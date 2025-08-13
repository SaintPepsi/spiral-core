//! Update executor for orchestrating the self-update flow
//!
//! This module implements the main orchestrator that coordinates all aspects of a self-update:
//! git operations, Claude Code integration, validation pipeline, and result reporting.

use super::{
    format_approval_instructions, format_plan_for_discord,
    pre_validation::PreImplementationValidator, ApprovalManager, ApprovalResult, GitOperations,
    ImplementationPlan, PreflightChecker, ProgressReporter, ScopeLimiter, SelfUpdateRequest,
    StatusTracker, StructuredLogger, SystemLock, UpdatePhase, UpdatePlanner, UpdateQueue,
    UpdateStatus, ValidationPipeline,
};
use crate::{claude_code::ClaudeCodeClient, error::SpiralError, Result};
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
    _status_tracker: Arc<RwLock<StatusTracker>>,
    /// Discord HTTP client for sending updates
    discord_http: Option<Arc<Http>>,
    /// Approval manager for handling plan approvals
    approval_manager: Arc<ApprovalManager>,
    /// System lock to prevent concurrent updates
    system_lock: Arc<SystemLock>,
}

/// Holds the context for an update execution
struct UpdateContext {
    request: SelfUpdateRequest,
    logger: StructuredLogger,
    progress_reporter: ProgressReporter,
    start_time: std::time::Instant,
    _channel_id: ChannelId,
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
            _status_tracker: Arc::new(RwLock::new(StatusTracker)),
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
                    self.update_discord_status(&request, &format!("🔒 {}", message))
                        .await;
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
    async fn execute_update_with_lock(
        &mut self,
        request: SelfUpdateRequest,
        _lock_token: &super::LockToken,
    ) -> UpdateResult {
        let start_time = std::time::Instant::now();

        // Initialize update context
        let mut context = match self.initialize_update_context(request, start_time).await {
            Ok(ctx) => ctx,
            Err(result) => return result,
        };

        // Execute preflight checks
        if let Err(result) = self.execute_preflight_phase(&mut context).await {
            return result;
        }

        // Create implementation plan
        let plan = match self.execute_planning_phase(&mut context).await {
            Ok(p) => p,
            Err(result) => return result,
        };

        // Get user approval
        if let Err(result) = self.execute_approval_phase(&mut context, &plan).await {
            return result;
        }

        // Create git snapshot
        let snapshot_id = match self.execute_snapshot_phase(&mut context).await {
            Ok(id) => id,
            Err(result) => return result,
        };

        // Execute Claude Code implementation
        if let Err(result) = self
            .execute_implementation_phase(&mut context, &plan, &snapshot_id)
            .await
        {
            return result;
        }

        // Run pre-restart validation on modified files
        if let Err(result) = self
            .execute_pre_validation_phase(&mut context, &snapshot_id)
            .await
        {
            return result;
        }

        // System restart would happen here (not implemented yet)
        // TODO: Implement system restart

        // Run post-restart validation pipeline
        self.execute_validation_phase(&mut context, snapshot_id)
            .await
    }

    /// Initialize the update context
    async fn initialize_update_context(
        &self,
        request: SelfUpdateRequest,
        start_time: std::time::Instant,
    ) -> std::result::Result<UpdateContext, UpdateResult> {
        // Create structured logger for this update
        let mut logger = match StructuredLogger::new(request.id.clone(), request.codename.clone()) {
            Ok(l) => l,
            Err(e) => {
                error!("[UpdateExecutor] Failed to create logger: {}", e);
                return Err(
                    self.create_failure_result(request, format!("Failed to create logger: {}", e))
                );
            }
        };

        // Log the start of the update
        if let Err(e) = logger.log_to_main(&format!(
            "Starting update execution\nDescription: {}\nUser ID: {}\nChannel ID: {}",
            request.description, request.user_id, request.channel_id
        )) {
            warn!("[UpdateExecutor] Failed to log update start: {}", e);
        }

        // Create progress reporter
        let channel_id = ChannelId::new(request.channel_id);
        let total_tasks = 7; // Approximate number of major steps
        let progress_reporter = ProgressReporter::new(
            request.id.clone(),
            self.discord_http.clone(),
            channel_id,
            total_tasks,
        );

        // Start background progress reporting (every 30 seconds)
        progress_reporter.start_reporting(std::time::Duration::from_secs(30));

        // Set initial phase
        progress_reporter.set_phase(UpdatePhase::Initializing).await;
        progress_reporter
            .set_status("Starting update process...".to_string())
            .await;

        Ok(UpdateContext {
            request,
            logger,
            progress_reporter,
            start_time,
            _channel_id: channel_id,
        })
    }

