# Rust Validation Pipeline Enhancement Prompt

## Objective

Modify the SPIRAL_CONSTELLATION_SELF_UPDATING_SYSTEM validation pipeline to implement a robust, two-phase Rust code quality assurance process with intelligent error handling and recovery mechanisms.

## Context

This validation pipeline operates on the working directory after Claude has made modifications but BEFORE the system restarts with the new code. The goal is to ensure changes are safe before applying them to the live running system.

## Pipeline Architecture Overview

The pipeline operates in two phases with conditional looping:

1. **Phase 1**: Advanced Quality Assurance (AQA)
2. **Phase 2**: Core Rust Compliance Checks (CRCC)

**Flow Logic**: If any compliance check in Phase 2 requires a retry, the entire pipeline loops back to Phase 1. Maximum 3 complete pipeline iterations before failure analysis.

## Phase 1: Advanced Quality Assurance

Execute the following checks in sequence. Each check has a maximum of 3 retry attempts with Claude Code instances:

### 1. Code Review & Standards Compliance

- **Execution**: Spawn Claude Code instance with prompt: "Perform comprehensive code review against project standards. Verify architectural consistency, naming conventions, error handling patterns, and adherence to established codebase guidelines. Provide specific improvement recommendations."
- **Success Criteria**: Full compliance with coding standards
- **Max Retries**: 3

### 2. Comprehensive Testing

- **Execution**: Spawn Claude Code instance with prompt: "Perform comprehensive testing analysis focused on pressure points and critical failure scenarios. Run all test suites and identify coverage gaps. Implement ONLY high-value test cases for: error boundaries, resource exhaustion, concurrent access patterns, data corruption scenarios, network failures, and system limits. Avoid trivial tests for straightforward function calls with clear implementations. Focus on edge cases that could cause system failure."
- **Success Criteria**: Zero test failures, critical scenarios covered
- **Max Retries**: 3

### 3. Security Audit

- **Execution**: Spawn Claude Code instance with prompt: "Conduct thorough security audit. Identify potential vulnerabilities, unsafe code patterns, dependency security issues, and data validation gaps. Provide specific remediation steps for any issues found."
- **Success Criteria**: Zero critical vulnerabilities
- **Max Retries**: 3

### 4. System Integration Verification

- **Execution**: Spawn Claude Code instance with prompt: "Verify system integration integrity. Test that changes don't break existing functionality, APIs remain compatible, and all integration points function correctly. Run integration test suites and verify system behaviour."
- **Success Criteria**: No integration regressions
- **Max Retries**: 3

## Phase 2: Core Rust Compliance Checks

Execute the following checks in sequence. **If ANY check requires a retry, the entire pipeline loops back to Phase 1.**

### 1. Compilation Verification (`cargo check`)

- **Purpose**: Ensure code compiles without errors
- **Failure Action**: Spawn Claude Code instance with prompt: "Fix all compilation errors identified by `cargo check`. Focus on type errors, missing dependencies, and syntax issues. Provide detailed explanation of changes made."
- **Success Criteria**: Zero compilation errors
- **Max Retries**: 3
- **Loop Trigger**: Any retry triggers return to Phase 1

### 2. Test Suite Validation (`cargo test`)

- **Purpose**: Verify all existing and new tests pass
- **Failure Action**: Spawn Claude Code instance with prompt: "Analyse and fix failing tests. DO NOT delete tests unless they are fundamentally invalid. Investigate root causes and implement proper fixes. Document any test modifications with justification."
- **Success Criteria**: 100% test pass rate
- **Max Retries**: 3
- **Loop Trigger**: Any retry triggers return to Phase 1

### 3. Code Formatting (`cargo fmt`)

- **Purpose**: Ensure consistent code style
- **Failure Action**: Spawn Claude Code instance with prompt: "Apply Rust standard formatting using `cargo fmt`. Resolve any formatting conflicts or issues that prevent automatic formatting."
- **Success Criteria**: No formatting violations
- **Max Retries**: 3
- **Loop Trigger**: Any retry triggers return to Phase 1

### 4. Linting Compliance (`cargo clippy`)

- **Purpose**: Identify and fix code quality issues
- **Failure Action**: Spawn Claude Code instance with prompt: "Fix all Clippy warnings and errors. Prioritise performance, correctness, and idiomatic Rust patterns. Explain reasoning for any clippy directives added."
- **Success Criteria**: Zero clippy violations
- **Max Retries**: 3
- **Loop Trigger**: Any retry triggers return to Phase 1

### 5. Documentation Generation (`cargo doc`)

- **Purpose**: Verify documentation builds successfully
- **Failure Action**: Spawn Claude Code instance with prompt: "Fix documentation build errors. Ensure all public APIs have proper documentation. Fix broken doc links and malformed doc comments."
- **Success Criteria**: Clean documentation build
- **Max Retries**: 3
- **Loop Trigger**: Any retry triggers return to Phase 1

## Pipeline Flow Control

### Happy Path Flow

