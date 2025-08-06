//! Progress reporting for self-update operations
//!
//! This module provides real-time progress updates to Discord during
//! the execution of self-updates, keeping users informed about the status.

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use serenity::http::Http;
use serenity::model::id::ChannelId;
use tracing::{debug, error, info};

/// Progress information for an ongoing update
#[derive(Debug, Clone)]
pub struct UpdateProgress {
    /// Update request ID
    pub update_id: String,
    /// Current phase of the update
    pub current_phase: UpdatePhase,
    /// Percentage complete (0-100)
    pub percent_complete: u8,
    /// Current status message
    pub status_message: String,
    /// Number of tasks completed
    pub tasks_completed: usize,
    /// Total number of tasks
    pub total_tasks: usize,
    /// When the update started
    pub started_at: Instant,
    /// Last time progress was reported
    pub last_report_at: Instant,
}

/// Phases of a self-update operation
#[derive(Debug, Clone, PartialEq)]
pub enum UpdatePhase {
    Initializing,
    PreflightChecks,
    Planning,
    AwaitingApproval,
    CreatingSnapshot,
    Implementing,
    Validating,
    Completing,
    Complete,
    Failed,
}

impl UpdatePhase {
    /// Get emoji for the phase
    pub fn emoji(&self) -> &str {
        match self {
            UpdatePhase::Initializing => "🔄",
            UpdatePhase::PreflightChecks => "🔍",
            UpdatePhase::Planning => "📋",
            UpdatePhase::AwaitingApproval => "⏳",
            UpdatePhase::CreatingSnapshot => "📸",
            UpdatePhase::Implementing => "🤖",
            UpdatePhase::Validating => "✅",
            UpdatePhase::Completing => "🏁",
            UpdatePhase::Complete => "🎉",
            UpdatePhase::Failed => "❌",
        }
    }
    
    /// Get human-readable name
    pub fn name(&self) -> &str {
        match self {
            UpdatePhase::Initializing => "Initializing",
            UpdatePhase::PreflightChecks => "Running Preflight Checks",
            UpdatePhase::Planning => "Creating Plan",
            UpdatePhase::AwaitingApproval => "Awaiting Approval",
            UpdatePhase::CreatingSnapshot => "Creating Git Snapshot",
            UpdatePhase::Implementing => "Implementing Changes",
            UpdatePhase::Validating => "Validating Changes",
            UpdatePhase::Completing => "Completing Update",
            UpdatePhase::Complete => "Complete",
            UpdatePhase::Failed => "Failed",
        }
    }
}

/// Progress reporter that sends periodic updates to Discord
pub struct ProgressReporter {
    /// Current progress state
    progress: Arc<RwLock<UpdateProgress>>,
    /// Discord HTTP client for sending updates
    discord_http: Option<Arc<Http>>,
    /// Channel to send updates to
    channel_id: ChannelId,
    /// Whether reporting is active
    active: Arc<RwLock<bool>>,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(
        update_id: String,
        discord_http: Option<Arc<Http>>,
        channel_id: ChannelId,
        total_tasks: usize,
    ) -> Self {
        let progress = UpdateProgress {
            update_id,
            current_phase: UpdatePhase::Initializing,
            percent_complete: 0,
            status_message: "Starting update...".to_string(),
            tasks_completed: 0,
            total_tasks,
            started_at: Instant::now(),
            last_report_at: Instant::now(),
        };
        
        Self {
            progress: Arc::new(RwLock::new(progress)),
            discord_http,
            channel_id,
            active: Arc::new(RwLock::new(true)),
        }
    }
    
    /// Start the background progress reporting task
    pub fn start_reporting(&self, report_interval: Duration) {
        let progress = self.progress.clone();
        let discord_http = self.discord_http.clone();
        let channel_id = self.channel_id;
        let active = self.active.clone();
        
        tokio::spawn(async move {
            let mut ticker = interval(report_interval);
            
            loop {
                ticker.tick().await;
                
                // Check if reporting is still active
                if !*active.read().await {
                    debug!("[ProgressReporter] Reporting stopped");
                    break;
                }
                
                // Get current progress
                let current = progress.read().await.clone();
                
                // Don't report if we just reported
                if current.last_report_at.elapsed() < Duration::from_secs(5) {
                    continue;
                }
                
                // Format progress message
                let elapsed = current.started_at.elapsed();
                let progress_bar = Self::create_progress_bar(current.percent_complete);
                
                let message = format!(
                    "{} **{}** ({}%)\n{}\n⏱️ {} | 📊 {}/{} tasks | 💬 {}",
                    current.current_phase.emoji(),
                    current.current_phase.name(),
                    current.percent_complete,
                    progress_bar,
                    Self::format_duration(elapsed),
                    current.tasks_completed,
                    current.total_tasks,
                    current.status_message
                );
                
                // Send to Discord if available
                if let Some(ref http) = discord_http {
                    if let Err(e) = channel_id.say(http, &message).await {
                        error!("[ProgressReporter] Failed to send progress update: {}", e);
                    }
                }
                
                // Update last report time
                progress.write().await.last_report_at = Instant::now();
                
                // Stop reporting if complete or failed
                if matches!(current.current_phase, UpdatePhase::Complete | UpdatePhase::Failed) {
                    *active.write().await = false;
                    break;
                }
            }
        });
    }
    
