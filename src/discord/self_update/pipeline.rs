//! Two-phase validation pipeline implementation
//!
//! Implements the validation pipeline described in SELF_UPDATE_PIPELINE_IMPROVEMENT.md:
//! - Phase 1: Advanced Quality Assurance (AQA)
//! - Phase 2: Core Rust Compliance Checks (CRCC)
//! - Pipeline looping: ANY Phase 2 retry triggers return to Phase 1
//! - Maximum 3 complete pipeline iterations

use crate::claude_code::{ClaudeCodeClient, CodeGenerationRequest};
use crate::config::ClaudeCodeConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{error, info, warn};

/// Maximum number of complete pipeline iterations allowed
const MAX_PIPELINE_ITERATIONS: u8 = 3;

/// Base timeout for individual agent operations (2 minutes)
const BASE_AGENT_TIMEOUT: Duration = Duration::from_secs(120);

/// Maximum timeout after exponential backoff (10 minutes)
const MAX_AGENT_TIMEOUT: Duration = Duration::from_secs(600);

/// Timeout multiplier for exponential backoff
const TIMEOUT_MULTIPLIER: f64 = 1.5;

/// Pipeline execution context passed between phases and to analysis agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineContext {
    /// Number of complete pipeline iterations (1-3)
    pub pipeline_iterations: u8,
    /// Total duration of pipeline execution in milliseconds
    pub total_duration_ms: u64,
    /// Final status of the pipeline run
    pub final_status: PipelineStatus,
    /// Results from Phase 1 checks (same for all iterations)
    pub phase1_results: Phase1Results,
    /// All Phase 2 attempts (one per pipeline iteration)
    pub phase2_attempts: Vec<Phase2Attempt>,
    /// Files modified during validation
    pub files_modified: Vec<String>,
    /// Changes applied during validation
    pub changes_applied: Vec<ChangeRecord>,
    /// Critical errors encountered
    pub critical_errors: Vec<String>,
    /// Non-blocking warnings
    pub warnings: Vec<String>,
    /// Patterns identified during execution
    pub patterns: ExecutionPatterns,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PipelineStatus {
    Success,
    SuccessWithRetries,
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase1Results {
    pub code_review: CheckResult,
    pub testing: CheckResult,
    pub security: CheckResult,
    pub integration: CheckResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub passed: bool,
    pub findings: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase2Attempt {
    pub iteration: u8,
    pub checks: Phase2Checks,
    pub triggered_loop: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase2Checks {
    pub compilation: ComplianceCheck,
    pub tests: ComplianceCheck,
    pub formatting: ComplianceCheck,
    pub clippy: ComplianceCheck,
    pub docs: ComplianceCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub passed: bool,
    pub retries: u8,
    pub errors: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeRecord {
    pub phase: String,
    pub description: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionPatterns {
    pub consistent_failures: Option<Vec<String>>,
    pub flakey_checks: Option<Vec<String>>,
    pub performance_bottlenecks: Option<Vec<String>>,
}

/// Response from Claude validation agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeValidationResponse {
    pub explanation: String,
    pub success: bool,
}

/// Main validation pipeline coordinator
pub struct ValidationPipeline {
    context: PipelineContext,
    start_time: Instant,
    claude_client: Option<ClaudeCodeClient>,
    snapshot_id: Option<String>,
}

impl PipelineContext {
    /// Serialize context to JSON for passing to Claude agents
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            crate::error::SpiralError::SystemError(format!("Failed to serialize context: {}", e))
        })
    }
}

impl ValidationPipeline {
    /// Create a new validation pipeline instance
    pub fn new() -> Self {
        Self {
            context: PipelineContext {
                pipeline_iterations: 0,
                total_duration_ms: 0,
                final_status: PipelineStatus::Failure,
                phase1_results: Phase1Results {
                    code_review: CheckResult {
                        passed: false,
                        findings: vec![],
                        duration_ms: 0,
                    },
                    testing: CheckResult {
                        passed: false,
                        findings: vec![],
                        duration_ms: 0,
                    },
                    security: CheckResult {
                        passed: false,
                        findings: vec![],
                        duration_ms: 0,
                    },
                    integration: CheckResult {
                        passed: false,
                        findings: vec![],
                        duration_ms: 0,
                    },
                },
                phase2_attempts: vec![],
                files_modified: vec![],
                changes_applied: vec![],
                critical_errors: vec![],
                warnings: vec![],
                patterns: ExecutionPatterns::default(),
            },
            start_time: Instant::now(),
            claude_client: None,
            snapshot_id: None,
        }
    }

    /// Create a new validation pipeline with Claude client
    pub async fn with_claude_client(config: ClaudeCodeConfig) -> Result<Self> {
        let claude_client = ClaudeCodeClient::new(config).await?;

        let mut pipeline = Self::new();
        pipeline.claude_client = Some(claude_client);

        Ok(pipeline)
    }

    /// Create a Git snapshot before making changes
    async fn create_validation_snapshot(&mut self) -> Result<()> {
        let snapshot_id = format!("validation-snapshot-{}", chrono::Utc::now().timestamp());

        info!("[ValidationPipeline] Creating snapshot: {}", snapshot_id);

        // Create a git stash with the snapshot ID
        let output = Command::new("git")
            .args(["stash", "push", "-m", &snapshot_id, "--include-untracked"])
            .output()
            .map_err(|e| {
                crate::error::SpiralError::SystemError(format!("Failed to create snapshot: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("[ValidationPipeline] Failed to create snapshot: {}", stderr);
            // Continue anyway - validation can proceed without snapshot
        } else {
            self.snapshot_id = Some(snapshot_id);
            info!("[ValidationPipeline] Snapshot created successfully");
        }

        Ok(())
    }

    /// Rollback to the snapshot if validation fails
    async fn rollback_to_snapshot(&self) -> Result<()> {
        if let Some(snapshot_id) = &self.snapshot_id {
            info!(
                "[ValidationPipeline] Rolling back to snapshot: {}",
                snapshot_id
            );

            // First, find the stash index
            let stash_list = Command::new("git")
                .args(["stash", "list"])
                .output()
                .map_err(|e| {
                    crate::error::SpiralError::SystemError(format!("Failed to list stashes: {}", e))
                })?;

            let stash_output = String::from_utf8_lossy(&stash_list.stdout);
            let stash_index = stash_output
                .lines()
                .position(|line| line.contains(snapshot_id))
                .ok_or_else(|| {
                    crate::error::SpiralError::SystemError(
                        "Snapshot not found in stash list".to_string(),
                    )
                })?;

            // Apply the stash
            let output = Command::new("git")
                .args(["stash", "pop", &format!("stash@{{{}}}", stash_index)])
                .output()
                .map_err(|e| {
                    crate::error::SpiralError::SystemError(format!("Failed to apply stash: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("[ValidationPipeline] Failed to rollback: {}", stderr);
                return Err(crate::error::SpiralError::SystemError(format!(
                    "Rollback failed: {}",
                    stderr
                )));
            }

            info!("[ValidationPipeline] Rollback completed successfully");
        } else {
            warn!("[ValidationPipeline] No snapshot to rollback to");
        }

        Ok(())
    }

    /// Clean up snapshot after successful validation
    async fn cleanup_snapshot(&self) -> Result<()> {
        if let Some(snapshot_id) = &self.snapshot_id {
            info!("[ValidationPipeline] Cleaning up snapshot: {}", snapshot_id);

            // Drop the stash
            let stash_list = Command::new("git")
                .args(["stash", "list"])
                .output()
                .map_err(|e| {
                    crate::error::SpiralError::SystemError(format!("Failed to list stashes: {}", e))
                })?;

            let stash_output = String::from_utf8_lossy(&stash_list.stdout);
            if let Some(index) = stash_output
                .lines()
                .position(|line| line.contains(snapshot_id))
            {
                Command::new("git")
                    .args(["stash", "drop", &format!("stash@{{{}}}", index)])
                    .output()
                    .map_err(|e| {
                        crate::error::SpiralError::SystemError(format!(
                            "Failed to drop stash: {}",
                            e
                        ))
                    })?;
            }
        }
        Ok(())
    }

    /// Execute the complete validation pipeline
    pub async fn execute(&mut self) -> Result<PipelineContext> {
        info!("[ValidationPipeline] ====== STARTING TWO-PHASE VALIDATION PIPELINE ======");
        info!(
            "[ValidationPipeline] Max iterations: {}",
            MAX_PIPELINE_ITERATIONS
        );

        // Create snapshot before making any changes
        if let Err(e) = self.create_validation_snapshot().await {
            warn!(
                "[ValidationPipeline] Failed to create snapshot: {} - continuing anyway",
                e
            );
        }

        // Main pipeline loop
        while self.context.pipeline_iterations < MAX_PIPELINE_ITERATIONS {
            self.context.pipeline_iterations += 1;
            info!(
                "[ValidationPipeline] ┌─── PIPELINE ITERATION {} / {} ───┐",
                self.context.pipeline_iterations, MAX_PIPELINE_ITERATIONS
            );

            // Phase 1: Advanced Quality Assurance
            info!("[ValidationPipeline] ├── Phase 1: Advanced Quality Assurance");
            self.execute_phase1().await?;

            // Check if Phase 1 failed critically
            if self.has_critical_phase1_failures() {
                error!("[ValidationPipeline] ├── CRITICAL: Phase 1 failures detected");
                self.context.final_status = PipelineStatus::Failure;
                self.add_critical_error(
                    "Phase 1 critical failures prevented Phase 2 execution".to_string(),
                );
                break;
            }

            // Phase 2: Core Rust Compliance Checks
            info!("[ValidationPipeline] ├── Phase 2: Core Rust Compliance Checks");
            let phase2_passed = self.execute_phase2().await?;

            if phase2_passed {
                // Success!
                self.context.final_status = if self.context.pipeline_iterations == 1 {
                    info!("[ValidationPipeline] └── SUCCESS: All checks passed on first attempt!");
                    PipelineStatus::Success
                } else {
                    info!(
                        "[ValidationPipeline] └── SUCCESS: All checks passed after {} iterations",
                        self.context.pipeline_iterations
                    );
                    PipelineStatus::SuccessWithRetries
                };
                break;
            }

            // Phase 2 failed - loop back to Phase 1 if we have iterations left
            if self.context.pipeline_iterations >= MAX_PIPELINE_ITERATIONS {
                error!("[ValidationPipeline] └── FAILURE: Maximum iterations exhausted");
                self.context.final_status = PipelineStatus::Failure;
                self.add_critical_error(format!(
                    "Pipeline failed after {} iterations",
                    MAX_PIPELINE_ITERATIONS
                ));
                break;
            } else {
                warn!("[ValidationPipeline] └── Phase 2 failed, looping back to Phase 1");
            }
        }

        // Calculate total duration
        self.context.total_duration_ms = self.start_time.elapsed().as_millis() as u64;
        info!(
            "[ValidationPipeline] Total execution time: {}ms",
            self.context.total_duration_ms
        );

        // Analyze patterns
        info!("[ValidationPipeline] Analyzing execution patterns...");
        self.analyze_patterns();

        // Run appropriate analysis agent based on outcome
        info!(
            "[ValidationPipeline] Running analysis agent for outcome: {:?}",
            self.context.final_status
        );
        self.run_analysis_agent().await?;

        // Final summary
        info!("[ValidationPipeline] ====== PIPELINE SUMMARY ======");
        info!(
            "[ValidationPipeline] Status: {:?}",
            self.context.final_status
        );
        info!(
            "[ValidationPipeline] Iterations: {}",
            self.context.pipeline_iterations
        );
        info!(
            "[ValidationPipeline] Duration: {}ms",
            self.context.total_duration_ms
        );
        info!(
            "[ValidationPipeline] Files modified: {}",
            self.context.files_modified.len()
        );
        info!(
            "[ValidationPipeline] Changes applied: {}",
            self.context.changes_applied.len()
        );
        info!(
            "[ValidationPipeline] Critical errors: {}",
            self.context.critical_errors.len()
        );
        info!(
            "[ValidationPipeline] Warnings: {}",
            self.context.warnings.len()
        );

        // Handle final result - rollback on failure, cleanup on success
        match self.context.final_status {
            PipelineStatus::Success | PipelineStatus::SuccessWithRetries => {
                info!("[ValidationPipeline] Validation successful - keeping changes");
                if let Err(e) = self.cleanup_snapshot().await {
                    warn!("[ValidationPipeline] Failed to cleanup snapshot: {}", e);
                }
            }
            PipelineStatus::Failure => {
                error!("[ValidationPipeline] Validation failed - rolling back changes");
                if let Err(e) = self.rollback_to_snapshot().await {
                    error!("[ValidationPipeline] CRITICAL: Rollback failed: {}", e);
                    self.add_critical_error(format!("Rollback failed: {}", e));
                }
            }
        }

        Ok(self.context.clone())
    }

    /// Execute Phase 1: Advanced Quality Assurance
    async fn execute_phase1(&mut self) -> Result<()> {
        info!("[ValidationPipeline] Executing Phase 1: Advanced Quality Assurance");

        // Only run Phase 1 on first iteration (results apply to all iterations)
        if self.context.pipeline_iterations > 1 {
            info!("[ValidationPipeline] Skipping Phase 1 (already completed)");
            return Ok(());
        }

        // Run each Phase 1 check
        self.context.phase1_results.code_review = self.run_code_review().await?;
        self.context.phase1_results.testing = self.run_comprehensive_testing().await?;
        self.context.phase1_results.security = self.run_security_audit().await?;
        self.context.phase1_results.integration = self.run_system_integration().await?;

        Ok(())
    }

    /// Execute Phase 2: Core Rust Compliance Checks
    async fn execute_phase2(&mut self) -> Result<bool> {
        info!("[Phase2] ┌─── CORE RUST COMPLIANCE CHECKS ───┐");

        let mut phase2_attempt = Phase2Attempt {
            iteration: self.context.pipeline_iterations,
            checks: Phase2Checks {
                compilation: ComplianceCheck {
                    passed: false,
                    retries: 0,
                    errors: None,
                },
                tests: ComplianceCheck {
                    passed: false,
                    retries: 0,
                    errors: None,
                },
                formatting: ComplianceCheck {
                    passed: false,
                    retries: 0,
                    errors: None,
                },
                clippy: ComplianceCheck {
                    passed: false,
                    retries: 0,
                    errors: None,
                },
                docs: ComplianceCheck {
                    passed: false,
                    retries: 0,
                    errors: None,
                },
            },
            triggered_loop: false,
        };

        // Run each Phase 2 check in sequence
        // If ANY check needs retry, we loop back to Phase 1

        // Compilation check
        phase2_attempt.checks.compilation = self.run_compilation_check().await?;
        if phase2_attempt.checks.compilation.retries > 0 {
            phase2_attempt.triggered_loop = true;
        }

        // Test check
        phase2_attempt.checks.tests = self.run_test_check().await?;
        if phase2_attempt.checks.tests.retries > 0 {
            phase2_attempt.triggered_loop = true;
        }

        // Formatting check
        phase2_attempt.checks.formatting = self.run_formatting_check().await?;
        if phase2_attempt.checks.formatting.retries > 0 {
            phase2_attempt.triggered_loop = true;
        }

        // Clippy check
        phase2_attempt.checks.clippy = self.run_clippy_check().await?;
        if phase2_attempt.checks.clippy.retries > 0 {
            phase2_attempt.triggered_loop = true;
        }

        // Documentation check
        phase2_attempt.checks.docs = self.run_doc_check().await?;
        if phase2_attempt.checks.docs.retries > 0 {
            phase2_attempt.triggered_loop = true;
        }

        // Store this attempt
        self.context.phase2_attempts.push(phase2_attempt.clone());

        // Return true if all checks passed without retries
        Ok(!phase2_attempt.triggered_loop && self.all_phase2_checks_passed(&phase2_attempt))
    }

    /// Check if Phase 1 has critical failures that should stop the pipeline
    fn has_critical_phase1_failures(&self) -> bool {
        // Security failures are always critical
        if !self.context.phase1_results.security.passed {
            return true;
        }

        // Multiple Phase 1 failures might be critical
        let failure_count = [
            &self.context.phase1_results.code_review,
            &self.context.phase1_results.testing,
            &self.context.phase1_results.integration,
        ]
        .iter()
        .filter(|check| !check.passed)
        .count();

        failure_count >= 2
    }

    /// Check if all Phase 2 checks passed
    fn all_phase2_checks_passed(&self, attempt: &Phase2Attempt) -> bool {
        attempt.checks.compilation.passed
            && attempt.checks.tests.passed
            && attempt.checks.formatting.passed
            && attempt.checks.clippy.passed
            && attempt.checks.docs.passed
    }

    /// Analyze execution patterns for the analysis agents
    fn analyze_patterns(&mut self) {
        let mut failure_counts: HashMap<String, u8> = HashMap::new();
        let mut performance_issues = vec![];

        // Analyze Phase 2 attempts for patterns
        for attempt in &self.context.phase2_attempts {
            if !attempt.checks.compilation.passed {
                *failure_counts.entry("compilation".to_string()).or_insert(0) += 1;
            }
            if !attempt.checks.tests.passed {
                *failure_counts.entry("tests".to_string()).or_insert(0) += 1;
            }
            if !attempt.checks.formatting.passed {
                *failure_counts.entry("formatting".to_string()).or_insert(0) += 1;
            }
            if !attempt.checks.clippy.passed {
                *failure_counts.entry("clippy".to_string()).or_insert(0) += 1;
            }
            if !attempt.checks.docs.passed {
                *failure_counts.entry("docs".to_string()).or_insert(0) += 1;
            }
        }

        // Identify consistent failures
        let total_attempts = self.context.phase2_attempts.len() as u8;
        let consistent_failures: Vec<String> = failure_counts
            .iter()
            .filter(|(_, count)| **count == total_attempts)
            .map(|(check, _)| check.clone())
            .collect();

        if !consistent_failures.is_empty() {
            self.context.patterns.consistent_failures = Some(consistent_failures);
        }

        // Identify flaky checks
        let flaky_checks: Vec<String> = failure_counts
            .iter()
            .filter(|(_, count)| **count > 0 && **count < total_attempts)
            .map(|(check, _)| check.clone())
            .collect();

        if !flaky_checks.is_empty() {
            self.context.patterns.flakey_checks = Some(flaky_checks);
        }

        // Identify performance bottlenecks
        if self.context.phase1_results.code_review.duration_ms > 60000 {
            performance_issues.push("code_review".to_string());
        }
        if self.context.phase1_results.testing.duration_ms > 120000 {
            performance_issues.push("testing".to_string());
        }

        if !performance_issues.is_empty() {
            self.context.patterns.performance_bottlenecks = Some(performance_issues);
        }
    }

    /// Run the appropriate analysis agent based on pipeline outcome
    async fn run_analysis_agent(&mut self) -> Result<()> {
        let context_clone = self.context.clone();

        match self.context.final_status {
            PipelineStatus::Success => {
                info!("[ValidationPipeline] Running success analyzer agent");
                let _ = self
                    .spawn_claude_agent(
                        ".claude/validation-agents/analysis/success-analyzer.md",
                        &context_clone,
                    )
                    .await;
            }
            PipelineStatus::SuccessWithRetries => {
                info!("[ValidationPipeline] Running success-with-issues analyzer agent");
                let _ = self
                    .spawn_claude_agent(
                        ".claude/validation-agents/analysis/success-with-issues-analyzer.md",
                        &context_clone,
                    )
                    .await;
            }
            PipelineStatus::Failure => {
                info!("[ValidationPipeline] Running failure analyzer agent");
                let _ = self
                    .spawn_claude_agent(
                        ".claude/validation-agents/analysis/failure-analyzer.md",
                        &context_clone,
                    )
                    .await;
            }
        }
        Ok(())
    }

    /// Calculate timeout for current iteration using exponential backoff
    fn calculate_timeout(&self) -> Duration {
        let backoff_factor = TIMEOUT_MULTIPLIER.powi((self.context.pipeline_iterations - 1) as i32);
        let timeout_secs = (BASE_AGENT_TIMEOUT.as_secs() as f64 * backoff_factor) as u64;

        // Cap at maximum timeout
        if timeout_secs > MAX_AGENT_TIMEOUT.as_secs() {
            MAX_AGENT_TIMEOUT
        } else {
            Duration::from_secs(timeout_secs)
        }
    }

    /// Run a command with timeout
    async fn run_command_with_timeout(
        &self,
        cmd: &str,
        args: &[&str],
    ) -> Result<std::process::Output> {
        let timeout_duration = Duration::from_secs(30); // 30 seconds for cargo commands

        let cmd_str = format!("{} {}", cmd, args.join(" "));
        info!(
            "[ValidationPipeline] Running command with timeout: {}",
            cmd_str
        );

        // Run command in blocking task with timeout
        let result = timeout(
            timeout_duration,
            tokio::task::spawn_blocking({
                let cmd = cmd.to_string();
                let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
                move || Command::new(cmd).args(&args).output()
            }),
        )
        .await;

        match result {
            Ok(Ok(Ok(output))) => Ok(output),
            Ok(Ok(Err(e))) => Err(crate::error::SpiralError::SystemError(format!(
                "Failed to run {}: {}",
                cmd_str, e
            ))),
            Ok(Err(e)) => Err(crate::error::SpiralError::SystemError(format!(
                "Command panicked: {}",
                e
            ))),
            Err(_) => Err(crate::error::SpiralError::SystemError(format!(
                "Command timed out after {:?}: {}",
                timeout_duration, cmd_str
            ))),
        }
    }

    /// Track a change made during validation
    fn track_change(&mut self, phase: &str, description: &str, files: Vec<String>) {
        self.context.changes_applied.push(ChangeRecord {
            phase: phase.to_string(),
            description: description.to_string(),
            files: files.clone(),
        });

        // Add files to modified list (deduplicated)
        for file in files {
            if !self.context.files_modified.contains(&file) {
                self.context.files_modified.push(file);
            }
        }
    }

    /// Add a critical error to the context
    fn add_critical_error(&mut self, error: String) {
        self.context.critical_errors.push(error);
    }

    /// Spawn a Claude Code agent with the given prompt file and context
    async fn spawn_claude_agent(
        &mut self,
        agent_path: &str,
        context: &PipelineContext,
    ) -> Result<ClaudeValidationResponse> {
        let timeout_duration = self.calculate_timeout();
        info!(
            "[ValidationPipeline] Spawning Claude agent: {} with timeout: {:?}",
            agent_path, timeout_duration
        );

        // 1. Read the agent prompt from agent_path
        let agent_prompt = tokio::fs::read_to_string(agent_path).await.map_err(|e| {
            crate::error::SpiralError::SystemError(format!(
                "Failed to read agent prompt {}: {}",
                agent_path, e
            ))
        })?;

        // 2. Serialize the context to JSON
        let context_json = context.to_json()?;

        // 3. Create full prompt with agent instructions + context
        let full_prompt = format!(
            "{}\n\n## Current Pipeline Context\n\n```json\n{}\n```\n\n## Task\n\nAnalyze the context and directly fix the issues using your available tools (file editing, command execution, etc.).\n\nAfter making the fixes, respond with a summary of what was done.",
            agent_prompt,
            context_json
        );

        // 4. Call Claude Code API (if client is available)
        if let Some(claude_client) = &self.claude_client {
            // Create code generation request
            let request = CodeGenerationRequest {
                language: "rust".to_string(),
                description: full_prompt,
                context: HashMap::new(),
                existing_code: None,
                requirements: vec![],
                session_id: None,
            };

            // Execute with timeout
            let result = timeout(timeout_duration, async {
                claude_client.generate_code(request).await
            })
            .await;

            match result {
                Ok(Ok(generation_result)) => {
                    // Claude has made the fixes directly
                    info!("[ValidationPipeline] Claude completed validation fixes");

                    // Track that Claude made changes (we don't know exactly what, but that's ok)
                    self.track_change(
                        "claude_validation",
                        &format!("Claude applied fixes via {}", agent_path),
                        vec![], // We don't track individual files since Claude handles it
                    );

                    Ok(ClaudeValidationResponse {
                        explanation: generation_result.explanation,
                        success: true,
                    })
                }
                Ok(Err(e)) => {
                    warn!("[ValidationPipeline] Claude Code API error: {}", e);
                    Ok(ClaudeValidationResponse {
                        explanation: format!("API error: {}", e),
                        success: false,
                    })
                }
                Err(_) => {
                    warn!("[ValidationPipeline] Claude Code request timed out");
                    Ok(ClaudeValidationResponse {
                        explanation: "Request timed out".to_string(),
                        success: false,
                    })
                }
            }
        } else {
            // No Claude client - return mock response
            info!("[ValidationPipeline] No Claude client configured - returning mock response");
            Ok(ClaudeValidationResponse {
                explanation: "No Claude client configured".to_string(),
                success: true, // Allow pipeline to continue
            })
        }
    }

    // Phase 1 check implementations (stubs for now)

    async fn run_code_review(&self) -> Result<CheckResult> {
        let start = Instant::now();
        info!("[Phase1] Running code review standards check");

        // TODO: Spawn Claude agent with .claude/validation-agents/phase1/code-review-standards.md

        Ok(CheckResult {
            passed: true,
            findings: vec!["Code review not yet implemented".to_string()],
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn run_comprehensive_testing(&self) -> Result<CheckResult> {
        let start = Instant::now();
        info!("[Phase1] Running comprehensive testing check");

        // TODO: Spawn Claude agent with .claude/validation-agents/phase1/comprehensive-testing.md

        Ok(CheckResult {
            passed: true,
            findings: vec!["Testing analysis not yet implemented".to_string()],
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn run_security_audit(&self) -> Result<CheckResult> {
        let start = Instant::now();
        info!("[Phase1] Running security audit");

        // TODO: Spawn Claude agent with .claude/validation-agents/phase1/security-audit.md

        Ok(CheckResult {
            passed: true,
            findings: vec!["Security audit not yet implemented".to_string()],
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn run_system_integration(&self) -> Result<CheckResult> {
        let start = Instant::now();
        info!("[Phase1] Running system integration check");

        // TODO: Spawn Claude agent with .claude/validation-agents/phase1/system-integration.md

        Ok(CheckResult {
            passed: true,
            findings: vec!["Integration check not yet implemented".to_string()],
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    // Phase 2 check implementations (stubs for now)

    async fn run_compilation_check(&mut self) -> Result<ComplianceCheck> {
        info!("[Phase2] Running compilation check");

        let mut retries = 0;
        let mut errors = vec![];

        // Try up to 2 times (initial + 1 retry with Claude)
        for attempt in 0..2 {
            let output = self
                .run_command_with_timeout("cargo", &["check", "--all-targets"])
                .await?;

            if output.status.success() {
                return Ok(ComplianceCheck {
                    passed: true,
                    retries,
                    errors: if errors.is_empty() {
                        None
                    } else {
                        Some(errors)
                    },
                });
            }

            // Compilation failed
            let error_msg = String::from_utf8_lossy(&output.stderr);
            errors.push(error_msg.to_string());

            if attempt == 0 {
                // First failure - try to fix with Claude
                warn!("[Phase2] Compilation check failed, attempting Claude fix");

                // Update context with compilation errors
                let mut fix_context = self.context.clone();
                fix_context.phase2_attempts.push(Phase2Attempt {
                    iteration: self.context.pipeline_iterations,
                    checks: Phase2Checks {
                        compilation: ComplianceCheck {
                            passed: false,
                            retries: 0,
                            errors: Some(vec![error_msg.to_string()]),
                        },
                        tests: ComplianceCheck {
                            passed: false,
                            retries: 0,
                            errors: None,
                        },
                        formatting: ComplianceCheck {
                            passed: false,
                            retries: 0,
                            errors: None,
                        },
                        clippy: ComplianceCheck {
                            passed: false,
                            retries: 0,
                            errors: None,
                        },
                        docs: ComplianceCheck {
                            passed: false,
                            retries: 0,
                            errors: None,
                        },
                    },
                    triggered_loop: false,
                });

                // Spawn Claude agent
                let claude_response = self
                    .spawn_claude_agent(
                        ".claude/validation-agents/phase2/compilation-fixer.md",
                        &fix_context,
                    )
                    .await?;

                if claude_response.success {
                    info!("[Phase2] Claude applied compilation fixes");
                    retries = 1;
                    // Loop will retry cargo check
                } else {
                    // Claude couldn't fix it
                    info!("[Phase2] Claude unable to fix compilation issues");
                    break;
                }
            }
        }

        Ok(ComplianceCheck {
            passed: false,
            retries,
            errors: Some(errors),
        })
    }

    async fn run_test_check(&self) -> Result<ComplianceCheck> {
        info!("[Phase2] Running test check");

        let output = Command::new("cargo").arg("test").output().map_err(|e| {
            crate::error::SpiralError::SystemError(format!("Failed to run cargo test: {}", e))
        })?;

        if output.status.success() {
            return Ok(ComplianceCheck {
                passed: true,
                retries: 0,
                errors: None,
            });
        }

        // Tests failed - spawn Claude agent to fix
        let errors = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        warn!("[Phase2] Test check failed");

        // TODO: Actually spawn Claude agent with test-failure-analyzer.md
        // For now, simulate that we tried once

        Ok(ComplianceCheck {
            passed: false,
            retries: 1,
            errors: Some(vec![format!("stdout: {}\nstderr: {}", stdout, errors)]),
        })
    }

    async fn run_formatting_check(&self) -> Result<ComplianceCheck> {
        info!("[Phase2] Running formatting check");

        let output = Command::new("cargo")
            .args(["fmt", "--", "--check"])
            .output()
            .map_err(|e| {
                crate::error::SpiralError::SystemError(format!("Failed to run cargo fmt: {}", e))
            })?;

        if output.status.success() {
            return Ok(ComplianceCheck {
                passed: true,
                retries: 0,
                errors: None,
            });
        }

        // Formatting incorrect - spawn Claude agent to fix
        warn!("[Phase2] Formatting check failed");

        // TODO: Actually spawn Claude agent with formatting-fixer.md
        // For now, simulate that we tried once and fixed it

        // Run cargo fmt to fix
        let _ = Command::new("cargo").arg("fmt").output();

        Ok(ComplianceCheck {
            passed: false,
            retries: 1,
            errors: Some(vec!["Formatting issues detected and fixed".to_string()]),
        })
    }

    async fn run_clippy_check(&self) -> Result<ComplianceCheck> {
        info!("[Phase2] Running clippy check");

        let output = Command::new("cargo")
            .args(["clippy", "--all-targets", "--", "-D", "warnings"])
            .output()
            .map_err(|e| {
                crate::error::SpiralError::SystemError(format!("Failed to run cargo clippy: {}", e))
            })?;

        if output.status.success() {
            return Ok(ComplianceCheck {
                passed: true,
                retries: 0,
                errors: None,
            });
        }

        // Clippy warnings/errors - spawn Claude agent to fix
        let errors = String::from_utf8_lossy(&output.stderr);
        warn!("[Phase2] Clippy check failed");

        // TODO: Actually spawn Claude agent with clippy-resolver.md
        // For now, simulate that we tried once

        Ok(ComplianceCheck {
            passed: false,
            retries: 1,
            errors: Some(vec![errors.to_string()]),
        })
    }

    async fn run_doc_check(&self) -> Result<ComplianceCheck> {
        info!("[Phase2] Running documentation check");

        let output = Command::new("cargo")
            .args(["doc", "--no-deps", "--quiet"])
            .output()
            .map_err(|e| {
                crate::error::SpiralError::SystemError(format!("Failed to run cargo doc: {}", e))
            })?;

        if output.status.success() {
            return Ok(ComplianceCheck {
                passed: true,
                retries: 0,
                errors: None,
            });
        }

        // Doc build failed - spawn Claude agent to fix
        let errors = String::from_utf8_lossy(&output.stderr);
        warn!("[Phase2] Documentation check failed");

        // TODO: Actually spawn Claude agent with doc-builder.md
        // For now, simulate that we tried once

        Ok(ComplianceCheck {
            passed: false,
            retries: 1,
            errors: Some(vec![errors.to_string()]),
        })
    }
}
