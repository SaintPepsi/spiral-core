# Claude Code Spawn Example

This document demonstrates how to spawn Claude Code with custom prompts for validation and code review tasks.

## Overview

The simplified approach allows you to spawn Claude Code instances with custom prompts without complex infrastructure. This is useful for:

- Additional validation beyond the standard 5-step pipeline
- Security reviews with specific focus areas
- Architecture analysis for large changes
- Custom code review scenarios

## Implementation

### 1. Simple Spawn Function

```rust
use spiral_core::discord::self_update::claude_spawn_example::simple_claude_spawn;

// Spawn Claude Code with a custom prompt
simple_claude_spawn("Review this code for security vulnerabilities").await?;
```

### 2. Integration Example

See `examples/claude_validation_example.rs` for a complete example that:

1. Runs standard validation (cargo check, test, fmt, clippy, doc)
2. Optionally spawns Claude Code based on criteria
3. Uses custom prompts for different scenarios

### 3. Spawn Methods

The example provides three integration patterns:

```rust
// Option 1: Claude Code CLI
let output = std::process::Command::new("claude-code")
    .arg("--prompt")
    .arg(prompt)
    .output()?;

// Option 2: Claude Code API
let client = ClaudeCodeClient::new(api_key);
let response = client.execute_prompt(prompt).await?;

// Option 3: Task API with specific agent
let task = Task {
    subagent_type: "security-inquisitor",
    description: "Security review",
    prompt: prompt.to_string(),
};
let result = client.create_task(task).await?;
```

## Usage in Self-Update Pipeline

To use Claude validation in your self-update workflow:

1. **Trigger Conditions**: Claude validation runs when:
   - Description contains "security", "auth", or "architecture"
   - User explicitly requests with "@claude" in the description
   - High-risk changes are detected

2. **Custom Prompts**: Use prompt templates for common scenarios:

   ```rust
   let prompt = prompt_templates::security_review(&changed_files);
   spawn_claude_with_prompt(&prompt).await?;
   ```

3. **Integration Points**: Add after standard validation:

   ```rust
   // Run standard validation
   UpdateValidator::validate_changes().await?;

   // Optionally add Claude validation
   if should_run_claude_validation(request) {
       spawn_claude_with_custom_prompt(prompt).await?;
   }
   ```

## Files

- **Implementation**: `src/discord/self_update/claude_spawn_example.rs`
- **Usage Example**: `examples/claude_validation_example.rs`
- **Placeholder Integration**: `src/discord/self_update/claude_validation.rs`

## Next Steps

When Claude Code Task API becomes available:

1. Replace placeholder implementations in `claude_validation.rs`
2. Update spawn functions to use actual API calls
3. Parse responses into `ValidationFinding` structs
4. Integrate findings into validation pipeline results

## Testing

Run the example:

```bash
cargo run --example claude_validation_example
```

Run tests:

```bash
cargo test claude_spawn_example --lib
```
