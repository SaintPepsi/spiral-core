//! Pre-restart validation to ensure changes are safe before applying to live system
//!
//! This module validates changes made to the working directory BEFORE the system
//! restarts with the new code. It implements the validation pipeline with:
//! - Phase 1: Engineering Review (engineers reviewing all the work)
//! - Phase 2: Final Assembly Checklist (ticking boxes before rolling off the line)

use super::{validation_agents, SelfUpdateRequest, StructuredLogger};
use crate::{claude_code::ClaudeCodeClient, error::SpiralError, Result};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Pre-implementation validator that runs checks on modified working directory
pub struct PreImplementationValidator {
    claude_client: Option<ClaudeCodeClient>,
    max_pipeline_iterations: u32,
    max_retries_per_check: u32,
}

/// Result of pre-implementation validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreValidationResult {
    pub engineering_review_passed: bool, // Phase 1: Engineering Review
    pub assembly_checklist_passed: bool, // Phase 2: Final Assembly Checklist
    pub pipeline_iterations: u32,
    pub total_checks_run: u32,
    pub checks_failed: Vec<String>,
    pub checks_passed: Vec<String>,
    pub error_details: Option<String>,
}

impl PreValidationResult {
    /// Check if all validation passed
    pub fn all_passed(&self) -> bool {
        self.engineering_review_passed && self.assembly_checklist_passed
    }
}

/// Individual check result
#[derive(Debug, Clone)]
struct CheckResult {
    pub name: String,
    pub passed: bool,
    pub _output: String,
    pub retries_used: u32,
}

impl PreImplementationValidator {
    /// Create a new pre-implementation validator
    pub fn new(claude_client: Option<ClaudeCodeClient>) -> Self {
        Self {
            claude_client,
            max_pipeline_iterations: 3,
            max_retries_per_check: 3,
        }
    }

    /// Validate the current working directory state
    pub async fn validate_current_state(
        &self,
        request: &SelfUpdateRequest,
        logger: &mut StructuredLogger,
    ) -> Result<PreValidationResult> {
        info!(
            "[PreValidator] Starting pre-restart validation for {}",
            request.id
        );

        let mut result = PreValidationResult {
            engineering_review_passed: false,
            assembly_checklist_passed: false,
            pipeline_iterations: 0,
            total_checks_run: 0,
            checks_failed: Vec::new(),
            checks_passed: Vec::new(),
            error_details: None,
        };

        // Run pipeline with loop-back logic
        for iteration in 1..=self.max_pipeline_iterations {
            info!(
                "[PreValidator] Pipeline iteration {}/{}",
                iteration, self.max_pipeline_iterations
            );
            result.pipeline_iterations = iteration;

            // Phase 1: Engineering Review (4 parts)
            let phase1_result = self.run_engineering_review(request, logger).await;
            result.total_checks_run += 4; // Engineering Review has 4 parts

            if !phase1_result {
                warn!(
                    "[PreValidator] Engineering Review failed on iteration {}",
                    iteration
                );
                result.engineering_review_passed = false;
                result.error_details = Some(format!(
                    "Phase 1: Engineering Review failed on iteration {}",
                    iteration
                ));
                break;
            }

            result.engineering_review_passed = true;
            let _ = logger
                .log_to_phase(
                    "PreValidation",
                    &format!(
                        "Phase 1: Engineering Review passed (iteration {})",
                        iteration
                    ),
                )
                .await;

            // Phase 2: Final Assembly Checklist (5 parts)
            let (phase2_result, required_retry) =
                self.run_assembly_checklist(request, logger).await;
            result.total_checks_run += 5; // Assembly Checklist has 5 parts

            if phase2_result {
                info!(
                    "[PreValidator] Final Assembly Checklist passed on iteration {}",
                    iteration
                );
                result.assembly_checklist_passed = true;
                let _ = logger
                    .log_to_phase("PreValidation", "All validation checks passed!")
                    .await;
                break;
            }

            if required_retry && iteration < self.max_pipeline_iterations {
                warn!("[PreValidator] Assembly Checklist required retry, looping back to Engineering Review");
                continue;
            }

            // Failed after max iterations
            result.assembly_checklist_passed = false;
            result.error_details = Some(format!(
                "Validation failed after {} pipeline iterations",
                iteration
            ));
            break;
        }

        Ok(result)
    }

