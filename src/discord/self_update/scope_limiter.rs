//! Change scope limiter to prevent runaway modifications
//!
//! This module enforces limits on the scope of changes that can be made
//! during a self-update to prevent accidental or malicious damage.

use crate::error::SpiralError;
use crate::Result;
use std::path::Path;
use tracing::{info, warn};

/// Limits for various types of changes
#[derive(Debug, Clone)]
pub struct ScopeLimits {
    /// Maximum number of files that can be modified
    pub max_files_modified: usize,
    /// Maximum number of lines changed per file
    pub max_lines_per_file: usize,
    /// Maximum total lines changed across all files
    pub max_total_lines: usize,
    /// Maximum number of new files created
    pub max_files_created: usize,
    /// Maximum number of files deleted
    pub max_files_deleted: usize,
    /// Directories that should never be modified
    pub protected_paths: Vec<String>,
    /// File extensions that require extra scrutiny
    pub sensitive_extensions: Vec<String>,
}

impl Default for ScopeLimits {
    fn default() -> Self {
        Self {
            max_files_modified: 20,
            max_lines_per_file: 500,
            max_total_lines: 2000,
            max_files_created: 10,
            max_files_deleted: 5,
            protected_paths: vec![
                ".git".to_string(),
                ".env".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
                ".claude".to_string(), // Protect validation agents
            ],
            sensitive_extensions: vec![
                ".env".to_string(),
                ".key".to_string(),
                ".pem".to_string(),
                ".cert".to_string(),
                ".secret".to_string(),
            ],
        }
    }
}

/// Tracks changes being made during an update
#[derive(Debug, Default)]
pub struct ChangeScope {
    /// Files that have been modified
    pub modified_files: Vec<String>,
    /// Files that have been created
    pub created_files: Vec<String>,
    /// Files that have been deleted  
    pub deleted_files: Vec<String>,
    /// Total lines changed across all files
    pub total_lines_changed: usize,
    /// Lines changed per file
    pub lines_per_file: std::collections::HashMap<String, usize>,
}

impl ChangeScope {
    /// Create a new change scope tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a file modification
    pub fn record_modification(&mut self, file_path: &str, lines_changed: usize) {
        if !self.modified_files.contains(&file_path.to_string()) {
            self.modified_files.push(file_path.to_string());
        }
        self.lines_per_file
            .insert(file_path.to_string(), lines_changed);
        self.total_lines_changed += lines_changed;
    }

    /// Record a file creation
    pub fn record_creation(&mut self, file_path: &str) {
        self.created_files.push(file_path.to_string());
    }

    /// Record a file deletion
    pub fn record_deletion(&mut self, file_path: &str) {
        self.deleted_files.push(file_path.to_string());
    }
}

/// Validates that changes are within acceptable scope limits
pub struct ScopeLimiter {
    limits: ScopeLimits,
}

impl ScopeLimiter {
    /// Create a new scope limiter with default limits
    pub fn new() -> Self {
        Self {
            limits: ScopeLimits::default(),
        }
    }

    /// Create a scope limiter with custom limits
    pub fn with_limits(limits: ScopeLimits) -> Self {
        Self { limits }
    }

    /// Check if a path is protected
    fn is_protected_path(&self, path: &str) -> bool {
        let path = Path::new(path);

        for protected in &self.limits.protected_paths {
            if path.starts_with(protected)
                || path
                    .components()
                    .any(|c| c.as_os_str() == protected.as_str())
            {
                return true;
            }
        }

        false
    }

    /// Check if a file has a sensitive extension
    fn has_sensitive_extension(&self, path: &str) -> bool {
        for ext in &self.limits.sensitive_extensions {
            if path.ends_with(ext) {
                return true;
            }
        }
        false
    }

    /// Validate that changes are within scope limits
    pub fn validate_scope(&self, scope: &ChangeScope) -> Result<()> {
        // Check file modification limits
        if scope.modified_files.len() > self.limits.max_files_modified {
            return Err(SpiralError::Validation(format!(
                "Too many files modified: {} (max: {})",
                scope.modified_files.len(),
                self.limits.max_files_modified
            )));
        }

        // Check file creation limits
        if scope.created_files.len() > self.limits.max_files_created {
            return Err(SpiralError::Validation(format!(
                "Too many files created: {} (max: {})",
                scope.created_files.len(),
                self.limits.max_files_created
            )));
        }

        // Check file deletion limits
        if scope.deleted_files.len() > self.limits.max_files_deleted {
            return Err(SpiralError::Validation(format!(
                "Too many files deleted: {} (max: {})",
                scope.deleted_files.len(),
                self.limits.max_files_deleted
            )));
        }

        // Check total lines changed
        if scope.total_lines_changed > self.limits.max_total_lines {
            return Err(SpiralError::Validation(format!(
                "Too many total lines changed: {} (max: {})",
                scope.total_lines_changed, self.limits.max_total_lines
            )));
        }

        // Check lines per file
        for (file, lines) in &scope.lines_per_file {
            if *lines > self.limits.max_lines_per_file {
                return Err(SpiralError::Validation(format!(
                    "Too many lines changed in {}: {} (max: {})",
                    file, lines, self.limits.max_lines_per_file
                )));
            }
        }

        // Check for protected paths
        for file in &scope.modified_files {
            if self.is_protected_path(file) {
                return Err(SpiralError::Validation(format!(
                    "Attempted to modify protected path: {}",
                    file
                )));
            }
        }

        for file in &scope.deleted_files {
            if self.is_protected_path(file) {
                return Err(SpiralError::Validation(format!(
                    "Attempted to delete protected path: {}",
                    file
                )));
            }
        }

        // Warn about sensitive files (but don't block)
        for file in &scope.modified_files {
            if self.has_sensitive_extension(file) {
                warn!("Modifying sensitive file: {}", file);
            }
        }

        for file in &scope.created_files {
            if self.has_sensitive_extension(file) {
                warn!("Creating sensitive file: {}", file);
            }
        }

        info!(
            "Scope validation passed: {} files modified, {} created, {} deleted, {} total lines",
            scope.modified_files.len(),
            scope.created_files.len(),
            scope.deleted_files.len(),
            scope.total_lines_changed
        );

        Ok(())
    }

