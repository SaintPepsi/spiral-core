//! Tests for the validation pipeline

use crate::discord::self_update::pipeline::{PipelineContext, PipelineStatus, ValidationPipeline};

/// Test creating a new ValidationPipeline
#[test]
fn test_pipeline_creation() {
    // We can only test that creation doesn't panic
    // since fields are private
    let _pipeline = ValidationPipeline::new();
}

/// Test PipelineStatus enum values
#[test]
fn test_pipeline_status_values() {
    // Just verify the enum values exist
    let _success = PipelineStatus::Success;
    let _success_with_retries = PipelineStatus::SuccessWithRetries;
    let _failure = PipelineStatus::Failure;
}

/// Integration test: pipeline creation doesn't panic
#[tokio::test]
async fn test_pipeline_creation_async() {
    // Create pipeline and verify it doesn't panic
    let _pipeline = ValidationPipeline::new();

    // Since execute() requires Claude client and Git operations,
    // we can't easily test it without mocking
}

/// Test that we can create a minimal context and serialize it
#[test]
fn test_minimal_context_serialization() {
    use crate::discord::self_update::pipeline::{
        CheckResult, ComplianceCheck, ExecutionPatterns, Phase1Results, Phase2Attempt, Phase2Checks,
    };

    // Create minimal valid context
    let context = PipelineContext {
        pipeline_iterations: 1,
        total_duration_ms: 1000,
        final_status: PipelineStatus::Success,
        phase1_results: Phase1Results {
            code_review: CheckResult {
                passed: true,
                findings: vec![],
                duration_ms: 100,
            },
            testing: CheckResult {
                passed: true,
                findings: vec![],
                duration_ms: 200,
            },
            security: CheckResult {
                passed: true,
                findings: vec![],
                duration_ms: 150,
            },
            integration: CheckResult {
                passed: true,
                findings: vec![],
                duration_ms: 100,
            },
        },
        phase2_attempts: vec![Phase2Attempt {
            iteration: 1,
            checks: Phase2Checks {
                compilation: ComplianceCheck {
                    passed: true,
                    retries: 0,
                    errors: None,
                },
                tests: ComplianceCheck {
                    passed: true,
                    retries: 0,
                    errors: None,
                },
                formatting: ComplianceCheck {
                    passed: true,
                    retries: 0,
                    errors: None,
                },
                clippy: ComplianceCheck {
                    passed: true,
                    retries: 0,
                    errors: None,
                },
                docs: ComplianceCheck {
                    passed: true,
                    retries: 0,
                    errors: None,
                },
            },
            triggered_loop: false,
        }],
        changes_applied: vec![],
        warnings: vec![],
        critical_errors: vec![],
        files_modified: vec![],
        patterns: ExecutionPatterns {
            consistent_failures: None,
            flakey_checks: None,
            performance_bottlenecks: None,
        },
    };

    // Test that serialization works
    let json_result = context.to_json();
    assert!(json_result.is_ok());

    let json = json_result.unwrap();
    assert!(json.contains("\"pipeline_iterations\": 1"));
    assert!(json.contains("\"final_status\": \"Success\""));
}