    /// Run Phase 1: Engineering Review - Like engineers reviewing all the work
    async fn run_engineering_review(
        &self,
        request: &SelfUpdateRequest,
        logger: &mut StructuredLogger,
    ) -> bool {
        info!("[PreValidator] Running Phase 1: Engineering Review");

        // Run the 4 parts of Engineering Review - deep quality inspection
        let checks = vec![
            self.check_code_standards(request, logger).await,
            self.check_testing_coverage(request, logger).await,
            self.check_security(request, logger).await,
            self.check_integration(request, logger).await,
        ];

        let all_passed = checks.iter().all(|check| check.passed);

        if !all_passed {
            let failed: Vec<_> = checks
                .iter()
                .filter(|c| !c.passed)
                .map(|c| c.name.clone())
                .collect();
            warn!(
                "[PreValidator] Engineering Review failed parts: {:?}",
                failed
            );
        }

        all_passed
    }

    /// Run Phase 2: Final Assembly Checklist - Ticking boxes before rolling off the line
    async fn run_assembly_checklist(
        &self,
        request: &SelfUpdateRequest,
        logger: &mut StructuredLogger,
    ) -> (bool, bool) {
        info!("[PreValidator] Running Phase 2: Final Assembly Checklist");

        let mut required_retry = false;
        let mut all_passed = true;

        // Part 1: ✓ Compilation check
        let compile_result = self.run_cargo_check(request, logger).await;
        if !compile_result.passed {
            all_passed = false;
            if compile_result.retries_used > 0 {
                required_retry = true;
            }
        }

        // Part 2: ✓ Test suite execution
        let test_result = self.run_cargo_test(request, logger).await;
        if !test_result.passed {
            all_passed = false;
            if test_result.retries_used > 0 {
                required_retry = true;
            }
        }

        // Part 3: ✓ Formatting check
        let fmt_result = self.run_cargo_fmt(request, logger).await;
        if !fmt_result.passed {
            all_passed = false;
            if fmt_result.retries_used > 0 {
                required_retry = true;
            }
        }

        // Part 4: ✓ Linting check
        let clippy_result = self.run_cargo_clippy(request, logger).await;
        if !clippy_result.passed {
            all_passed = false;
            if clippy_result.retries_used > 0 {
                required_retry = true;
            }
        }

        // Part 5: ✓ Documentation build
        let doc_result = self.run_cargo_doc(request, logger).await;
        if !doc_result.passed {
            all_passed = false;
            if doc_result.retries_used > 0 {
                required_retry = true;
            }
        }

        (all_passed, required_retry)
    }

