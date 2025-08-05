//! Two-phase validation pipeline implementation
#![allow(clippy::uninlined_format_args)]
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
use tracing::{debug, error, info, warn};

/// Maximum number of complete pipeline iterations allowed
const MAX_PIPELINE_ITERATIONS: u8 = 3;

/// Base timeout for individual agent operations (2 minutes)
const BASE_AGENT_TIMEOUT: Duration = Duration::from_secs(120);

/// Maximum timeout after exponential backoff (10 minutes)
const MAX_AGENT_TIMEOUT: Duration = Duration::from_secs(600);

/// Timeout multiplier for exponential backoff
const TIMEOUT_MULTIPLIER: f64 = 1.5;

// Phase 2 Claude agent paths - single source of truth for agent locations
const CLAUDE_AGENT_COMPILATION_FIX: &str = ".claude/validation-agents/phase2/compilation-fixer.md";
const CLAUDE_AGENT_TEST_FIX: &str = ".claude/validation-agents/phase2/test-failure-analyzer.md";
const CLAUDE_AGENT_FORMAT_FIX: &str = ".claude/validation-agents/phase2/formatting-fixer.md";
const CLAUDE_AGENT_CLIPPY_FIX: &str = ".claude/validation-agents/phase2/clippy-resolver.md";
const CLAUDE_AGENT_DOC_FIX: &str = ".claude/validation-agents/phase2/doc-builder.md";

// Analysis agent paths
const CLAUDE_AGENT_SUCCESS_ANALYZER: &str =
    ".claude/validation-agents/analysis/success-analyzer.md";
const CLAUDE_AGENT_SUCCESS_WITH_ISSUES: &str =
    ".claude/validation-agents/analysis/success-with-issues-analyzer.md";
const CLAUDE_AGENT_FAILURE_ANALYZER: &str =
    ".claude/validation-agents/analysis/failure-analyzer.md";

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
    /// Whether Claude executed successfully (not whether the fix worked)
    pub success: bool,
}

/// Main validation pipeline coordinator
pub struct ValidationPipeline {
    context: PipelineContext,
    start_time: Instant,
    claude_client: Option<ClaudeCodeClient>,
    snapshot_id: Option<String>,
}

/// 📐 SOLID: Trait for running validation checks
/// Responsibility: Execute a validation command
/// Dependencies: None - pure interface
#[async_trait::async_trait]
pub trait CheckRunner: Send + Sync {
    async fn run_check(&self) -> Result<CheckOutput>;
    fn name(&self) -> &str;
}

/// Output from running a check
pub struct CheckOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// 📐 SOLID: Trait for handling validation fixes
/// Responsibility: Abstract fix application strategy
/// Dependencies: None - pure interface
/// Extension: Implement for different fix strategies
#[async_trait::async_trait]
pub trait FixHandler: Send + Sync {
    /// Attempt to fix a validation issue
    async fn attempt_fix(&self, check_name: &str, error_msg: &str) -> Result<bool>;
}

/// 🔄 DRY PATTERN: Generic validation runner with retry
/// Replaces: All duplicate retry logic
/// Usage: Pass any check runner and fix handler
pub struct ValidationRunner;

