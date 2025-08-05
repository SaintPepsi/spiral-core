//! Update executor for orchestrating the self-update flow
//!
//! This module implements the main orchestrator that coordinates all aspects of a self-update:
//! git operations, Claude Code integration, validation pipeline, and result reporting.

use super::{
    format_plan_for_discord, GitOperations, ImplementationPlan, PreflightChecker,
    SelfUpdateRequest, StatusTracker, UpdatePlanner, UpdateQueue, UpdateStatus, ValidationPipeline,
};
use crate::{claude_code::ClaudeCodeClient, Result};
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
    /// Validation pipeline for checking changes
    pipeline: ValidationPipeline,
    /// Status tracker for monitoring progress
    status_tracker: Arc<RwLock<StatusTracker>>,
    /// Discord HTTP client for sending updates
    discord_http: Option<Arc<Http>>,
}

impl UpdateExecutor {
    /// Create a new update executor
    pub fn new(
        queue: Arc<UpdateQueue>,
        claude_client: Option<ClaudeCodeClient>,
        discord_http: Option<Arc<Http>>,
    ) -> Self {
        Self {
            queue,
            claude_client,
            pipeline: ValidationPipeline::new(),
            status_tracker: Arc::new(RwLock::new(StatusTracker)),
            discord_http,
        }
    }

    /// Process a single update request through the full pipeline
    pub async fn process_request(&mut self, request: SelfUpdateRequest) -> UpdateResult {
        info!(
            "[UpdateExecutor] Processing update request: {} ({})",
            request.id, request.codename
        );

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
        let plan = match UpdatePlanner::create_plan(&request).await {
            Ok(p) => p,
            Err(e) => {
                error!("[UpdateExecutor] Failed to create plan: {}", e);
                return self.create_failure_result(request, format!("Planning failed: {}", e));
            }
        };

        // Step 3: Present plan for approval
        self.present_plan_for_approval(&request, &plan).await;
        
        // Wait for plan approval (this will be implemented in the next task)
        // For now, we'll continue as if approved
        info!("[UpdateExecutor] Plan created, continuing with implementation (approval pending)");

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
        let claude_result = self.execute_claude_update(&request).await;
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
    async fn execute_claude_update(&self, request: &SelfUpdateRequest) -> Result<()> {
        debug!(
            "[UpdateExecutor] Executing Claude update for {}",
            request.id
        );

        // For now, we'll simulate the execution since Claude integration isn't complete
        // In a real implementation, this would:
        // 1. Create a prompt from the request
        // 2. Execute Claude with the prompt
        // 3. Wait for completion
        // 4. Return success/failure

        warn!(
            "[UpdateExecutor] Claude Code integration not yet implemented - simulating execution"
        );

        // Simulate some processing time
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // For now, always return success in simulation
        info!("[UpdateExecutor] Simulated Claude update completed");
        Ok(())
    }

    /// Run the validation pipeline on the changes
    async fn run_validation_pipeline(&mut self, request: &SelfUpdateRequest) -> Result<String> {
        debug!(
            "[UpdateExecutor] Running validation pipeline for {}",
            request.id
        );

        // Run the validation pipeline
        let result = self.pipeline.execute().await?;

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
    async fn present_plan_for_approval(&self, request: &SelfUpdateRequest, plan: &ImplementationPlan) {
        if let Some(ref http) = self.discord_http {
            let channel_id = ChannelId::new(request.channel_id);
            let plan_message = format_plan_for_discord(plan);
            
            if let Err(e) = channel_id.say(http, plan_message).await {
                warn!(
                    "[UpdateExecutor] Failed to send plan for approval: {}",
                    e
                );
            }
        }
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
        let executor = UpdateExecutor::new(queue, None, None);

        assert!(executor.discord_http.is_none());
        assert!(executor.claude_client.is_none());
    }

    #[tokio::test]
    async fn test_failure_result_creation() {
        let queue = Arc::new(UpdateQueue::new());
        let executor = UpdateExecutor::new(queue, None, None);

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
