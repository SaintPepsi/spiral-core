# Aggressive Proximity Audit Documentation Standard

## Overview

This document defines the standard for aggressive proximity audit documentation in the Spiral Core codebase. These comments provide critical context directly in the code where it matters most, enabling faster code reviews, better security audits, and clearer architectural understanding.

## Core Principle: Documentation Lives with Code

> "The best documentation is written where the decision is made."

Documentation should be aggressive (immediately visible), proximate (next to the relevant code), and auditable (traceable to requirements and decisions).

## Required Documentation Patterns

### 1. Architecture Decisions

```rust
/// 🏗️ ARCHITECTURE DECISION: [Decision summary]
/// Why: [Core reasoning - why this approach]
/// Alternative: [What was considered and rejected]
/// Audit: [What to verify during review]
/// Trade-off: [What we gain vs what we lose]
```

**Example:**

```rust
/// 🏗️ ARCHITECTURE DECISION: Two-phase validation pipeline over single pass
/// Why: Separates quality checks from compliance, allowing targeted fixes
/// Alternative: Single validation pass (rejected: mixes concerns, harder to debug)
/// Audit: Verify Phase 2 doesn't depend on Phase 1 for SOLID compliance
/// Trade-off: More complex but more maintainable and debuggable
pub struct ValidationPipeline { ... }
```

### 2. Security Decisions

```rust
/// 🛡️ SECURITY DECISION: [Security control]
/// Why: [Threat being mitigated]
/// Attack Vector: [What attack this prevents]
/// Audit: [Security verification steps]
```

**Example:**

```rust
/// 🛡️ SECURITY DECISION: Input validation with strict regex patterns
/// Why: Prevents injection attacks and resource exhaustion
/// Attack Vector: XSS, SQL injection, command injection via user input
/// Audit: Verify all user input passes through this validator
const SAFE_CONTENT_REGEX: &str = r"^[a-zA-Z0-9\s\-_.,!?@#$%^&*()\[\]{}|\\/<>:;'\"+=~`]+$";
```

### 3. Performance Decisions

```rust
/// ⚡ PERFORMANCE DECISION: [Optimization approach]
/// Why: [Performance gain explanation]
/// Benchmark: [Expected or measured impact]
/// Alternative: [What simpler approach was avoided]
```

**Example:**

```rust
/// ⚡ PERFORMANCE DECISION: Reuse command instances with Arc
/// Why: Prevents allocation overhead in hot path
/// Benchmark: 40% reduction in command execution time
/// Alternative: Create new Command each time (rejected: GC pressure)
let cmd = Arc::new(Command::new("cargo"));
```

### 4. Critical Audit Checkpoints

```rust
/// 🔍 AUDIT CHECKPOINT: [What must be verified]
/// Risk: [What could go wrong]
/// Verify: [Specific verification steps]
```

**Example:**

```rust
/// 🔍 AUDIT CHECKPOINT: Claude agent spawning with user context
/// Risk: Privilege escalation if context includes sensitive data
/// Verify: Context is sanitized, no secrets in prompt, timeout enforced
async fn spawn_claude_agent(&mut self, agent_path: &str, context: &PipelineContext)
```

### 5. DRY Pattern Documentation

```rust
/// 🔄 DRY PATTERN: [Pattern name]
/// Replaces: [What duplicate code this eliminates]
/// Usage: [When to use this pattern]
/// Example: [Simple usage example]
```

**Example:**

```rust
/// 🔄 DRY PATTERN: Generic check-fix-retry
/// Replaces: 5 duplicate Phase 2 check implementations (~300 lines)
/// Usage: Any validation that needs retry with optional auto-fix
/// Example: self.run_check_with_retry("tests", "cargo", &["test"], agent, None)
async fn run_check_with_retry(...) -> Result<ComplianceCheck>
```

### 6. SOLID Principle Application

```rust
/// 📐 SOLID: [Principle being applied]
/// Responsibility: [Single clear responsibility]
/// Dependencies: [What this depends on/doesn't depend on]
/// Extension: [How this can be extended]
```

**Example:**

```rust
/// 📐 SOLID: Single Responsibility + Dependency Inversion
/// Responsibility: Execute Phase 2 checks ONLY - no Phase 1 concerns
/// Dependencies: None on Phase 1 - can run completely standalone
/// Extension: Add new checks by implementing ComplianceCheck trait
pub struct Phase2Executor {
    pub claude_client: Option<ClaudeCodeClient>,
    pub start_time: Instant,
}
```

### 7. Error Recovery Strategy

```rust
/// 🔧 ERROR RECOVERY: [Recovery approach]
/// Failure Mode: [What failures are expected]
/// Recovery: [How we recover]
/// Fallback: [Ultimate fallback if recovery fails]
```

**Example:**

```rust
/// 🔧 ERROR RECOVERY: Git snapshot with rollback
/// Failure Mode: Validation changes break the build
/// Recovery: Rollback to snapshot on validation failure
/// Fallback: Manual git reset if snapshot corrupted
async fn create_validation_snapshot(&mut self) -> Result<()>
```

## Enforcement Strategy

### 1. Add to CLAUDE.md

Add this requirement to CLAUDE.md:

```markdown
## 🔍 Aggressive Proximity Audit Documentation

