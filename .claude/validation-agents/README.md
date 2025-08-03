# Validation Pipeline Agents

This directory contains specialized agents for the two-phase validation pipeline.

## Pipeline Context Structure

All analysis agents receive a structured context object with the following information:

```typescript
interface PipelineContext {
  // Overall execution
  pipelineIterations: number;      // 1-3
  totalDurationMs: number;
  finalStatus: "success" | "success_with_retries" | "failure";
  
  // Phase 1 results (same for all iterations)
  phase1Results: {
    codeReview: { passed: boolean; findings: string[]; duration: number };
    testing: { passed: boolean; findings: string[]; duration: number };
    security: { passed: boolean; findings: string[]; duration: number };
    integration: { passed: boolean; findings: string[]; duration: number };
  };
  
  // Phase 2 attempts (one per pipeline iteration)
  phase2Attempts: Array<{
    iteration: number;
    checks: {
      compilation: { passed: boolean; retries: number; errors?: string[] };
      tests: { passed: boolean; retries: number; failures?: string[] };
      formatting: { passed: boolean; retries: number; files?: number };
      clippy: { passed: boolean; retries: number; warnings?: string[] };
      docs: { passed: boolean; retries: number; issues?: string[] };
    };
    triggeredLoop: boolean;  // Did this attempt cause return to Phase 1?
  }>;
  
  // Changes made
  filesModified: string[];       // Absolute paths
  changesApplied: Array<{
    phase: string;
    description: string;
    files: string[];
  }>;
  
  // Issues encountered
  criticalErrors: string[];      // Errors that blocked progress
  warnings: string[];            // Non-blocking issues
  
  // Improvement areas identified
  patterns: {
    consistentFailures?: string[];  // Checks that failed every iteration
    flakeyChecks?: string[];        // Checks that sometimes pass/fail
    performanceBottlenecks?: string[];  // Slow operations
  };
}
```

## Agent Directory Structure

### Phase 1 Agents (Advanced Quality Assurance)

- `phase1/code-review-standards.md` - Reviews code against project standards
- `phase1/comprehensive-testing.md` - Analyzes test coverage and quality
- `phase1/security-audit.md` - Identifies security vulnerabilities
- `phase1/system-integration.md` - Verifies system integration integrity

### Phase 2 Agents (Core Rust Compliance)

- `phase2/compilation-fixer.md` - Fixes compilation errors
- `phase2/test-failure-analyzer.md` - Analyzes and fixes test failures
- `phase2/formatting-fixer.md` - Applies standard formatting
- `phase2/clippy-resolver.md` - Resolves Clippy warnings
- `phase2/doc-builder.md` - Fixes documentation issues

### Analysis Agents (Pipeline Outcome Analysis)

- `analysis/success-analyzer.md` - Analyzes successful runs (no retries) ✓
- `analysis/success-with-issues-analyzer.md` - Analyzes runs that needed retries ✓
- `analysis/failure-analyzer.md` - Analyzes failed runs (3 iterations exhausted) ✓

## Pipeline Flow

1. **Phase 1**: All quality checks run in sequence
2. **Phase 2**: Compliance checks run in sequence
   - If ANY check needs retry → Loop back to Phase 1
   - Maximum 3 complete iterations
3. **Analysis**: Based on outcome, run appropriate analyzer with context

## Key Design Principles

1. **Focused Agents**: Each agent has ONE specific job
2. **Structured Context**: Clean data instead of raw logs
3. **Clear Success Criteria**: Measurable pass/fail conditions
4. **No Redundancy**: Agents are purposefully separate for clarity
5. **Actionable Output**: Each agent provides specific next steps