    /// Part 1: Code Standards Review - Engineers reviewing architecture and patterns
    async fn check_code_standards(
        &self,
        request: &SelfUpdateRequest,
        logger: &mut StructuredLogger,
    ) -> CheckResult {
        debug!("[PreValidator] Checking code standards with Claude agent");

        // Use Claude agent if available
        if let Some(ref claude_client) = self.claude_client {
            // Read the agent prompt
            let agent_prompt = match validation_agents::read_agent_prompt(
                validation_agents::engineering_review::CODE_STANDARDS,
            )
            .await
            {
                Ok(prompt) => prompt,
                Err(e) => {
                    warn!(
                        "[PreValidator] Failed to read code review agent prompt: {}",
                        e
                    );
                    return self.fallback_code_standards_check(logger).await;
                }
            };

            // Get list of changed files from git
            let changed_files = match self.get_changed_files().await {
                Ok(files) => files,
                Err(e) => {
                    warn!("[PreValidator] Failed to get changed files: {}", e);
                    return self.fallback_code_standards_check(logger).await;
                }
            };

            // Build Claude request with agent prompt and changed files context
            let full_prompt = format!(
                "{}\n\n## Changed Files\n\nThe following files have been modified:\n{}\n\n## Task\n\nReview these changes for compliance with project standards.",
                agent_prompt,
                changed_files.join("\n")
            );

            let code_request = crate::claude_code::CodeGenerationRequest {
                language: "markdown".to_string(),
                description: full_prompt,
                context: std::collections::HashMap::from([
                    ("validation_type".to_string(), "code_standards".to_string()),
                    ("request_id".to_string(), request.id.clone()),
                ]),
                existing_code: None,
                requirements: vec![
                    "Review code against CODING_STANDARDS.md".to_string(),
                    "Check for SOLID violations".to_string(),
                    "Verify error handling patterns".to_string(),
                    "Check for fake implementations".to_string(),
                ],
                session_id: Some(format!("validation-{}-code-standards", request.id)),
            };

            match claude_client.generate_code(code_request).await {
                Ok(result) => {
                    // Parse Claude's response to determine if standards pass
                    let passed = result.explanation.contains("COMPLIANCE STATUS: PASS");
                    let _ = logger
                        .log_to_phase(
                            "PreValidation",
                            &format!(
                                "Code standards (Claude): {}",
                                if passed { "PASSED" } else { "FAILED" }
                            ),
                        )
                        .await;

                    CheckResult {
                        name: "Code Standards Review".to_string(),
                        passed,
                        _output: result.explanation,
                        retries_used: 0,
                    }
                }
                Err(e) => {
                    warn!("[PreValidator] Claude code standards check failed: {}", e);
                    self.fallback_code_standards_check(logger).await
                }
            }
        } else {
            // No Claude client, use fallback
            self.fallback_code_standards_check(logger).await
        }
    }

