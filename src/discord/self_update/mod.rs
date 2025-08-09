//! 🔄 SELF UPDATE MODULE: Autonomous system update functionality
//!
//! This module provides safe, controlled system updates through Claude Code integration.
//! It ensures atomic updates with automatic rollback capabilities and comprehensive validation.
//!
//! # Architecture
//!
//! The self-update system is designed with safety as the primary concern:
//! - **Bounded Queues**: Prevents memory exhaustion from unlimited update requests
//! - **Git Snapshots**: Every update creates a restorable snapshot before changes
//! - **Atomic Operations**: Updates either fully succeed or are completely rolled back
//! - **Multi-Stage Validation**: Pre-flight checks, compilation, tests, and security validation
//!
//! # Components
//!
//! - `UpdateQueue`: Thread-safe queue for managing pending update requests
//! - `GitOperations`: Safe git operations for snapshots and rollbacks
//! - `UpdateValidator`: Validates requests and system changes
//! - `PreflightChecker`: Ensures system is ready for updates
//!
//! # Safety Guarantees
//!
//! 1. **Input Sanitization**: All user inputs are sanitized to prevent injection attacks
//! 2. **Resource Limits**: Queue size and content size are bounded to prevent DoS
//! 3. **Authorization**: Only authorized users can trigger updates
//! 4. **Rollback Capability**: Any failed update can be rolled back to previous state
//!
//! # Usage Example
//!
//! ```rust,no_run
//! use spiral_core::discord::self_update::{UpdateQueue, SelfUpdateRequest, UpdateStatus};
//! use serenity::model::id::{UserId, ChannelId};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let queue = UpdateQueue::new();
//! let request = SelfUpdateRequest {
//!     id: "update-123".to_string(),
//!     codename: "feature-x".to_string(),
//!     description: "Add feature X".to_string(),
//!     user_id: 123456789,
//!     channel_id: 987654321,
//!     message_id: 111222333,
//!     combined_messages: vec!["Add feature X functionality".to_string()],
//!     timestamp: chrono::Utc::now().to_rfc3339(),
//!     retry_count: 0,
//!     status: UpdateStatus::Queued,
//! };
//!
//! // Add request to queue
//! queue.try_add_request(request).await?;
//! # Ok(())
//! # }
//! ```

mod approval;
mod executor;
mod git_ops;
mod health_monitor;
mod message_templates;
pub mod pipeline;
mod planner;
mod pre_validation;
mod progress_reporter;
mod queue;
mod scope_limiter;
mod status_tracker;
mod structured_logger;
mod system_lock;
mod types;
mod validation;
mod validation_agents;

pub mod claude_spawn_example;

pub use approval::{ApprovalManager, ApprovalResult, PendingApproval, format_approval_instructions};
pub use executor::{UpdateExecutor, UpdateResult};
pub use git_ops::{GitOperations, SnapshotManager};
pub use health_monitor::{HealthMonitor, HealthCheckResult, HealthCheck, HealthCategory};
pub use message_templates::UpdateMessageTemplates;
pub use pipeline::{
    CheckResult, ClaudeValidationResponse, ComplianceCheck, ExecutionPatterns, Phase1Results,
    Phase2Attempt, Phase2Checks, PipelineContext, PipelineStatus, ValidationPipeline,
};
pub use planner::{
    format_plan_for_discord, ApprovalStatus, ImplementationPlan, PlannedTask, ResourceRequirements,
    RiskLevel as PlanRiskLevel, TaskCategory, UpdatePlanner,
};
pub use progress_reporter::{ProgressReporter, UpdatePhase, UpdateProgress};
pub use queue::{UpdateQueue, UpdateQueueStatus};
pub use scope_limiter::{ScopeLimiter, ScopeLimits, ChangeScope};
pub use status_tracker::{ImplementationProgress, StatusTracker, UpdateType};
pub use structured_logger::StructuredLogger;
pub use system_lock::{SystemLock, LockToken};
pub use types::{SelfUpdateRequest, UpdateStatus};
pub use validation::{PreflightChecker, UpdateValidator};

// Re-export constants
pub const MAX_QUEUE_SIZE: usize = 10;
pub const MAX_UPDATE_CONTENT_SIZE: usize = 64 * 1024; // 64KB

#[cfg(test)]
mod tests;
#[cfg(test)]
mod structured_logger_test;
#[cfg(test)]
mod pre_validation_tests;
#[cfg(test)]
pub mod test_harness;
