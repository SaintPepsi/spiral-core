//! 🔄 GIT OPERATIONS: Safe git snapshot and rollback functionality
//!
//! This module provides safe wrappers around git operations for creating
//! system snapshots and performing rollbacks when updates fail.
//!
//! # Security
//!
//! All inputs are sanitized to prevent command injection:
//! - Codenames are restricted to alphanumeric characters plus hyphens/underscores
//! - No shell expansion or special characters are allowed
//! - All git commands use explicit argument arrays, not shell strings
//!
//! # Snapshot Strategy
//!
//! Each update creates a git commit with a unique identifier. This allows:
//! - Quick rollback to any previous state
//! - Audit trail of all system changes
//! - Recovery from failed updates without data loss

use crate::error::{Result, SpiralError};
use std::process::Command;
use tracing::{info, warn};

pub struct GitOperations;

impl GitOperations {
    /// Verify git is available and repository is valid
    pub async fn verify_git_available() -> Result<()> {
        // Check if git command exists
        let version_output = Command::new("git")
            .args(["--version"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Git not found: {e}")))?;

        if !version_output.status.success() {
            return Err(SpiralError::SystemError("Git command failed".to_string()));
        }

        // Check if we're in a git repository
        let rev_parse_output = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to check git repo: {e}")))?;

        if !rev_parse_output.status.success() {
            return Err(SpiralError::SystemError(
                "Not in a git repository".to_string(),
            ));
        }

        Ok(())
    }

    /// Create a git snapshot with sanitized branch name
    pub async fn create_snapshot(codename: &str) -> Result<String> {
        let safe_codename = Self::sanitize_codename(codename)?;
        let snapshot_id = format!(
            "pre-update-snapshot-{}-{}",
            safe_codename,
            chrono::Utc::now().timestamp()
        );

        info!("[GitOps] Creating snapshot: {}", snapshot_id);

        // Check git status first
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to check git status: {e}")))?;

        if !status_output.status.success() {
            return Err(SpiralError::SystemError(
                "Git repository not found or corrupted".to_string(),
            ));
        }

        // Stage all changes
        let add_output = Command::new("git")
            .args(["add", "-A"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to stage changes: {e}")))?;

        if !add_output.status.success() {
            let stderr = String::from_utf8_lossy(&add_output.stderr);
            warn!("[GitOps] Failed to stage changes: {}", stderr);
        }

        // Create commit as snapshot
        let commit_message = format!("Auto-update snapshot: {safe_codename}");
        let commit_output = Command::new("git")
            .args(["commit", "-m", &commit_message, "--allow-empty"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to create commit: {e}")))?;

        if !commit_output.status.success() {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            return Err(SpiralError::SystemError(format!(
                "Failed to create snapshot commit: {stderr}"
            )));
        }

        // Get the commit hash
        let hash_output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to get commit hash: {e}")))?;

        if hash_output.status.success() {
            let commit_hash = String::from_utf8_lossy(&hash_output.stdout)
                .trim()
                .to_string();
            info!(
                "[GitOps] Created snapshot {} with commit {}",
                snapshot_id, commit_hash
            );
            Ok(snapshot_id)
        } else {
            Err(SpiralError::SystemError(
                "Failed to retrieve snapshot commit hash".to_string(),
            ))
        }
    }

    /// Rollback to a previous snapshot
    pub async fn rollback_to_snapshot(snapshot_id: &str) -> Result<()> {
        info!("[GitOps] Rolling back to snapshot: {}", snapshot_id);

        // Validate snapshot ID format and sanitize
        if !snapshot_id.starts_with("pre-update-snapshot-") {
            return Err(SpiralError::Validation(
                "Invalid snapshot ID format".to_string(),
            ));
        }

        // Additional safety: ensure snapshot_id doesn't contain shell metacharacters
        if snapshot_id.contains(
            &[
                '$', '`', '\\', '"', '\'', ';', '&', '|', '<', '>', '(', ')', '{', '}', '[', ']',
                '*', '?', '~',
            ][..],
        ) {
            return Err(SpiralError::Validation(
                "Invalid characters in snapshot ID".to_string(),
            ));
        }

        // Find the commit with this snapshot ID in the message
        let log_output = Command::new("git")
            .args(["log", "--oneline", "--grep", snapshot_id, "-n", "1"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to search git log: {e}")))?;

        if !log_output.status.success() || log_output.stdout.is_empty() {
            return Err(SpiralError::NotFound(format!(
                "Snapshot {snapshot_id} not found"
            )));
        }

        let log_line = String::from_utf8_lossy(&log_output.stdout);
        let commit_hash = log_line
            .split_whitespace()
            .next()
            .ok_or_else(|| SpiralError::SystemError("Failed to parse commit hash".to_string()))?;

        // First, stash any uncommitted changes as a safety measure
        let stash_output = Command::new("git")
            .args([
                "stash",
                "push",
                "-m",
                &format!("Pre-rollback stash for {snapshot_id}"),
            ])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to stash changes: {e}")))?;

        if !stash_output.status.success() {
            warn!("[GitOps] No changes to stash before rollback");
        }

        // Perform hard reset to the snapshot commit
        let reset_output = Command::new("git")
            .args(["reset", "--hard", commit_hash])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to reset to snapshot: {e}")))?;

        if reset_output.status.success() {
            info!(
                "[GitOps] Successfully rolled back to snapshot {}",
                snapshot_id
            );
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&reset_output.stderr);
            Err(SpiralError::SystemError(format!(
                "Failed to rollback: {stderr}"
            )))
        }
    }

    /// Commit validated changes with descriptive message
    pub async fn commit_validated_changes(codename: &str, description: &str) -> Result<String> {
        let safe_codename = Self::sanitize_codename(codename)?;
        
        info!("[GitOps] Committing validated changes for {}", safe_codename);
        
        // Stage all changes
        let add_output = Command::new("git")
            .args(["add", "-A"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to stage changes: {e}")))?;
        
        if !add_output.status.success() {
            let stderr = String::from_utf8_lossy(&add_output.stderr);
            return Err(SpiralError::SystemError(format!(
                "Failed to stage changes: {stderr}"
            )));
        }
        
        // Check if there are changes to commit
        let status_output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to check git status: {e}")))?;
        
        if status_output.stdout.is_empty() {
            warn!("[GitOps] No changes to commit");
            return Ok("No changes to commit".to_string());
        }
        
        // Create descriptive commit message
        let commit_message = format!(
            "🔄 Self-update: {}\n\nCodename: {}\nDescription: {}\nValidated: ✅\nTimestamp: {}",
            safe_codename,
            safe_codename,
            description,
            chrono::Utc::now().to_rfc3339()
        );
        
        // Create commit
        let commit_output = Command::new("git")
            .args(["commit", "-m", &commit_message])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to create commit: {e}")))?;
        
        if !commit_output.status.success() {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            return Err(SpiralError::SystemError(format!(
                "Failed to commit changes: {stderr}"
            )));
        }
        
        // Get commit hash
        let hash_output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to get commit hash: {e}")))?;
        
        if hash_output.status.success() {
            let commit_hash = String::from_utf8_lossy(&hash_output.stdout)
                .trim()
                .to_string();
            info!("[GitOps] Committed changes with hash: {}", commit_hash);
            Ok(commit_hash)
        } else {
            Err(SpiralError::SystemError(
                "Failed to retrieve commit hash".to_string(),
            ))
        }
    }
    
    /// Push committed changes to remote repository
    pub async fn push_to_remote(branch: Option<&str>) -> Result<()> {
        info!("[GitOps] Pushing changes to remote repository");
        
        // Get current branch if not specified
        let current_branch = if let Some(branch) = branch {
            branch.to_string()
        } else {
            let branch_output = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .output()
                .map_err(|e| SpiralError::SystemError(format!("Failed to get current branch: {e}")))?;
            
            if !branch_output.status.success() {
                return Err(SpiralError::SystemError(
                    "Failed to determine current branch".to_string(),
                ));
            }
            
            String::from_utf8_lossy(&branch_output.stdout)
                .trim()
                .to_string()
        };
        
        info!("[GitOps] Pushing to branch: {}", current_branch);
        
        // Push to remote
        let push_output = Command::new("git")
            .args(["push", "origin", &current_branch])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to push to remote: {e}")))?;
        
        if push_output.status.success() {
            info!("[GitOps] Successfully pushed changes to remote");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&push_output.stderr);
            
            // Check if it's just because we're up to date
            if stderr.contains("Everything up-to-date") {
                info!("[GitOps] Remote is already up-to-date");
                Ok(())
            } else {
                Err(SpiralError::SystemError(format!(
                    "Failed to push to remote: {stderr}"
                )))
            }
        }
    }
    
    /// Check if there are unpushed commits
    pub async fn has_unpushed_commits() -> Result<bool> {
        let output = Command::new("git")
            .args(["status", "-sb"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to check git status: {e}")))?;
        
        if output.status.success() {
            let status = String::from_utf8_lossy(&output.stdout);
            // Check if status contains "ahead" which indicates unpushed commits
            Ok(status.contains("ahead"))
        } else {
            Err(SpiralError::SystemError(
                "Failed to check for unpushed commits".to_string(),
            ))
        }
    }
    
    /// Sanitize codename for safe use in git operations
    fn sanitize_codename(codename: &str) -> Result<String> {
        // Remove any characters that could be dangerous in git commands
        let safe_name = codename
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>()
            .to_lowercase();

        // Ensure it's not empty and doesn't start with dash
        if safe_name.is_empty() {
            return Err(SpiralError::Validation(
                "Codename cannot be empty".to_string(),
            ));
        }

        if safe_name.starts_with('-') {
            return Err(SpiralError::Validation(
                "Codename cannot start with dash".to_string(),
            ));
        }

        // Limit length
        if safe_name.len() > 32 {
            Ok(safe_name.chars().take(32).collect())
        } else {
            Ok(safe_name)
        }
    }
}

pub struct SnapshotManager;

impl SnapshotManager {
    /// List available snapshots
    pub async fn list_snapshots() -> Result<Vec<String>> {
        let log_output = Command::new("git")
            .args(["log", "--grep=pre-update-snapshot", "--oneline", "-n", "20"])
            .output()
            .map_err(|e| SpiralError::SystemError(format!("Failed to list snapshots: {e}")))?;

        if log_output.status.success() {
            let snapshots = String::from_utf8_lossy(&log_output.stdout)
                .lines()
                .filter_map(|line| {
                    // Extract snapshot ID from commit message
                    if let Some(idx) = line.find("pre-update-snapshot-") {
                        let snapshot_part = &line[idx..];
                        let snapshot_id = snapshot_part.split_whitespace().next()?;
                        Some(snapshot_id.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(snapshots)
        } else {
            Ok(Vec::new())
        }
    }

    /// Clean up old snapshots (keep last N)
    pub async fn cleanup_old_snapshots(keep_count: usize) -> Result<()> {
        let snapshots = Self::list_snapshots().await?;

        if snapshots.len() <= keep_count {
            info!("[GitOps] No snapshots to clean up");
            return Ok(());
        }

        // Git log returns newest first, so skip the first keep_count
        let to_remove = snapshots.into_iter().skip(keep_count).collect::<Vec<_>>();

        info!(
            "[GitOps] Found {} old snapshots that could be cleaned up",
            to_remove.len()
        );
        info!("[GitOps] Snapshot cleanup is currently disabled for safety");
        // Note: We intentionally don't implement automatic cleanup to preserve history
        // Manual cleanup can be done via git commands if needed

        Ok(())
    }
}
