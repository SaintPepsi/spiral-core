# Coding Examples

This document contains detailed code examples extracted from CLAUDE.md to reduce its size while preserving important patterns.

## Architecture Verification Example

```bash
# ❌ WRONG: Assuming separate services based on common patterns
# "Most systems have separate API and bot processes, so I'll create both"

# ✅ RIGHT: Check the actual architecture first
grep -r "Discord" src/ --include="*.rs"  # Find how Discord is integrated
cargo tree | grep discord  # Check if Discord is a dependency
# Discovery: Discord bot is in src/discord/, integrated into main binary
```

## Script Creation Examples

❌ **BAD - Complex inline commands**:

```bash
echo "=== Build Check ===" && cargo build --lib 2>&1 | grep -c "error" && echo "=== Test Check ===" && cargo test --lib --quiet 2>&1 | grep -E "test result"
```

✅ **GOOD - Create a reusable script**:

```bash
# Create scripts/validate-quality.sh
#!/bin/bash
set -e

echo "=== Running comprehensive checks ==="
echo "1. Build check..."
cargo build --lib

echo "2. Test check..."
cargo test --lib

echo "3. Format check..."
cargo fmt -- --check

echo "4. Clippy check..."
cargo clippy --all-targets

echo "✅ All checks passed!"
```

## Rust String Formatting Examples

```rust
// ✅ GOOD - Inline variables (modern Rust style)
format!("User {user_id} has {count} items")
println!("Error: {error}")
log::info!("Processing file: {file_path}")
response.push_str(&format!("{completion_message}\n\n"))
write!(f, "Task {task_id} status: {status}")

// ❌ BAD - Positional arguments (causes Clippy warnings)
format!("User {} has {} items", user_id, count)
println!("Error: {}", error)
log::info!("Processing file: {}", file_path)
response.push_str(&format!("{}\n\n", completion_message))
write!(f, "Task {} status: {}", task_id, status)
```

**Exception**: Use positional when you need special formatting:

```rust
format!("{:.2}", value)  // Format with 2 decimal places
format!("{:?}", debug_struct)  // Debug formatting
format!("{:>10}", padded)  // Right-aligned with padding
```

## YAGNI Examples

```rust
// ❌ BAD - Creating unused agent types "for the future"
enum AgentType {
    SoftwareDeveloper,  // ✅ Implemented
    ProjectManager,     // ✅ Implemented
    QualityAssurance,   // ❌ Not implemented
    DecisionMaker,      // ❌ Not implemented
    CreativeInnovator,  // ❌ Not implemented
}

// ✅ GOOD - Only what's implemented
enum AgentType {
    SoftwareDeveloper,
    ProjectManager,
    // Add more when actually implemented
}
```

## Problem Space Boundary Examples

- ✅ GOOD: "Let me check Cargo.toml to see what HTTP client we're using"
- ❌ BAD: "Let's use reqwest for HTTP" (without checking if it's already in use)
- ✅ GOOD: "I see we're using tokio, so I'll use tokio::spawn"
- ❌ BAD: "Let's add async-std for better performance" (introducing new dependency)

## No Bullshit Code Examples

- ❌ BAD: `start_discord_bot()` that just sleeps and logs "Discord bot active"
- ✅ GOOD: Omit the function entirely if Discord is integrated into main process
- ❌ BAD: `check_database()` that always returns true without checking
- ✅ GOOD: `# Database check not implemented` comment instead of fake function
- ❌ BAD: Creating functions for every "expected" service in a startup script
- ✅ GOOD: Only create functions for services that actually exist

## No Half-Broken State Examples

- ❌ BAD: Creating service modules but not integrating them into the orchestrator
- ✅ GOOD: Creating services AND updating orchestrator to use them
- ❌ BAD: Leaving both old and new implementations side by side
- ✅ GOOD: Fully replacing old implementation with new one

## God Object Prevention Example

```rust
// ❌ BAD: God object with 11+ responsibilities
pub struct AgentOrchestrator {
    agents: Arc<RwLock<HashMap<AgentType, Box<dyn Agent>>>>,
    task_queue: Arc<Mutex<Vec<Task>>>,
    results: Arc<Mutex<HashMap<String, TaskResult>>>,
    statuses: Arc<RwLock<HashMap<AgentType, AgentStatus>>>,
    // ... 7 more collections of responsibilities
}

// ✅ GOOD: Composed services with single responsibilities  
pub struct AgentOrchestrator {
    task_queue: TaskQueue,        // Only queuing
    agent_registry: AgentRegistry, // Only registration
    result_store: ResultStore,     // Only storage
    status_manager: StatusManager, // Only status tracking
}
```

## Proximity Audit Documentation Example

```rust
/// 🏗️ ARCHITECTURE DECISION: Two-phase validation pipeline
/// Why: Separates quality checks from compliance for targeted fixes
/// Alternative: Single pass (rejected: mixes concerns)
/// Audit: Verify Phase 2 doesn't depend on Phase 1
/// Trade-off: More complex but more maintainable
pub struct ValidationPipeline { ... }
```

## Check-Fix-Retry Pattern

```rust
async fn run_check_with_retry(
    &mut self,
    check_name: &str,
    command: &str,
    args: &[&str],
    fix_agent: Option<&str>,     // Claude agent to fix issues
    auto_fix: Option<(&str, &[&str])>, // Auto-fix command
) -> Result<CheckResult> {
    const MAX_ATTEMPTS: u8 = 3;

    for attempt in 1..=MAX_ATTEMPTS {
        // Run check
        let result = run_command(command, args).await?;
        if result.success() { return Ok(CheckResult::Success); }

        // Try fix if not last attempt
        if attempt < MAX_ATTEMPTS {
            // Try auto-fix first, then Claude agent
            if let Some(fix) = auto_fix { /* run fix */ }
            if let Some(agent) = fix_agent { /* spawn agent */ }
        }
    }

    Err("Check failed after max attempts")
}
```

## Error Handling Pattern

```rust
// ✅ GOOD - Early return pattern
if !condition {
    return Err("Condition not met");
}
// Continue with happy path

// ❌ BAD - Nested if blocks
if condition {
    // Happy path code
} else {
    return Err("Condition not met");
}
```

## Result Chaining

```rust
// ✅ GOOD - Chain operations
let result = operation1()?
    .operation2()
    .and_then(|x| operation3(x))?;

// ❌ BAD - Nested match statements
match operation1() {
    Ok(val1) => match val1.operation2() {
        Ok(val2) => operation3(val2),
        Err(e) => Err(e),
    },
    Err(e) => Err(e),
}
```

## Code Templates

### New Check Implementation

```rust
async fn run_[check_name]_check(&mut self) -> Result<ComplianceCheck> {
    self.run_check_with_retry(
        "[check_name]",           // Check identifier
        "command",                 // Command to run
        &["args"],                // Command arguments
        Some("path/to/agent.md"), // Optional Claude agent
        Some(("fix_cmd", &["fix_args"])), // Optional auto-fix
    ).await
}
```

### Agent Integration Pattern

```rust
let context = self.create_context(error_info);
let response = self.spawn_claude_agent(agent_path, &context).await?;

if response.success {
    // Agent executed, retry the operation
    retries += 1;
} else {
    warn!("Agent failed: {}", response.explanation);
}
```
