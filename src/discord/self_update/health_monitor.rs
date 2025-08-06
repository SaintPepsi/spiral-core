//! Post-update health monitoring
//!
//! This module verifies system stability after self-updates by running
//! a series of health checks to ensure the system is functioning correctly.

use crate::Result;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tracing::{debug, error, info};
use serde::{Deserialize, Serialize};

/// Health check results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Overall health status
    pub healthy: bool,
    /// Individual check results
    pub checks: Vec<HealthCheck>,
    /// Total duration of health checks
    pub duration: Duration,
    /// Any critical issues found
    pub critical_issues: Vec<String>,
    /// Any warnings found
    pub warnings: Vec<String>,
}

/// Individual health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Name of the check
    pub name: String,
    /// Category of check
    pub category: HealthCategory,
    /// Whether the check passed
    pub passed: bool,
    /// Check execution duration
    pub duration: Duration,
    /// Error message if failed
    pub error: Option<String>,
    /// Additional details
    pub details: Option<String>,
}

/// Categories of health checks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthCategory {
    Compilation,
    Tests,
    BinaryExecution,
    Dependencies,
    Documentation,
    GitStatus,
}

impl HealthCategory {
    /// Get emoji for the category
    pub fn emoji(&self) -> &str {
        match self {
            HealthCategory::Compilation => "🔨",
            HealthCategory::Tests => "🧪",
            HealthCategory::BinaryExecution => "🚀",
            HealthCategory::Dependencies => "📦",
            HealthCategory::Documentation => "📚",
            HealthCategory::GitStatus => "🔍",
        }
    }
}

/// Health monitor for post-update verification
pub struct HealthMonitor {
    /// Timeout for individual checks
    check_timeout: Duration,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new() -> Self {
        Self {
            check_timeout: Duration::from_secs(60),
        }
    }
    
    /// Run all health checks
    pub async fn run_health_checks(&self) -> Result<HealthCheckResult> {
        info!("[HealthMonitor] Starting post-update health checks");
        let start = Instant::now();
        
        let mut checks = Vec::new();
        let mut critical_issues = Vec::new();
        let mut warnings = Vec::new();
        
        // Run individual health checks
        checks.push(self.check_compilation().await);
        checks.push(self.check_tests().await);
        checks.push(self.check_binary_execution().await);
        checks.push(self.check_dependencies().await);
        checks.push(self.check_documentation().await);
        checks.push(self.check_git_status().await);
        
        // Analyze results
        let mut has_critical = false;
        for check in &checks {
            if !check.passed {
                match check.category {
                    HealthCategory::Compilation | HealthCategory::BinaryExecution => {
                        has_critical = true;
                        critical_issues.push(format!(
                            "{} failed: {}",
                            check.name,
                            check.error.as_ref().unwrap_or(&"Unknown error".to_string())
                        ));
                    }
                    HealthCategory::Tests => {
                        warnings.push(format!(
                            "Tests are failing: {}",
                            check.error.as_ref().unwrap_or(&"Unknown error".to_string())
                        ));
                    }
                    _ => {
                        warnings.push(format!(
                            "{} check failed: {}",
                            check.name,
                            check.error.as_ref().unwrap_or(&"Unknown error".to_string())
                        ));
                    }
                }
            }
        }
        
        let healthy = !has_critical && critical_issues.is_empty();
        let duration = start.elapsed();
        
        let result = HealthCheckResult {
            healthy,
            checks,
            duration,
            critical_issues: critical_issues.clone(),
            warnings,
        };
        
        if healthy {
            info!("[HealthMonitor] System is healthy (duration: {:?})", duration);
        } else {
            error!("[HealthMonitor] System has health issues: {:?}", critical_issues);
        }
        
        Ok(result)
    }
    
