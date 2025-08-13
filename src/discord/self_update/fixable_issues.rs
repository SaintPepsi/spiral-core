//! 🔧 FIXABLE ISSUES: Tracking system for auto-fixable problems
//!
//! This module tracks issues that the self-update system could automatically fix.
//! When warnings or errors occur that have known solutions, they're logged here
//! for potential automated resolution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Maximum number of issues to track
const MAX_TRACKED_ISSUES: usize = 100;

/// Categories of fixable issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueCategory {
    /// Discord message size limits
    MessageTooLarge,
    /// Compilation warnings
    CompilationWarning,
    /// Formatting issues
    FormattingIssue,
    /// Clippy lints
    ClippyWarning,
    /// Test failures
    TestFailure,
    /// Documentation issues
    DocumentationIssue,
    /// Performance issues
    PerformanceIssue,
    /// Security vulnerabilities
    SecurityIssue,
    /// Other categorized issues
    Other(String),
}

/// A fixable issue that was detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixableIssue {
    /// Unique identifier for this issue
    pub id: String,
    /// Category of the issue
    pub category: IssueCategory,
    /// Description of the issue
    pub description: String,
    /// Suggested fix or action
    pub suggested_fix: String,
    /// Context (e.g., file path, function name)
    pub context: Option<String>,
    /// When the issue was detected
    pub timestamp: DateTime<Utc>,
    /// Severity level (1-5, 5 being most severe)
    pub severity: u8,
    /// Whether this has been attempted to fix
    pub fix_attempted: bool,
}

/// Tracker for fixable issues
#[derive(Clone)]
pub struct FixableIssueTracker {
    issues: Arc<RwLock<VecDeque<FixableIssue>>>,
}

impl Default for FixableIssueTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl FixableIssueTracker {
    pub fn new() -> Self {
        Self {
            issues: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Log a new fixable issue
    pub async fn log_issue(&self, issue: FixableIssue) {
        let mut issues = self.issues.write().await;

        // Log the issue for visibility
        match issue.severity {
            5 => warn!(
                "🚨 CRITICAL fixable issue detected: {} - {}",
                issue.category_str(),
                issue.description
            ),
            4 => warn!(
                "⚠️ HIGH priority fixable issue: {} - {}",
                issue.category_str(),
                issue.description
            ),
            3 => info!(
                "🔧 MEDIUM fixable issue: {} - {}",
                issue.category_str(),
                issue.description
            ),
            _ => info!(
                "💡 LOW priority fixable issue: {} - {}",
                issue.category_str(),
                issue.description
            ),
        }

        // Add to tracking queue
        issues.push_back(issue);

        // Keep queue bounded
        while issues.len() > MAX_TRACKED_ISSUES {
            issues.pop_front();
        }
    }

    /// Get all current fixable issues
    pub async fn get_issues(&self) -> Vec<FixableIssue> {
        let issues = self.issues.read().await;
        issues.iter().cloned().collect()
    }

    /// Get issues by category
    pub async fn get_issues_by_category(&self, category: &IssueCategory) -> Vec<FixableIssue> {
        let issues = self.issues.read().await;
        issues
            .iter()
            .filter(|issue| &issue.category == category)
            .cloned()
            .collect()
    }

    /// Get high priority issues (severity >= 4)
    pub async fn get_high_priority_issues(&self) -> Vec<FixableIssue> {
        let issues = self.issues.read().await;
        issues
            .iter()
            .filter(|issue| issue.severity >= 4 && !issue.fix_attempted)
            .cloned()
            .collect()
    }

    /// Mark an issue as fix attempted
    pub async fn mark_fix_attempted(&self, issue_id: &str) {
        let mut issues = self.issues.write().await;
        if let Some(issue) = issues.iter_mut().find(|i| i.id == issue_id) {
            issue.fix_attempted = true;
        }
    }

    /// Clear all tracked issues
    pub async fn clear_issues(&self) {
        let mut issues = self.issues.write().await;
        issues.clear();
    }

    /// Generate a summary report of current issues
    pub async fn generate_report(&self) -> String {
        let issues = self.issues.read().await;

        if issues.is_empty() {
            return "✅ No fixable issues currently tracked".to_string();
        }

        let mut report = format!("📊 **Fixable Issues Report** ({} issues)\n\n", issues.len());

        // Group by category
        let mut by_category: std::collections::HashMap<String, Vec<&FixableIssue>> =
            std::collections::HashMap::new();

        for issue in issues.iter() {
            by_category
                .entry(issue.category_str())
                .or_default()
                .push(issue);
        }

        for (category, category_issues) in by_category {
            report.push_str(&format!(
                "**{}** ({} issues)\n",
                category,
                category_issues.len()
            ));
            for issue in category_issues.iter().take(3) {
                report.push_str(&format!(
                    "  • [Sev {}] {}\n",
                    issue.severity, issue.description
                ));
            }
            if category_issues.len() > 3 {
                report.push_str(&format!("  • ...and {} more\n", category_issues.len() - 3));
            }
            report.push('\n');
        }

        report
    }
}

impl FixableIssue {
    /// Create a new fixable issue
    pub fn new(
        category: IssueCategory,
        description: impl Into<String>,
        suggested_fix: impl Into<String>,
        severity: u8,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            category,
            description: description.into(),
            suggested_fix: suggested_fix.into(),
            context: None,
            timestamp: Utc::now(),
            severity: severity.min(5).max(1),
            fix_attempted: false,
        }
    }

    /// Add context to the issue
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Get a string representation of the category
    pub fn category_str(&self) -> String {
        match &self.category {
            IssueCategory::MessageTooLarge => "Message Too Large".to_string(),
            IssueCategory::CompilationWarning => "Compilation Warning".to_string(),
            IssueCategory::FormattingIssue => "Formatting Issue".to_string(),
            IssueCategory::ClippyWarning => "Clippy Warning".to_string(),
            IssueCategory::TestFailure => "Test Failure".to_string(),
            IssueCategory::DocumentationIssue => "Documentation Issue".to_string(),
            IssueCategory::PerformanceIssue => "Performance Issue".to_string(),
            IssueCategory::SecurityIssue => "Security Issue".to_string(),
            IssueCategory::Other(s) => s.clone(),
        }
    }
}

/// Helper functions for common issues
impl FixableIssueTracker {
    /// Log a Discord message size issue
    pub async fn log_message_too_large(&self, context: &str, actual_size: usize) {
        let issue = FixableIssue::new(
            IssueCategory::MessageTooLarge,
            format!("Discord message exceeded 2000 char limit (was {} chars)", actual_size),
            "Reduce message size by making content more concise or splitting into multiple messages",
            4, // High severity as it prevents user from seeing response
        )
        .with_context(context);

        self.log_issue(issue).await;
    }