impl ValidationRunner {
    /// 🏗️ ARCHITECTURE DECISION: Truly generic retry mechanism
    /// Why: Complete separation of concerns - running vs fixing
    /// Alternative: Coupled implementation (rejected: violates SOLID)
    /// Audit: Verify check and fix are properly abstracted
    pub async fn run_with_retry<C, F, Fut>(
        check: &C,
        fix_handler: F,
        max_attempts: u8,
    ) -> Result<ComplianceCheck>
    where
        C: CheckRunner,
        F: Fn(&str, u8) -> Fut,
        Fut: std::future::Future<Output = Result<bool>>,
    {
        let mut retries = 0;
        let mut all_errors = vec![];
        let check_name = check.name();

        info!("[ValidationRunner] Running {} check", check_name);

        for attempt in 1..=max_attempts {
            info!(
                "[ValidationRunner] {} check attempt {}/{}",
                check_name, attempt, max_attempts
            );

            // Run the check
            let output = check.run_check().await?;

            if output.success {
                info!(
                    "[ValidationRunner] {} check passed on attempt {}",
                    check_name, attempt
                );
                return Ok(ComplianceCheck {
                    passed: true,
                    retries,
                    errors: if all_errors.is_empty() {
                        None
                    } else {
                        Some(all_errors)
                    },
                });
            }

            // Check failed - capture error
            let error_msg = if check_name == "tests" {
                format!("stdout: {}\nstderr: {}", output.stdout, output.stderr)
            } else if check_name == "formatting" {
                "Code is not properly formatted".to_string()
            } else {
                output.stderr
            };

            all_errors.push(format!("Attempt {}: {}", attempt, error_msg));

            // Log the error details (first 500 chars to avoid spam)
            let error_preview = if error_msg.len() > 500 {
                format!("{}... [truncated]", &error_msg[..500])
            } else {
                error_msg.clone()
            };

            warn!(
                "[ValidationRunner] {} check failed on attempt {}/{}\nError: {}",
                check_name, attempt, max_attempts, error_preview
            );

            // Always try to fix after a failure (even on last attempt)
            info!(
                "[ValidationRunner] Attempting fix for {} issues",
                check_name
            );

            match fix_handler(&error_msg, retries).await {
                Ok(true) => {
                    info!("[ValidationRunner] Fix applied successfully");
                    retries += 1;

                    // Only retry the check if we haven't exhausted attempts
                    if attempt < max_attempts {
                        continue; // Retry the check
                    } else {
                        info!("[ValidationRunner] Fix applied on final attempt, but no more retries allowed");
                    }
                }
                Ok(false) => {
                    warn!("[ValidationRunner] Fix handler couldn't resolve the issue");
                    retries += 1;
                }
                Err(e) => {
                    warn!("[ValidationRunner] Fix handler error: {}", e);
                    retries += 1;
                }
            }
        }

        // Log final failure with summary of errors
        error!(
            "[ValidationRunner] {} check failed after {} attempts with {} retries",
            check_name, max_attempts, retries
        );

        // Log a summary of errors from each attempt
        for (i, err) in all_errors.iter().enumerate() {
            error!("[ValidationRunner] └── {}", err);
            if i >= 2 {
                // Only show first 3 errors to avoid spam
                if all_errors.len() > 3 {
                    error!(
                        "[ValidationRunner] └── ... and {} more errors",
                        all_errors.len() - 3
                    );
                }
                break;
            }
        }

        Ok(ComplianceCheck {
            passed: false,
            retries,
            errors: Some(all_errors),
        })
    }
}

/// Concrete check runner for cargo commands
pub struct CargoCheck {
    name: String,
    command: String,
    args: Vec<String>,
}

impl CargoCheck {
    pub fn new(name: impl Into<String>, command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            args,
        }
    }
}