    /// Check if the code compiles
    async fn check_compilation(&self) -> HealthCheck {
        let start = Instant::now();
        let name = "Compilation Check".to_string();
        
        debug!("[HealthMonitor] Running compilation check");
        
        let output = match tokio::time::timeout(
            self.check_timeout,
            Command::new("cargo")
                .args(&["check", "--all-targets"])
                .output()
        ).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return HealthCheck {
                    name,
                    category: HealthCategory::Compilation,
                    passed: false,
                    duration: start.elapsed(),
                    error: Some(format!("Failed to run cargo check: {}", e)),
                    details: None,
                };
            }
            Err(_) => {
                return HealthCheck {
                    name,
                    category: HealthCategory::Compilation,
                    passed: false,
                    duration: start.elapsed(),
                    error: Some("Compilation check timed out".to_string()),
                    details: None,
                };
            }
        };
        
        let passed = output.status.success();
        let error = if !passed {
            Some(String::from_utf8_lossy(&output.stderr).to_string())
        } else {
            None
        };
        
        HealthCheck {
            name,
            category: HealthCategory::Compilation,
            passed,
            duration: start.elapsed(),
            error,
            details: Some(format!("Exit code: {}", output.status.code().unwrap_or(-1))),
        }
    }
    
    /// Check if tests pass
    async fn check_tests(&self) -> HealthCheck {
        let start = Instant::now();
        let name = "Test Suite".to_string();
        
        debug!("[HealthMonitor] Running test check");
        
        let output = match tokio::time::timeout(
            Duration::from_secs(120), // Tests get more time
            Command::new("cargo")
                .args(&["test", "--", "--test-threads=4", "--nocapture"])
                .output()
        ).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return HealthCheck {
                    name,
                    category: HealthCategory::Tests,
                    passed: false,
                    duration: start.elapsed(),
                    error: Some(format!("Failed to run cargo test: {}", e)),
                    details: None,
                };
            }
            Err(_) => {
                return HealthCheck {
                    name,
                    category: HealthCategory::Tests,
                    passed: false,
                    duration: start.elapsed(),
                    error: Some("Test suite timed out".to_string()),
                    details: None,
                };
            }
        };
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let passed = output.status.success();
        
        // Extract test statistics if available
        let details = if let Some(summary_line) = stdout.lines().find(|l| l.contains("test result:")) {
            Some(summary_line.to_string())
        } else {
            None
        };
        
        let error = if !passed {
            Some("One or more tests failed".to_string())
        } else {
            None
        };
        
        HealthCheck {
            name,
            category: HealthCategory::Tests,
            passed,
            duration: start.elapsed(),
            error,
            details,
        }
    }
    
    /// Check if the main binary executes
    async fn check_binary_execution(&self) -> HealthCheck {
        let start = Instant::now();
        let name = "Binary Execution".to_string();
        
        debug!("[HealthMonitor] Checking binary execution");
        
        let output = match tokio::time::timeout(
            Duration::from_secs(10),
            Command::new("cargo")
                .args(&["run", "--bin", "spiral-core", "--", "--version"])
                .output()
        ).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return HealthCheck {
                    name,
                    category: HealthCategory::BinaryExecution,
                    passed: false,
                    duration: start.elapsed(),
                    error: Some(format!("Failed to execute binary: {}", e)),
                    details: None,
                };
            }
            Err(_) => {
                return HealthCheck {
                    name,
                    category: HealthCategory::BinaryExecution,
                    passed: false,
                    duration: start.elapsed(),
                    error: Some("Binary execution timed out".to_string()),
                    details: None,
                };
            }
        };
        
        let passed = output.status.success();
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        HealthCheck {
            name,
            category: HealthCategory::BinaryExecution,
            passed,
            duration: start.elapsed(),
            error: if !passed {
                Some("Binary failed to execute".to_string())
            } else {
                None
            },
            details: if passed {
                Some(format!("Version: {}", version))
            } else {
                None
            },
        }
    }
    
    /// Check dependencies are valid
    async fn check_dependencies(&self) -> HealthCheck {
        let start = Instant::now();
        let name = "Dependency Check".to_string();
        
        debug!("[HealthMonitor] Checking dependencies");
        
        let output = match tokio::time::timeout(
            self.check_timeout,
            Command::new("cargo")
                .args(&["tree", "--depth", "1"])
                .output()
        ).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return HealthCheck {
                    name,
                    category: HealthCategory::Dependencies,
                    passed: false,
                    duration: start.elapsed(),
                    error: Some(format!("Failed to check dependencies: {}", e)),
                    details: None,
                };
            }
            Err(_) => {
                return HealthCheck {
                    name,
                    category: HealthCategory::Dependencies,
                    passed: false,
                    duration: start.elapsed(),
                    error: Some("Dependency check timed out".to_string()),
                    details: None,
                };
            }
        };
        
        let passed = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Count dependencies
        let dep_count = stdout.lines().filter(|l| l.contains("├──") || l.contains("└──")).count();
        
        HealthCheck {
            name,
            category: HealthCategory::Dependencies,
            passed,
            duration: start.elapsed(),
            error: if !passed {
                Some("Dependency tree has issues".to_string())
            } else {
                None
            },
            details: Some(format!("{} direct dependencies", dep_count)),
        }
    }
    
    /// Check documentation builds
    async fn check_documentation(&self) -> HealthCheck {
        let start = Instant::now();
        let name = "Documentation Build".to_string();
        
        debug!("[HealthMonitor] Checking documentation");
        
        let output = match tokio::time::timeout(
            self.check_timeout,
            Command::new("cargo")
                .args(&["doc", "--no-deps", "--quiet"])
                .output()
        ).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return HealthCheck {
                    name,
                    category: HealthCategory::Documentation,
                    passed: false,
                    duration: start.elapsed(),
                    error: Some(format!("Failed to build docs: {}", e)),
                    details: None,
                };
            }
            Err(_) => {
                return HealthCheck {
                    name,
                    category: HealthCategory::Documentation,
                    passed: false,
                    duration: start.elapsed(),
                    error: Some("Documentation build timed out".to_string()),
                    details: None,
                };
            }
        };
        
        let passed = output.status.success();
        
        HealthCheck {
            name,
            category: HealthCategory::Documentation,
            passed,
            duration: start.elapsed(),
            error: if !passed {
                Some("Documentation has errors".to_string())
            } else {
                None
            },
            details: None,
        }
    }
    
    /// Check git repository status
    async fn check_git_status(&self) -> HealthCheck {
        let start = Instant::now();
        let name = "Git Repository".to_string();
        
        debug!("[HealthMonitor] Checking git status");
        
        let output = match Command::new("git")
            .args(&["status", "--porcelain"])
            .output()
            .await
        {
            Ok(output) => output,
            Err(e) => {
                return HealthCheck {
                    name,
                    category: HealthCategory::GitStatus,
                    passed: false,
                    duration: start.elapsed(),
                    error: Some(format!("Failed to check git status: {}", e)),
                    details: None,
                };
            }
        };
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let has_changes = !stdout.trim().is_empty();
        let change_count = stdout.lines().count();
        
        HealthCheck {
            name,
            category: HealthCategory::GitStatus,
            passed: true, // Git changes aren't a failure
            duration: start.elapsed(),
            error: None,
            details: if has_changes {
                Some(format!("{} uncommitted changes", change_count))
            } else {
                Some("Working directory clean".to_string())
            },
        }
    }
    
    /// Format health check results for Discord
    pub fn format_for_discord(result: &HealthCheckResult) -> String {
        let status_emoji = if result.healthy { "✅" } else { "⚠️" };
        let status_text = if result.healthy { "HEALTHY" } else { "ISSUES DETECTED" };
        
        let mut message = format!(
            "{} **System Health: {}**\n",
            status_emoji, status_text
        );
        
        message.push_str("```\n");
        for check in &result.checks {
            let check_emoji = if check.passed { "✓" } else { "✗" };
            message.push_str(&format!(
                "{} {} {}: {}\n",
                check.category.emoji(),
                check_emoji,
                check.name,
                if check.passed {
                    check.details.as_ref().unwrap_or(&"OK".to_string()).clone()
                } else {
                    check.error.as_ref().unwrap_or(&"Failed".to_string()).clone()
                }
            ));
        }
        message.push_str("```\n");
        
        if !result.critical_issues.is_empty() {
            message.push_str("**🚨 Critical Issues:**\n");
            for issue in &result.critical_issues {
                message.push_str(&format!("• {}\n", issue));
            }
        }
        
        if !result.warnings.is_empty() {
            message.push_str("**⚠️ Warnings:**\n");
            for warning in &result.warnings {
                message.push_str(&format!("• {}\n", warning));
            }
        }
        
        message.push_str(&format!(
            "\n⏱️ Health check completed in {:.2}s",
            result.duration.as_secs_f32()
        ));
        
        message
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_health_category_emoji() {
        assert_eq!(HealthCategory::Compilation.emoji(), "🔨");
        assert_eq!(HealthCategory::Tests.emoji(), "🧪");
        assert_eq!(HealthCategory::BinaryExecution.emoji(), "🚀");
    }
    
    #[test]
    fn test_format_for_discord() {
        let result = HealthCheckResult {
            healthy: true,
            checks: vec![
                HealthCheck {
                    name: "Compilation Check".to_string(),
                    category: HealthCategory::Compilation,
                    passed: true,
                    duration: Duration::from_secs(2),
                    error: None,
                    details: Some("All targets compiled".to_string()),
                },
                HealthCheck {
                    name: "Test Suite".to_string(),
                    category: HealthCategory::Tests,
                    passed: false,
                    duration: Duration::from_secs(5),
                    error: Some("2 tests failed".to_string()),
                    details: None,
                },
            ],
            duration: Duration::from_secs(7),
            critical_issues: vec![],
            warnings: vec!["Tests are failing: 2 tests failed".to_string()],
        };
        
        let formatted = HealthMonitor::format_for_discord(&result);
        assert!(formatted.contains("System Health"));
        assert!(formatted.contains("Compilation Check"));
        assert!(formatted.contains("Test Suite"));
        assert!(formatted.contains("Warnings"));
    }
    
    #[tokio::test]
    async fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new();
        assert_eq!(monitor.check_timeout, Duration::from_secs(60));
    }
}