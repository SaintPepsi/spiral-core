//! Update executor for orchestrating the self-update flow
//!
//! This module implements the main orchestrator that coordinates all aspects of a self-update:
//! git operations, Claude Code integration, validation pipeline, and result reporting.

use super::{
    format_plan_for_discord, ApprovalManager, ApprovalResult, GitOperations, ImplementationPlan,
    PreflightChecker, SelfUpdateRequest, StatusTracker, SystemLock, UpdatePlanner, 
    UpdateQueue, UpdateStatus, ValidationPipeline, format_approval_instructions,
};
use crate::{claude_code::ClaudeCodeClient, Result, error::SpiralError};
use serenity::{http::Http, model::id::ChannelId};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// The result of an update execution
#[derive(Debug, Clone)]
pub struct UpdateResult {
    /// The original request
    pub request: SelfUpdateRequest,
    /// Whether the update succeeded
    pub success: bool,
    /// Description of the outcome
    pub message: String,
    /// Git snapshot ID if created
    pub snapshot_id: Option<String>,
    /// Detailed error if failed
    pub error: Option<String>,
    /// Validation pipeline results if run
    pub validation_results: Option<String>,
}

/// The main update executor that orchestrates self-updates
pub struct UpdateExecutor {
    /// Update queue for managing requests
    queue: Arc<UpdateQueue>,
    /// Claude Code client for AI operations
    claude_client: Option<ClaudeCodeClient>,
    /// Status tracker for monitoring progress
    status_tracker: Arc<RwLock<StatusTracker>>,
    /// Discord HTTP client for sending updates
    discord_http: Option<Arc<Http>>,
    /// Approval manager for handling plan approvals
    approval_manager: Arc<ApprovalManager>,
    /// System lock to prevent concurrent updates
    system_lock: Arc<SystemLock>,
}

impl UpdateExecutor {
    /// Create a new update executor
    pub fn new(
        queue: Arc<UpdateQueue>,
        claude_client: Option<ClaudeCodeClient>,
        discord_http: Option<Arc<Http>>,
        approval_manager: Arc<ApprovalManager>,
        system_lock: Arc<SystemLock>,
    ) -> Self {
        Self {
            queue,
            claude_client,
            status_tracker: Arc::new(RwLock::new(StatusTracker)),
            discord_http,
            approval_manager,
            system_lock,
        }
    }

    /// Process a single update request through the full pipeline
    pub async fn process_request(&mut self, request: SelfUpdateRequest) -> UpdateResult {
        info!(
            "[UpdateExecutor] Processing update request: {} ({})",
            request.id, request.codename
        );

        // Try to acquire system lock
        let lock_token = match self.system_lock.try_acquire(request.id.clone()).await {
            Ok(Some(token)) => token,
            Ok(None) => {
                // Another update is in progress
                if let Some((holder_id, duration)) = self.system_lock.current_holder().await {
                    let message = format!(
                        "Another update ({}) is currently in progress (running for {} seconds). Please wait for it to complete.",
                        holder_id,
                        duration.as_secs()
                    );
                    warn!("[UpdateExecutor] {}", message);
                    self.update_discord_status(&request, &format!("🔒 {}", message)).await;
                    return self.create_failure_result(request, message);
                } else {
                    return self.create_failure_result(
                        request,
                        "System is locked by another update. Please try again later.".to_string(),
                    );
                }
            }
            Err(e) => {
                error!("[UpdateExecutor] Failed to acquire system lock: {}", e);
                return self.create_failure_result(
                    request,
                    format!("Failed to acquire system lock: {}", e),
                );
            }
        };
        
        info!("[UpdateExecutor] System lock acquired for {}", request.id);
        
        // Create a scope to ensure lock is released at the end
        let result = self.execute_update_with_lock(request, &lock_token).await;
        
        // Release the lock
        self.system_lock.release(lock_token).await;
        
        result
    }
    
