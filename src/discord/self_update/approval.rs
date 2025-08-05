//! Plan approval flow management for self-updates
//!
//! This module handles the interactive approval process where users
//! review and approve/reject implementation plans before execution.

use super::planner::{ApprovalStatus, ImplementationPlan};
use crate::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use tracing::{info, warn};

/// Manages pending plan approvals
pub struct ApprovalManager {
    /// Maps message IDs to pending plans awaiting approval
    pending_approvals: Arc<RwLock<HashMap<u64, PendingApproval>>>,
}

/// A plan waiting for user approval
#[derive(Debug, Clone)]
pub struct PendingApproval {
    /// The implementation plan
    pub plan: ImplementationPlan,
    /// Original request ID
    pub request_id: String,
    /// User who must approve (original requester)
    pub user_id: u64,
    /// Channel where approval was requested
    pub channel_id: u64,
    /// Message ID of the plan presentation
    pub plan_message_id: u64,
    /// When the approval was requested
    pub requested_at: std::time::Instant,
}

/// Result of waiting for approval
#[derive(Debug, Clone)]
pub enum ApprovalResult {
    /// Plan was approved
    Approved,
    /// Plan was rejected with reason
    Rejected(String),
    /// User requested modifications
    ModifyRequested(String),
    /// Approval timed out
    TimedOut,
}

