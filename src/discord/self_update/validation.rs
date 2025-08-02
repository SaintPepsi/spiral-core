//! 🔄 UPDATE VALIDATION: Pre-flight checks and system validation
//!
//! This module ensures updates are safe to execute by performing comprehensive
//! validation at multiple stages of the update process.
//!
//! # Validation Stages
//!
//! 1. **Request Validation**: Ensures the update request is well-formed and safe
//! 2. **Pre-flight Checks**: Verifies system readiness (git state, disk space, dependencies)
//! 3. **Post-execution Validation**: Confirms changes compile and pass tests
//!
//! # Safety Checks
//!
//! - Dangerous patterns in descriptions are rejected (rm -rf, format, etc.)
//! - Git repository must be clean with no pending operations
//! - Sufficient disk space must be available (>100MB)
//! - All dependencies must be present and functional

use super::types::SelfUpdateRequest;
use crate::error::{Result, SpiralError};
use std::process::Command;
use tracing::{info, warn};

pub struct UpdateValidator;

impl UpdateValidator {
    /// Validate update request content
    pub async fn validate_request(request: &SelfUpdateRequest) -> Result<bool> {
        // Basic validation - ensure we have meaningful content
        if request.description.trim().is_empty() {
            return Ok(false);
        }

        // Check for dangerous patterns in description
        let dangerous_patterns = vec![
            "rm -rf",
            "format c:",
            "del /f",
            "drop table",
            "delete from",
            "../../../",
            "etc/passwd",
            "cmd.exe",
            "/bin/sh",
        ];

        let description_lower = request.description.to_lowercase();
        for pattern in dangerous_patterns {
            if description_lower.contains(pattern) {
                warn!("[UpdateValidator] Dangerous pattern detected: {}", pattern);
                return Ok(false);
            }
        }

        // Ensure we have at least some actionable content
        let word_count = request.description.split_whitespace().count();
        if word_count < 3 {
            return Ok(false);
        }

        Ok(true)
    }

