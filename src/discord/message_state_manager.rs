use serenity::model::id::{ChannelId, MessageId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// Represents a message update that should be applied
#[derive(Debug, Clone)]
pub enum MessageUpdate {
    Content(String),
    Reaction(String),
    Edit { content: String },
}

/// Tracks the state of a pending Discord message
#[derive(Debug, Clone)]
pub struct PendingMessage {
    pub message_id: MessageId,
    pub channel_id: ChannelId,
    pub expected_updates: Vec<MessageUpdate>,
    pub current_state: String,
    pub created_at: Instant,
    pub last_update_attempt: Option<Instant>,
    pub retry_count: u32,
}

impl PendingMessage {
    pub fn new(message_id: MessageId, channel_id: ChannelId, initial_content: String) -> Self {
        Self {
            message_id,
            channel_id,
            expected_updates: Vec::new(),
            current_state: initial_content,
            created_at: Instant::now(),
            last_update_attempt: None,
            retry_count: 0,
        }
    }

    /// Check if this message has timed out
    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        self.created_at.elapsed() > timeout
    }

    /// Check if ready for retry
    pub fn ready_for_retry(&self, retry_interval: Duration) -> bool {
        match self.last_update_attempt {
            None => true,
            Some(last_attempt) => last_attempt.elapsed() >= retry_interval,
        }
    }
}

/// Configuration for message state recovery
#[derive(Debug, Clone)]
pub struct MessageStateConfig {
    /// Maximum time to track a pending message
    pub message_timeout: Duration,
    /// Time between retry attempts
    pub retry_interval: Duration,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Cleanup interval for expired messages
    pub cleanup_interval: Duration,
}