    /// Fallback code standards check when Claude is unavailable
    async fn fallback_code_standards_check(&self, logger: &mut StructuredLogger) -> CheckResult {
        debug!("[PreValidator] Using fallback code standards check");

        // Basic compilation check as fallback
        let result = Command::new("cargo")
            .args(&["check", "--workspace"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match result {
            Ok(output) => {
                let passed = output.status.success();
                let _ = logger
                    .log_to_phase(
                        "PreValidation",
                        &format!(
                            "Code standards (fallback): {}",
                            if passed { "PASSED" } else { "FAILED" }
                        ),
                    )
                    .await;

                CheckResult {
                    name: "Code Standards (Basic)".to_string(),
                    passed,
                    _output: String::from_utf8_lossy(&output.stderr).to_string(),
                    retries_used: 0,
                }
            }
            Err(e) => {
                error!(
                    "[PreValidator] Failed to run fallback code standards check: {}",
                    e
                );
                CheckResult {
                    name: "Code Standards (Basic)".to_string(),
                    passed: false,
                    _output: e.to_string(),
                    retries_used: 0,
                }
            }
        }
    }

    /// Get list of changed files from git
    async fn get_changed_files(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(&["diff", "--name-only", "HEAD"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| SpiralError::SystemError(format!("Failed to get changed files: {}", e)))?;

        if !output.status.success() {
            return Err(SpiralError::SystemError(
                "Failed to get git diff".to_string(),
            ));
        }

        let files = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();

        Ok(files)
    }

    /// Part 2: Test Coverage Analysis - Engineers ensuring critical paths are tested
    async fn check_testing_coverage(
        &self,
        request: &SelfUpdateRequest,
        logger: &mut StructuredLogger,
    ) -> CheckResult {
        debug!("[PreValidator] Checking testing coverage with Claude agent");

        if let Some(ref claude_client) = self.claude_client {
            // Read the testing analysis agent prompt
            let agent_prompt = match validation_agents::read_agent_prompt(
                validation_agents::engineering_review::TEST_COVERAGE,
            )
            .await
            {
                Ok(prompt) => prompt,
                Err(e) => {
                    warn!("[PreValidator] Failed to read testing agent prompt: {}", e);
                    return self.fallback_testing_check(logger).await;
                }
            };

            // Get changed files to focus testing analysis
            let changed_files = match self.get_changed_files().await {
                Ok(files) => files,
                Err(_) => vec![],
            };

            let full_prompt = format!(
                "{}\n\n## Changed Files\n\n{}\n\n## Task\n\nAnalyze testing coverage for critical pressure points and implement high-value tests only.",
                agent_prompt,
                changed_files.join("\n")
            );

            let code_request = crate::claude_code::CodeGenerationRequest {
                language: "rust".to_string(),
                description: full_prompt,
                context: std::collections::HashMap::from([
                    (
                        "validation_type".to_string(),
                        "testing_analysis".to_string(),
                    ),
                    ("request_id".to_string(), request.id.clone()),
                ]),
                existing_code: None,
                requirements: vec![
                    "Focus on pressure points and critical failure scenarios".to_string(),
                    "Avoid trivial tests".to_string(),
                    "Test error boundaries and edge cases".to_string(),
                ],
                session_id: Some(format!("validation-{}-testing", request.id)),
            };

            match claude_client.generate_code(code_request).await {
                Ok(result) => {
                    let passed = result
                        .explanation
                        .contains("TEST COVERAGE STATUS: ADEQUATE");
                    let _ = logger
                        .log_to_phase(
                            "PreValidation",
                            &format!(
                                "Testing analysis (Claude): {}",
                                if passed { "PASSED" } else { "FAILED" }
                            ),
                        )
                        .await;

                    CheckResult {
                        name: "Testing Coverage Analysis".to_string(),
                        passed,
                        _output: result.explanation,
                        retries_used: 0,
                    }
                }
                Err(e) => {
                    warn!("[PreValidator] Claude testing analysis failed: {}", e);
                    self.fallback_testing_check(logger).await
                }
            }
        } else {
            self.fallback_testing_check(logger).await
        }
    }

    /// Fallback testing check when Claude is unavailable
    async fn fallback_testing_check(&self, logger: &mut StructuredLogger) -> CheckResult {
        debug!("[PreValidator] Using fallback testing check");

        let result = Command::new("cargo")
            .args(&["test", "--no-run"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match result {
            Ok(output) => {
                let passed = output.status.success();
                let _ = logger
                    .log_to_phase(
                        "PreValidation",
                        &format!(
                            "Testing (fallback): {}",
                            if passed { "COMPILED" } else { "FAILED" }
                        ),
                    )
                    .await;

                CheckResult {
                    name: "Testing (Basic)".to_string(),
                    passed,
                    _output: String::from_utf8_lossy(&output.stderr).to_string(),
                    retries_used: 0,
                }
            }
            Err(e) => CheckResult {
                name: "Testing (Basic)".to_string(),
                passed: false,
                _output: e.to_string(),
                retries_used: 0,
            },
        }
    }

    /// Part 3: Security Inspection - Engineers reviewing for vulnerabilities
    async fn check_security(
        &self,
        request: &SelfUpdateRequest,
        logger: &mut StructuredLogger,
    ) -> CheckResult {
        debug!("[PreValidator] Checking security with Claude agent");

        if let Some(ref claude_client) = self.claude_client {
            // Read the security audit agent prompt
            let agent_prompt = match validation_agents::read_agent_prompt(
                validation_agents::engineering_review::SECURITY,
            )
            .await
            {
                Ok(prompt) => prompt,
                Err(e) => {
                    warn!("[PreValidator] Failed to read security agent prompt: {}", e);
                    return self.fallback_security_check(logger).await;
                }
            };

            // Get changed files and their contents for security analysis
            let changed_files = match self.get_changed_files().await {
                Ok(files) => files,
                Err(_) => vec![],
            };

            let full_prompt = format!(
                "{}\n\n## Changed Files\n\n{}\n\n## Task\n\nConduct security audit on these changes. Look for vulnerabilities, unsafe patterns, and security issues.",
                agent_prompt,
                changed_files.join("\n")
            );

            let code_request = crate::claude_code::CodeGenerationRequest {
                language: "markdown".to_string(),
                description: full_prompt,
                context: std::collections::HashMap::from([
                    ("validation_type".to_string(), "security_audit".to_string()),
                    ("request_id".to_string(), request.id.clone()),
                ]),
                existing_code: None,
                requirements: vec![
                    "Check for injection vulnerabilities".to_string(),
                    "Verify authentication and authorization".to_string(),
                    "Check for unsafe code patterns".to_string(),
                    "Audit dependencies for known CVEs".to_string(),
                ],
                session_id: Some(format!("validation-{}-security", request.id)),
            };

            match claude_client.generate_code(code_request).await {
                Ok(result) => {
                    let passed = result.explanation.contains("SECURITY STATUS: PASS");
                    let _ = logger
                        .log_to_phase(
                            "PreValidation",
                            &format!(
                                "Security audit (Claude): {}",
                                if passed { "PASSED" } else { "FAILED" }
                            ),
                        )
                        .await;

                    CheckResult {
                        name: "Security Audit".to_string(),
                        passed,
                        _output: result.explanation,
                        retries_used: 0,
                    }
                }
                Err(e) => {
                    warn!("[PreValidator] Claude security audit failed: {}", e);
                    self.fallback_security_check(logger).await
                }
            }
        } else {
            self.fallback_security_check(logger).await
        }
    }

    /// Fallback security check when Claude is unavailable
    async fn fallback_security_check(&self, logger: &mut StructuredLogger) -> CheckResult {
        debug!("[PreValidator] Using fallback security check");

        // Basic check: ensure no .env files are being tracked
        let result = Command::new("git")
            .args(&["ls-files", ".env"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match result {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let passed = output_str.trim().is_empty();

                let _ = logger
                    .log_to_phase(
                        "PreValidation",
                        &format!(
                            "Security (fallback): {}",
                            if passed { "PASSED" } else { "FAILED" }
                        ),
                    )
                    .await;

                CheckResult {
                    name: "Security (Basic)".to_string(),
                    passed,
                    _output: if passed {
                        "No .env files tracked".to_string()
                    } else {
                        "WARNING: .env file is being tracked!".to_string()
                    },
                    retries_used: 0,
                }
            }
            Err(e) => CheckResult {
                name: "Security (Basic)".to_string(),
                passed: false,
                _output: e.to_string(),
                retries_used: 0,
            },
        }
    }

    /// Part 4: Integration Review - Engineers verifying system cohesion
    async fn check_integration(
        &self,
        request: &SelfUpdateRequest,
        logger: &mut StructuredLogger,
    ) -> CheckResult {
        debug!("[PreValidator] Checking system integration with Claude agent");

        if let Some(ref claude_client) = self.claude_client {
            // Read the integration verification agent prompt
            let agent_prompt = match validation_agents::read_agent_prompt(
                validation_agents::engineering_review::INTEGRATION,
            )
            .await
            {
                Ok(prompt) => prompt,
                Err(e) => {
                    warn!(
                        "[PreValidator] Failed to read integration agent prompt: {}",
                        e
                    );
                    return self.fallback_integration_check(logger).await;
                }
            };

            let changed_files = match self.get_changed_files().await {
                Ok(files) => files,
                Err(_) => vec![],
            };

            let full_prompt = format!(
                "{}\n\n## Changed Files\n\n{}\n\n## Task\n\nVerify system integration integrity. Check that changes don't break existing functionality or APIs.",
                agent_prompt,
                changed_files.join("\n")
            );

            let code_request = crate::claude_code::CodeGenerationRequest {
                language: "markdown".to_string(),
                description: full_prompt,
                context: std::collections::HashMap::from([
                    (
                        "validation_type".to_string(),
                        "integration_verification".to_string(),
                    ),
                    ("request_id".to_string(), request.id.clone()),
                ]),
                existing_code: None,
                requirements: vec![
                    "Verify API compatibility".to_string(),
                    "Check integration points".to_string(),
                    "Ensure no breaking changes".to_string(),
                    "Validate system components work together".to_string(),
                ],
                session_id: Some(format!("validation-{}-integration", request.id)),
            };

            match claude_client.generate_code(code_request).await {
                Ok(result) => {
                    let passed = result.explanation.contains("INTEGRATION STATUS: PASS");
                    let _ = logger
                        .log_to_phase(
                            "PreValidation",
                            &format!(
                                "Integration verification (Claude): {}",
                                if passed { "PASSED" } else { "FAILED" }
                            ),
                        )
                        .await;

                    CheckResult {
                        name: "System Integration".to_string(),
                        passed,
                        _output: result.explanation,
                        retries_used: 0,
                    }
                }
                Err(e) => {
                    warn!("[PreValidator] Claude integration check failed: {}", e);
                    self.fallback_integration_check(logger).await
                }
            }
        } else {
            self.fallback_integration_check(logger).await
        }
    }

    /// Fallback integration check when Claude is unavailable
    async fn fallback_integration_check(&self, logger: &mut StructuredLogger) -> CheckResult {
        debug!("[PreValidator] Using fallback integration check");

        // Verify key integration points still compile
        let result = Command::new("cargo")
            .args(&["check", "--lib"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match result {
            Ok(output) => {
                let passed = output.status.success();
                let _ = logger
                    .log_to_phase(
                        "PreValidation",
                        &format!(
                            "Integration (fallback): {}",
                            if passed { "PASSED" } else { "FAILED" }
                        ),
                    )
                    .await;

                CheckResult {
                    name: "Integration (Basic)".to_string(),
                    passed,
                    _output: String::from_utf8_lossy(&output.stderr).to_string(),
                    retries_used: 0,
                }
            }
            Err(e) => CheckResult {
                name: "Integration (Basic)".to_string(),
                passed: false,
                _output: e.to_string(),
                retries_used: 0,
            },
        }
    }

    /// Run cargo check
    async fn run_cargo_check(
        &self,
        _request: &SelfUpdateRequest,
        logger: &mut StructuredLogger,
    ) -> CheckResult {
        debug!("[PreValidator] Running cargo check");

        let mut retries_used = 0;
        let mut last_output = String::new();

        for attempt in 1..=self.max_retries_per_check {
            let result = Command::new("cargo")
                .args(&["check", "--all-targets"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await;

            match result {
                Ok(output) => {
                    last_output = String::from_utf8_lossy(&output.stderr).to_string();

                    if output.status.success() {
                        let _ = logger
                            .log_to_phase("PreValidation", "Cargo check: PASSED")
                            .await;
                        return CheckResult {
                            name: "Cargo Check".to_string(),
                            passed: true,
                            _output: last_output,
                            retries_used,
                        };
                    }

                    if attempt < self.max_retries_per_check {
                        warn!(
                            "[PreValidator] Cargo check failed, attempt {}/{}",
                            attempt, self.max_retries_per_check
                        );
                        retries_used += 1;

                        // If Claude client is available, try to fix
                        if let Some(ref _client) = self.claude_client {
                            // TODO: Spawn Claude agent to fix compilation errors
                            warn!("[PreValidator] Claude fix not yet implemented");
                        }
                    }
                }
                Err(e) => {
                    last_output = e.to_string();
                    error!("[PreValidator] Failed to run cargo check: {}", e);
                }
            }
        }

        let _ = logger
            .log_to_phase(
                "PreValidation",
                &format!("Cargo check: FAILED after {} retries", retries_used),
            )
            .await;
        CheckResult {
            name: "Cargo Check".to_string(),
            passed: false,
            _output: last_output,
            retries_used,
        }
    }

    /// Run cargo test
    async fn run_cargo_test(
        &self,
        _request: &SelfUpdateRequest,
        logger: &mut StructuredLogger,
    ) -> CheckResult {
        debug!("[PreValidator] Running cargo test");

        let result = Command::new("cargo")
            .args(&["test", "--workspace"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match result {
            Ok(output) => {
                let passed = output.status.success();
                let _ = logger
                    .log_to_phase(
                        "PreValidation",
                        &format!("Cargo test: {}", if passed { "PASSED" } else { "FAILED" }),
                    )
                    .await;

                CheckResult {
                    name: "Cargo Test".to_string(),
                    passed,
                    _output: String::from_utf8_lossy(&output.stdout).to_string(),
                    retries_used: 0,
                }
            }
            Err(e) => CheckResult {
                name: "Cargo Test".to_string(),
                passed: false,
                _output: e.to_string(),
                retries_used: 0,
            },
        }
    }

    /// Run cargo fmt check
    async fn run_cargo_fmt(
        &self,
        _request: &SelfUpdateRequest,
        logger: &mut StructuredLogger,
    ) -> CheckResult {
        debug!("[PreValidator] Running cargo fmt");

        let result = Command::new("cargo")
            .args(&["fmt", "--", "--check"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match result {
            Ok(output) => {
                let passed = output.status.success();

                if !passed {
                    // Auto-fix by running cargo fmt
                    let _ = Command::new("cargo").args(&["fmt"]).output().await;

                    let _ = logger
                        .log_to_phase("PreValidation", "Cargo fmt: AUTO-FIXED")
                        .await;

                    return CheckResult {
                        name: "Cargo Format".to_string(),
                        passed: true,
                        _output: "Formatting issues auto-fixed".to_string(),
                        retries_used: 1,
                    };
                }

                let _ = logger
                    .log_to_phase("PreValidation", "Cargo fmt: PASSED")
                    .await;
                CheckResult {
                    name: "Cargo Format".to_string(),
                    passed: true,
                    _output: String::new(),
                    retries_used: 0,
                }
            }
            Err(e) => CheckResult {
                name: "Cargo Format".to_string(),
                passed: false,
                _output: e.to_string(),
                retries_used: 0,
            },
        }
    }

    /// Run cargo clippy
    async fn run_cargo_clippy(
        &self,
        _request: &SelfUpdateRequest,
        logger: &mut StructuredLogger,
    ) -> CheckResult {
        debug!("[PreValidator] Running cargo clippy");

        let result = Command::new("cargo")
            .args(&["clippy", "--all-targets", "--", "-D", "warnings"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match result {
            Ok(output) => {
                let passed = output.status.success();
                let _ = logger
                    .log_to_phase(
                        "PreValidation",
                        &format!("Cargo clippy: {}", if passed { "PASSED" } else { "FAILED" }),
                    )
                    .await;

                CheckResult {
                    name: "Cargo Clippy".to_string(),
                    passed,
                    _output: String::from_utf8_lossy(&output.stderr).to_string(),
                    retries_used: 0,
                }
            }
            Err(e) => CheckResult {
                name: "Cargo Clippy".to_string(),
                passed: false,
                _output: e.to_string(),
                retries_used: 0,
            },
        }
    }

    /// Run cargo doc
    async fn run_cargo_doc(
        &self,
        _request: &SelfUpdateRequest,
        logger: &mut StructuredLogger,
    ) -> CheckResult {
        debug!("[PreValidator] Running cargo doc");

        let result = Command::new("cargo")
            .args(&["doc", "--no-deps", "--workspace"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match result {
            Ok(output) => {
                let passed = output.status.success();
                let _ = logger
                    .log_to_phase(
                        "PreValidation",
                        &format!("Cargo doc: {}", if passed { "PASSED" } else { "FAILED" }),
                    )
                    .await;

                CheckResult {
                    name: "Cargo Doc".to_string(),
                    passed,
                    _output: String::from_utf8_lossy(&output.stderr).to_string(),
                    retries_used: 0,
                }
            }
            Err(e) => CheckResult {
                name: "Cargo Doc".to_string(),
                passed: false,
                _output: e.to_string(),
                retries_used: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pre_validator_creation() {
        let validator = PreImplementationValidator::new(None);
        assert_eq!(validator.max_pipeline_iterations, 3);
        assert_eq!(validator.max_retries_per_check, 3);
    }

    #[test]
    fn test_validation_result_all_passed() {
        let mut result = PreValidationResult {
            engineering_review_passed: true,
            assembly_checklist_passed: true,
            pipeline_iterations: 1,
            total_checks_run: 9,
            checks_failed: vec![],
            checks_passed: vec!["all".to_string()],
            error_details: None,
        };
        assert!(result.all_passed());

        result.engineering_review_passed = false;
        assert!(!result.all_passed());
    }
}