    /// Analyze git diff to build change scope
    pub async fn analyze_diff(&self, diff_output: &str) -> Result<ChangeScope> {
        let mut scope = ChangeScope::new();

        // Parse git diff output
        let mut current_file: Option<String> = None;
        let mut is_new_file = false;
        let mut is_deleted_file = false;
        let mut additions = 0;
        let mut deletions = 0;

        for line in diff_output.lines() {
            if line.starts_with("diff --git") {
                // Save previous file stats
                if let Some(ref file) = current_file {
                    if !is_new_file && !is_deleted_file && (additions > 0 || deletions > 0) {
                        let total_changes = additions + deletions;
                        scope.record_modification(file, total_changes);
                    }
                    additions = 0;
                    deletions = 0;
                    is_new_file = false;
                    is_deleted_file = false;
                }

                // Extract file path from diff header
                if let Some(path) = line.split(" b/").nth(1) {
                    current_file = Some(path.to_string());
                }
            } else if line.starts_with("new file mode") {
                if let Some(ref file) = current_file {
                    scope.record_creation(file);
                    is_new_file = true;
                }
            } else if line.starts_with("deleted file mode") {
                if let Some(ref file) = current_file {
                    scope.record_deletion(file);
                    is_deleted_file = true;
                }
            } else if line.starts_with('+') && !line.starts_with("+++") {
                additions += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                deletions += 1;
            }
        }

        // Save last file stats
        if let Some(ref file) = current_file {
            if !is_new_file && !is_deleted_file && (additions > 0 || deletions > 0) {
                let total_changes = additions + deletions;
                scope.record_modification(file, total_changes);
            }
        }

        // Validate the scope
        self.validate_scope(&scope)?;

        Ok(scope)
    }
}

impl Default for ScopeLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_limits_validation() {
        let limiter = ScopeLimiter::new();
        let mut scope = ChangeScope::new();

        // Should pass with no changes
        assert!(limiter.validate_scope(&scope).is_ok());

        // Add some changes within limits
        scope.record_modification("src/main.rs", 50);
        scope.record_creation("src/new_module.rs");
        assert!(limiter.validate_scope(&scope).is_ok());

        // Exceed file modification limit
        for i in 0..25 {
            scope.record_modification(&format!("src/file_{}.rs", i), 10);
        }
        assert!(limiter.validate_scope(&scope).is_err());
    }

    #[test]
    fn test_protected_paths() {
        let limiter = ScopeLimiter::new();
        let mut scope = ChangeScope::new();

        // Should fail when modifying protected path
        scope.record_modification(".git/config", 10);
        assert!(limiter.validate_scope(&scope).is_err());

        // Should fail when deleting protected path
        let mut scope = ChangeScope::new();
        scope.record_deletion(".env");
        assert!(limiter.validate_scope(&scope).is_err());
    }

    #[test]
    fn test_diff_parsing() {
        let diff = r#"diff --git a/src/main.rs b/src/main.rs
index 123..456 789
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,5 +1,6 @@
+use std::io;
 fn main() {
-    println!("Hello");
+    println!("Hello, world!");
+    let input = io::stdin();
 }
diff --git a/src/new.rs b/src/new.rs
new file mode 100644
index 0000000..1234567
--- /dev/null
+++ b/src/new.rs
@@ -0,0 +1,3 @@
+fn new_function() {
+    // New code
+}"#;

        let limiter = ScopeLimiter::new();
        let scope = tokio_test::block_on(limiter.analyze_diff(diff));

        assert!(scope.is_ok());
        let scope = scope.unwrap();
        assert_eq!(scope.modified_files.len(), 1); // main.rs was modified
        assert_eq!(scope.created_files.len(), 1); // new.rs was created
        assert_eq!(scope.deleted_files.len(), 0);
    }
}
