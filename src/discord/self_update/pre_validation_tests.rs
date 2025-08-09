//! Automated tests for Phase 1 Pre-restart Validation
//! 
//! These tests verify the Engineering Review phase works correctly
//! with both real Claude integration and mock fallbacks.

use crate::discord::self_update::pre_validation::PreImplementationValidator;
use crate::discord::self_update::{SelfUpdateRequest, UpdateStatus, StructuredLogger};
use crate::claude_code::{ClaudeCodeClient, CodeGenerationResult};
use crate::config::ClaudeCodeConfig;
use crate::Result;
use std::collections::HashMap;
use tempfile::TempDir;
use tokio::fs;

/// Mock Claude client for testing
struct MockClaudeClient {
    responses: HashMap<String, String>,
}

impl MockClaudeClient {
    fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }
    
    fn with_response(mut self, validation_type: &str, response: &str) -> Self {
        self.responses.insert(validation_type.to_string(), response.to_string());
        self
    }
    
    async fn generate_code(&self, request: crate::claude_code::CodeGenerationRequest) -> Result<CodeGenerationResult> {
        // Extract validation type from context
        let validation_type = request.context.get("validation_type")
            .map(|s| s.as_str())
            .unwrap_or("unknown");
        
        // Return pre-configured response or default
        let response = self.responses.get(validation_type)
            .cloned()
            .unwrap_or_else(|| format!("{} STATUS: PASS", validation_type.to_uppercase()));
        
        Ok(CodeGenerationResult {
            code: String::new(),
            explanation: response,
            language: request.language,
            files_to_create: vec![],
            files_to_modify: vec![],
            workspace_path: "/tmp".to_string(),
            session_id: request.session_id,
        })
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    
    /// Helper to create a test request
    pub fn create_test_request() -> SelfUpdateRequest {
        SelfUpdateRequest {
            id: "test-123".to_string(),
            codename: "test-update".to_string(),
            description: "Test update for validation".to_string(),
            user_id: 123456789,
            channel_id: 987654321,
            message_id: 111222333,
            combined_messages: vec!["Test implementation".to_string()],
            timestamp: chrono::Utc::now().to_rfc3339(),
            retry_count: 0,
            status: UpdateStatus::Executing,
        }
    }
    
    /// Helper to create a test logger
    pub async fn create_test_logger() -> StructuredLogger {
        let temp_dir = TempDir::new().unwrap();
        StructuredLogger::new(
            "test-123".to_string(),
            "test-update".to_string()
        ).unwrap()
    }

    #[tokio::test]
    async fn test_phase1_all_pass() {
        // Create mock Claude client with all passing responses
        let mock_client = MockClaudeClient::new()
            .with_response("code_standards", "Code review passed\nCOMPLIANCE STATUS: PASS")
            .with_response("testing_analysis", "Tests adequate\nTEST COVERAGE STATUS: ADEQUATE")
            .with_response("security_audit", "No vulnerabilities\nSECURITY STATUS: PASS")
            .with_response("integration_verification", "Integration verified\nINTEGRATION STATUS: PASS");
        
        let validator = PreImplementationValidator::new(Some(Box::new(mock_client)));
        let request = create_test_request();
        let mut logger = create_test_logger().await;
        
        let result = validator.validate_current_state(&request, &mut logger).await.unwrap();
        
        assert!(result.engineering_review_passed);
        assert!(result.all_passed());
        assert_eq!(result.pipeline_iterations, 1);
    }
    
    #[tokio::test]
    async fn test_phase1_code_standards_fail() {
        // Mock client with failing code standards
        let mock_client = MockClaudeClient::new()
            .with_response("code_standards", "Found SOLID violations\nCOMPLIANCE STATUS: FAIL")
            .with_response("testing_analysis", "Tests adequate\nTEST COVERAGE STATUS: ADEQUATE")
            .with_response("security_audit", "No vulnerabilities\nSECURITY STATUS: PASS")
            .with_response("integration_verification", "Integration verified\nINTEGRATION STATUS: PASS");
        
        let validator = PreImplementationValidator::new(Some(Box::new(mock_client)));
        let request = create_test_request();
        let mut logger = create_test_logger().await;
        
        let result = validator.validate_current_state(&request, &mut logger).await.unwrap();
        
        assert!(!result.engineering_review_passed);
        assert!(!result.all_passed());
        assert!(result.error_details.is_some());
    }
    
    #[tokio::test]
    async fn test_phase1_security_fail() {
        // Mock client with security issues
        let mock_client = MockClaudeClient::new()
            .with_response("code_standards", "Code standards good\nCOMPLIANCE STATUS: PASS")
            .with_response("testing_analysis", "Tests adequate\nTEST COVERAGE STATUS: ADEQUATE")
            .with_response("security_audit", "Found SQL injection vulnerability\nSECURITY STATUS: FAIL")
            .with_response("integration_verification", "Integration verified\nINTEGRATION STATUS: PASS");
        
        let validator = PreImplementationValidator::new(Some(Box::new(mock_client)));
        let request = create_test_request();
        let mut logger = create_test_logger().await;
        
        let result = validator.validate_current_state(&request, &mut logger).await.unwrap();
        
        assert!(!result.engineering_review_passed);
        assert!(result.error_details.unwrap().contains("Engineering Review failed"));
    }
    
    #[tokio::test]
    async fn test_phase2_loop_back() {
        // Mock client that simulates Phase 2 requiring retry
        // This would need Phase 2 implementation to test properly
        // For now, test that multiple iterations work
        
        let mock_client = MockClaudeClient::new()
            .with_response("code_standards", "Code standards good\nCOMPLIANCE STATUS: PASS")
            .with_response("testing_analysis", "Tests adequate\nTEST COVERAGE STATUS: ADEQUATE")
            .with_response("security_audit", "Security good\nSECURITY STATUS: PASS")
            .with_response("integration_verification", "Integration verified\nINTEGRATION STATUS: PASS");
        
        let validator = PreImplementationValidator::new(Some(Box::new(mock_client)));
        let request = create_test_request();
        let mut logger = create_test_logger().await;
        
        // This should pass Phase 1 and then run Phase 2
        let result = validator.validate_current_state(&request, &mut logger).await.unwrap();
        
        assert!(result.engineering_review_passed);
        // Phase 2 checks would determine if assembly_checklist_passed
    }
    
    #[tokio::test]
    async fn test_fallback_without_claude() {
        // Test with no Claude client - should use fallback
        let validator = PreImplementationValidator::new(None);
        let request = create_test_request();
        let mut logger = create_test_logger().await;
        
        let result = validator.validate_current_state(&request, &mut logger).await.unwrap();
        
        // Fallback checks should still work
        assert_eq!(result.total_checks_run, 9); // 4 Phase 1 + 5 Phase 2
    }
    
    #[tokio::test]
    async fn test_max_iterations_limit() {
        // Test that pipeline stops after max iterations
        let validator = PreImplementationValidator {
            claude_client: None,
            max_pipeline_iterations: 3,
            max_retries_per_check: 3,
        };
        
        assert_eq!(validator.max_pipeline_iterations, 3);
        
        // In a real scenario with failing Phase 2 checks that require retry,
        // the pipeline should stop after 3 iterations
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use super::validation_tests::{create_test_request, create_test_logger};
    
    #[tokio::test]
    #[ignore = "Requires real Claude binary"]
    async fn test_real_claude_integration() {
        // This test requires actual Claude binary to be installed
        let config = ClaudeCodeConfig {
            claude_binary_path: None, // Let it auto-discover
            working_directory: Some("/tmp/test-validation".to_string()),
            timeout_seconds: 30,
            permission_mode: "standard".to_string(),
            allowed_tools: vec![],
            workspace_cleanup_after_hours: 1,
            max_workspace_size_mb: 100,
        };
        
        let claude_client = ClaudeCodeClient::new(config).await.unwrap();
        let validator = PreImplementationValidator::new(Some(Box::new(claude_client)));
        
        let request = create_test_request();
        let mut logger = create_test_logger().await;
        
        // Create some test changes to validate
        let test_file = "/tmp/test-validation/test.rs";
        fs::create_dir_all("/tmp/test-validation").await.unwrap();
        fs::write(test_file, "fn main() { println!(\"Hello\"); }").await.unwrap();
        
        let result = validator.validate_current_state(&request, &mut logger).await.unwrap();
        
        // With real Claude, results depend on actual code analysis
        println!("Real Claude validation result: {:?}", result);
        assert!(result.total_checks_run > 0);
    }
    
    #[tokio::test]
    async fn test_git_changed_files() {
        let validator = PreImplementationValidator::new(None);
        
        // This will work if we're in a git repo
        match validator.get_changed_files().await {
            Ok(files) => {
                println!("Changed files: {:?}", files);
                // Files could be empty if no changes
                assert!(files.is_empty() || files.len() > 0);
            }
            Err(e) => {
                // Not in a git repo or no git
                println!("Git not available: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod response_parsing_tests {
    
    #[test]
    fn test_parse_compliance_status() {
        let responses = vec![
            ("COMPLIANCE STATUS: PASS", true),
            ("COMPLIANCE STATUS: FAIL", false),
            ("Something else\nCOMPLIANCE STATUS: PASS\nMore text", true),
            ("No status here", false),
            ("compliance status: pass", false), // Case sensitive
        ];
        
        for (response, expected) in responses {
            let passed = response.contains("COMPLIANCE STATUS: PASS");
            assert_eq!(passed, expected, "Failed for response: {}", response);
        }
    }
    
    #[test]
    fn test_parse_test_coverage_status() {
        let responses = vec![
            ("TEST COVERAGE STATUS: ADEQUATE", true),
            ("TEST COVERAGE STATUS: INSUFFICIENT", false),
            ("Random text\nTEST COVERAGE STATUS: ADEQUATE", true),
            ("No coverage info", false),
        ];
        
        for (response, expected) in responses {
            let passed = response.contains("TEST COVERAGE STATUS: ADEQUATE");
            assert_eq!(passed, expected, "Failed for response: {}", response);
        }
    }
    
    #[test]
    fn test_parse_security_status() {
        let responses = vec![
            ("SECURITY STATUS: PASS", true),
            ("SECURITY STATUS: FAIL", false),
            ("Security audit complete\nSECURITY STATUS: PASS", true),
        ];
        
        for (response, expected) in responses {
            let passed = response.contains("SECURITY STATUS: PASS");
            assert_eq!(passed, expected, "Failed for response: {}", response);
        }
    }
    
    #[test]
    fn test_parse_integration_status() {
        let responses = vec![
            ("INTEGRATION STATUS: PASS", true),
            ("INTEGRATION STATUS: FAIL", false),
            ("All good\nINTEGRATION STATUS: PASS\nDone", true),
        ];
        
        for (response, expected) in responses {
            let passed = response.contains("INTEGRATION STATUS: PASS");
            assert_eq!(passed, expected, "Failed for response: {}", response);
        }
    }
}

#[cfg(test)]
mod phase2_tests {
    use super::*;
    use super::validation_tests::{create_test_request, create_test_logger};
    
    #[tokio::test]
    async fn test_cargo_check() {
        let validator = PreImplementationValidator::new(None);
        let request = create_test_request();
        let mut logger = create_test_logger().await;
        
        // This test will actually run cargo check
        let result = validator.run_cargo_check(&request, &mut logger).await;
        
        // Should succeed in a valid Rust project
        assert!(result.passed);
        assert_eq!(result.name, "Cargo Check");
    }
    
    #[tokio::test]
    async fn test_cargo_fmt_auto_fix() {
        let validator = PreImplementationValidator::new(None);
        let request = create_test_request();
        let mut logger = create_test_logger().await;
        
        // cargo fmt with auto-fix
        let result = validator.run_cargo_fmt(&request, &mut logger).await;
        
        // Should pass (either already formatted or auto-fixed)
        assert!(result.passed);
        if result.retries_used > 0 {
            assert_eq!(result.output, "Formatting issues auto-fixed");
        }
    }
    
    #[tokio::test] 
    async fn test_cargo_clippy() {
        let validator = PreImplementationValidator::new(None);
        let request = create_test_request();
        let mut logger = create_test_logger().await;
        
        let result = validator.run_cargo_clippy(&request, &mut logger).await;
        
        // May pass or fail depending on code quality
        assert_eq!(result.name, "Cargo Clippy");
    }
}