# Claude Improver Agent Usage Guide

## When to Use

The Claude Improver agent should be invoked when you notice:

- Duplicate code patterns
- Long, complex functions
- SOLID principle violations
- WET (Write Everything Twice) code

## How to Invoke

### Manual Invocation

```bash
# When you see duplicate code, invoke the improver
claude-code --agent .claude/utility-agents/claude-improver.md \
           --context "Review src/discord/self_update/pipeline.rs for DRY violations"
```

### Automatic Triggers

The agent is automatically suggested when:

- Same code pattern appears 3+ times
- Function exceeds 50 lines
- Cyclomatic complexity > 10
- Nested conditions go 3+ levels deep

## Example Workflow

### Before: WET Code

```rust
// File: src/checks/mod.rs
async fn check_compilation() -> Result<()> {
    for attempt in 1..=3 {
        let output = Command::new("cargo")
            .args(&["check"])
            .output()
            .await?;
        if output.status.success() {
            return Ok(());
        }
        if attempt < 3 {
            spawn_claude_agent("fix-compilation.md").await?;
        }
    }
    Err("Compilation failed after 3 attempts")
}

async fn check_tests() -> Result<()> {
    for attempt in 1..=3 {
        let output = Command::new("cargo")
            .args(&["test"])
            .output()
            .await?;
        if output.status.success() {
            return Ok(());
        }
        if attempt < 3 {
            spawn_claude_agent("fix-tests.md").await?;
        }
    }
    Err("Tests failed after 3 attempts")
}

async fn check_clippy() -> Result<()> {
    for attempt in 1..=3 {
        let output = Command::new("cargo")
            .args(&["clippy"])
            .output()
            .await?;
        if output.status.success() {
            return Ok(());
        }
        if attempt < 3 {
            spawn_claude_agent("fix-clippy.md").await?;
        }
    }
    Err("Clippy failed after 3 attempts")
}
```

### Invoke Claude Improver

```bash
# Claude Improver detects the DRY violation
claude-code --agent .claude/utility-agents/claude-improver.md \
           --context "Refactor duplicate check patterns in src/checks/mod.rs"
```

### After: DRY Code

```rust
// File: src/checks/mod.rs
async fn run_check_with_retry(
    check_name: &str,
    command: &str,
    args: &[&str],
    fix_agent: Option<&str>,
) -> Result<()> {
    const MAX_ATTEMPTS: u8 = 3;

    for attempt in 1..=MAX_ATTEMPTS {
        let output = Command::new(command)
            .args(args)
            .output()
            .await?;

        if output.status.success() {
            return Ok(());
        }

        if attempt < MAX_ATTEMPTS {
            if let Some(agent) = fix_agent {
                spawn_claude_agent(agent).await?;
            }
        }
    }

    Err(format!("{} failed after {} attempts", check_name, MAX_ATTEMPTS))
}

async fn check_compilation() -> Result<()> {
    run_check_with_retry(
        "Compilation",
        "cargo",
        &["check"],
        Some("fix-compilation.md"),
    ).await
}

async fn check_tests() -> Result<()> {
    run_check_with_retry(
        "Tests",
        "cargo",
        &["test"],
        Some("fix-tests.md"),
    ).await
}

async fn check_clippy() -> Result<()> {
    run_check_with_retry(
        "Clippy",
        "cargo",
        &["clippy"],
        Some("fix-clippy.md"),
    ).await
}
```

## Metrics Report

After refactoring, Claude Improver provides metrics:

```markdown
## Code Quality Analysis Complete

### DRY Violations Fixed

- Extracted common pattern into `run_check_with_retry()`
- Eliminated 3 duplicate implementations

### Metrics Improvement

- Lines of Code: 60 → 35 (-42%)
- Duplication: 70% → 0% (-70%)
- Cyclomatic Complexity: 9 → 3 (-66%)
- Maintainability Index: 45 → 85 (+89%)

### Tests Status

✅ All tests passing after refactoring
✅ No functionality changed
✅ Code coverage maintained at 95%
```

## Integration with CI/CD

Add to your CI pipeline:

```yaml
# .github/workflows/code-quality.yml
name: Code Quality Check

on: [pull_request]

jobs:
  check-dry:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Check for DRY violations
        run: |
          claude-code --agent .claude/utility-agents/claude-improver.md \
                     --mode analyze \
                     --fail-on-duplication 10
```

## Best Practices

1. **Run regularly**: Invoke after implementing new features
2. **Review suggestions**: Not all abstractions are beneficial
3. **Test after refactoring**: Ensure behavior is preserved
4. **Track metrics**: Monitor improvement over time
5. **Document patterns**: Add new patterns to PATTERNS.md

## Common Refactoring Patterns Applied

| Pattern                               | When Applied          | Benefit                  |
| ------------------------------------- | --------------------- | ------------------------ |
| Extract Method                        | Code block > 10 lines | Improves readability     |
| Extract Variable                      | Complex expression    | Clarifies intent         |
| Replace Magic Number                  | Hardcoded values      | Improves maintainability |
| Introduce Parameter Object            | > 3 parameters        | Reduces complexity       |
| Replace Conditional with Polymorphism | Large match/if chains | Enables extension        |

## Troubleshooting

### Agent suggests over-abstraction

- Use judgment - not every duplication needs abstraction
- Consider if the "duplicate" code might diverge in the future
- Prefer clarity over cleverness

### Tests fail after refactoring

- The agent preserves behavior but edge cases might be missed
- Review the changes carefully
- Add more test coverage for the refactored code

### Performance degradation

- Some abstractions add overhead
- Profile before and after if performance is critical
- Consider inline hints for hot paths

## Remember

> "Premature optimization is the root of all evil" - Donald Knuth

But also:

> "Copy and paste is a design error" - David Parnas

Balance is key. Use Claude Improver to identify opportunities, but apply human judgment on whether the refactoring improves the code.
