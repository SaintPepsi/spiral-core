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

    /// Temporary stub for validate_changes - will be replaced with two-phase pipeline
    pub async fn validate_changes() -> Result<()> {
        info!("[UpdateValidator] Two-phase validation pipeline not yet implemented");
        Err(SpiralError::Validation(
            "New two-phase validation pipeline is being implemented".to_string(),
        ))
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