    /// Execute the update while holding the lock
    async fn execute_update_with_lock(&mut self, request: SelfUpdateRequest, _lock_token: &super::LockToken) -> UpdateResult {

        // Update status to processing
        self.update_discord_status(&request, "🔧 Processing update request...")
            .await;

        // Step 1: Run preflight checks
        let preflight_result = self.run_preflight_checks(&request).await;
        if let Err(e) = preflight_result {
            error!("[UpdateExecutor] Preflight checks failed: {}", e);
            return self.create_failure_result(request, format!("Preflight checks failed: {}", e));
        }

        // Step 2: Create implementation plan
        self.update_discord_status(&request, "📋 Creating implementation plan...")
            .await;
        let planner = UpdatePlanner::new(self.claude_client.clone());
        let plan = match planner.create_plan(&request).await {
            Ok(p) => p,
            Err(e) => {
                error!("[UpdateExecutor] Failed to create plan: {}", e);
                return self.create_failure_result(request, format!("Planning failed: {}", e));
            }
        };

        // Step 3: Present plan for approval and wait
        let plan_message_id = self.present_plan_for_approval(&request, &plan).await;
        
        // Register plan for approval
        self.approval_manager
            .register_for_approval(
                plan.clone(),
                request.id.clone(),
                request.user_id,
                request.channel_id,
                plan_message_id,
            )
            .await;
        
        // Wait for approval (10 minute timeout)
        let approval_timeout = tokio::time::Duration::from_secs(600);
        let (approval_result, _) = match self
            .approval_manager
            .wait_for_approval(plan_message_id, approval_timeout)
            .await 
        {
            Ok(result) => result,
            Err(e) => {
                error!("[UpdateExecutor] Failed to wait for approval: {}", e);
                return self.create_failure_result(request, format!("Approval wait failed: {}", e));
            }
        };
        
        match approval_result {
            ApprovalResult::Approved => {
                info!("[UpdateExecutor] Plan approved, proceeding with implementation");
                self.update_discord_status(&request, "✅ Plan approved! Starting implementation...")
                    .await;
            }
            ApprovalResult::Rejected(reason) => {
                info!("[UpdateExecutor] Plan rejected: {}", reason);
                return self.create_failure_result(
                    request,
                    format!("Update cancelled: Plan rejected - {}", reason),
                );
            }
            ApprovalResult::ModifyRequested(details) => {
                info!("[UpdateExecutor] Modifications requested: {}", details);
                return self.create_failure_result(
                    request,
                    format!("Update paused: Modifications requested - {}", details),
                );
            }
            ApprovalResult::TimedOut => {
                info!("[UpdateExecutor] Approval timed out");
                return self.create_failure_result(
                    request,
                    "Update cancelled: Approval timed out (10 minutes)".to_string(),
                );
            }
        }

        // Step 4: Create git snapshot
        self.update_discord_status(&request, "📸 Creating git snapshot...")
            .await;
        let snapshot_result = self.create_snapshot(&request).await;
        let snapshot_id = match snapshot_result {
            Ok(id) => {
                info!("[UpdateExecutor] Created snapshot: {}", id);
                Some(id)
            }
            Err(e) => {
                error!("[UpdateExecutor] Failed to create snapshot: {}", e);
                return self
                    .create_failure_result(request, format!("Snapshot creation failed: {}", e));
            }
        };

        // Step 5: Execute Claude Code to implement changes
        self.update_discord_status(&request, "🤖 Implementing changes with Claude Code...")
            .await;
        let claude_result = self.execute_claude_update(&request, &plan).await;
        if let Err(e) = claude_result {
            error!("[UpdateExecutor] Claude execution failed: {}", e);
            // Rollback if we have a snapshot
            if let Some(ref id) = snapshot_id {
                self.rollback_changes(id).await;
            }
            return self
                .create_failure_result(request, format!("Claude Code execution failed: {}", e));
        }

        // Step 6: Run validation pipeline
        self.update_discord_status(&request, "✅ Validating changes...")
            .await;
        let validation_result = self.run_validation_pipeline(&request).await;
        match validation_result {
            Ok(results) => {
                info!("[UpdateExecutor] Validation passed");
                self.update_discord_status(&request, "🎉 Update completed successfully!")
                    .await;
                UpdateResult {
                    request,
                    success: true,
                    message: "Update completed successfully".to_string(),
                    snapshot_id,
                    error: None,
                    validation_results: Some(results),
                }
            }
            Err(e) => {
                error!("[UpdateExecutor] Validation failed: {}", e);
                // Rollback changes
                if let Some(ref id) = snapshot_id {
                    self.rollback_changes(id).await;
                }
                self.create_failure_result(request, format!("Validation failed: {}", e))
            }
        }
    }