```
Phase 1 (AQA) → Phase 2 (CRCC - no retries) → System Restart → Post-Restart Validation → Git Commit & Push → Success Analysis Protocol
```

### Critical Path Flow (Maximum Iterations)

```
Phase 1 (AQA) → Phase 2 (CRCC - retry occurred) →
Phase 1 (AQA) → Phase 2 (CRCC - retry occurred) →
Phase 1 (AQA) → Phase 2 (CRCC - retry occurred) →
Phase 1 (AQA) → Phase 2 (CRCC - retry occurred) →
Failure Analysis Protocol
```

### Sad Path Flow (Success with Issues)

```
Phase 1 (AQA) → Phase 2 (CRCC - retry occurred) →
Phase 1 (AQA) → Phase 2 (CRCC - no retries) →
Success with Issues Analysis Protocol
```

## Analysis Protocols

### Success Analysis Protocol

When pipeline completes without any Phase 2 retries:

**Execution**: Spawn Claude Code instance with prompt:

```
PIPELINE SUCCESS ANALYSIS

The validation pipeline has completed successfully with no compliance check retries required.

Tasks:
1. Generate comprehensive success report including:
   - Summary of all checks performed and their outcomes
   - Code quality metrics and improvements made
   - Security audit findings and resolutions
   - Test coverage analysis and new tests added
   - Performance implications of changes made
2. Document best practices identified during this validation
3. Recommend any optimisations for future pipeline runs
4. Create deployment readiness checklist
5. Generate change impact summary for stakeholders
6. Confirm successful git push to remote repository
7. Document commit hash and push timestamp

Focus on providing actionable insights and maintaining high code quality standards.
```

### Success with Issues Analysis Protocol

When pipeline completes after Phase 2 retries but ultimately succeeds:

**Execution**: Spawn Claude Code instance with prompt:

```
PIPELINE SUCCESS WITH ISSUES ANALYSIS

The validation pipeline has completed successfully but required compliance check retries, indicating potential areas for improvement.

Analyse the following:
- Which compliance checks required retries and why
- Pattern analysis of issues encountered
- Root cause analysis of compliance failures
- Code quality concerns that led to retries

Tasks:
1. Generate detailed report on issues encountered and resolved
2. Identify systemic problems that caused compliance failures
3. Recommend process improvements to prevent similar issues
4. Assess if additional training or guidelines are needed
5. Document lessons learned for future development
6. Create monitoring recommendations for similar patterns
7. Generate risk assessment for deployment

Provide specific, actionable recommendations to improve code quality and reduce future pipeline iterations.
```

### Failure Analysis Protocol

When pipeline fails after 3 complete iterations:

**Execution**: Spawn Claude Code instance with prompt:

```
CRITICAL FAILURE ANALYSIS REQUIRED

The Rust validation pipeline has failed after 3 complete iterations.

Analyse the following failure data:
- Iteration logs: [LOGS]
- Prompts used: [PROMPTS]
- Error patterns: [ERRORS]
- Failure points: [FAILURE_POINTS]
- Phase 1 vs Phase 2 failure distribution: [PHASE_ANALYSIS]

Tasks:
1. Identify root causes of pipeline failures
2. Determine if failures are due to:
   - Inadequate prompt specificity
   - Systemic code issues requiring architectural changes
   - Tool limitations or configuration problems
   - Unrealistic success criteria
   - Phase interaction problems
3. Propose specific pipeline modifications:
   - Enhanced prompts with better context
   - Modified success criteria if appropriate
   - Additional validation steps if needed
   - Alternative approaches for persistent issues
   - Phase flow optimisations
4. Generate updated pipeline configuration
5. Recommend immediate remediation steps
6. Assess if the two-phase approach needs refinement

Focus on creating a more resilient, intelligent pipeline that can handle edge cases and provide better guidance to Claude Code instances.
```

## Error Handling & Recovery Strategy

### Timeout Management

- **Claude Timeout**: Implement immediate retry with progress update
- **Extended Timeout**: Apply exponential backoff (1s, 2s, 4s) with detailed status messages
- **Maximum Timeout Retries**: 3 attempts before escalation

### Phase Loop Logic

- **Phase 1 Completion**: Proceed to Phase 2 regardless of retry count within Phase 1
- **Phase 2 Retry Trigger**: Any single retry in Phase 2 immediately returns to Phase 1
- **Maximum Pipeline Iterations**: 3 complete Phase 1 → Phase 2 cycles
- **Context Preservation**: Each Phase 1 restart receives full context about previous Phase 2 failures

## Success Metrics

- **Optimal**: Single pipeline iteration (Happy Path)
- **Acceptable**: 2-3 iterations with ultimate success
- **Critical**: Pipeline failure after 3 iterations

## Pipeline Resilience Features

- Two-phase validation with intelligent looping
- Progressive context building between iterations
- Detailed analysis protocols for all outcome scenarios
- Phase-specific failure handling
- Comprehensive audit trail of all decisions and changes
