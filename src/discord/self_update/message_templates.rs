//! Message templates for consistent Discord communication during self-updates
//!
//! This module provides standardized message formatting for all update phases,
//! ensuring consistent and professional communication with users.

use super::{
    ApprovalResult, HealthCheckResult, ImplementationPlan, PlanRiskLevel, SelfUpdateRequest,
    UpdatePhase,
};

/// Message builder for self-update communications
pub struct UpdateMessageTemplates;

impl UpdateMessageTemplates {
    /// Format the initial acknowledgment message
    pub fn acknowledgment(request: &SelfUpdateRequest) -> String {
        format!(
            "📥 **Update Request Received**\n\n\
            **Codename**: `{}`\n\
            **Request ID**: `{}`\n\
            **Description**: {}\n\
            **Status**: Request has been queued for processing\n\n\
            I'll begin processing this update shortly and keep you informed of progress.",
            request.codename, request.id, request.description
        )
    }

    /// Format preflight checks status
    pub fn preflight_status(
        request: &SelfUpdateRequest,
        passed: bool,
        details: Option<&str>,
    ) -> String {
        if passed {
            format!(
                "✅ **Preflight Checks Passed**\n\n\
                **Update**: `{}`\n\
                **Checks completed**:\n\
                • Git repository status: Clean\n\
                • Disk space: Sufficient\n\
                • System resources: Available\n\
                • Dependencies: Verified\n\n\
                Proceeding to planning phase...",
                request.codename
            )
        } else {
            format!(
                "❌ **Preflight Checks Failed**\n\n\
                **Update**: `{}`\n\
                **Issue**: {}\n\n\
                The update cannot proceed. Please resolve the issue and try again.",
                request.codename,
                details.unwrap_or("Unknown preflight failure")
            )
        }
    }

    /// Format planning phase notification
    pub fn planning_started(request: &SelfUpdateRequest) -> String {
        format!(
            "📋 **Creating Implementation Plan**\n\n\
            **Update**: `{}`\n\
            **Status**: Analyzing requirements and generating plan with Claude Code\n\n\
            This may take a moment as I:\n\
            • Analyze the codebase\n\
            • Identify affected components\n\
            • Design implementation approach\n\
            • Assess risks and complexity",
            request.codename
        )
    }