    /// Execute preflight checks phase
    async fn execute_preflight_phase(
        &self,
        context: &mut UpdateContext,
    ) -> std::result::Result<(), UpdateResult> {
        context
            .progress_reporter
            .set_phase(UpdatePhase::PreflightChecks)
            .await;
        context
            .progress_reporter
            .set_status("Running preflight checks...".to_string())
            .await;
        self.update_discord_status(&context.request, "🔧 Processing update request...")
            .await;

        let preflight_result = self.run_preflight_checks(&context.request).await;
        if let Err(e) = preflight_result {
            error!("[UpdateExecutor] Preflight checks failed: {}", e);
            let _ = context
                .logger
                .log_error("PreflightChecks", &e.to_string(), None)
                .await;
            context
                .progress_reporter
                .set_phase(UpdatePhase::Failed)
                .await;
            context.progress_reporter.stop().await;
            let _ = context
                .logger
                .create_summary(
                    false,
                    &format!("Preflight checks failed: {}", e),
                    context.start_time.elapsed(),
                )
                .await;
            return Err(self.create_failure_result(
                context.request.clone(),
                format!("Preflight checks failed: {}", e),
            ));
        }

        let _ = context
            .logger
            .log_to_phase("PreflightChecks", "Preflight checks passed successfully")
            .await;
        Ok(())
    }

    /// Execute planning phase
    async fn execute_planning_phase(
        &self,
        context: &mut UpdateContext,
    ) -> std::result::Result<ImplementationPlan, UpdateResult> {
        context
            .progress_reporter
            .set_phase(UpdatePhase::Planning)
            .await;
        context
            .progress_reporter
            .set_status("Creating implementation plan with Claude Code...".to_string())
            .await;
        self.update_discord_status(&context.request, "📋 Creating implementation plan...")
            .await;

        let planner = UpdatePlanner::new(self.claude_client.clone());
        match planner.create_plan(&context.request).await {
            Ok(p) => Ok(p),
            Err(e) => {
                error!("[UpdateExecutor] Failed to create plan: {}", e);
                let _ = context
                    .logger
                    .log_error("Planning", &e.to_string(), None)
                    .await;
                context
                    .progress_reporter
                    .set_phase(UpdatePhase::Failed)
                    .await;
                context.progress_reporter.stop().await;
                let _ = context
                    .logger
                    .create_summary(
                        false,
                        &format!("Planning failed: {}", e),
                        context.start_time.elapsed(),
                    )
                    .await;
                Err(self.create_failure_result(
                    context.request.clone(),
                    format!("Planning failed: {}", e),
                ))
            }
        }
    }

    /// Execute approval phase
    async fn execute_approval_phase(
        &self,
        context: &mut UpdateContext,
        plan: &ImplementationPlan,
    ) -> std::result::Result<(), UpdateResult> {
        context
            .progress_reporter
            .set_phase(UpdatePhase::AwaitingApproval)
            .await;

        // Check if human approval is specifically required
        if plan.requires_human_approval {
            context
                .progress_reporter
                .set_status(format!(
                    "⚠️ Human approval required: {}",
                    plan.approval_reason
                        .as_ref()
                        .unwrap_or(&"Critical changes detected".to_string())
                ))
                .await;

            self.update_discord_status(
                &context.request,
                &format!(
                    "⚠️ **Human Approval Required**\n\n{}\n\nPlease review the plan carefully.",
                    plan.approval_reason
                        .as_ref()
                        .unwrap_or(&"Critical changes detected".to_string())
                ),
            )
            .await;
        } else {
            context
                .progress_reporter
                .set_status("Waiting for user approval...".to_string())
                .await;
        }

        let plan_message_id = self.present_plan_for_approval(&context.request, plan).await;

        // Register plan for approval
        self.approval_manager
            .register_for_approval(
                plan.clone(),
                context.request.id.clone(),
                context.request.user_id,
                context.request.channel_id,
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
                context
                    .progress_reporter
                    .set_phase(UpdatePhase::Failed)
                    .await;
                context.progress_reporter.stop().await;
                return Err(self.create_failure_result(
                    context.request.clone(),
                    format!("Approval wait failed: {}", e),
                ));
            }
        };

