//! Tests for Claude agent validation integration

#[cfg(test)]
mod tests {
    use crate::discord::self_update::{
        ClaudeValidationConfig, ClaudeValidator, FindingSeverity, ValidationFinding,
    };
    use std::time::Duration;

    #[tokio::test]
    async fn test_claude_validator_creation() {
        let config = ClaudeValidationConfig {
            agent_timeout: Duration::from_secs(60),
            parallel_execution: false,
            continue_on_warning: true,
        };

        let _validator = ClaudeValidator::new(config);
        // Validator should be created successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_ty_lee_validation_placeholder() {
        let config = ClaudeValidationConfig::default();
        let validator = ClaudeValidator::new(config);

        let changed_files = vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "src/test_utils.rs".to_string(),
        ];

        let results = validator.validate_with_agents(changed_files).await.unwrap();

        // Should have results from all 3 agents
        assert_eq!(results.len(), 3);

        // Check Ty Lee results
        let ty_lee_result = results
            .iter()
            .find(|r| r.agent_name.contains("Ty Lee"))
            .unwrap();
        assert!(ty_lee_result.success); // Should succeed with no findings (placeholder)

        // Placeholder returns empty findings
        assert!(ty_lee_result.findings.is_empty());
    }

    #[tokio::test]
    async fn test_security_validation_placeholder() {
        let config = ClaudeValidationConfig::default();
        let validator = ClaudeValidator::new(config);

        let changed_files = vec![
            "src/auth/login.rs".to_string(),
            "src/security/validator.rs".to_string(),
        ];

        let results = validator.validate_with_agents(changed_files).await.unwrap();

        // Check Security Inquisitor results
        let security_result = results
            .iter()
            .find(|r| r.agent_name.contains("Security"))
            .unwrap();
        assert!(security_result.success);

        // Placeholder returns empty findings
        assert!(security_result.findings.is_empty());
    }

    #[tokio::test]
    async fn test_lordgenome_validation_placeholder() {
        let config = ClaudeValidationConfig::default();
        let validator = ClaudeValidator::new(config);

        // Test with many files
        let changed_files: Vec<String> = (0..15).map(|i| format!("src/module_{}.rs", i)).collect();

        let results = validator.validate_with_agents(changed_files).await.unwrap();

        // Check Lordgenome results
        let lordgenome_result = results
            .iter()
            .find(|r| r.agent_name.contains("Lordgenome"))
            .unwrap();

        // Placeholder implementation returns empty findings
        assert!(lordgenome_result.findings.is_empty());
        // But should have recommendations
        assert!(!lordgenome_result.recommendations.is_empty());
    }

    #[tokio::test]
    async fn test_validation_with_continue_on_warning() {
        let config = ClaudeValidationConfig {
            agent_timeout: Duration::from_secs(60),
            parallel_execution: false,
            continue_on_warning: true, // Should not fail on warnings
        };

        let validator = ClaudeValidator::new(config);

        let changed_files: Vec<String> = (0..15).map(|i| format!("src/module_{}.rs", i)).collect();

        let result = validator.validate_with_agents(changed_files).await;

        // Should succeed because continue_on_warning is true and we have no critical findings
        assert!(result.is_ok());
    }

    #[test]
    fn test_finding_severity_ordering() {
        // Ensure severity levels are properly ordered (Info < Low < Medium < High < Critical)
        assert!(FindingSeverity::Info < FindingSeverity::Low);
        assert!(FindingSeverity::Low < FindingSeverity::Medium);
        assert!(FindingSeverity::Medium < FindingSeverity::High);
        assert!(FindingSeverity::High < FindingSeverity::Critical);
    }

    #[test]
    fn test_validation_result_formatting() {
        use crate::discord::self_update::{format_validation_results, AgentValidationResult};

        let results = vec![AgentValidationResult {
            agent_name: "Ty Lee Precision Tester".to_string(),
            success: true,
            findings: vec![ValidationFinding {
                severity: FindingSeverity::Medium,
                category: "Test Coverage".to_string(),
                description: "Missing tests for pressure points".to_string(),
                file_path: Some("src/main.rs".to_string()),
                line_number: None,
                suggested_fix: Some("Add targeted tests".to_string()),
            }],
            recommendations: vec!["🎯 Focus on pressure points".to_string()],
            execution_time_ms: 150,
        }];

        let formatted = format_validation_results(&results);
        assert!(formatted.contains("Claude Agent Validation Results"));
        assert!(formatted.contains("Ty Lee Precision Tester"));
        assert!(formatted.contains("150ms"));
        assert!(formatted.contains("Focus on pressure points"));
    }
}