#[async_trait::async_trait]
impl CheckRunner for CargoCheck {
    async fn run_check(&self) -> Result<CheckOutput> {
        let output = Command::new(&self.command)
            .args(&self.args)
            .output()
            .map_err(|e| {
                error!(
                    "[CargoCheck] Failed to execute {} {}: {}",
                    self.command,
                    self.args.join(" "),
                    e
                );
                crate::error::SpiralError::SystemError(format!(
                    "Failed to run {} {}: {}",
                    self.command,
                    self.args.join(" "),
                    e
                ))
            })?;

        // Log exit code if command failed
        if !output.status.success() {
            if let Some(code) = output.status.code() {
                debug!("[CargoCheck] {} exited with code {}", self.command, code);
            }
        }

        Ok(CheckOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Composite fix handler that tries auto-fix first, then Claude
pub struct CompositeFixHandler {
    auto_fix: Option<Box<dyn FixHandler>>,
    claude_fix: Option<Box<dyn FixHandler>>,
}

impl CompositeFixHandler {
    pub fn new(
        auto_fix: Option<Box<dyn FixHandler>>,
        claude_fix: Option<Box<dyn FixHandler>>,
    ) -> Self {
        Self {
            auto_fix,
            claude_fix,
        }
    }
}

#[async_trait::async_trait]
impl FixHandler for CompositeFixHandler {
    async fn attempt_fix(&self, check_name: &str, error_msg: &str) -> Result<bool> {
        // Try auto-fix first
        if let Some(auto_handler) = &self.auto_fix {
            if let Ok(true) = auto_handler.attempt_fix(check_name, error_msg).await {
                return Ok(true);
            }
            // Continue to Claude
        }

        // Try Claude fix
        if let Some(claude_handler) = &self.claude_fix {
            return claude_handler.attempt_fix(check_name, error_msg).await;
        }

        Ok(false)
    }
}

/// Auto-fix handler for simple fixes like cargo fmt
pub struct AutoFixHandler {
    command: String,
    args: Vec<String>,
}

impl AutoFixHandler {
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            command: command.into(),
            args,
        }
    }
}

#[async_trait::async_trait]
impl FixHandler for AutoFixHandler {
    async fn attempt_fix(&self, check_name: &str, _error_msg: &str) -> Result<bool> {
        info!(
            "[AutoFixHandler] Running {} {} for {}",
            self.command,
            self.args.join(" "),
            check_name
        );

        let output = Command::new(&self.command)
            .args(&self.args)
            .output()
            .map_err(|e| {
                crate::error::SpiralError::SystemError(format!("Failed to run auto-fix: {}", e))
            })?;

        Ok(output.status.success())
    }
}

/// 🏗️ ARCHITECTURE DECISION: Claude-based fix handler
/// Why: Claude can understand and fix complex errors autonomously
/// Alternative: Manual fixes (available via NoOpFixHandler)
/// Audit: Verify prompts don't expose sensitive data
pub struct ClaudeFixHandler {
    claude_client: ClaudeCodeClient,
    agent_path: String,
}

impl ClaudeFixHandler {
    pub fn new(claude_client: ClaudeCodeClient, agent_path: impl Into<String>) -> Self {
        Self {
            claude_client,
            agent_path: agent_path.into(),
        }
    }
}

#[async_trait::async_trait]
impl FixHandler for ClaudeFixHandler {
    async fn attempt_fix(&self, check_name: &str, error_msg: &str) -> Result<bool> {
        use std::collections::HashMap;

        // Read the agent prompt template
        let agent_prompt = tokio::fs::read_to_string(&self.agent_path)
            .await
            .map_err(|e| {
                crate::error::SpiralError::SystemError(format!(
                    "Failed to read agent prompt {}: {}",
                    self.agent_path, e
                ))
            })?;

        // Truncate error message to avoid exceeding Claude's limits
        // Keep first 2000 chars for context (leaves ~8000 for agent prompt + task)
        let truncated_error = if error_msg.len() > 2000 {
            // For test/compilation errors, try to extract the most relevant parts
            let lines: Vec<&str> = error_msg.lines().collect();
            let error_lines: Vec<&str> = lines
                .iter()
                .filter(|line| {
                    line.contains("error:")
                        || line.contains("warning:")
                        || line.contains("-->")
                        || line.contains("help:")
                        || line.trim().starts_with('|')
                })
                .take(30) // Take first 30 relevant lines
                .copied()
                .collect();

            let truncated = if !error_lines.is_empty() {
                format!(
                    "{}\n[Truncated {} more lines from {} total chars]",
                    error_lines.join("\n"),
                    lines.len().saturating_sub(error_lines.len()),
                    error_msg.len()
                )
            } else {
                // Fallback to simple truncation if no error patterns found
                format!(
                    "{}...\n[Truncated {} more characters]",
                    &error_msg[..2000],
                    error_msg.len() - 2000
                )
            };

            info!(
                "[ClaudeFixHandler] Truncated error from {} to {} chars",
                error_msg.len(),
                truncated.len()
            );
            truncated
        } else {
            error_msg.to_string()
        };

        // Create a focused prompt for Claude
        let full_prompt = format!(
            "{}\n\n## Current Issue\n\nThe {} check is failing with the following error:\n\n```\n{}\n```\n\n## Task\n\nAnalyze and fix this issue directly in the code. Use your available tools to:\n1. Understand the error\n2. Locate the problematic code\n3. Apply the necessary fix\n4. Verify the fix if possible\n\nRespond with a brief summary of what you fixed.",
            agent_prompt,
            check_name,
            truncated_error
        );

        // Log prompt size for debugging
        info!(
            "[ClaudeFixHandler] Total prompt size: {} chars (agent: {}, error: {})",
            full_prompt.len(),
            agent_prompt.len(),
            truncated_error.len()
        );

        // Create code generation request
        let request = CodeGenerationRequest {
            language: "rust".to_string(),
            description: full_prompt,
            context: HashMap::new(),
            existing_code: None,
            requirements: vec![
                format!("Fix the {} validation error", check_name),
                "Make minimal necessary changes".to_string(),
                "Preserve existing functionality".to_string(),
            ],
            session_id: Some(format!("phase2-{}-fix", check_name)),
        };

        // Execute Claude fix with timeout
        let timeout_duration = Duration::from_secs(120);
        match tokio::time::timeout(timeout_duration, self.claude_client.generate_code(request))
            .await
        {
            Ok(Ok(result)) => {
                info!("[ClaudeFixHandler] Fix completed: {}", result.explanation);
                Ok(true)
            }
            Ok(Err(e)) => {
                warn!("[ClaudeFixHandler] Fix error: {}", e);
                Ok(false)
            }
            Err(_) => {
                warn!("[ClaudeFixHandler] Fix timed out after 2 minutes");
                Ok(false)
            }
        }
    }
}

/// No-op fix handler for when Claude is not available
pub struct NoOpFixHandler;

#[async_trait::async_trait]
impl FixHandler for NoOpFixHandler {
    async fn attempt_fix(&self, _check_name: &str, _error_msg: &str) -> Result<bool> {
        Ok(false) // Always returns false - no fix attempted
    }
}

/// 📐 SOLID: Single Responsibility + Dependency Inversion
/// Responsibility: Execute Phase 2 checks ONLY - no Phase 1 concerns
/// Dependencies: Optional Claude client for fixes
/// Extension: Add new fix strategies by implementing FixHandler trait
pub struct Phase2Executor {
    pub claude_client: Option<ClaudeCodeClient>,
    pub start_time: Instant,
}

impl Default for Phase2Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Phase2Executor {
    /// Create a new Phase 2 executor without any Phase 1 dependencies
    pub fn new() -> Self {
        Self {
            claude_client: None,
            start_time: Instant::now(),
        }
    }