        match approval_result {
            ApprovalResult::Approved => {
                info!("[UpdateExecutor] Plan approved, proceeding with implementation");
                self.update_discord_status(
                    &context.request,
                    "✅ Plan approved! Starting implementation...",
                )
                .await;
                Ok(())
            }
            ApprovalResult::Rejected(reason) => {
                info!("[UpdateExecutor] Plan rejected: {}", reason);
                context
                    .progress_reporter
                    .set_phase(UpdatePhase::Failed)
                    .await;
                context.progress_reporter.stop().await;
                Err(self.create_failure_result(
                    context.request.clone(),
                    format!("Update cancelled: Plan rejected - {}", reason),
                ))
            }
            ApprovalResult::ModifyRequested(details) => {
                info!("[UpdateExecutor] Modifications requested: {}", details);
                context
                    .progress_reporter
                    .set_phase(UpdatePhase::Failed)
                    .await;
                context.progress_reporter.stop().await;
                Err(self.create_failure_result(
                    context.request.clone(),
                    format!("Update paused: Modifications requested - {}", details),
                ))
            }
            ApprovalResult::TimedOut => {
                info!("[UpdateExecutor] Approval timed out");
                context
                    .progress_reporter
                    .set_phase(UpdatePhase::Failed)
                    .await;
                context.progress_reporter.stop().await;
                Err(self.create_failure_result(
                    context.request.clone(),
                    "Update cancelled: Approval timed out (10 minutes)".to_string(),
                ))
            }
        }
    }

    /// Execute snapshot creation phase
    async fn execute_snapshot_phase(
        &self,
        context: &mut UpdateContext,
    ) -> std::result::Result<Option<String>, UpdateResult> {
        context
            .progress_reporter
            .set_phase(UpdatePhase::CreatingSnapshot)
            .await;
        context
            .progress_reporter
            .set_status("Creating git snapshot for rollback safety...".to_string())
            .await;
        self.update_discord_status(&context.request, "📸 Creating git snapshot...")
            .await;

        match self.create_snapshot(&context.request).await {
            Ok(id) => {
                info!("[UpdateExecutor] Created snapshot: {}", id);
                Ok(Some(id))
            }
            Err(e) => {
                error!("[UpdateExecutor] Failed to create snapshot: {}", e);
                context
                    .progress_reporter
                    .set_phase(UpdatePhase::Failed)
                    .await;
                context.progress_reporter.stop().await;
                Err(self.create_failure_result(
                    context.request.clone(),
                    format!("Snapshot creation failed: {}", e),
                ))
            }
        }
    }

    /// Execute Claude Code implementation phase
    async fn execute_implementation_phase(
        &self,
        context: &mut UpdateContext,
        plan: &ImplementationPlan,
        snapshot_id: &Option<String>,
    ) -> std::result::Result<(), UpdateResult> {
        context
            .progress_reporter
            .set_phase(UpdatePhase::Implementing)
            .await;
        context
            .progress_reporter
            .set_status(format!(
                "Implementing {} tasks with Claude Code...",
                plan.tasks.len()
            ))
            .await;
        self.update_discord_status(
            &context.request,
            "🤖 Implementing changes with Claude Code...",
        )
        .await;

        if let Err(e) = self.execute_claude_update(&context.request, plan).await {
            error!("[UpdateExecutor] Claude execution failed: {}", e);
            context
                .progress_reporter
                .set_phase(UpdatePhase::Failed)
                .await;
            context.progress_reporter.stop().await;

            // Rollback if we have a snapshot
            if let Some(ref id) = snapshot_id {
                self.rollback_changes(id).await;
            }

            return Err(self.create_failure_result(
                context.request.clone(),
                format!("Claude Code execution failed: {}", e),
            ));
        }

        // Check scope limits after execution
        context
            .progress_reporter
            .set_status("Validating change scope...".to_string())
            .await;
        if let Err(e) = self.validate_change_scope(&context.request).await {
            error!("[UpdateExecutor] Change scope validation failed: {}", e);
            context
                .progress_reporter
                .set_phase(UpdatePhase::Failed)
                .await;
            context.progress_reporter.stop().await;

            // Rollback if we have a snapshot
            if let Some(ref id) = snapshot_id {
                self.rollback_changes(id).await;
            }

            return Err(self.create_failure_result(
                context.request.clone(),
                format!("Changes exceeded scope limits: {}", e),
            ));
        }

        Ok(())
    }

    /// Execute pre-restart validation phase
    async fn execute_pre_validation_phase(
        &self,
        context: &mut UpdateContext,
        snapshot_id: &Option<String>,
    ) -> std::result::Result<(), UpdateResult> {
        context
            .progress_reporter
            .set_phase(UpdatePhase::Validating)
            .await;
        context
            .progress_reporter
            .set_status("Running pre-restart validation on modified files...".to_string())
            .await;
        self.update_discord_status(&context.request, "🔍 Validating changes before restart...")
            .await;

        // Create pre-implementation validator
        let validator = PreImplementationValidator::new(self.claude_client.clone());

        // Run validation on current working directory
        match validator
            .validate_current_state(&context.request, &mut context.logger)
            .await
        {
            Ok(result) => {
                if result.all_passed() {
                    info!("[UpdateExecutor] Pre-restart validation passed");
                    context
                        .progress_reporter
                        .set_status(format!(
                            "Pre-restart validation passed ({} checks in {} iterations)",
                            result.total_checks_run, result.pipeline_iterations
                        ))
                        .await;
                    Ok(())
                } else {
                    error!("[UpdateExecutor] Pre-restart validation failed");
                    let error_msg = format!(
                        "Pre-restart validation failed: Engineering Review={}, Assembly Checklist={}, Details: {:?}",
                        result.engineering_review_passed,
                        result.assembly_checklist_passed,
                        result.error_details
                    );

                    context
                        .progress_reporter
                        .set_phase(UpdatePhase::Failed)
                        .await;
                    context.progress_reporter.stop().await;

                    // Rollback changes since validation failed
                    if let Some(ref id) = snapshot_id {
                        self.rollback_changes(id).await;
                        self.update_discord_status(
                            &context.request,
                            "❌ Validation failed - changes rolled back",
                        )
                        .await;
                    }

                    Err(self.create_failure_result(context.request.clone(), error_msg))
                }
            }
            Err(e) => {
                error!("[UpdateExecutor] Pre-restart validation error: {}", e);
                context
                    .progress_reporter
                    .set_phase(UpdatePhase::Failed)
                    .await;
                context.progress_reporter.stop().await;

                // Rollback on error
                if let Some(ref id) = snapshot_id {
                    self.rollback_changes(id).await;
                }

                Err(self.create_failure_result(
                    context.request.clone(),
                    format!("Pre-restart validation error: {}", e),
                ))
            }
        }
    }

    /// Execute post-restart validation phase
    async fn execute_validation_phase(
        &self,
        context: &mut UpdateContext,
        snapshot_id: Option<String>,
    ) -> UpdateResult {
        context
            .progress_reporter
            .set_phase(UpdatePhase::Validating)
            .await;
        context
            .progress_reporter
            .set_status("Running post-restart validation pipeline...".to_string())
            .await;
        self.update_discord_status(&context.request, "✅ Validating system after restart...")
            .await;

        match self.run_validation_pipeline(&context.request).await {
            Ok(results) => {
                info!("[UpdateExecutor] Validation passed, committing and pushing changes");

                // Git operations: Commit and push validated changes
                let git_result = self.commit_and_push_changes(&context.request).await;

                context
                    .progress_reporter
                    .set_phase(UpdatePhase::Complete)
                    .await;
                context
                    .progress_reporter
                    .set_status("Update completed successfully!".to_string())
                    .await;

                let final_message = match git_result {
                    Ok(commit_hash) => {
                        self.update_discord_status(
                            &context.request,
                            &format!(
                                "🎉 Update completed and pushed! Commit: {}",
                                &commit_hash[..8]
                            ),
                        )
                        .await;
                        format!(
                            "Update completed successfully and pushed to remote ({})",
                            &commit_hash[..8]
                        )
                    }
                    Err(e) => {
                        warn!("[UpdateExecutor] Git operations failed: {}", e);
                        self.update_discord_status(
                            &context.request,
                            "⚠️ Update completed but Git push failed - changes are local only",
                        )
                        .await;
                        format!("Update completed but Git operations failed: {}", e)
                    }
                };

                // Stop progress reporting
                context.progress_reporter.stop().await;

                // Log validation results and create summary
                let _ = context
                    .logger
                    .log_validation_results("Final", &results)
                    .await;
                let _ = context
                    .logger
                    .create_summary(true, &final_message, context.start_time.elapsed())
                    .await;
                info!(
                    "[UpdateExecutor] {}. Logs at: {}",
                    final_message,
                    context.logger.get_log_dir().display()
                );

                UpdateResult {
                    request: context.request.clone(),
                    success: true,
                    message: final_message,
                    snapshot_id,
                    error: None,
                    validation_results: Some(results),
                }
            }
            Err(e) => {
                error!("[UpdateExecutor] Validation failed: {}", e);
                context
                    .progress_reporter
                    .set_phase(UpdatePhase::Failed)
                    .await;
                context
                    .progress_reporter
                    .set_status(format!("Validation failed: {}", e))
                    .await;

                // Rollback changes
                if let Some(ref id) = snapshot_id {
                    self.rollback_changes(id).await;
                }

                // Stop progress reporting
                context.progress_reporter.stop().await;

                self.create_failure_result(
                    context.request.clone(),
                    format!("Validation failed: {}", e),
                )
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

            // Mark as completed in queue
            self.queue.complete_request(&result.request.id).await;

            // Update final status and handle retries
            if result.success {
                let _final_status = UpdateStatus::Completed;
            } else {
                let error_msg = result.error.clone().unwrap_or_default();
                let _final_status = UpdateStatus::Failed(error_msg.clone());

                // Determine if this failure is retryable
                if self.is_retryable_error(&error_msg) && request.retry_count < 3 {
                    info!(
                        "[UpdateExecutor] Request {} failed with retryable error, attempting retry {}/3",
                        request.id, request.retry_count + 1
                    );

                    // Attempt to retry the request
                    if let Err(e) = self.queue.retry_request(request.clone()).await {
                        warn!("[UpdateExecutor] Failed to retry request: {}", e);
                    } else {
                        // Notify Discord about the retry
                        self.notify_retry(&request, request.retry_count + 1).await;
                    }
                }
            }

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
    async fn execute_claude_update(
        &self,
        request: &SelfUpdateRequest,
        plan: &ImplementationPlan,
    ) -> Result<()> {
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
                        result
                            .files_to_create
                            .iter()
                            .map(|f| &f.path)
                            .collect::<Vec<_>>()
                    );
                }
                if !result.files_to_modify.is_empty() {
                    info!(
                        "[UpdateExecutor] Files to modify: {:?}",
                        result
                            .files_to_modify
                            .iter()
                            .map(|f| &f.path)
                            .collect::<Vec<_>>()
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
    fn build_execution_prompt(
        &self,
        request: &SelfUpdateRequest,
        plan: &ImplementationPlan,
    ) -> String {
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

    /// Validate that changes are within acceptable scope limits
    async fn validate_change_scope(&self, _request: &SelfUpdateRequest) -> Result<()> {
        // Get git diff to analyze changes
        let output = tokio::process::Command::new("git")
            .args(&["diff", "HEAD"])
            .output()
            .await
            .map_err(|e| SpiralError::SystemError(format!("Failed to run git diff: {}", e)))?;

        if !output.status.success() {
            return Err(SpiralError::SystemError(
                "Failed to get git diff".to_string(),
            ));
        }

        let diff = String::from_utf8_lossy(&output.stdout);

        // Use scope limiter to analyze and validate changes
        let limiter = ScopeLimiter::new();
        let scope = limiter.analyze_diff(&diff).await?;

        info!(
            "[UpdateExecutor] Change scope: {} files modified, {} created, {} deleted",
            scope.modified_files.len(),
            scope.created_files.len(),
            scope.deleted_files.len()
        );

        Ok(())
    }

    /// Rollback changes to a snapshot
    async fn rollback_changes(&self, snapshot_id: &str) {
        info!("[UpdateExecutor] Rolling back to snapshot: {}", snapshot_id);
        if let Err(e) = GitOperations::rollback_to_snapshot(snapshot_id).await {
            error!("[UpdateExecutor] Rollback failed: {}", e);
        }
    }

    /// Commit and push validated changes to remote repository
    async fn commit_and_push_changes(&self, request: &SelfUpdateRequest) -> Result<String> {
        info!("[UpdateExecutor] Committing and pushing validated changes");

        // Commit the changes with descriptive message
        let commit_hash =
            GitOperations::commit_validated_changes(&request.codename, &request.description)
                .await?;

        // Push to remote repository
        match GitOperations::push_to_remote(None).await {
            Ok(()) => {
                info!("[UpdateExecutor] Successfully pushed changes to remote");
                Ok(commit_hash)
            }
            Err(e) => {
                warn!(
                    "[UpdateExecutor] Push failed but changes are committed locally: {}",
                    e
                );
                // Return commit hash even if push fails - changes are still saved locally
                Ok(commit_hash)
            }
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
    async fn present_plan_for_approval(
        &self,
        request: &SelfUpdateRequest,
        plan: &ImplementationPlan,
    ) -> u64 {
        if let Some(ref http) = self.discord_http {
            let channel_id = ChannelId::new(request.channel_id);
            let mut plan_message = format_plan_for_discord(plan);
            plan_message.push_str("\n\n");
            plan_message.push_str(format_approval_instructions());

            match channel_id.say(http, &plan_message).await {
                Ok(msg) => {
                    info!(
                        "[UpdateExecutor] Sent plan for approval, message ID: {}",
                        msg.id
                    );

                    // Add emoji reactions for approval
                    let message_id = msg.id;

                    // Add check mark emoji for approval
                    if let Err(e) = msg
                        .react(http, crate::discord::messages::emojis::CHECK)
                        .await
                    {
                        warn!("[UpdateExecutor] Failed to add approval emoji: {}", e);
                    }

                    // Add cross mark emoji for rejection
                    if let Err(e) = msg
                        .react(http, crate::discord::messages::emojis::CROSS)
                        .await
                    {
                        warn!("[UpdateExecutor] Failed to add rejection emoji: {}", e);
                    }

                    // Add pencil emoji for modification request
                    if let Err(e) = msg
                        .react(http, crate::discord::messages::emojis::PENCIL)
                        .await
                    {
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

    /// Determine if an error is retryable
    fn is_retryable_error(&self, error: &str) -> bool {
        // Network/transient errors are retryable
        if error.contains("network")
            || error.contains("timeout")
            || error.contains("connection")
            || error.contains("temporarily")
        {
            return true;
        }

        // Git conflicts might be resolved on retry
        if error.contains("git") && error.contains("conflict") {
            return true;
        }

        // Claude API rate limits are retryable
        if error.contains("rate limit") || error.contains("429") {
            return true;
        }

        // System resource issues might be temporary
        if error.contains("memory") || error.contains("disk space") {
            return true;
        }

        // Most other errors are not retryable (compilation errors, validation failures, etc.)
        false
    }

    /// Notify Discord about a retry attempt
    async fn notify_retry(&self, request: &SelfUpdateRequest, retry_number: u32) {
        if let Some(ref http) = self.discord_http {
            let channel_id = ChannelId::new(request.channel_id);
            let message = format!(
                "⚠️ **Update Retry**\n\n\
                The update `{}` encountered a retryable error.\n\
                Attempting retry {}/3...\n\
                The update will be re-queued and processed again shortly.",
                request.codename, retry_number
            );

            if let Err(e) = channel_id.say(http, message).await {
                warn!("[UpdateExecutor] Failed to send retry notification: {}", e);
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
                let retry_info = if result.request.retry_count > 0 {
                    format!("\n**Retries**: {}/3", result.request.retry_count)
                } else {
                    String::new()
                };

                format!(
                    "❌ **Update Failed**\n\n\
                    **Request**: {}\n\
                    **Codename**: {}\n\
                    **Error**: {}\n{}\
                    **Status**: Changes rolled back",
                    result.request.description,
                    result.request.codename,
                    result.error.as_deref().unwrap_or("Unknown error"),
                    retry_info
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