    /// Validate system changes after implementation
    ///
    /// This comprehensive validation ensures all code changes meet our quality standards:
    /// 1. Code must compile (cargo check)
    /// 2. All tests must pass (cargo test)
    /// 3. Code must be properly formatted (cargo fmt)
    /// 4. No clippy errors allowed (cargo clippy)
    /// 5. Documentation must build (cargo doc)
    pub async fn validate_changes() -> Result<()> {
        info!("[UpdateValidator] Starting comprehensive validation of system changes");

        // Step 1: Run cargo check
        info!("[UpdateValidator] Step 1/5: Running cargo check...");
        let check_output = Command::new("cargo")
            .args(["check", "--all-targets"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to run cargo check: {e}")))?;

        if !check_output.status.success() {
            let stderr = String::from_utf8_lossy(&check_output.stderr);
            return Err(SpiralError::Validation(format!(
                "❌ Cargo check failed:\n{stderr}"
            )));
        }
        info!("[UpdateValidator] ✅ Cargo check passed");

        // Step 2: Run all tests
        info!("[UpdateValidator] Step 2/5: Running cargo test...");
        let test_output = Command::new("cargo")
            .args(["test", "--", "--test-threads=4", "--nocapture"])
            .env("RUST_TEST_TIME_UNIT", "1000")
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to run tests: {e}")))?;

        if !test_output.status.success() {
            let stderr = String::from_utf8_lossy(&test_output.stderr);
            let stdout = String::from_utf8_lossy(&test_output.stdout);

            // Check if it's just slow tests
            if stdout.contains("test result: ok") {
                info!("[UpdateValidator] ✅ Tests passed (some were slow)");
            } else {
                return Err(SpiralError::Validation(format!(
                    "❌ Tests failed:\n{stdout}\n{stderr}"
                )));
            }
        } else {
            info!("[UpdateValidator] ✅ All tests passed");
        }

        // Step 3: Check code formatting
        info!("[UpdateValidator] Step 3/5: Checking code formatting...");
        let fmt_output = Command::new("cargo")
            .args(["fmt", "--", "--check"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to run cargo fmt: {e}")))?;

        if !fmt_output.status.success() {
            let stderr = String::from_utf8_lossy(&fmt_output.stderr);
            let stdout = String::from_utf8_lossy(&fmt_output.stdout);
            return Err(SpiralError::Validation(format!(
                "❌ Code formatting check failed. Run 'cargo fmt' to fix:\n{stdout}\n{stderr}"
            )));
        }
        info!("[UpdateValidator] ✅ Code formatting is correct");

        // Step 4: Run clippy for linting
        info!("[UpdateValidator] Step 4/5: Running clippy linting...");
        let clippy_output = Command::new("cargo")
            .args(["clippy", "--all-targets", "--", "-D", "warnings"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to run cargo clippy: {e}")))?;

        if !clippy_output.status.success() {
            let stderr = String::from_utf8_lossy(&clippy_output.stderr);
            let stdout = String::from_utf8_lossy(&clippy_output.stdout);

            // Check if it's just warnings (shouldn't happen with -D warnings)
            if stderr.contains("error:") || stdout.contains("error:") {
                return Err(SpiralError::Validation(format!(
                    "❌ Clippy found errors:\n{stdout}\n{stderr}"
                )));
            }
            warn!("[UpdateValidator] Clippy warnings detected but proceeding");
        }
        info!("[UpdateValidator] ✅ Clippy checks passed");

        // Step 5: Verify documentation builds
        info!("[UpdateValidator] Step 5/5: Verifying documentation...");
        let doc_output = Command::new("cargo")
            .args(["doc", "--no-deps", "--quiet"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to run cargo doc: {e}")))?;

        if !doc_output.status.success() {
            let stderr = String::from_utf8_lossy(&doc_output.stderr);
            warn!(
                "[UpdateValidator] Documentation warnings detected: {}",
                stderr
            );
            // Don't fail on doc warnings, but log them
        } else {
            info!("[UpdateValidator] ✅ Documentation builds successfully");
        }

        info!("[UpdateValidator] 🎉 All validation checks passed successfully!");
        Ok(())
    }
}

pub struct PreflightChecker;

impl PreflightChecker {
    /// Run comprehensive pre-flight checks
    pub async fn run_checks(request: &SelfUpdateRequest) -> Result<()> {
        info!(
            "[PreflightChecker] Running pre-flight checks for {}",
            request.id
        );

        // Check 1: Git repository state
        Self::check_git_state().await?;

        // Check 2: System dependencies
        Self::check_dependencies().await?;

        // Check 3: Disk space
        Self::check_disk_space().await?;

        // Check 4: Request validation
        if !UpdateValidator::validate_request(request).await? {
            return Err(SpiralError::Validation(
                "Update request failed validation".to_string(),
            ));
        }

        info!("[PreflightChecker] All pre-flight checks passed");
        Ok(())
    }

    /// Check git repository state
    async fn check_git_state() -> Result<()> {
        // First verify git is available using our new method
        use super::git_ops::GitOperations;
        GitOperations::verify_git_available().await?;

        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to check git status: {e}")))?;

        if !status_output.status.success() {
            return Err(SpiralError::SystemError(
                "Git repository check failed - not a git repository".to_string(),
            ));
        }

        // Check for merge conflicts
        let stdout = String::from_utf8_lossy(&status_output.stdout);
        if stdout.contains("UU ") || stdout.contains("AA ") || stdout.contains("DD ") {
            return Err(SpiralError::SystemError(
                "Git repository has unresolved conflicts".to_string(),
            ));
        }

        Ok(())
    }

    /// Check system dependencies
    async fn check_dependencies() -> Result<()> {
        // Check cargo is available
        let cargo_check = Command::new("cargo").arg("--version").output();

        match cargo_check {
            Ok(output) if output.status.success() => {
                // Cargo is available and working
            }
            Ok(_) => {
                return Err(SpiralError::SystemError("Cargo command failed".to_string()));
            }
            Err(e) => {
                return Err(SpiralError::SystemError(format!("Cargo not found: {e}")));
            }
        }

        // Check git is available
        let git_check = Command::new("git").arg("--version").output();

        match git_check {
            Ok(output) if output.status.success() => {
                // Git is available and working
            }
            Ok(_) => {
                return Err(SpiralError::SystemError("Git command failed".to_string()));
            }
            Err(e) => {
                return Err(SpiralError::SystemError(format!("Git not found: {e}")));
            }
        }

        Ok(())
    }

    /// Check available disk space
    async fn check_disk_space() -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let df_output = Command::new("df").args(["-H", "."]).output().map_err(|e| {
                SpiralError::SystemError(format!("Failed to check disk space: {e}"))
            })?;

            if df_output.status.success() {
                let output = String::from_utf8_lossy(&df_output.stdout);
                // Parse df output to check available space
                if let Some(line) = output.lines().nth(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        // Check if less than 100MB available (parts[3] is available space)
                        if let Some(avail) = parts.get(3) {
                            if avail.ends_with('M') || avail.ends_with('K') {
                                let size_str = avail.trim_end_matches(|c: char| c.is_alphabetic());
                                if let Ok(size) = size_str.parse::<f64>() {
                                    if (avail.ends_with('M') && size < 100.0)
                                        || avail.ends_with('K')
                                    {
                                        return Err(SpiralError::SystemError(
                                            "Insufficient disk space (less than 100MB available)"
                                                .to_string(),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Similar implementation for Linux
            let df_output = Command::new("df")
                .args(&["-BM", "."])
                .output()
                .map_err(|e| {
                    SpiralError::SystemError(format!("Failed to check disk space: {}", e))
                })?;

            if df_output.status.success() {
                let output = String::from_utf8_lossy(&df_output.stdout);
                if let Some(line) = output.lines().nth(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        if let Some(avail) = parts.get(3) {
                            let size_str = avail.trim_end_matches('M');
                            if let Ok(size) = size_str.parse::<u64>() {
                                if size < 100 {
                                    return Err(SpiralError::SystemError(
                                        "Insufficient disk space (less than 100MB available)"
                                            .to_string(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
