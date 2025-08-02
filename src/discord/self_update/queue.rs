//! 🔄 UPDATE QUEUE: Thread-safe queue management for update requests
//!
//! This module provides a bounded, thread-safe queue for managing self-update requests.
//! The queue prevents resource exhaustion attacks while ensuring fair processing order.
//!
//! # Thread Safety
//!
//! The queue uses `Arc<Mutex<>>` for thread safety across async contexts. This allows
//! multiple Discord event handlers to safely add requests while a background task
//! processes them.
//!
//! # Resource Limits
//!
//! - Maximum queue size: 10 requests
//! - Maximum content size per request: 64KB
//! - Duplicate codenames are rejected to prevent spam

use super::types::{SelfUpdateRequest, UpdateStatus};
use super::{MAX_QUEUE_SIZE, MAX_UPDATE_CONTENT_SIZE};
use crate::error::{Result, SpiralError};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct UpdateQueue {
    inner: Arc<Mutex<UpdateQueueInner>>,
}

#[derive(Debug)]
struct UpdateQueueInner {
    requests: VecDeque<SelfUpdateRequest>,
    is_processing: bool,
    current_request: Option<String>, // Current request ID
    rejected_count: u64,             // Track rejected requests for monitoring
}

impl UpdateQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(UpdateQueueInner {
                requests: VecDeque::new(),
                is_processing: false,
                current_request: None,
                rejected_count: 0,
            })),
        }
    }

    /// Attempt to add a request with bounds checking and content validation
    pub async fn try_add_request(&self, request: SelfUpdateRequest) -> Result<()> {
        let mut inner = self.inner.lock().await;

        // Check queue size limit
        if inner.requests.len() >= MAX_QUEUE_SIZE {
            inner.rejected_count += 1;
            return Err(SpiralError::QueueFull);
        }

        // Validate content size to prevent memory exhaustion
        let total_content_size = request.description.len()
            + request
                .combined_messages
                .iter()
                .map(|m| m.len())
                .sum::<usize>();

        if total_content_size > MAX_UPDATE_CONTENT_SIZE {
            inner.rejected_count += 1;
            return Err(SpiralError::Validation(format!(
                "Update content too large: {total_content_size} bytes (max: {MAX_UPDATE_CONTENT_SIZE})"
            )));
        }

        // Check for duplicate codenames
        if inner
            .requests
            .iter()
            .any(|r| r.codename == request.codename)
        {
            inner.rejected_count += 1;
            return Err(SpiralError::Validation(format!(
                "Duplicate request codename: {}",
                request.codename
            )));
        }

        inner.requests.push_back(request);
        Ok(())
    }

    /// Get next request from queue and mark as processing
    pub async fn next_request(&self) -> Option<SelfUpdateRequest> {
        let mut inner = self.inner.lock().await;

        if inner.is_processing {
            return None;
        }

        if let Some(mut request) = inner.requests.pop_front() {
            inner.is_processing = true;
            inner.current_request = Some(request.id.clone());
            request.status = UpdateStatus::PreflightChecks;
            Some(request)
        } else {
            None
        }
    }

    /// Mark current request as completed and clear processing flag
    pub async fn complete_request(&self, request_id: &str) {
        let mut inner = self.inner.lock().await;

        if inner.current_request.as_deref() == Some(request_id) {
            inner.is_processing = false;
            inner.current_request = None;
        }
    }

    /// Get queue status for monitoring
    pub async fn get_status(&self) -> UpdateQueueStatus {
        let inner = self.inner.lock().await;

        UpdateQueueStatus {
            queue_size: inner.requests.len(),
            max_size: MAX_QUEUE_SIZE,
            rejected_count: inner.rejected_count,
            is_processing: inner.is_processing,
            current_request: inner.current_request.clone(),
        }
    }

    /// Clear all pending requests (emergency use only)
    pub async fn clear_queue(&self) {
        let mut inner = self.inner.lock().await;
        inner.requests.clear();
        inner.is_processing = false;
        inner.current_request = None;
    }

    /// Mark current request as completed
    pub async fn mark_completed(&self) {
        let mut inner = self.inner.lock().await;
        inner.is_processing = false;
        inner.current_request = None;
    }

    /// Mark current request as failed and clear queue
    pub async fn mark_failed(&self) {
        let mut inner = self.inner.lock().await;
        inner.is_processing = false;
        inner.current_request = None;
        // Clear entire queue on failure as per spec
        inner.requests.clear();
    }

    /// Check if system is shutting down
    pub async fn is_shutting_down(&self) -> bool {
        // For now, just check if we're not processing
        // In future, could add explicit shutdown flag
        let _inner = self.inner.lock().await;
        false // Placeholder - would check shutdown flag
    }

    /// Initiate shutdown
    pub async fn shutdown(&self) {
        let mut inner = self.inner.lock().await;
        // Clear queue and mark as not processing
        inner.requests.clear();
        inner.is_processing = false;
        inner.current_request = None;
    }
}

impl Default for UpdateQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Queue status information
#[derive(Debug, Clone)]
pub struct UpdateQueueStatus {
    pub queue_size: usize,
    pub max_size: usize,
    pub rejected_count: u64,
    pub is_processing: bool,
    pub current_request: Option<String>,
}