    /// Create with Claude client for automatic fixes
    pub async fn with_claude(config: ClaudeCodeConfig) -> Result<Self> {
        let claude_client = ClaudeCodeClient::new(config).await?;
        Ok(Self {
            claude_client: Some(claude_client),
            start_time: Instant::now(),
        })
    }

    /// Run all Phase 2 checks independently
    pub async fn execute(&mut self) -> Result<Phase2Attempt> {
        info!("[Phase2Executor] Starting independent Phase 2 validation");

        let compilation = self.run_compilation_check().await?;
        let tests = self.run_test_check().await?;
        let formatting = self.run_formatting_check().await?;
        let clippy = self.run_clippy_check().await?;
        let docs = self.run_doc_check().await?;

        let triggered_loop = compilation.retries > 0
            || tests.retries > 0
            || formatting.retries > 0
            || clippy.retries > 0
            || docs.retries > 0;

        Ok(Phase2Attempt {
            iteration: 1, // Standalone is always iteration 1
            checks: Phase2Checks {
                compilation,
                tests,
                formatting,
                clippy,
                docs,
            },
            triggered_loop,
        })
    }

    async fn run_compilation_check(&mut self) -> Result<ComplianceCheck> {
        let check = CargoCheck::new(
            "compilation",
            "cargo",
            vec!["check".to_string(), "--all-targets".to_string()],
        );

        let claude_client = self.claude_client.clone();
        let fix_handler = move |error_msg: &str, _retries: u8| {
            let client = claude_client.clone();
            let error = error_msg.to_string();
            async move {
                if let Some(client) = client {
                    // Spawn Claude agent to handle compilation fixes
                    let fix_handler = ClaudeFixHandler::new(client, CLAUDE_AGENT_COMPILATION_FIX);
                    fix_handler.attempt_fix("compilation", &error).await
                } else {
                    Ok(false)
                }
            }
        };

        ValidationRunner::run_with_retry(&check, fix_handler, 3).await
    }

