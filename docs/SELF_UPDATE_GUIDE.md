# 🔄 Spiral Core Self-Update System Guide

This guide explains how to use the self-update system that allows Spiral Core to improve itself through Claude Code integration.

## 🚀 Quick Start

### Triggering an Update

To trigger a self-update, mention the bot with an update-related keyword:

```
@SpiralConstellation update the error handling in the Discord bot
```

**Update Keywords**:

- update, fix, modify, change, improve, enhance, repair, correct, adjust, patch, upgrade

**Examples**:

```
@SpiralConstellation fix the rate limiting bug
@SpiralConstellation improve the error messages
@SpiralConstellation update the command handler to be more robust
```

## 🔐 Authorization

Only authorized users can trigger self-updates. Check `.env` for the `DISCORD_AUTHORIZED_USERS` list.

## 📋 Update Process

When you trigger an update, the system follows these steps:

1. **🔍 Pre-flight Checks**

   - Verifies git repository is clean
   - Checks available disk space (>100MB required)
   - Ensures dependencies are available
   - Validates Claude Code connectivity

2. **📸 Snapshot Creation**

   - Creates a git snapshot with unique identifier
   - Allows rollback if update fails

3. **🔧 Execution**

   - Claude Code analyzes the request
   - Implements necessary changes
   - Focuses on minimal, targeted modifications

4. **✅ Validation**

   - Runs `cargo check` for compilation
   - Executes unit tests
   - Performs security validation

5. **🎉 Completion**
   - Update marked as completed
   - Or rolled back if validation fails

## 📊 Queue Management

- **Max Queue Size**: 10 requests
- **Max Content Size**: 64KB per request
- **Duplicate Prevention**: Same codename cannot be queued twice

## 🛡️ Safety Features

1. **Bounded Resources**: Prevents DoS through queue limits
2. **Input Sanitization**: All inputs sanitized for git operations
3. **Atomic Operations**: Updates fully succeed or fully rollback
4. **Authorization Required**: Only trusted users can update

## 📝 Status Messages

The bot will send status updates for each phase:

- "🔍 Running pre-flight checks..."
- "📸 Creating system snapshot..."
- "🚀 Executing system changes..."
- "✅ Validating changes..."
- "🎉 Update completed successfully!"

## 🚨 Troubleshooting

### Update Rejected

- Check if queue is full (max 10 requests)
- Ensure description isn't too large (64KB limit)
- Verify you're authorized

### Pre-flight Failed

- Commit any pending changes
- Ensure >100MB disk space
- Check git repository health

### Validation Failed

- Review error messages
- Check compilation errors
- Verify test failures

## 🔧 Advanced Usage

### Checking Queue Status

```rust
let status = queue.get_status().await;
println!("Queue: {}/{}", status.queue_size, status.max_size);
```

### Manual Rollback

```bash
# List available snapshots
git log --grep=pre-update-snapshot --oneline -n 20

# Rollback to specific snapshot
git reset --hard <commit-hash>
```

## Best Practices

Be specific about what's broken. Fix one thing at a time.

## 🌟 Example Updates

Good examples:

- `@SpiralConstellation fix the memory leak in message handler`
- `@SpiralConstellation add error handling to the rate limiter`
- `@SpiralConstellation improve the agent selection logic`

Poor examples:

- `@SpiralConstellation make it better` (too vague)
- `@SpiralConstellation rewrite everything` (too broad)
- `@SpiralConstellation update` (no description)

---

*Self-updates enable continuous system improvement through practical collaboration.*
