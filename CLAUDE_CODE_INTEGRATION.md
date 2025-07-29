# Claude Code Integration

This document describes the Claude Code CLI integration in Spiral Core and how to handle permissions and limitations.

## Overview

Spiral Core now uses Claude Code CLI instead of the direct Claude API, providing enhanced capabilities:

- **Codebase Awareness**: Understanding of project structure and files
- **Built-in Tools**: Direct file operations, git integration, testing tools
- **Better Context**: Access to full project environment
- **Session Management**: Continuity across related tasks using Claude Code's session system
- **Workspace Isolation**: Each session gets its own secure workspace directory

## Permission Configuration

### Environment Variables

```bash
# Optional: specify Claude binary location
CLAUDE_BINARY_PATH=/usr/local/bin/claude

# Optional: set relative working directory for Claude operations
# Note: For security, workspaces are always created as child directories
# of the current working directory, never in /tmp or absolute paths
CLAUDE_WORKING_DIR=workspaces

# Claude execution timeout (seconds)
CLAUDE_TIMEOUT_SECONDS=300

# Permission mode: acceptEdits, bypassPermissions, default, plan
CLAUDE_PERMISSION_MODE=acceptEdits

# Allowed tools (comma-separated)
CLAUDE_ALLOWED_TOOLS=Edit,Write,Read,Bash,MultiEdit,Glob,Grep,TodoWrite,NotebookEdit,WebFetch
```

### Permission Modes

1. **`acceptEdits`** (default for production)
   - Automatically accepts file edits and writes
   - Safe for trusted environments
   - Requires explicit tool allowlist

2. **`bypassPermissions`** (default for debug builds)
   - Bypasses all permission checks
   - Use only in sandboxed environments
   - Maximum capability access

3. **`default`**
   - Interactive prompts for permissions
   - Not suitable for automated systems

4. **`plan`**
   - Creates execution plans without running
   - Useful for review workflows

## Automatic Limitation Detection

The system monitors Claude Code responses for common limitation patterns:

### Permission Issues

- "need permission" → Check `--permission-mode` setting
- "cannot write" → Verify `Write,Edit,MultiEdit` in allowed tools
- "cannot read" → Verify `Read,Glob,Grep` in allowed tools
- "access denied" → Check working directory permissions

### Resource Issues

- "timeout" → Increase `CLAUDE_TIMEOUT_SECONDS`
- "rate limit" → Implement backoff strategy
- "quota exceeded" → Monitor API usage

### Tool Issues

- "command not found" → Verify tool availability in PATH
- "invalid tool" → Check `--allowedTools` configuration

## Fallback Behavior

The system automatically attempts fallback when permission issues are detected:

1. **Initial Attempt**: Uses configured permission mode
2. **Fallback**: Switches to `bypassPermissions` for permission failures
3. **Logging**: All limitation patterns are logged with improvement suggestions

## Best Practices

### Development Environment

```bash
CLAUDE_PERMISSION_MODE=bypassPermissions
CLAUDE_ALLOWED_TOOLS=Edit,Write,Read,Bash,MultiEdit,Glob,Grep,TodoWrite,WebFetch
```

### Production Environment

```bash
CLAUDE_PERMISSION_MODE=acceptEdits
CLAUDE_ALLOWED_TOOLS=Edit,Write,Read,MultiEdit,Glob,Grep
CLAUDE_WORKING_DIR=/app/workspace
```

### Sandboxed Environment

```bash
CLAUDE_PERMISSION_MODE=bypassPermissions
CLAUDE_ALLOWED_TOOLS=Edit,Write,Read,Bash,MultiEdit,Glob,Grep,TodoWrite,NotebookEdit,WebFetch
CLAUDE_WORKING_DIR=/sandbox
```

## Monitoring

The system logs detailed information about:

- **Successful Operations**: File creations, modifications, implementations
- **Limitation Patterns**: Permission denials, access issues, tool restrictions
- **Performance Metrics**: Execution duration, API costs, token usage
- **Fallback Attempts**: When and why permission fallbacks occur

## Testing Claude Code Integration

### Running API Tests

1. **Start the server with Claude Code configuration:**

```bash
# Copy environment file for testing
cp .env.hurl .env

# Start the API server
cargo run --bin spiral-core
```

2. **Run Claude Code integration tests:**

```bash
# Test the Claude Code integration specifically
hurl --test src/api/tests/hurl/claude-code-integration.hurl --variables-file .env.hurl

# Run all API tests
hurl --test src/api/tests/hurl/*.hurl --variables-file .env.hurl
```

3. **Manual testing:**

```bash
# Test simple code generation
curl -X POST http://localhost:3000/tasks \
  -H "x-api-key: test-api-key-1234567890123456789012345678901234567890" \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "SoftwareDeveloper",
    "content": "Create a simple calculator function in Rust",
    "priority": "Medium"
  }'

# Test task analysis
curl -X POST http://localhost:3000/tasks/{TASK_ID}/analyze \
  -H "x-api-key: test-api-key-1234567890123456789012345678901234567890" \
  -H "Content-Type: application/json" \
  -d '{
    "agent_type": "SoftwareDeveloper",
    "content": "Create a simple calculator function in Rust",
    "priority": "Medium"
  }'
```

### Session Management & Workspace Inspection

Check generated workspaces and sessions:

```bash
# View all workspaces (from your project root)
ls -la ./claude-workspaces/

# Inspect session-specific workspaces (persistent across related tasks)
ls -la ./claude-workspaces/session-{TASK_ID}/

# Inspect one-time workspaces (for isolated tasks)
ls -la ./claude-workspaces/workspace-{UUID}/

# View all workspaces with details
find ./claude-workspaces -name "session-*" -o -name "workspace-*" -type d -exec ls -la {} \;
```

### Session Continuity

When creating related tasks, use the same task ID context to maintain session continuity:

```json
{
  "agent_type": "SoftwareDeveloper",
  "content": "Continue building the calculator by adding error handling",
  "context": {
    "continues_from": "previous_task_id",
    "project_name": "calculator"
  }
}
```

This ensures Claude Code continues in the same workspace with full context of previous work.

## Troubleshooting

### Common Issues

1. **"I need permission to write files"**
   - Solution: Set `CLAUDE_PERMISSION_MODE=acceptEdits` or `bypassPermissions`
   - Add `Write,Edit,MultiEdit` to `CLAUDE_ALLOWED_TOOLS`

2. **"Command not found"**
   - Solution: Ensure tools are available in PATH
   - For Bash operations, add `Bash` to allowed tools

3. **"Access denied"**
   - Solution: Check working directory permissions
   - Verify `CLAUDE_WORKING_DIR` is writable

4. **API rate limiting**
   - Solution: Implement exponential backoff
   - Monitor usage patterns in logs

5. **500 Internal Server Error on task analysis**
   - Solution: Check server logs for Claude Code execution errors
   - Verify Claude Code binary is accessible: `which claude`
   - Ensure proper permissions are configured

6. **"cd to '/path' was blocked" errors**
   - Solution: Workspaces are created as child directories for security
   - Claude Code only allows access to child directories of the current working directory
   - Absolute paths in `CLAUDE_WORKING_DIR` are converted to relative paths automatically

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=spiral_core::claude_code=debug cargo run --bin spiral-core
```

This provides:

- Full Claude Code command execution details
- Raw JSON responses
- Limitation detection analysis
- Fallback attempt information
- Workspace creation and isolation details