    /// Update the current phase
    pub async fn set_phase(&self, phase: UpdatePhase) {
        let mut progress = self.progress.write().await;
        progress.current_phase = phase.clone();
        
        // Update percentage based on phase
        progress.percent_complete = match phase {
            UpdatePhase::Initializing => 0,
            UpdatePhase::PreflightChecks => 10,
            UpdatePhase::Planning => 20,
            UpdatePhase::AwaitingApproval => 30,
            UpdatePhase::CreatingSnapshot => 40,
            UpdatePhase::Implementing => 50,
            UpdatePhase::Validating => 80,
            UpdatePhase::Completing => 95,
            UpdatePhase::Complete => 100,
            UpdatePhase::Failed => progress.percent_complete, // Keep current
        };
        
        info!(
            "[ProgressReporter] Phase changed to {:?} ({}%)",
            phase, progress.percent_complete
        );
    }
    
    /// Update the status message
    pub async fn set_status(&self, message: String) {
        let mut progress = self.progress.write().await;
        progress.status_message = message.clone();
        debug!("[ProgressReporter] Status: {}", message);
    }
    
    /// Update task completion count
    pub async fn increment_tasks_completed(&self) {
        let mut progress = self.progress.write().await;
        progress.tasks_completed += 1;
        
        // Update percentage if in implementation phase
        if progress.current_phase == UpdatePhase::Implementing {
            let task_progress = (progress.tasks_completed as f32 / progress.total_tasks as f32) * 30.0;
            progress.percent_complete = 50 + task_progress as u8;
        }
    }
    
    /// Set custom percentage
    pub async fn set_percent(&self, percent: u8) {
        let mut progress = self.progress.write().await;
        progress.percent_complete = percent.min(100);
    }
    
    /// Stop progress reporting
    pub async fn stop(&self) {
        *self.active.write().await = false;
    }
    
    /// Create a visual progress bar
    fn create_progress_bar(percent: u8) -> String {
        let filled = (percent as usize * 20) / 100;
        let empty = 20 - filled;
        
        format!(
            "[{}{}]",
            "█".repeat(filled),
            "░".repeat(empty)
        )
    }
    
    /// Format duration as human-readable string
    fn format_duration(duration: Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_progress_bar_creation() {
        assert_eq!(ProgressReporter::create_progress_bar(0), "[░░░░░░░░░░░░░░░░░░░░]");
        assert_eq!(ProgressReporter::create_progress_bar(25), "[█████░░░░░░░░░░░░░░░]");
        assert_eq!(ProgressReporter::create_progress_bar(50), "[██████████░░░░░░░░░░]");
        assert_eq!(ProgressReporter::create_progress_bar(75), "[███████████████░░░░░]");
        assert_eq!(ProgressReporter::create_progress_bar(100), "[████████████████████]");
    }
    
    #[test]
    fn test_duration_formatting() {
        assert_eq!(ProgressReporter::format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(ProgressReporter::format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(ProgressReporter::format_duration(Duration::from_secs(3661)), "1h 1m");
    }
    
    #[tokio::test]
    async fn test_phase_progression() {
        let reporter = ProgressReporter::new(
            "test-123".to_string(),
            None,
            ChannelId::new(123),
            5,
        );
        
        // Test phase changes update percentage
        reporter.set_phase(UpdatePhase::PreflightChecks).await;
        assert_eq!(reporter.progress.read().await.percent_complete, 10);
        
        reporter.set_phase(UpdatePhase::Planning).await;
        assert_eq!(reporter.progress.read().await.percent_complete, 20);
        
        reporter.set_phase(UpdatePhase::Complete).await;
        assert_eq!(reporter.progress.read().await.percent_complete, 100);
    }
    
    #[tokio::test]
    async fn test_task_completion_tracking() {
        let reporter = ProgressReporter::new(
            "test-456".to_string(),
            None,
            ChannelId::new(456),
            4,
        );
        
        // Set to implementing phase
        reporter.set_phase(UpdatePhase::Implementing).await;
        
        // Complete tasks and check percentage
        reporter.increment_tasks_completed().await;
        assert_eq!(reporter.progress.read().await.tasks_completed, 1);
        
        reporter.increment_tasks_completed().await;
        assert_eq!(reporter.progress.read().await.tasks_completed, 2);
        
        // Should be at 50% + (2/4 * 30%) = 65%
        assert_eq!(reporter.progress.read().await.percent_complete, 65);
    }
}