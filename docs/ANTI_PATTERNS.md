# 🚫 Anti-Patterns Documentation

This document catalogs anti-patterns discovered during development to prevent their recurrence. Each anti-pattern includes the problematic code, why it happened, and how to avoid it.

## 1. The Placeholder Function Anti-Pattern

### What Happened

Created a `start_discord_bot()` function in a startup script that didn't actually start anything because the Discord bot was integrated into the main binary.

### The Bad Code

```bash
# Start the Discord bot (if it's a separate process)
start_discord_bot() {
    log_info "Starting Discord bot integration..."

    # The Discord bot is integrated into spiral-core, so this is just a health check
    sleep 2

    # Check if Discord connection is established (you could add actual health check here)
    log_success "Discord bot integration active"
}
```

### Why It Happened

1. **Pattern Matching**: Applied common multi-service architecture patterns without verifying actual architecture
2. **Gap Filling**: Created functions for "expected" components rather than actual components
3. **Misleading Success**: Logged success messages for operations that didn't occur

### How to Prevent

```bash
# ✅ GOOD: Only create functions for actual operations
start_spiral_core() {
    log_info "Starting Spiral Core server (includes Discord bot)..."
    # Actual startup code here
}

# ✅ GOOD: Verify integration if needed, with accurate naming
verify_discord_integration() {
    # Actually check logs or status
    if grep -q "Discord connected" logs/spiral-core.log; then
        log_success "Discord integration confirmed"
    fi
}
```

### Prevention Checklist

- [ ] Does the function name accurately describe what happens?
- [ ] Does the function actually DO something meaningful?
- [ ] Would removing this function break anything?

## 2. The Mock Data Anti-Pattern

### What Happened

Creating fake results or status messages instead of implementing real checks.

### The Bad Code

```rust
// ❌ BAD: Fake status without actual checking
fn get_agent_status() -> String {
    "🟢 Active".to_string()  // Not actually checking anything
}

// ❌ BAD: Mock data to satisfy structure
let phase1_results = Phase1Results {
    passed: true,  // Not based on actual validation
    errors: vec![], // Empty regardless of reality
};
```

### How to Prevent

```rust
// ✅ GOOD: Honest about unimplemented features
fn get_agent_status() -> String {
    "❓ Status check not implemented".to_string()
}

// ✅ GOOD: Actually perform the check or refactor
fn get_agent_status() -> Result<AgentStatus> {
    // Real implementation that checks process, memory, etc.
    let pid = check_process_id()?;
    let memory = get_memory_usage(pid)?;
    Ok(AgentStatus::Active { pid, memory })
}
```

## 3. The Pattern Assumption Anti-Pattern

### What Happened

Assuming this project follows common architectural patterns without verification.

### The Bad Assumption

"Most systems have separate services for API, database, and bot, so I'll create startup functions for each."

### How to Prevent

```bash
# ALWAYS verify architecture before implementing
# 1. Check directory structure
ls -la src/

# 2. Search for integration points
grep -r "Discord" src/ --include="*.rs"

# 3. Check dependencies
cargo tree | grep discord

# 4. Read existing documentation
cat docs/ARCHITECTURE.md
```

## 4. The Comment Excuse Anti-Pattern

### What Happened

Adding a comment acknowledging fake functionality instead of fixing it.

### The Bad Code

```rust
// This doesn't actually check the database but returns success for now
fn check_database() -> bool {
    true  // TODO: Implement real check
}
```

### How to Prevent

```rust
// ✅ GOOD: Don't implement until ready
// Database check will be implemented when database is added

// ✅ GOOD: Make it optional
fn check_database() -> Option<bool> {
    None  // Not implemented
}

// ✅ GOOD: Return error
fn check_database() -> Result<bool> {
    Err(anyhow!("Database check not implemented"))
}
```

## 5. The Kitchen Sink Anti-Pattern

### What Happened

Including every conceivable feature/function because they "might be needed" or are "commonly found" in similar systems.

### The Bad Code

```rust
struct SystemMonitor {
    cpu_monitor: CpuMonitor,      // Not used
    memory_monitor: MemMonitor,    // Not used
    disk_monitor: DiskMonitor,     // Not used
    network_monitor: NetMonitor,   // Not used
    gpu_monitor: GpuMonitor,       // App doesn't use GPU
}
```

### How to Prevent

- Start minimal, add only when needed
- YAGNI (You Aren't Gonna Need It) principle
- Remove unused code immediately

## 6. The Success Theater Anti-Pattern

### What Happened

Logging success messages for operations that didn't actually succeed or weren't performed.

### The Bad Code

```rust
println!("✅ Database optimized");  // No optimization occurred
log::info!("Cache cleared successfully");  // No cache exists
```

### How to Prevent

- Only log what actually happened
- Use conditional logging based on actual results
- Include metrics in success messages when possible

## Prevention Strategy

### Before Writing Any Function

1. **Verify Architecture**

   ```bash
   # Check if component exists
   find . -name "*.rs" | xargs grep -l "ComponentName"
   ```

2. **Question Necessity**

   - Is this solving a real problem?
   - Is this required by the current task?
   - Would the system work without this?

3. **Name Honestly**

   - Does the name describe what actually happens?
   - Could someone understand the function from its name alone?

4. **Implement or Omit**
   - Either implement it properly
   - Or don't create it at all
   - Never create placeholders

### During Code Review

Check for these red flags:

- 🚩 Functions that only log messages
- 🚩 Always-true validation functions
- 🚩 Success messages without operations
- 🚩 Comments explaining why code is fake
- 🚩 TODO comments that could be avoided
- 🚩 Mock data or hardcoded results
- 🚩 Functions named "start" that don't start anything
- 🚩 Functions named "check" that don't check anything

## The Golden Rule

> "If removing this code wouldn't break anything, it shouldn't exist."

Every line of code should have a purpose. Every function should do real work. Every log message should reflect actual events. No bullshit code.