    async fn run_test_check(&mut self) -> Result<ComplianceCheck> {
        let check = CargoCheck::new("tests", "cargo", vec!["test".to_string()]);

        let claude_client = self.claude_client.clone();
        let fix_handler = move |error_msg: &str, _retries: u8| {
            let client = claude_client.clone();
            let error = error_msg.to_string();
            async move {
                if let Some(client) = client {
                    // Spawn Claude agent to handle test failure analysis
                    let fix_handler = ClaudeFixHandler::new(client, CLAUDE_AGENT_TEST_FIX);
                    fix_handler.attempt_fix("tests", &error).await
                } else {
                    Ok(false)
                }
            }
        };

        ValidationRunner::run_with_retry(&check, fix_handler, 3).await
    }

    async fn run_formatting_check(&mut self) -> Result<ComplianceCheck> {
        let check = CargoCheck::new(
            "formatting",
            "cargo",
            vec!["fmt".to_string(), "--".to_string(), "--check".to_string()],
        );

        let claude_client = self.claude_client.clone();
        let fix_handler = move |error_msg: &str, retries: u8| {
            let client = claude_client.clone();
            let error = error_msg.to_string();
            async move {
                // Try auto-fix first with cargo fmt
                if retries == 0 {
                    let auto_fix = AutoFixHandler::new("cargo", vec!["fmt".to_string()]);
                    if auto_fix.attempt_fix("formatting", &error).await? {
                        return Ok(true);
                    }
                }

                // Then try Claude if auto-fix didn't work
                if let Some(client) = client {
                    let fix_handler = ClaudeFixHandler::new(client, CLAUDE_AGENT_FORMAT_FIX);
                    fix_handler.attempt_fix("formatting", &error).await
                } else {
                    Ok(false)
                }
            }
        };

        ValidationRunner::run_with_retry(&check, fix_handler, 3).await
    }

    async fn run_clippy_check(&mut self) -> Result<ComplianceCheck> {
        let check = CargoCheck::new(
            "clippy",
            "cargo",
            vec![
                "clippy".to_string(),
                "--all-targets".to_string(),
                "--".to_string(),
                "-D".to_string(),
                "warnings".to_string(),
            ],
        );

        let claude_client = self.claude_client.clone();
        let fix_handler = move |error_msg: &str, _retries: u8| {
            let client = claude_client.clone();
            let error = error_msg.to_string();
            async move {
                if let Some(client) = client {
                    // Spawn Claude agent to resolve clippy warnings
                    let fix_handler = ClaudeFixHandler::new(client, CLAUDE_AGENT_CLIPPY_FIX);
                    fix_handler.attempt_fix("clippy", &error).await
                } else {
                    Ok(false)
                }
            }
        };

        ValidationRunner::run_with_retry(&check, fix_handler, 3).await
    }