EVERY significant function, struct, and decision point MUST have aggressive proximity audit documentation.

Before implementing ANY feature:

1. Document the ARCHITECTURE DECISION
2. Identify SECURITY DECISION points
3. Mark AUDIT CHECKPOINT locations
4. Note PERFORMANCE DECISION trade-offs

Use the patterns from [AUDIT_DOCUMENTATION_STANDARD.md](docs/AUDIT_DOCUMENTATION_STANDARD.md).

**Rule**: If a reviewer would ask "why?", the answer should already be in a comment.
```

### 2. Code Review Checklist

Add to code review requirements:

- [ ] Architecture decisions documented
- [ ] Security decisions explained
- [ ] Audit checkpoints marked
- [ ] Performance trade-offs noted
- [ ] DRY patterns documented
- [ ] SOLID principles noted where applied

### 3. Claude Improver Integration

Add triggers to `.claude/utility-agents/claude-improver.md`:

- "Missing architecture decision"
- "Undocumented security control"
- "No audit checkpoint"
- "Unclear performance trade-off"

## Examples of Missing Documentation

### ❌ BAD - No context

```rust
pub async fn execute(&mut self) -> Result<PipelineContext> {
    // 600 lines of complex logic with no explanation
}
```

### ✅ GOOD - Aggressive proximity documentation

```rust
/// 🏗️ ARCHITECTURE DECISION: Two-phase validation with pipeline looping
/// Why: Quality checks (Phase 1) inform compliance fixes (Phase 2)
/// Alternative: Single pass (rejected: can't detect systemic issues)
/// Audit: Verify MAX_PIPELINE_ITERATIONS prevents infinite loops
/// Trade-off: Longer execution for higher confidence
pub async fn execute(&mut self) -> Result<PipelineContext> {
    /// 🔍 AUDIT CHECKPOINT: Snapshot creation for rollback safety
    /// Risk: Validation changes could break the system
    /// Verify: Snapshot created before ANY changes
    self.create_validation_snapshot().await?;

    // ... rest of implementation
}
```

## Documentation Priorities

1. **CRITICAL** - Must document immediately:

   - Security boundaries
   - Authentication/authorization points
   - Resource management decisions
   - External API calls
   - Subprocess execution

2. **HIGH** - Document before merging:

   - Architectural patterns
   - Performance optimizations
   - Error recovery strategies
   - SOLID principle applications

3. **MEDIUM** - Document during refactoring:
   - DRY pattern extractions
   - Complex algorithms
   - Business logic decisions

## Benefits

1. **Faster Reviews**: Reviewers understand intent immediately
2. **Better Security**: Audit points clearly marked
3. **Easier Debugging**: Decisions documented at point of implementation
4. **Knowledge Transfer**: New developers understand "why" not just "what"
5. **Compliance**: Audit trail for security and architectural reviews

## Tooling Support

### VS Code Snippets

Add these snippets for quick documentation:

```json
{
  "Architecture Decision": {
    "prefix": "arch",
    "body": [
      "/// 🏗️ ARCHITECTURE DECISION: ${1:decision}",
      "/// Why: ${2:reasoning}",
      "/// Alternative: ${3:rejected} (rejected: ${4:why})",
      "/// Audit: ${5:verify}",
      "/// Trade-off: ${6:gain_vs_loss}"
    ]
  },
  "Security Decision": {
    "prefix": "sec",
    "body": [
      "/// 🛡️ SECURITY DECISION: ${1:control}",
      "/// Why: ${2:threat}",
      "/// Attack Vector: ${3:attacks}",
      "/// Audit: ${4:verify}"
    ]
  }
}
```

## Metrics for Success

Track these metrics to measure documentation quality:

1. **Coverage**: % of public functions with documentation
2. **Audit Points**: Number of marked audit checkpoints
3. **Review Speed**: Time to review PRs (should decrease)
4. **Security Issues**: Found during review vs production (review should catch more)
5. **Onboarding Time**: New developer productivity (should improve)

## Remember

> "Code tells you how, comments tell you why, aggressive proximity audit documentation tells you both at the point of decision."

The goal is not more documentation, but better documentation that lives where it's needed most.
