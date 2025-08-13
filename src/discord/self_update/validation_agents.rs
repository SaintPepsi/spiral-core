//! Centralized validation agent paths - Single source of truth
//!
//! This module defines all validation agent file paths in one place,
//! following the DRY principle to avoid repetition across modules.

/// Phase 1: Engineering Review - agent paths (deep quality inspection)
pub mod engineering_review {
    /// Part 1: Code Standards Review - Engineers reviewing architecture and patterns
    pub const CODE_STANDARDS: &str = ".claude/validation-agents/phase1/code-review-standards.md";

    /// Part 2: Test Coverage Analysis - Engineers ensuring critical paths are tested
    pub const TEST_COVERAGE: &str = ".claude/validation-agents/phase1/comprehensive-testing.md";

    /// Part 3: Security Inspection - Engineers reviewing for vulnerabilities
    pub const SECURITY: &str = ".claude/validation-agents/phase1/security-audit.md";

    /// Part 4: Integration Review - Engineers verifying system cohesion
    pub const INTEGRATION: &str = ".claude/validation-agents/phase1/system-integration.md";
}

/// Phase 2: Final Assembly Checklist - agent paths (mechanical verification)
pub mod assembly_checklist {
    /// Part 1: ✓ Compilation Check
    pub const COMPILATION_FIX: &str = ".claude/validation-agents/phase2/compilation-fixer.md";

    /// Part 2: ✓ Test Execution
    pub const TEST_FIX: &str = ".claude/validation-agents/phase2/test-failure-analyzer.md";

    /// Part 3: ✓ Formatting Check
    pub const FORMAT_FIX: &str = ".claude/validation-agents/phase2/formatting-fixer.md";

    /// Part 4: ✓ Linting Check
    pub const CLIPPY_FIX: &str = ".claude/validation-agents/phase2/clippy-resolver.md";

    /// Part 5: ✓ Documentation Build
    pub const DOC_FIX: &str = ".claude/validation-agents/phase2/doc-builder.md";
}

/// Analysis agents - for pipeline outcome analysis
pub mod analysis {
    /// Success analysis when all checks pass first time
    pub const SUCCESS: &str = ".claude/validation-agents/analysis/success-analyzer.md";

    /// Success with issues when retries were needed
    pub const SUCCESS_WITH_ISSUES: &str =
        ".claude/validation-agents/analysis/success-with-issues-analyzer.md";

    /// Failure analysis when pipeline fails after max iterations
    pub const FAILURE: &str = ".claude/validation-agents/analysis/failure-analyzer.md";
}

/// Helper function to read an agent prompt file
pub async fn read_agent_prompt(path: &str) -> Result<String, std::io::Error> {
    tokio::fs::read_to_string(path).await
}