impl ApprovalManager {
    /// Create a new approval manager
    pub fn new() -> Self {
        Self {
            pending_approvals: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a plan for approval
    pub async fn register_for_approval(
        &self,
        plan: ImplementationPlan,
        request_id: String,
        user_id: u64,
        channel_id: u64,
        plan_message_id: u64,
    ) {
        let pending = PendingApproval {
            plan,
            request_id: request_id.clone(),
            user_id,
            channel_id,
            plan_message_id,
            requested_at: std::time::Instant::now(),
        };

        let mut approvals = self.pending_approvals.write().await;
        approvals.insert(plan_message_id, pending);
        
        info!(
            "[ApprovalManager] Registered plan for approval: {} (message: {})",
            request_id, plan_message_id
        );
    }

    /// Wait for approval with timeout
    pub async fn wait_for_approval(
        &self,
        plan_message_id: u64,
        timeout_duration: Duration,
    ) -> Result<(ApprovalResult, Option<ImplementationPlan>)> {
        info!(
            "[ApprovalManager] Waiting for approval on message {} (timeout: {:?})",
            plan_message_id, timeout_duration
        );

        // Simple polling approach - check for approval status changes
        let start_time = std::time::Instant::now();
        
        loop {
            // Check if approval was processed
            let approval_status = {
                let approvals = self.pending_approvals.read().await;
                if let Some(pending) = approvals.get(&plan_message_id) {
                    // Clone the approval status to check it after dropping the lock
                    pending.plan.approval_status.clone()
                } else {
                    // Approval was removed (shouldn't happen)
                    return Ok((ApprovalResult::TimedOut, None));
                }
            };
            
            // Check if approval status changed
            match approval_status {
                ApprovalStatus::Approved => {
                    let plan = self.remove_pending_approval(plan_message_id).await;
                    return Ok((ApprovalResult::Approved, plan.map(|p| p.plan)));
                }
                ApprovalStatus::Rejected(reason) => {
                    let plan = self.remove_pending_approval(plan_message_id).await;
                    return Ok((ApprovalResult::Rejected(reason), plan.map(|p| p.plan)));
                }
                ApprovalStatus::Modified => {
                    let plan = self.remove_pending_approval(plan_message_id).await;
                    return Ok((ApprovalResult::ModifyRequested("Modifications requested".to_string()), plan.map(|p| p.plan)));
                }
                ApprovalStatus::Pending => {
                    // Still waiting, continue loop
                }
            }
            
            // Check for timeout
            if start_time.elapsed() >= timeout_duration {
                info!("[ApprovalManager] Approval timed out for message {}", plan_message_id);
                let plan = self.remove_pending_approval(plan_message_id).await;
                return Ok((ApprovalResult::TimedOut, plan.map(|p| p.plan)));
            }
            
            // Sleep briefly before checking again
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    /// Process an approval response from a user
    pub async fn process_approval_response(
        &self,
        user_id: u64,
        channel_id: u64,
        response: &str,
    ) -> Option<(String, ApprovalResult)> {
        let response_lower = response.trim().to_lowercase();
        
        // Find pending approval for this user in this channel
        let approvals = self.pending_approvals.read().await;
        let pending = approvals.values().find(|p| {
            p.user_id == user_id && p.channel_id == channel_id
        })?;
        
        let plan_message_id = pending.plan_message_id;
        let request_id = pending.request_id.clone();
        drop(approvals);

        // Determine the approval result
        let result = if response_lower == "approve" || response_lower.starts_with("approve") {
            ApprovalResult::Approved
        } else if response_lower == "reject" || response_lower.starts_with("reject") {
            let reason = if response_lower.len() > 6 {
                response[6..].trim().to_string()
            } else {
                "No reason provided".to_string()
            };
            ApprovalResult::Rejected(reason)
        } else if response_lower == "modify" || response_lower.starts_with("modify") {
            let details = if response_lower.len() > 6 {
                response[6..].trim().to_string()
            } else {
                "No details provided".to_string()
            };
            ApprovalResult::ModifyRequested(details)
        } else {
            // Not an approval response
            return None;
        };

        // Update the plan status in the stored pending approval
        {
            let mut approvals = self.pending_approvals.write().await;
            if let Some(pending) = approvals.get_mut(&plan_message_id) {
                match &result {
                    ApprovalResult::Approved => {
                        pending.plan.approval_status = ApprovalStatus::Approved;
                    }
                    ApprovalResult::Rejected(reason) => {
                        pending.plan.approval_status = ApprovalStatus::Rejected(reason.clone());
                    }
                    ApprovalResult::ModifyRequested(_) => {
                        pending.plan.approval_status = ApprovalStatus::Modified;
                    }
                    _ => {}
                }
            }
        }

        info!(
            "[ApprovalManager] Processed approval response: {:?} for request {}",
            result, request_id
        );

        Some((request_id, result))
    }

    /// Remove and return a pending approval
    async fn remove_pending_approval(&self, plan_message_id: u64) -> Option<PendingApproval> {
        let mut approvals = self.pending_approvals.write().await;
        approvals.remove(&plan_message_id)
    }

    /// Clean up old pending approvals (older than 1 hour)
    pub async fn cleanup_old_approvals(&self) {
        let mut approvals = self.pending_approvals.write().await;
        let now = std::time::Instant::now();
        
        approvals.retain(|_, pending| {
            let age = now.duration_since(pending.requested_at);
            if age > Duration::from_secs(3600) {
                warn!(
                    "[ApprovalManager] Removing old approval request: {} (age: {:?})",
                    pending.request_id, age
                );
                false
            } else {
                true
            }
        });
    }

    /// Check if there's a pending approval for a user in a channel
    pub async fn has_pending_approval(&self, user_id: u64, channel_id: u64) -> bool {
        let approvals = self.pending_approvals.read().await;
        approvals.values().any(|p| p.user_id == user_id && p.channel_id == channel_id)
    }

    /// Get the pending approval for a user in a channel
    pub async fn get_pending_approval(
        &self,
        user_id: u64,
        channel_id: u64,
    ) -> Option<PendingApproval> {
        let approvals = self.pending_approvals.read().await;
        approvals
            .values()
            .find(|p| p.user_id == user_id && p.channel_id == channel_id)
            .cloned()
    }
}

impl Default for ApprovalManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Format approval instructions for Discord
pub fn format_approval_instructions() -> &'static str {
    "\n📋 **Plan Review Instructions**\n\
    React with:\n\
    • ✅ to **approve** and proceed with implementation\n\
    • ❌ to **reject** and cancel the update\n\
    • ✏️ to **request modifications**\n\
    \n\
    Or reply with:\n\
    • **approve** to proceed with the implementation\n\
    • **reject [reason]** to cancel the update\n\
    • **modify [details]** to request changes\n\
    \n\
    ⏱️ You have 10 minutes to respond before the request times out"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discord::self_update::planner::{PlannedTask, ResourceRequirements, TaskCategory};

    fn create_test_plan() -> ImplementationPlan {
        ImplementationPlan {
            plan_id: "test-plan-123".to_string(),
            request_id: "test-request-123".to_string(),
            summary: "Test plan".to_string(),
            risk_level: super::super::planner::RiskLevel::Low,
            tasks: vec![PlannedTask {
                id: "task-1".to_string(),
                description: "Test task".to_string(),
                category: TaskCategory::CodeChange,
                complexity: 2,
                dependencies: vec![],
                affected_components: vec!["test.rs".to_string()],
                validation_steps: vec!["Run tests".to_string()],
            }],
            identified_risks: vec!["Low risk".to_string()],
            rollback_strategy: "Git rollback".to_string(),
            success_criteria: vec!["Tests pass".to_string()],
            resource_requirements: ResourceRequirements {
                required_agents: vec!["Claude Code".to_string()],
                special_requirements: vec![],
            },
            approval_status: ApprovalStatus::Pending,
        }
    }

    #[tokio::test]
    async fn test_approval_registration() {
        let manager = ApprovalManager::new();
        let plan = create_test_plan();
        
        manager
            .register_for_approval(
                plan.clone(),
                "test-request-123".to_string(),
                123456,
                789012,
                111222,
            )
            .await;

        assert!(manager.has_pending_approval(123456, 789012).await);
    }

    #[tokio::test]
    async fn test_approval_response_processing() {
        let manager = ApprovalManager::new();
        let plan = create_test_plan();
        
        manager
            .register_for_approval(
                plan.clone(),
                "test-request-123".to_string(),
                123456,
                789012,
                111222,
            )
            .await;

        // Test approve
        let result = manager
            .process_approval_response(123456, 789012, "approve")
            .await;
        assert!(result.is_some());
        if let Some((request_id, approval_result)) = result {
            assert_eq!(request_id, "test-request-123");
            assert!(matches!(approval_result, ApprovalResult::Approved));
        }
    }

    #[tokio::test]
    async fn test_approval_timeout() {
        let manager = ApprovalManager::new();
        let plan = create_test_plan();
        
        manager
            .register_for_approval(
                plan.clone(),
                "test-request-123".to_string(),
                123456,
                789012,
                111222,
            )
            .await;

        // Wait with very short timeout
        let result = manager
            .wait_for_approval(111222, Duration::from_millis(10))
            .await
            .unwrap();
        
        assert!(matches!(result.0, ApprovalResult::TimedOut));
    }
}