    /// Log a compilation warning
    pub async fn log_compilation_warning(&self, warning: &str, file: Option<&str>) {
        let issue = FixableIssue::new(
            IssueCategory::CompilationWarning,
            warning,
            "Fix the compilation warning in the specified file",
            2,
        );

        let issue = if let Some(f) = file {
            issue.with_context(f)
        } else {
            issue
        };

        self.log_issue(issue).await;
    }

    /// Log a test failure
    pub async fn log_test_failure(&self, test_name: &str, error: &str) {
        let issue = FixableIssue::new(
            IssueCategory::TestFailure,
            format!("Test '{}' failed", test_name),
            "Fix the failing test or update test expectations",
            3,
        )
        .with_context(error);

        self.log_issue(issue).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_issue_tracking() {
        let tracker = FixableIssueTracker::new();

        // Log a test issue
        tracker.log_message_too_large("help.rs", 2500).await;

        // Check it was tracked
        let issues = tracker.get_issues().await;
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, IssueCategory::MessageTooLarge);
    }

    #[tokio::test]
    async fn test_issue_categories() {
        let tracker = FixableIssueTracker::new();

        // Log different categories
        tracker.log_message_too_large("test", 3000).await;
        tracker
            .log_compilation_warning("unused variable", Some("main.rs"))
            .await;
        tracker
            .log_test_failure("test_example", "assertion failed")
            .await;

        // Check categorization
        let message_issues = tracker
            .get_issues_by_category(&IssueCategory::MessageTooLarge)
            .await;
        assert_eq!(message_issues.len(), 1);

        let high_priority = tracker.get_high_priority_issues().await;
        assert_eq!(high_priority.len(), 1); // Only message too large is severity 4
    }
}
