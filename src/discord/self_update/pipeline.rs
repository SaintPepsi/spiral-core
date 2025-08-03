//! Two-phase validation pipeline implementation
//!
//! Implements the validation pipeline described in SELF_UPDATE_PIPELINE_IMPROVEMENT.md:
//! - Phase 1: Advanced Quality Assurance (AQA)
//! - Phase 2: Core Rust Compliance Checks (CRCC)
//! - Pipeline looping: ANY Phase 2 retry triggers return to Phase 1
//! - Maximum 3 complete pipeline iterations

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Maximum number of complete pipeline iterations allowed
const MAX_PIPELINE_ITERATIONS: u8 = 3;

/// Timeout for individual agent operations (5 minutes)
const AGENT_TIMEOUT: Duration = Duration::from_secs(300);

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

/// Main validation pipeline coordinator
pub struct ValidationPipeline {
    context: PipelineContext,
    start_time: Instant,
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
        }
    }

    /// Execute the complete validation pipeline
    pub async fn execute(&mut self) -> Result<PipelineContext> {
        info!("[ValidationPipeline] Starting two-phase validation pipeline");

        // Main pipeline loop
        while self.context.pipeline_iterations < MAX_PIPELINE_ITERATIONS {
            self.context.pipeline_iterations += 1;
            info!(
                "[ValidationPipeline] Starting pipeline iteration {}",
                self.context.pipeline_iterations
            );

            // Phase 1: Advanced Quality Assurance
            self.execute_phase1().await?;

            // Check if Phase 1 failed critically
            if self.has_critical_phase1_failures() {
                self.context.final_status = PipelineStatus::Failure;
                break;
            }

            // Phase 2: Core Rust Compliance Checks
            let phase2_passed = self.execute_phase2().await?;

            if phase2_passed {
                // Success!
                self.context.final_status = if self.context.pipeline_iterations == 1 {
                    PipelineStatus::Success
                } else {
                    PipelineStatus::SuccessWithRetries
                };
                break;
            }

            // Phase 2 failed - loop back to Phase 1 if we have iterations left
            if self.context.pipeline_iterations >= MAX_PIPELINE_ITERATIONS {
                warn!("[ValidationPipeline] Maximum iterations reached - pipeline failed");
                self.context.final_status = PipelineStatus::Failure;
                break;
            }
        }

        // Calculate total duration
        self.context.total_duration_ms = self.start_time.elapsed().as_millis() as u64;

        // Analyze patterns
        self.analyze_patterns();

        // Run appropriate analysis agent based on outcome
        self.run_analysis_agent().await?;

        info!(
            "[ValidationPipeline] Pipeline completed with status: {:?}",
            self.context.final_status
        );

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
        info!("[ValidationPipeline] Executing Phase 2: Core Rust Compliance Checks");

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
    async fn run_analysis_agent(&self) -> Result<()> {
        match self.context.final_status {
            PipelineStatus::Success => {
                info!("[ValidationPipeline] Running success analyzer agent");
                self.spawn_claude_agent(
                    ".claude/validation-agents/analysis/success-analyzer.md",
                    &self.context,
                )
                .await?;
            }
            PipelineStatus::SuccessWithRetries => {
                info!("[ValidationPipeline] Running success-with-issues analyzer agent");
                self.spawn_claude_agent(
                    ".claude/validation-agents/analysis/success-with-issues-analyzer.md",
                    &self.context,
                )
                .await?;
            }
            PipelineStatus::Failure => {
                info!("[ValidationPipeline] Running failure analyzer agent");
                self.spawn_claude_agent(
                    ".claude/validation-agents/analysis/failure-analyzer.md",
                    &self.context,
                )
                .await?;
            }
        }
        Ok(())
    }

    /// Spawn a Claude Code agent with the given prompt file and context
    async fn spawn_claude_agent(&self, agent_path: &str, _context: &PipelineContext) -> Result<()> {
        // TODO: Implement actual Claude Code integration
        // For now, this is a placeholder that logs the intent
        info!(
            "[ValidationPipeline] Would spawn Claude agent: {} with context",
            agent_path
        );

        // In real implementation:
        // 1. Read the agent prompt from agent_path
        // 2. Serialize the context to JSON
        // 3. Call Claude Code API with prompt + context
        // 4. Process response and apply any recommended changes

        Ok(())
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

    async fn run_compilation_check(&self) -> Result<ComplianceCheck> {
        info!("[Phase2] Running compilation check");

        let output = Command::new("cargo")
            .args(["check", "--all-targets"])
            .output()
            .map_err(|e| {
                crate::error::SpiralError::SystemError(format!("Failed to run cargo check: {}", e))
            })?;

        if output.status.success() {
            return Ok(ComplianceCheck {
                passed: true,
                retries: 0,
                errors: None,
            });
        }

        // Compilation failed - spawn Claude agent to fix
        let errors = String::from_utf8_lossy(&output.stderr);
        warn!("[Phase2] Compilation check failed: {}", errors);

        // TODO: Actually spawn Claude agent with compilation-fixer.md
        // For now, simulate that we tried once

        Ok(ComplianceCheck {
            passed: false,
            retries: 1,
            errors: Some(vec![errors.to_string()]),
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