impl Default for MessageStateConfig {
    fn default() -> Self {
        Self {
            message_timeout: Duration::from_secs(300), // 5 minutes
            retry_interval: Duration::from_secs(5),
            max_retries: 3,
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

/// Manages message state and recovery for Discord bot
pub struct MessageStateManager {
    pending_messages: Arc<RwLock<HashMap<MessageId, PendingMessage>>>,
    config: MessageStateConfig,
    recovery_stats: Arc<Mutex<RecoveryStats>>,
    // 🔧 RESOURCE LEAK FIX: Add task lifecycle management
    cleanup_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    shutdown_signal_sender: Arc<Mutex<Option<mpsc::Sender<()>>>>,
}

#[derive(Debug, Default)]
struct RecoveryStats {
    successful_recoveries: u64,
    failed_recoveries: u64,
    timed_out_messages: u64,
}

impl MessageStateManager {
    pub fn new(config: MessageStateConfig) -> Self {
        Self {
            pending_messages: Arc::new(RwLock::new(HashMap::new())),
            config,
            recovery_stats: Arc::new(Mutex::new(RecoveryStats::default())),
            // 🔧 RESOURCE LEAK FIX: Initialize task management
            cleanup_handle: Arc::new(Mutex::new(None)),
            shutdown_signal_sender: Arc::new(Mutex::new(None)),
        }
    }

    /// Register a new message for state tracking
    pub async fn register_message(
        &self,
        message_id: MessageId,
        channel_id: ChannelId,
        initial_content: String,
    ) {
        let mut messages = self.pending_messages.write().await;
        let pending = PendingMessage::new(message_id, channel_id, initial_content);
        messages.insert(message_id, pending);

        debug!("Registered message {} for state tracking", message_id);
    }

    /// Add an expected update to a message
    pub async fn add_expected_update(&self, message_id: MessageId, update: MessageUpdate) {
        let mut messages = self.pending_messages.write().await;
        if let Some(pending) = messages.get_mut(&message_id) {
            pending.expected_updates.push(update);
            debug!("Added expected update for message {}", message_id);
        }
    }

    /// Mark a message update as successful
    pub async fn mark_update_successful(&self, message_id: MessageId, new_content: Option<String>) {
        let mut messages = self.pending_messages.write().await;
        if let Some(pending) = messages.get_mut(&message_id) {
            if let Some(content) = new_content {
                pending.current_state = content;
            }
            // Remove the first expected update as it was successful
            if !pending.expected_updates.is_empty() {
                pending.expected_updates.remove(0);
            }
            pending.retry_count = 0; // Reset retry count on success

            // If no more updates expected, remove from tracking
            if pending.expected_updates.is_empty() {
                messages.remove(&message_id);
                let mut stats = self.recovery_stats.lock().await;
                stats.successful_recoveries += 1;
                info!("Message {} successfully completed all updates", message_id);
            }
        }
    }

    /// Mark a message update as failed
    pub async fn mark_update_failed(&self, message_id: MessageId) {
        let mut messages = self.pending_messages.write().await;
        if let Some(pending) = messages.get_mut(&message_id) {
            pending.retry_count += 1;
            pending.last_update_attempt = Some(Instant::now());

            if pending.retry_count >= self.config.max_retries {
                warn!("Message {} exceeded max retries, abandoning", message_id);
                messages.remove(&message_id);
                let mut stats = self.recovery_stats.lock().await;
                stats.failed_recoveries += 1;
            } else {
                debug!(
                    "Message {} update failed, will retry ({}/{})",
                    message_id, pending.retry_count, self.config.max_retries
                );
            }
        }
    }

    /// Get messages that need retry
    pub async fn get_messages_for_retry(&self) -> Vec<PendingMessage> {
        let messages = self.pending_messages.read().await;
        messages
            .values()
            .filter(|msg| {
                !msg.expected_updates.is_empty()
                    && msg.ready_for_retry(self.config.retry_interval)
                    && !msg.is_timed_out(self.config.message_timeout)
            })
            .cloned()
            .collect()
    }

    /// Clean up expired messages
    pub async fn cleanup_expired_messages(&self) {
        let mut messages = self.pending_messages.write().await;
        let mut expired_count = 0;

        messages.retain(|id, msg| {
            if msg.is_timed_out(self.config.message_timeout) {
                warn!("Message {} timed out, removing from tracking", id);
                expired_count += 1;
                false
            } else {
                true
            }
        });

        if expired_count > 0 {
            let mut stats = self.recovery_stats.lock().await;
            stats.timed_out_messages += expired_count;
            info!("Cleaned up {} expired messages", expired_count);
        }
    }

    /// Get recovery statistics
    pub async fn get_stats(&self) -> MessageRecoveryStats {
        let stats = self.recovery_stats.lock().await;
        let pending_count = self.pending_messages.read().await.len();

        MessageRecoveryStats {
            pending_messages: pending_count as u64,
            successful_recoveries: stats.successful_recoveries,
            failed_recoveries: stats.failed_recoveries,
            timed_out_messages: stats.timed_out_messages,
        }
    }

    /// Start background cleanup task with proper lifecycle management
    /// 🔧 RESOURCE LEAK FIX: Properly managed background task with shutdown capability
    pub async fn start_cleanup_task(self: Arc<Self>) {
        let cleanup_interval = self.config.cleanup_interval;
        let (shutdown_signal_sender, mut shutdown_signal_receiver) = mpsc::channel::<()>(1);

        // Store the shutdown channel for later cleanup
        {
            let mut sender_guard = self.shutdown_signal_sender.lock().await;
            *sender_guard = Some(shutdown_signal_sender);
        }

        let manager_clone = Arc::clone(&self);
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        manager_clone.cleanup_expired_messages().await;
                    }
                    _ = shutdown_signal_receiver.recv() => {
                        info!("MessageStateManager cleanup task shutting down gracefully");
                        break;
                    }
                }
            }
        });

        // Store the handle for proper cleanup
        {
            let mut handle_guard = self.cleanup_handle.lock().await;
            *handle_guard = Some(handle);
        }

        info!("MessageStateManager cleanup task started with graceful shutdown capability");
    }

    /// Shutdown cleanup task and free resources
    /// 🔧 RESOURCE LEAK FIX: Proper task shutdown and resource cleanup
    pub async fn shutdown(&self) {
        info!("Shutting down MessageStateManager...");

        // Signal shutdown
        if let Some(sender) = self.shutdown_signal_sender.lock().await.take() {
            let _ = sender.send(()).await; // Ignore send errors if receiver is dropped
        }

        // Wait for task to complete
        if let Some(handle) = self.cleanup_handle.lock().await.take() {
            if let Err(e) = handle.await {
                warn!("Error waiting for cleanup task to complete: {}", e);
            }
        }

        info!("MessageStateManager shutdown complete");
    }
}

#[derive(Debug, Clone)]
pub struct MessageRecoveryStats {
    pub pending_messages: u64,
    pub successful_recoveries: u64,
    pub failed_recoveries: u64,
    pub timed_out_messages: u64,
}
