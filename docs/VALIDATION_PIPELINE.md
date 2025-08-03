# Validation Pipeline Documentation

## Overview

The Spiral Core self-update system uses a sophisticated two-phase validation pipeline to ensure code quality and system stability. This pipeline integrates with Claude Code to automatically fix issues and maintain high code standards.

## Architecture

### Two-Phase Design

The pipeline operates in two distinct phases with intelligent looping:

1. **Phase 1: Advanced Quality Assurance (AQA)** - Deep analysis and quality checks
2. **Phase 2: Core Rust Compliance Checks (CRCC)** - Standard Rust toolchain validation

### Flow Logic

```
Start → Phase 1 (AQA) → Phase 2 (CRCC) → Success
           ↑                    ↓
           └── Retry Required ──┘
```

**Critical Rule**: If ANY check in Phase 2 requires a retry, the entire pipeline loops back to Phase 1. This ensures that fixes don't introduce new issues.

## Phase 1: Advanced Quality Assurance (AQA)

Phase 1 performs deep analysis using specialized Claude Code agents:

### 1. Code Review & Standards Compliance

- **Agent**: `.claude/validation-agents/phase1/code-review-standards.md`
- **Purpose**: Verify architectural consistency, naming conventions, error handling
- **Max Retries**: 3

### 2. Comprehensive Testing

- **Agent**: `.claude/validation-agents/phase1/comprehensive-testing.md`
- **Purpose**: Focus on pressure points, edge cases, critical failure scenarios
- **Max Retries**: 3

### 3. Security Audit

- **Agent**: `.claude/validation-agents/phase1/security-audit.md`
- **Purpose**: Identify vulnerabilities, unsafe patterns, dependency issues
- **Max Retries**: 3

### 4. System Integration Verification

- **Agent**: `.claude/validation-agents/phase1/system-integration.md`
- **Purpose**: Ensure no breaking changes or regressions
- **Max Retries**: 3

## Phase 2: Core Rust Compliance Checks (CRCC)

Phase 2 runs standard Rust toolchain checks with automatic fixing:

### 1. Compilation Verification

- **Command**: `cargo check --all-targets`
- **Agent**: `.claude/validation-agents/phase2/compilation-fixer.md`
- **Auto-Fix**: Compilation errors

### 2. Test Execution

- **Command**: `cargo test`
- **Agent**: `.claude/validation-agents/phase2/test-failure-analyzer.md`
- **Auto-Fix**: Test failures

### 3. Code Formatting

- **Command**: `cargo fmt`
- **Agent**: `.claude/validation-agents/phase2/formatting-fixer.md`
- **Auto-Fix**: Formatting issues

### 4. Linting (Clippy)

- **Command**: `cargo clippy --all-targets`
- **Agent**: `.claude/validation-agents/phase2/clippy-resolver.md`
- **Auto-Fix**: Clippy warnings

### 5. Documentation Build

- **Command**: `cargo doc --no-deps`
- **Agent**: `.claude/validation-agents/phase2/doc-builder.md`
- **Auto-Fix**: Documentation issues

## Analysis Agents

Based on the pipeline outcome, specialized analysis agents provide insights:

### Success Analysis

- **Agent**: `.claude/validation-agents/analysis/success-analyzer.md`
- **Triggered**: When pipeline succeeds on first attempt
- **Output**: Commendation and quality metrics

### Success with Issues Analysis

- **Agent**: `.claude/validation-agents/analysis/success-with-issues-analyzer.md`
- **Triggered**: When pipeline succeeds after retries
- **Output**: Detailed report on issues encountered and improvements

### Failure Analysis

- **Agent**: `.claude/validation-agents/analysis/failure-analyzer.md`
- **Triggered**: When pipeline fails after 3 iterations
- **Output**: Root cause analysis and remediation recommendations

## Pipeline Configuration

### Constants

```rust
const MAX_PIPELINE_ITERATIONS: u32 = 3;  // Maximum complete pipeline cycles
const MAX_PHASE_RETRIES: u32 = 3;        // Max retries per Phase 1 check
const BASE_TIMEOUT_SECS: u64 = 1;        // Initial timeout (exponential backoff)
```

