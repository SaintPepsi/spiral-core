//! 🔄 SELF UPDATE TYPES: Core data structures for update system
//!
//! This module defines the fundamental types used throughout the self-update system.
//! All types are designed to be serializable for persistence and network transmission.

use serde::{Deserialize, Serialize};

/// Represents a request to update the system through Claude Code
///
/// Each request captures all necessary context from the Discord conversation,
/// including the user's intent, authorization details, and tracking information.
///
/// # Security Considerations
///
/// - `codename` is sanitized before use in git operations
/// - `description` and `combined_messages` are validated for size limits
/// - `user_id` must be in the authorized users list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfUpdateRequest {
    /// Unique identifier combining codename and timestamp
    pub id: String,

    /// Human-readable identifier for the update (e.g., "fix-memory-leak")
    /// Must be alphanumeric with hyphens/underscores only
    pub codename: String,

    /// ISO 8601 timestamp of request creation
    pub timestamp: String,

    /// Discord user ID of the requester (must be authorized)
    pub user_id: u64,

    /// Discord channel ID for sending status updates
    pub channel_id: u64,

    /// Original Discord message ID that triggered the update
    pub message_id: u64,

    /// Clear description of what the update should accomplish
    pub description: String,

    /// All user messages in the conversation thread for full context
    pub combined_messages: Vec<String>,

    /// Number of retry attempts (max 3 before permanent failure)
    pub retry_count: u32,

    /// Current status of the update request
    pub status: UpdateStatus,
}

/// Tracks the current state of an update request through its lifecycle
///
/// Updates progress through these states sequentially, with the ability
/// to fail or rollback at any stage after snapshot creation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpdateStatus {
    /// Request is waiting in queue to be processed
    Queued,

    /// Running pre-flight checks (git status, disk space, etc.)
    PreflightChecks,

    /// Creating git snapshot for rollback capability
    CreatingSnapshot,

    /// Executing changes via Claude Code
    Executing,

    /// Running validation tests on changes
    Testing,

    /// Update completed successfully
    Completed,

    /// Update failed with error message
    Failed(String),

    /// Update was rolled back to previous state
    RolledBack,
}