    /// Process all pending requests in the queue
    pub async fn process_queue(&mut self) {
        info!("[UpdateExecutor] Starting queue processing");

        loop {
            // Get next request from queue
            let request = self.queue.next_request().await;
            let Some(mut request) = request else {
                // No requests, sleep and check again
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            };

            // Update request status
            request.status = UpdateStatus::Executing;

            // Process the request
            let result = self.process_request(request.clone()).await;

            // Update final status
            let _final_status = if result.success {
                UpdateStatus::Completed
            } else {
                UpdateStatus::Failed(result.error.clone().unwrap_or_default())
            };
            // Queue will handle status updates internally

            // Send final result to Discord
            self.send_final_result(&result).await;
        }
    }

    /// Run preflight checks
    async fn run_preflight_checks(&self, request: &SelfUpdateRequest) -> Result<()> {
        debug!(
            "[UpdateExecutor] Running preflight checks for {}",
            request.id
        );
        PreflightChecker::run_checks(request).await
    }

    /// Create a git snapshot
    async fn create_snapshot(&self, request: &SelfUpdateRequest) -> Result<String> {
        debug!("[UpdateExecutor] Creating snapshot for {}", request.id);
        GitOperations::create_snapshot(&request.codename).await
    }

    /// Execute Claude Code to implement the requested changes
    async fn execute_claude_update(&self, request: &SelfUpdateRequest, plan: &ImplementationPlan) -> Result<()> {
        debug!(
            "[UpdateExecutor] Executing Claude update for {}",
            request.id
        );

        // Check if Claude client is available
        let claude_client = match &self.claude_client {
            Some(client) => client,
            None => {
                warn!("[UpdateExecutor] No Claude client configured - cannot execute update");
                return Err(SpiralError::SystemError(
                    "Claude Code client not configured".to_string(),
                ));
            }
        };

        // Build the execution prompt from the plan
        let execution_prompt = self.build_execution_prompt(request, plan);
        
        info!(
            "[UpdateExecutor] Executing {} tasks for update {}",
            plan.tasks.len(),
            request.id
        );

        // Create a code generation request
        let code_request = crate::claude_code::CodeGenerationRequest {
            language: "rust".to_string(), // Primary language is Rust
            description: execution_prompt,
            context: std::collections::HashMap::from([
                ("task_type".to_string(), "self_update_execution".to_string()),
                ("request_id".to_string(), request.id.clone()),
                ("plan_id".to_string(), plan.plan_id.clone()),
                ("codename".to_string(), request.codename.clone()),
            ]),
            existing_code: None,
            requirements: vec![
                "Follow the implementation plan exactly".to_string(),
                "Make only the changes specified in the tasks".to_string(),
                "Ensure all changes compile and pass tests".to_string(),
                "Add appropriate error handling".to_string(),
                "Follow existing code patterns and conventions".to_string(),
            ],
            session_id: Some(format!("update-{}-execution", request.id)),
        };

        // Execute the update via Claude Code
        match claude_client.generate_code(code_request).await {
            Ok(result) => {
                info!(
                    "[UpdateExecutor] Claude Code execution completed: {}",
                    result.explanation
                );
                
                // Log files that were generated/modified
                if !result.files_to_create.is_empty() {
                    info!(
                        "[UpdateExecutor] Files to create: {:?}",
                        result.files_to_create.iter().map(|f| &f.path).collect::<Vec<_>>()
                    );
                }
                if !result.files_to_modify.is_empty() {
                    info!(
                        "[UpdateExecutor] Files to modify: {:?}",
                        result.files_to_modify.iter().map(|f| &f.path).collect::<Vec<_>>()
                    );
                }
                
                Ok(())
            }
            Err(e) => {
                error!("[UpdateExecutor] Claude Code execution failed: {}", e);
                Err(SpiralError::Agent {
                    message: format!("UpdateExecutor: Claude execution failed: {}", e),
                })
            }
        }
    }
    