### Timeout Management

The pipeline uses exponential backoff for timeouts:

- Initial: 1 second
- Backoff: 2x per retry (1s → 2s → 4s → 8s...)
- Maximum: 600 seconds (10 minutes)

## Safety Features

### Git Snapshots

Before any validation:

1. Create Git stash snapshot
2. Run validation pipeline
3. On success: Clean up snapshot
4. On failure: Rollback to snapshot

### Resource Management

- Bounded iteration count prevents infinite loops
- Timeout caps prevent hanging operations
- Memory-efficient context passing between phases

## Context Tracking

The `PipelineContext` maintains comprehensive state:

```rust
pub struct PipelineContext {
    pub pipeline_iterations: u32,
    pub phase1_retries: HashMap<String, u32>,
    pub phase2_retries: HashMap<String, u32>,
    pub current_phase: String,
    pub warnings: Vec<String>,
    pub critical_errors: Vec<String>,
    pub files_modified: Vec<String>,
    pub patterns: ValidationPatterns,
    pub final_status: PipelineStatus,
}
```

## Pattern Detection

The pipeline identifies recurring issues:

- **Recurring Issues**: Similar errors across iterations
- **Systemic Problems**: Architectural or design flaws
- **Performance Bottlenecks**: Slow operations or resource issues

## Integration Points

### With Self-Update System

The validation pipeline integrates at:

- `UpdateValidator::validate_changes()` - Entry point
- `ValidationPipeline::execute()` - Main execution
- Returns `Result<()>` for success/failure

### With Claude Code

Each agent interaction:

1. Reads agent prompt from `.claude/validation-agents/`
2. Serializes context to JSON
3. Spawns Claude Code with prompt + context (defaults to Sonnet model)
4. Claude makes fixes directly using its tools
5. Pipeline verifies fixes in next iteration

**Model Configuration**: The pipeline defaults to using Claude 3.5 Sonnet (`--model sonnet`) for optimal balance of speed and capability.

## Usage Example

```rust
use spiral_core::discord::self_update::ValidationPipeline;

async fn validate_update() -> Result<()> {
    let mut pipeline = ValidationPipeline::new();
    let context = pipeline.execute().await?;
    
    match context.final_status {
        PipelineStatus::Success => {
            println!("Validation passed on first attempt!");
        }
        PipelineStatus::SuccessWithRetries => {
            println!("Validation passed after {} iterations", 
                     context.pipeline_iterations);
        }
        PipelineStatus::Failure => {
            eprintln!("Validation failed: {:?}", 
                      context.critical_errors);
        }
    }
    
    Ok(())
}
```

## Testing

The pipeline includes comprehensive tests in `src/discord/self_update/tests.rs`:

- Unit tests for context serialization
- Timeout calculation tests
- Retry tracking tests
- Pattern detection tests
- File modification deduplication tests

Run tests with:

```bash
cargo test self_update::tests
```

## Future Enhancements

Potential improvements for the pipeline:

1. **Parallel Phase 1 Checks**: Run AQA checks concurrently
2. **Custom Agent Configuration**: Allow project-specific agents
3. **Metrics Collection**: Track success rates and common issues
4. **Learning System**: Adapt based on historical patterns
5. **Progressive Validation**: Quick checks first, deep analysis later

## Troubleshooting

### Common Issues

1. **Pipeline Timeout**: Increase timeout limits or optimize checks
2. **Excessive Retries**: Review agent prompts for clarity
3. **Rollback Failures**: Ensure Git repository is clean before updates
4. **Claude API Errors**: Check API key and rate limits

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=debug cargo run
```

### Manual Pipeline Execution

For testing, you can manually trigger the pipeline:

```rust
let pipeline = ValidationPipeline::new();
let result = pipeline.execute().await;
```

## Related Documentation

- [Self-Update Pipeline Specification](SELF_UPDATE_PIPELINE_IMPROVEMENT.md)
- [Coding Standards](CODING_STANDARDS.md)
- [Self-Update Guide](SELF_UPDATE_GUIDE.md)
- [Claude Completion Checklist](CLAUDE_COMPLETION_CHECKLIST.md)