    /// Format plan presentation for approval
    pub fn plan_presentation(plan: &ImplementationPlan) -> String {
        let risk_emoji = match plan.risk_level {
            PlanRiskLevel::Unknown => "❓",
            PlanRiskLevel::Low => "🟢",
            PlanRiskLevel::Potential => "🟡",
            PlanRiskLevel::Medium => "🟠",
            PlanRiskLevel::Certain => "🔴",
            PlanRiskLevel::High => "⚫",
            PlanRiskLevel::Nuclear => "☢️",
            PlanRiskLevel::DoNotImplement => "⛔",
        };

        let complexity_total: u32 = plan.tasks.iter().map(|t| t.complexity as u32).sum();

        format!(
            "📋 **Implementation Plan Ready**\n\n\
            **Plan ID**: `{}`\n\
            **Risk Level**: {} {:?}\n\
            **Total Complexity**: {}\n\
            **Tasks**: {} planned\n\n\
            **Summary**:\n{}\n\n\
            **Success Criteria**:\n{}\n\n\
            Please review and approve this plan:\n\
            • ✅ React to approve\n\
            • ❌ React to reject\n\
            • ✏️ React to request modifications",
            plan.plan_id,
            risk_emoji,
            plan.risk_level,
            complexity_total,
            plan.tasks.len(),
            plan.summary,
            plan.success_criteria
                .iter()
                .map(|c| format!("• {}", c))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    /// Format approval status message
    pub fn approval_status(result: &ApprovalResult, codename: &str) -> String {
        match result {
            ApprovalResult::Approved => {
                format!(
                    "✅ **Plan Approved**\n\n\
                    **Update**: `{}`\n\
                    **Status**: Implementation authorized\n\n\
                    Starting implementation phase...",
                    codename
                )
            }
            ApprovalResult::Rejected(reason) => {
                format!(
                    "❌ **Plan Rejected**\n\n\
                    **Update**: `{}`\n\
                    **Reason**: {}\n\n\
                    The update has been cancelled.",
                    codename, reason
                )
            }
            ApprovalResult::ModifyRequested(details) => {
                format!(
                    "✏️ **Modifications Requested**\n\n\
                    **Update**: `{}`\n\
                    **Details**: {}\n\n\
                    The plan will be revised based on your feedback.",
                    codename, details
                )
            }
            ApprovalResult::TimedOut => {
                format!(
                    "⏱️ **Approval Timeout**\n\n\
                    **Update**: `{}`\n\
                    **Status**: No response received within 10 minutes\n\n\
                    The update has been cancelled due to timeout.",
                    codename
                )
            }
        }
    }

    /// Format snapshot creation message
    pub fn snapshot_created(codename: &str, snapshot_id: &str) -> String {
        format!(
            "📸 **Safety Snapshot Created**\n\n\
            **Update**: `{}`\n\
            **Snapshot ID**: `{}`\n\n\
            A git snapshot has been created for rollback safety.\n\
            If anything goes wrong, we can restore to this point.",
            codename, snapshot_id
        )
    }

    /// Format implementation progress
    pub fn implementation_progress(
        codename: &str,
        task_num: usize,
        total_tasks: usize,
        task_desc: &str,
    ) -> String {
        format!(
            "🔧 **Implementation Progress**\n\n\
            **Update**: `{}`\n\
            **Task**: {}/{}\n\
            **Current**: {}\n\n\
            Claude Code is implementing the approved changes...",
            codename, task_num, total_tasks, task_desc
        )
    }

    /// Format validation status
    pub fn validation_status(
        codename: &str,
        phase: &str,
        passed: bool,
        details: Option<&str>,
    ) -> String {
        if passed {
            format!(
                "✅ **Validation {} Passed**\n\n\
                **Update**: `{}`\n\
                **Checks**: All validation tests passed\n\n\
                {}",
                phase,
                codename,
                details.unwrap_or("Continuing to next phase...")
            )
        } else {
            format!(
                "❌ **Validation {} Failed**\n\n\
                **Update**: `{}`\n\
                **Issue**: {}\n\n\
                Rolling back changes to previous state...",
                phase,
                codename,
                details.unwrap_or("Validation checks did not pass")
            )
        }
    }

    /// Format rollback notification
    pub fn rollback_notification(codename: &str, snapshot_id: &str, reason: &str) -> String {
        format!(
            "🔄 **Rolling Back Changes**\n\n\
            **Update**: `{}`\n\
            **Snapshot**: `{}`\n\
            **Reason**: {}\n\n\
            The system is being restored to the previous state.\n\
            All changes from this update are being reverted.",
            codename, snapshot_id, reason
        )
    }

    /// Format health check results
    pub fn health_check_results(codename: &str, result: &HealthCheckResult) -> String {
        let status_emoji = if result.healthy { "✅" } else { "⚠️" };

        let checks = result
            .checks
            .iter()
            .map(|check| {
                let emoji = if check.passed { "🟢" } else { "🔴" };
                let status = if check.passed {
                    "Passed".to_string()
                } else {
                    check.error.clone().unwrap_or_else(|| "Failed".to_string())
                };
                format!("• {} {}: {}", emoji, check.name, status)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let issues_section = if !result.critical_issues.is_empty() {
            format!(
                "\n\n**Critical Issues**:\n{}",
                result
                    .critical_issues
                    .iter()
                    .map(|i| format!("• ❗ {}", i))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        };

        let warnings_section = if !result.warnings.is_empty() {
            format!(
                "\n\n**Warnings**:\n{}",
                result
                    .warnings
                    .iter()
                    .map(|w| format!("• ⚠️ {}", w))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        };

        format!(
            "{} **Post-Update Health Check**\n\n\
            **Update**: `{}`\n\
            **Overall Status**: {}\n\
            **Duration**: {}ms\n\n\
            **Health Checks**:\n{}{}{}",
            status_emoji,
            codename,
            if result.healthy {
                "Healthy"
            } else {
                "Issues Detected"
            },
            result.duration.as_millis(),
            checks,
            issues_section,
            warnings_section
        )
    }

    /// Format completion message
    pub fn completion(
        request: &SelfUpdateRequest,
        success: bool,
        snapshot_id: Option<&str>,
        duration: std::time::Duration,
        error: Option<&str>,
    ) -> String {
        if success {
            format!(
                "🎉 **Update Completed Successfully!**\n\n\
                **Update**: `{}`\n\
                **Duration**: {}m {}s\n\
                **Snapshot**: `{}`\n\n\
                **Summary**:\n\
                • All changes implemented successfully\n\
                • Validation tests passed\n\
                • System health verified\n\n\
                The update has been successfully applied to the system.",
                request.codename,
                duration.as_secs() / 60,
                duration.as_secs() % 60,
                snapshot_id.unwrap_or("N/A")
            )
        } else {
            format!(
                "❌ **Update Failed**\n\n\
                **Update**: `{}`\n\
                **Duration**: {}m {}s\n\
                **Error**: {}\n\n\
                The update could not be completed. {}",
                request.codename,
                duration.as_secs() / 60,
                duration.as_secs() % 60,
                error.unwrap_or("Unknown error"),
                if snapshot_id.is_some() {
                    "Changes have been rolled back."
                } else {
                    "No changes were applied."
                }
            )
        }
    }

    /// Format queue status message
    pub fn queue_status(
        position: usize,
        total: usize,
        estimated_wait: Option<std::time::Duration>,
    ) -> String {
        let wait_str = if let Some(duration) = estimated_wait {
            format!("\n**Estimated wait**: ~{} minutes", duration.as_secs() / 60)
        } else {
            String::new()
        };

        format!(
            "⏳ **Update Queued**\n\n\
            **Queue position**: {} of {}\n\
            **Status**: Waiting for processing{}\n\n\
            Your update will begin processing when it reaches the front of the queue.",
            position, total, wait_str
        )
    }

    /// Format error message with context
    pub fn error_with_context(
        codename: &str,
        phase: UpdatePhase,
        error: &str,
        suggestion: Option<&str>,
    ) -> String {
        let phase_str = format!("{:?}", phase);

        format!(
            "❌ **Error During {}**\n\n\
            **Update**: `{}`\n\
            **Error**: {}\n\n\
            {}\n\n\
            The update cannot continue. {}",
            phase_str,
            codename,
            error,
            suggestion.unwrap_or("Please check the error details and try again if appropriate."),
            if matches!(phase, UpdatePhase::Implementing | UpdatePhase::Validating) {
                "Changes are being rolled back."
            } else {
                "No changes were made to the system."
            }
        )
    }

    /// Format progress percentage with visual bar
    pub fn progress_bar(current: usize, total: usize, phase: UpdatePhase) -> String {
        let percentage = if total > 0 {
            (current * 100) / total
        } else {
            0
        };

        let filled = (percentage / 5).min(20);
        let empty = 20 - filled;
        let bar = format!("{}{}", "▰".repeat(filled), "▱".repeat(empty));

        format!(
            "**Progress**: {} {}%\n\
            **Phase**: {:?}\n\
            **Tasks**: {}/{}",
            bar, percentage, phase, current, total
        )
    }
}