    async fn run_doc_check(&mut self) -> Result<ComplianceCheck> {
        let check = CargoCheck::new(
            "docs",
            "cargo",
            vec![
                "doc".to_string(),
                "--no-deps".to_string(),
                "--quiet".to_string(),
            ],
        );

        let claude_client = self.claude_client.clone();
        let fix_handler = move |error_msg: &str, _retries: u8| {
            let client = claude_client.clone();
            let error = error_msg.to_string();
            async move {
                if let Some(client) = client {
                    // Spawn Claude agent to fix documentation issues
                    let fix_handler = ClaudeFixHandler::new(client, CLAUDE_AGENT_DOC_FIX);
                    fix_handler.attempt_fix("docs", &error).await
                } else {
                    Ok(false)
                }
            }
        };

        ValidationRunner::run_with_retry(&check, fix_handler, 3).await
    }
}

impl PipelineContext {
    /// Serialize context to JSON for passing to Claude agents
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            crate::error::SpiralError::SystemError(format!("Failed to serialize context: {e}"))
        })
    }
}

impl Default for ValidationPipeline {
    fn default() -> Self {
        Self::new()
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
    /// Execute Phase 2 checks independently without requiring Phase 1
    /// This follows SOLID principles - Phase 2 doesn't depend on Phase 1
    pub async fn run_phase2_independent(&mut self) -> Result<Phase2Attempt> {
        info!("[Phase2] ┌─── CORE RUST COMPLIANCE CHECKS (STANDALONE) ───┐");

        let mut phase2_attempt = Phase2Attempt {
            iteration: 1, // Standalone run is always iteration 1
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
        phase2_attempt.checks.compilation = self.run_compilation_check().await?;
        if phase2_attempt.checks.compilation.retries > 0 {
            phase2_attempt.triggered_loop = true;
        }

        phase2_attempt.checks.tests = self.run_test_check().await?;
        if phase2_attempt.checks.tests.retries > 0 {
            phase2_attempt.triggered_loop = true;
        }

        phase2_attempt.checks.formatting = self.run_formatting_check().await?;
        if phase2_attempt.checks.formatting.retries > 0 {
            phase2_attempt.triggered_loop = true;
        }

        phase2_attempt.checks.clippy = self.run_clippy_check().await?;
        if phase2_attempt.checks.clippy.retries > 0 {
            phase2_attempt.triggered_loop = true;
        }

        phase2_attempt.checks.docs = self.run_doc_check().await?;
        if phase2_attempt.checks.docs.retries > 0 {
            phase2_attempt.triggered_loop = true;
        }

        info!("[Phase2] └─── PHASE 2 COMPLETE ───┘");

        Ok(phase2_attempt)
    }

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
                    .spawn_claude_agent(CLAUDE_AGENT_SUCCESS_ANALYZER, &context_clone)
                    .await;
            }
            PipelineStatus::SuccessWithRetries => {
                info!("[ValidationPipeline] Running success-with-issues analyzer agent");
                let _ = self
                    .spawn_claude_agent(CLAUDE_AGENT_SUCCESS_WITH_ISSUES, &context_clone)
                    .await;
            }
            PipelineStatus::Failure => {
                info!("[ValidationPipeline] Running failure analyzer agent");
                let _ = self
                    .spawn_claude_agent(CLAUDE_AGENT_FAILURE_ANALYZER, &context_clone)
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
    #[allow(dead_code)] // Utility method for future use
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

    /// Create a fix context for Phase 2 checks
    #[allow(dead_code)] // Utility method for future use
    fn create_fix_context(
        &self,
        check_name: &str,
        error_msg: &str,
        retries: u8,
    ) -> PipelineContext {
        PipelineContext {
            pipeline_iterations: self.context.pipeline_iterations,
            total_duration_ms: self.start_time.elapsed().as_millis() as u64,
            final_status: PipelineStatus::Failure,
            phase1_results: self.context.phase1_results.clone(),
            phase2_attempts: vec![Phase2Attempt {
                iteration: self.context.pipeline_iterations,
                checks: Phase2Checks {
                    compilation: if check_name == "compilation" {
                        ComplianceCheck {
                            passed: false,
                            retries,
                            errors: Some(vec![error_msg.to_string()]),
                        }
                    } else {
                        ComplianceCheck {
                            passed: false,
                            retries: 0,
                            errors: None,
                        }
                    },
                    tests: if check_name == "tests" {
                        ComplianceCheck {
                            passed: false,
                            retries,
                            errors: Some(vec![error_msg.to_string()]),
                        }
                    } else {
                        ComplianceCheck {
                            passed: false,
                            retries: 0,
                            errors: None,
                        }
                    },
                    formatting: if check_name == "formatting" {
                        ComplianceCheck {
                            passed: false,
                            retries,
                            errors: Some(vec![error_msg.to_string()]),
                        }
                    } else {
                        ComplianceCheck {
                            passed: false,
                            retries: 0,
                            errors: None,
                        }
                    },
                    clippy: if check_name == "clippy" {
                        ComplianceCheck {
                            passed: false,
                            retries,
                            errors: Some(vec![error_msg.to_string()]),
                        }
                    } else {
                        ComplianceCheck {
                            passed: false,
                            retries: 0,
                            errors: None,
                        }
                    },
                    docs: if check_name == "docs" {
                        ComplianceCheck {
                            passed: false,
                            retries,
                            errors: Some(vec![error_msg.to_string()]),
                        }
                    } else {
                        ComplianceCheck {
                            passed: false,
                            retries: 0,
                            errors: None,
                        }
                    },
                },
                triggered_loop: false,
            }],
            files_modified: self.context.files_modified.clone(),
            changes_applied: self.context.changes_applied.clone(),
            critical_errors: vec![],
            warnings: vec![],
            patterns: ExecutionPatterns {
                consistent_failures: None,
                flakey_checks: None,
                performance_bottlenecks: None,
            },
        }
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
                        success: true, // Claude executed successfully
                    })
                }
                Ok(Err(e)) => {
                    warn!("[ValidationPipeline] Claude Code API error: {}", e);
                    Ok(ClaudeValidationResponse {
                        explanation: format!("API error: {}", e),
                        success: false, // Claude failed to execute
                    })
                }
                Err(_) => {
                    warn!("[ValidationPipeline] Claude Code request timed out");
                    Ok(ClaudeValidationResponse {
                        explanation: "Request timed out".to_string(),
                        success: false, // Claude failed to execute
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

    // Phase 1 check implementations

    async fn run_code_review(&self) -> Result<CheckResult> {
        let start = Instant::now();
        info!("[Phase1] Running code review standards check");

        // Note: Claude agent integration pending
        // Agent path: .claude/validation-agents/phase1/code-review-standards.md

        Ok(CheckResult {
            passed: true,
            findings: vec!["Code review agent integration pending".to_string()],
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn run_comprehensive_testing(&self) -> Result<CheckResult> {
        let start = Instant::now();
        info!("[Phase1] Running comprehensive testing check");

        // Note: Claude agent integration pending
        // Agent path: .claude/validation-agents/phase1/comprehensive-testing.md

        Ok(CheckResult {
            passed: true,
            findings: vec!["Testing analysis agent integration pending".to_string()],
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn run_security_audit(&self) -> Result<CheckResult> {
        let start = Instant::now();
        info!("[Phase1] Running security audit");

        // Note: Claude agent integration pending
        // Agent path: .claude/validation-agents/phase1/security-audit.md

        Ok(CheckResult {
            passed: true,
            findings: vec!["Security audit agent integration pending".to_string()],
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn run_system_integration(&self) -> Result<CheckResult> {
        let start = Instant::now();
        info!("[Phase1] Running system integration check");

        // Note: Claude agent integration pending
        // Agent path: .claude/validation-agents/phase1/system-integration.md

        Ok(CheckResult {
            passed: true,
            findings: vec!["Integration check agent integration pending".to_string()],
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    // Phase 2 check implementations

    async fn run_compilation_check(&mut self) -> Result<ComplianceCheck> {
        let check = CargoCheck::new(
            "compilation",
            "cargo",
            vec!["check".to_string(), "--all-targets".to_string()],
        );

        let claude_client = self.claude_client.clone();
        let fix_handler = move |error_msg: &str, _retries: u8| {
            let client = claude_client.clone();
            let error = error_msg.to_string();
            async move {
                if let Some(client) = client {
                    // Spawn Claude agent to handle compilation fixes
                    let fix_handler = ClaudeFixHandler::new(client, CLAUDE_AGENT_COMPILATION_FIX);
                    fix_handler.attempt_fix("compilation", &error).await
                } else {
                    Ok(false)
                }
            }
        };

        ValidationRunner::run_with_retry(&check, fix_handler, 3).await
    }

    async fn run_test_check(&mut self) -> Result<ComplianceCheck> {
        let check = CargoCheck::new("tests", "cargo", vec!["test".to_string()]);

        let claude_client = self.claude_client.clone();
        let fix_handler = move |error_msg: &str, _retries: u8| {
            let client = claude_client.clone();
            let error = error_msg.to_string();
            async move {
                if let Some(client) = client {
                    // Spawn Claude agent to handle test failure analysis
                    let fix_handler = ClaudeFixHandler::new(client, CLAUDE_AGENT_TEST_FIX);
                    fix_handler.attempt_fix("tests", &error).await
                } else {
                    Ok(false)
                }
            }
        };

        ValidationRunner::run_with_retry(&check, fix_handler, 3).await
    }

    async fn run_formatting_check(&mut self) -> Result<ComplianceCheck> {
        let check = CargoCheck::new(
            "formatting",
            "cargo",
            vec!["fmt".to_string(), "--".to_string(), "--check".to_string()],
        );

        let claude_client = self.claude_client.clone();
        let fix_handler = move |error_msg: &str, retries: u8| {
            let client = claude_client.clone();
            let error = error_msg.to_string();
            async move {
                // Try auto-fix first with cargo fmt
                if retries == 0 {
                    let auto_fix = AutoFixHandler::new("cargo", vec!["fmt".to_string()]);
                    if auto_fix.attempt_fix("formatting", &error).await? {
                        return Ok(true);
                    }
                }

                // Then try Claude if auto-fix didn't work
                if let Some(client) = client {
                    let fix_handler = ClaudeFixHandler::new(client, CLAUDE_AGENT_FORMAT_FIX);
                    fix_handler.attempt_fix("formatting", &error).await
                } else {
                    Ok(false)
                }
            }
        };

        ValidationRunner::run_with_retry(&check, fix_handler, 3).await
    }

    async fn run_clippy_check(&mut self) -> Result<ComplianceCheck> {
        let check = CargoCheck::new(
            "clippy",
            "cargo",
            vec![
                "clippy".to_string(),
                "--all-targets".to_string(),
                "--".to_string(),
                "-D".to_string(),
                "warnings".to_string(),
            ],
        );

        let claude_client = self.claude_client.clone();
        let fix_handler = move |error_msg: &str, _retries: u8| {
            let client = claude_client.clone();
            let error = error_msg.to_string();
            async move {
                if let Some(client) = client {
                    // Spawn Claude agent to resolve clippy warnings
                    let fix_handler = ClaudeFixHandler::new(client, CLAUDE_AGENT_CLIPPY_FIX);
                    fix_handler.attempt_fix("clippy", &error).await
                } else {
                    Ok(false)
                }
            }
        };

        ValidationRunner::run_with_retry(&check, fix_handler, 3).await
    }

    async fn run_doc_check(&mut self) -> Result<ComplianceCheck> {
        let check = CargoCheck::new(
            "docs",
            "cargo",
            vec![
                "doc".to_string(),
                "--no-deps".to_string(),
                "--quiet".to_string(),
            ],
        );

        let claude_client = self.claude_client.clone();
        let fix_handler = move |error_msg: &str, _retries: u8| {
            let client = claude_client.clone();
            let error = error_msg.to_string();
            async move {
                if let Some(client) = client {
                    // Spawn Claude agent to fix documentation issues
                    let fix_handler = ClaudeFixHandler::new(client, CLAUDE_AGENT_DOC_FIX);
                    fix_handler.attempt_fix("docs", &error).await
                } else {
                    Ok(false)
                }
            }
        };

        ValidationRunner::run_with_retry(&check, fix_handler, 3).await
    }
}