    /// Build the execution prompt from the implementation plan
    fn build_execution_prompt(&self, request: &SelfUpdateRequest, plan: &ImplementationPlan) -> String {
        let task_details = plan.tasks
            .iter()
            .map(|task| {
                format!(
                    "- Task {}: {}\n  Category: {:?}\n  Complexity: {}\n  Components: {}\n  Validation: {}",
                    task.id,
                    task.description,
                    task.category,
                    task.complexity,
                    task.affected_components.join(", "),
                    task.validation_steps.join("; ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        
        format!(
            r#"You are implementing a self-update for the Spiral Core system.

## Update Request
- ID: {}
- Codename: {}
- Description: {}
- User Messages:
{}

## Approved Implementation Plan
- Plan ID: {}
- Risk Level: {:?}
- Summary: {}

## Tasks to Implement
{}

## Implementation Guidelines
1. Follow the task list in order, respecting dependencies
2. Make only the changes specified - no extra "improvements"
3. Follow project coding standards from CLAUDE.md
4. Ensure all changes compile and existing tests pass
5. If a task is unclear, implement conservatively

## Success Criteria
{}

## Important Notes
- This is a live system update - be careful and precise
- If you encounter unexpected issues, document them clearly
- Maintain backward compatibility unless explicitly changing APIs
- Add tests for new functionality where appropriate

Implement all tasks according to the plan."#,
            request.id,
            request.codename,
            request.description,
            request.combined_messages.join("\n"),
            plan.plan_id,
            plan.risk_level,
            plan.summary,
            task_details,
            plan.success_criteria.join("\n")
        )
    }

    /// Run the validation pipeline on the changes
    async fn run_validation_pipeline(&self, request: &SelfUpdateRequest) -> Result<String> {
        debug!(
            "[UpdateExecutor] Running validation pipeline for {}",
            request.id
        );

        // Create a new validation pipeline for this run
        let mut pipeline = ValidationPipeline::new();
        
        // Run the validation pipeline
        let result = pipeline.execute().await?;

        // Format results for return
        let results = format!(
            "Phase 1: {:?}\nPhase 2 Attempts: {:?}\nFinal Status: {:?}\nIterations: {}",
            result.phase1_results,
            result.phase2_attempts,
            result.final_status,
            result.pipeline_iterations
        );

        Ok(results)
    }

    /// Rollback changes to a snapshot
    async fn rollback_changes(&self, snapshot_id: &str) {
        info!("[UpdateExecutor] Rolling back to snapshot: {}", snapshot_id);
        if let Err(e) = GitOperations::rollback_to_snapshot(snapshot_id).await {
            error!("[UpdateExecutor] Rollback failed: {}", e);
        }
    }

    /// Create a failure result
    fn create_failure_result(&self, request: SelfUpdateRequest, error: String) -> UpdateResult {
        UpdateResult {
            request,
            success: false,
            message: "Update failed".to_string(),
            snapshot_id: None,
            error: Some(error),
            validation_results: None,
        }
    }

    /// Update Discord status
    async fn update_discord_status(&self, request: &SelfUpdateRequest, status: &str) {
        if let Some(ref http) = self.discord_http {
            let channel_id = ChannelId::new(request.channel_id);
            if let Err(e) = channel_id.say(http, status).await {
                warn!(
                    "[UpdateExecutor] Failed to send Discord status update: {}",
                    e
                );
            }
        }
    }

    /// Present the implementation plan for user approval
    async fn present_plan_for_approval(&self, request: &SelfUpdateRequest, plan: &ImplementationPlan) -> u64 {
        if let Some(ref http) = self.discord_http {
            let channel_id = ChannelId::new(request.channel_id);
            let mut plan_message = format_plan_for_discord(plan);
            plan_message.push_str("\n\n");
            plan_message.push_str(format_approval_instructions());
            
            match channel_id.say(http, &plan_message).await {
                Ok(msg) => {
                    info!("[UpdateExecutor] Sent plan for approval, message ID: {}", msg.id);
                    
                    // Add emoji reactions for approval
                    let message_id = msg.id;
                    
                    // Add check mark emoji for approval
                    if let Err(e) = msg.react(http, crate::discord::messages::emojis::CHECK).await {
                        warn!("[UpdateExecutor] Failed to add approval emoji: {}", e);
                    }
                    
                    // Add cross mark emoji for rejection
                    if let Err(e) = msg.react(http, crate::discord::messages::emojis::CROSS).await {
                        warn!("[UpdateExecutor] Failed to add rejection emoji: {}", e);
                    }
                    
                    // Add pencil emoji for modification request
                    if let Err(e) = msg.react(http, crate::discord::messages::emojis::PENCIL).await {
                        warn!("[UpdateExecutor] Failed to add modification emoji: {}", e);
                    }
                    
                    return message_id.get();
                }
                Err(e) => {
                    warn!("[UpdateExecutor] Failed to send plan for approval: {}", e);
                }
            }
        }
        // Return a default message ID if we couldn't send
        0
    }

    /// Send final result to Discord
    async fn send_final_result(&self, result: &UpdateResult) {
        if let Some(ref http) = self.discord_http {
            let channel_id = ChannelId::new(result.request.channel_id);

            let message = if result.success {
                format!(
                    "✅ **Update Completed Successfully**\n\n\
                    **Request**: {}\n\
                    **Codename**: {}\n\
                    **Snapshot**: {}\n\
                    **Results**: Update applied and validated successfully",
                    result.request.description,
                    result.request.codename,
                    result.snapshot_id.as_deref().unwrap_or("N/A")
                )
            } else {
                format!(
                    "❌ **Update Failed**\n\n\
                    **Request**: {}\n\
                    **Codename**: {}\n\
                    **Error**: {}\n\
                    **Status**: Changes rolled back",
                    result.request.description,
                    result.request.codename,
                    result.error.as_deref().unwrap_or("Unknown error")
                )
            };

            if let Err(e) = channel_id.say(http, message).await {
                warn!(
                    "[UpdateExecutor] Failed to send final result to Discord: {}",
                    e
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_update_executor_creation() {
        let queue = Arc::new(UpdateQueue::new());
        let approval_manager = Arc::new(ApprovalManager::new());
        let system_lock = Arc::new(SystemLock::new());
        let executor = UpdateExecutor::new(queue, None, None, approval_manager, system_lock);

        assert!(executor.discord_http.is_none());
        assert!(executor.claude_client.is_none());
    }

    #[tokio::test]
    async fn test_failure_result_creation() {
        let queue = Arc::new(UpdateQueue::new());
        let approval_manager = Arc::new(ApprovalManager::new());
        let system_lock = Arc::new(SystemLock::new());
        let executor = UpdateExecutor::new(queue, None, None, approval_manager, system_lock);

        let request = SelfUpdateRequest {
            id: "test-123".to_string(),
            codename: "test-update".to_string(),
            description: "Test update".to_string(),
            user_id: 123456,
            channel_id: 789012,
            message_id: 345678,
            combined_messages: vec!["Test message".to_string()],
            timestamp: chrono::Utc::now().to_rfc3339(),
            retry_count: 0,
            status: UpdateStatus::Queued,
        };

        let result = executor.create_failure_result(request.clone(), "Test error".to_string());

        assert!(!result.success);
        assert_eq!(result.request.id, "test-123");
        assert_eq!(result.error, Some("Test error".to_string()));
        assert!(result.snapshot_id.is_none());
        assert!(result.validation_results.is_none());
    }
}
