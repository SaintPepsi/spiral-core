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
use std::collections::{VecDeque, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct UpdateQueue {
    inner: Arc<Mutex<UpdateQueueInner>>,
}

#[derive(Debug)]
struct UpdateQueueInner {
    requests: VecDeque<SelfUpdateRequest>,
    processing_requests: HashSet<String>, // Track multiple concurrent requests
    max_concurrent: usize,                // Maximum concurrent updates allowed
    rejected_count: u64,                  // Track rejected requests for monitoring
}

impl UpdateQueue {
    pub fn new() -> Self {
        Self::with_max_concurrent(3) // Default to 3 concurrent updates
    }
    
    /// Create a new queue with specified max concurrent updates
    pub fn with_max_concurrent(max_concurrent: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(UpdateQueueInner {
                requests: VecDeque::new(),
                processing_requests: HashSet::new(),
                max_concurrent: max_concurrent.max(1), // Ensure at least 1
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

    /// Get next request from queue if under concurrent limit
    pub async fn next_request(&self) -> Option<SelfUpdateRequest> {
        let mut inner = self.inner.lock().await;

        // Check if we've reached the concurrent limit
        if inner.processing_requests.len() >= inner.max_concurrent {
            return None;
        }

        // Find the next request that isn't already being processed
        while let Some(mut request) = inner.requests.pop_front() {
            // Skip if this request is somehow already being processed
            if inner.processing_requests.contains(&request.id) {
                continue;
            }
            
            // Mark as processing
            inner.processing_requests.insert(request.id.clone());
            request.status = UpdateStatus::PreflightChecks;
            return Some(request);
        }
        
        None
    }

    /// Mark a specific request as completed
    pub async fn complete_request(&self, request_id: &str) {
        let mut inner = self.inner.lock().await;
        inner.processing_requests.remove(request_id);
    }

    /// Get queue status for monitoring
    pub async fn get_status(&self) -> UpdateQueueStatus {
        let inner = self.inner.lock().await;

        UpdateQueueStatus {
            queue_size: inner.requests.len(),
            max_size: MAX_QUEUE_SIZE,
            rejected_count: inner.rejected_count,
            is_processing: !inner.processing_requests.is_empty(),
            current_request: None, // Deprecated - use processing_requests instead
            processing_requests: inner.processing_requests.clone(),
            max_concurrent: inner.max_concurrent,
        }
    }

    /// Clear all pending requests (emergency use only)
    pub async fn clear_queue(&self) {
        let mut inner = self.inner.lock().await;
        inner.requests.clear();
        // Note: We don't clear processing_requests as those are actively running
    }

    /// Mark a specific request as completed (deprecated - use complete_request)
    pub async fn mark_completed(&self, request_id: &str) {
        self.complete_request(request_id).await;
    }

    /// Mark a specific request as failed
    pub async fn mark_failed(&self, request_id: &str, clear_queue: bool) {
        let mut inner = self.inner.lock().await;
        inner.processing_requests.remove(request_id);
        
        // Optionally clear entire queue on critical failure
        if clear_queue {
            inner.requests.clear();
        }
    }
    
    /// Retry a failed request by re-queuing it with incremented retry count
    pub async fn retry_request(&self, mut request: SelfUpdateRequest) -> Result<()> {
        const MAX_RETRIES: u32 = 3;
        
        // Check if we've exceeded max retries
        if request.retry_count >= MAX_RETRIES {
            return Err(SpiralError::Validation(format!(
                "Request {} has exceeded maximum retries ({})",
                request.id, MAX_RETRIES
            )));
        }
        
        // Increment retry count
        request.retry_count += 1;
        
        // Reset status to queued
        request.status = UpdateStatus::Queued;
        
        // Generate new ID with retry suffix
        if !request.id.contains("-retry-") {
            request.id = format!("{}-retry-{}", request.id, request.retry_count);
        } else {
            // Update existing retry suffix
            let base_id = request.id.split("-retry-").next().unwrap_or(&request.id);
            request.id = format!("{}-retry-{}", base_id, request.retry_count);
        }
        
        // Add back to queue
        self.try_add_request(request).await
    }

    /// Check if system is shutting down
    pub async fn is_shutting_down(&self) -> bool {
        // For now, just check if we're not processing
        // In future, could add explicit shutdown flag
        let _inner = self.inner.lock().await;
        false // Placeholder - would check shutdown flag
    }
    
    /// Get the number of currently processing requests
    pub async fn processing_count(&self) -> usize {
        let inner = self.inner.lock().await;
        inner.processing_requests.len()
    }
    
    /// Check if a specific request is currently being processed
    pub async fn is_processing(&self, request_id: &str) -> bool {
        let inner = self.inner.lock().await;
        inner.processing_requests.contains(request_id)
    }

    /// Initiate shutdown
    pub async fn shutdown(&self) {
        let mut inner = self.inner.lock().await;
        // Clear queue but keep processing_requests to track what's still running
        inner.requests.clear();
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
    pub current_request: Option<String>, // Deprecated - kept for compatibility
    pub processing_requests: HashSet<String>, // Currently processing request IDs
    pub max_concurrent: usize, // Maximum concurrent updates allowed
}
