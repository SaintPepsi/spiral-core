# Common Code Patterns for Spiral Core

This document provides reusable patterns to maintain DRY code and consistent implementation across the codebase.

## Table of Contents

1. [Validation Patterns](#validation-patterns)
2. [Error Handling Patterns](#error-handling-patterns)
3. [Async Patterns](#async-patterns)
4. [Testing Patterns](#testing-patterns)
5. [Agent Integration Patterns](#agent-integration-patterns)

## Validation Patterns

### Generic Check-Fix-Retry Pattern (attempt_action_with_retry)

Use this pattern for any validation that may need fixes. This is our core DRY pattern for Phase 2 validation:

**Real Example from Validation Pipeline:**

```rust
// Define agent paths as constants - single source of truth
const CLAUDE_AGENT_COMPILATION_FIX: &str = ".claude/validation-agents/phase2/compilation-resolver.md";
const CLAUDE_AGENT_TEST_FIX: &str = ".claude/validation-agents/phase2/test-resolver.md";
const CLAUDE_AGENT_FORMAT_FIX: &str = ".claude/validation-agents/phase2/formatting-resolver.md";
const CLAUDE_AGENT_CLIPPY_FIX: &str = ".claude/validation-agents/phase2/clippy-resolver.md";
const CLAUDE_AGENT_DOC_FIX: &str = ".claude/validation-agents/phase2/doc-builder.md";

/// Generic action-with-retry pattern - the DRY approach
/// This replaced 5 duplicate implementations (~300 lines) with one generic method
async fn attempt_action_with_retry(
    &mut self,
    action_name: &str,
    command: &str,
    args: &[&str],
    claude_agent_path: Option<&str>,
    auto_fix_command: Option<(&str, &[&str])>,
) -> Result<ComplianceCheck> {
    const MAX_ATTEMPTS: u8 = 3;
    let mut retries = 0;
    let mut all_errors = vec![];
    
    for attempt in 1..=MAX_ATTEMPTS {
        info!("{} - Attempt {}/{}", action_name, attempt, MAX_ATTEMPTS);
        
        // Run the check
        let output = self.run_command_with_timeout(command, args).await?;
        
        if output.status.success() {
            return Ok(ComplianceCheck { 
                passed: true, 
                retries, 
                errors: if all_errors.is_empty() { None } else { Some(all_errors) }
            });
        }
        
        // Capture error
        let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
        all_errors.push(format!("Attempt {}: {}", attempt, error_msg));
        
        if attempt < MAX_ATTEMPTS {
            // Try auto-fix first if available (e.g., cargo fmt)
            if let Some((fix_cmd, fix_args)) = auto_fix_command {
                let fix_output = self.run_command_with_timeout(fix_cmd, fix_args).await?;
                if fix_output.status.success() {
                    retries += 1;
                    continue; // Retry the check
                }
            }
            
            // If auto-fix failed or unavailable, spawn Claude agent
            if let Some(agent) = claude_agent_path {
                let fix_context = self.create_fix_context(action_name, &error_msg, retries);
                let response = self.spawn_claude_agent(agent, &fix_context).await?;
                retries += 1;
            }
        }
    }
    
    Ok(ComplianceCheck { 
        passed: false, 
        retries, 
        errors: Some(all_errors) 
    })
}

// Usage - clean, DRY implementation replacing ~300 lines of WET code:
async fn run_compilation_check(&mut self) -> Result<ComplianceCheck> {
    self.attempt_action_with_retry(
        "Compilation Check",
        "cargo",
        &["check", "--all-targets"],
        Some(CLAUDE_AGENT_COMPILATION_FIX),
        None,  // No auto-fix for compilation
    ).await
}

async fn run_formatting_check(&mut self) -> Result<ComplianceCheck> {
    self.attempt_action_with_retry(
        "Formatting Check",
        "cargo",
        &["fmt", "--", "--check"],
        Some(CLAUDE_AGENT_FORMAT_FIX),
        Some(("cargo", &["fmt"])),  // Auto-fix available
    ).await
}

async fn run_test_check(&mut self) -> Result<ComplianceCheck> {
    self.attempt_action_with_retry(
        "Test Suite",
        "cargo",
        &["test"],
        Some(CLAUDE_AGENT_TEST_FIX),
        None,  // Tests need Claude to fix
    ).await
}
```

### Command Execution with Timeout

```rust
async fn run_command_with_timeout(
    &self,
    command: &str,
    args: &[&str],
    timeout_duration: Duration,
) -> Result<Output> {
    let cmd_string = format!("{} {}", command, args.join(" "));
    
    timeout(timeout_duration, async {
        Command::new(command)
            .args(args)
            .output()
            .await
            .map_err(|e| format!("Failed to run {}: {}", cmd_string, e))
    })
    .await
    .map_err(|_| format!("{} timed out after {:?}", cmd_string, timeout_duration))?
}
```

## Error Handling Patterns

### Early Return Pattern

Always use early returns with negative conditions:

```rust
// ✅ GOOD
pub async fn process_task(&mut self, task: Task) -> Result<()> {
    // Validate input first
    if task.content.is_empty() {
        return Err("Task content cannot be empty");
    }
    
    if task.priority > Priority::Critical {
        return Err("Invalid priority level");
    }
    
    // Happy path continues without nesting
    self.execute_task(task).await
}

// ❌ BAD - Nested conditions
pub async fn process_task(&mut self, task: Task) -> Result<()> {
    if !task.content.is_empty() {
        if task.priority <= Priority::Critical {
            self.execute_task(task).await
        } else {
            Err("Invalid priority level")
        }
    } else {
        Err("Task content cannot be empty")
    }
}
```

### Result Chain Pattern

```rust
async fn process_pipeline(&mut self) -> Result<Output> {
    self.validate_input()?
        .transform_data()
        .and_then(|data| self.apply_rules(data))?
        .save_results()
        .await
}
```

## Async Patterns

### Parallel Execution Pattern

Run independent async operations in parallel:

```rust
use tokio::join;

async fn run_all_checks(&mut self) -> Result<AllCheckResults> {
    let (compilation, tests, formatting, clippy, docs) = join!(
        self.run_compilation_check(),
        self.run_test_check(),
        self.run_formatting_check(),
        self.run_clippy_check(),
        self.run_doc_check()
    );
    
    Ok(AllCheckResults {
        compilation: compilation?,
        tests: tests?,
        formatting: formatting?,
        clippy: clippy?,
        docs: docs?,
    })
}
```

### Retry with Exponential Backoff

```rust
async fn retry_with_backoff<F, T>(
    operation: F,
    max_retries: u32,
    base_delay: Duration,
) -> Result<T>
where
    F: Fn() -> Result<T>,
{
    let mut delay = base_delay;
    
    for attempt in 0..max_retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries - 1 => {
                warn!("Attempt {} failed: {}. Retrying in {:?}", attempt + 1, e, delay);
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
    
    unreachable!()
}
```

## Testing Patterns

### Test Helper Pattern

Create test helpers to reduce duplication:

```rust
#[cfg(test)]
mod test_helpers {
    use super::*;
    
    pub fn create_test_context() -> TestContext {
        TestContext {
            config: Config::test_config(),
            client: MockClient::new(),
            // ... other test setup
        }
    }
    
    pub async fn run_test_with_cleanup<F>(test_fn: F)
    where
        F: FnOnce(TestContext) -> Result<()>,
    {
        let context = create_test_context();
        let result = test_fn(context);
        cleanup_test_resources().await;
        result
    }
}

#[tokio::test]
async fn test_feature() {
    run_test_with_cleanup(|ctx| {
        // Test logic here
        Ok(())
    }).await;
}
```

### Parameterized Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_check_variations() {
        let test_cases = vec![
            ("cargo", vec!["check"], true),
            ("cargo", vec!["test"], true),
            ("cargo", vec!["invalid"], false),
        ];
        
        for (cmd, args, expected) in test_cases {
            let result = run_command(cmd, &args).await;
            assert_eq!(result.is_ok(), expected, 
                      "Failed for {} {}", cmd, args.join(" "));
        }
    }
}
```

## Agent Integration Patterns

### Claude Agent Spawn Pattern

```rust
async fn spawn_claude_fix(
    &mut self,
    agent_path: &str,
    error_context: &ErrorContext,
) -> Result<bool> {
    // 1. Create context
    let context = self.create_agent_context(error_context);
    
    // 2. Spawn agent
    let response = self.spawn_claude_agent(agent_path, &context).await?;
    
    // 3. Handle response
    match response.success {
        true => {
            info!("Claude agent succeeded: {}", response.explanation);
            self.track_change("claude_fix", &response.explanation);
            Ok(true)
        }
        false => {
            warn!("Claude agent failed: {}", response.explanation);
            Ok(false)
        }
    }
}
```

### Agent Context Builder

```rust
impl AgentContextBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_error(mut self, error: String) -> Self {
        self.errors.push(error);
        self
    }
    
    pub fn with_file(mut self, file: String) -> Self {
        self.files.push(file);
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    pub fn build(self) -> AgentContext {
        AgentContext {
            errors: self.errors,
            files: self.files,
            metadata: self.metadata,
            timestamp: Utc::now(),
        }
    }
}

// Usage
let context = AgentContextBuilder::new()
    .with_error(error_msg)
    .with_file("src/main.rs")
    .with_metadata("check_type", "compilation")
    .build();
```

## Anti-Patterns to Avoid

### ❌ Copy-Paste Programming

```rust
// BAD - Same logic repeated
async fn check_a() {
    let result = command().await;
    if !result { fix().await; }
    retry().await;
}

async fn check_b() {
    let result = command().await;
    if !result { fix().await; }
    retry().await;
}
```

### ❌ God Functions

```rust
// BAD - Function doing too many things
async fn process_everything() {
    // 200 lines of code
    // Multiple responsibilities
    // Hard to test
}
```

### ❌ Magic Numbers

```rust
// BAD
if retries > 3 { /* ... */ }

// GOOD
const MAX_RETRIES: u8 = 3;
if retries > MAX_RETRIES { /* ... */ }
```

### ❌ Stringly-Typed Code

```rust
// BAD
fn process(type: &str) {
    if type == "compilation" { /* ... */ }
    else if type == "test" { /* ... */ }
}

// GOOD
enum CheckType {
    Compilation,
    Test,
}

fn process(check_type: CheckType) {
    match check_type {
        CheckType::Compilation => { /* ... */ }
        CheckType::Test => { /* ... */ }
    }
}
```

## Pattern Selection Guide

| Scenario | Pattern to Use |
|----------|---------------|
| Multiple similar checks | Generic check-fix-retry |
| Async operations that can run together | Parallel execution |
| Operation that might fail transiently | Retry with backoff |
| Complex object construction | Builder pattern |
| Shared test setup | Test helper pattern |
| Multiple test cases with same logic | Parameterized tests |
| Agent integration | Claude agent spawn pattern |

## Remember

- **DRY**: Don't Repeat Yourself - extract common patterns
- **KISS**: Keep It Simple, Stupid - don't over-engineer
- **YAGNI**: You Aren't Gonna Need It - don't add unused abstractions
- **SOLID**: Follow the five principles for maintainable code

When in doubt, invoke the `claude-improver.md` agent to analyze and suggest refactoring opportunities!