# Claude Agent Validation Integration

This document describes the Claude agent validation system integrated into the Spiral Core self-update pipeline.

## Overview

The validation pipeline now includes three specialized Claude Code agents that perform comprehensive code review:

1. **Ty Lee Precision Tester** - Identifies critical pressure points and test coverage gaps
2. **Security Inquisitor** - Performs security analysis and threat modeling
3. **Lordgenome Spiral King** - Reviews architectural implications and long-term stability

## Implementation Status

### ✅ Completed

- Simple integration that spawns Claude Code agents
- Mock implementations for testing without Claude Code API
- Integration into the main validation pipeline as Step 6
- Comprehensive test coverage
- Error handling and result formatting

### 🚧 Pending Claude Code Integration

The system is designed to work with Claude Code's Task API but currently uses mock implementations. To complete the integration:

1. **Import Claude Code SDK/Client**

   ```rust
   use claude_code::{Client, Task, TaskResult};
   ```

2. **Replace Mock Calls**

   ```rust
   // Current mock implementation:
   let findings = self.mock_ty_lee_validation(changed_files).await?;
   
   // Replace with simple Task API call:
   let task_result = claude_client.create_task(Task {
       subagent_type: "ty-lee-precision-tester",
       description: "Review changed files",
       prompt: format!("Review these changed files:\n{}", changed_files.join("\n")),
   }).await?;
   
   let findings = self.parse_agent_response(&task_result.response)?;
   ```

3. **Parse Agent Responses**
   Implement parsing logic to convert agent responses into `ValidationFinding` structs.

## Agent Files

Claude Code automatically loads agent definitions from:

- `.claude/agents/ty-lee-precision-tester.md`
- `.claude/agents/security-inquisitor.md`
- `.claude/agents/lordgenome-spiral-king.md`

These files contain the complete agent personalities and instructions. Claude Code handles loading them when you specify the `subagent_type`.

## Configuration

```rust
let config = ClaudeValidationConfig {
    agent_timeout: Duration::from_secs(300), // 5 minutes per agent
    parallel_execution: false,               // Sequential execution
    continue_on_warning: true,               // Continue on non-critical findings
};
```

## Severity Levels

Findings are categorized by severity:

- **Critical** - Must fix, blocks deployment
- **High** - Should fix, security or stability risk
- **Medium** - Consider fixing, quality issue
- **Low** - Nice to fix, minor improvement
- **Info** - Informational only

## Usage

The validation is automatically run as part of `UpdateValidator::validate_changes()`:

```rust
// Step 6: Run Claude agent validation
let changed_files = Self::get_changed_files().await?;
let claude_validator = ClaudeValidator::new(config);
let results = claude_validator.validate_with_agents(changed_files).await?;
```

## Output Format

Results are formatted for Discord display:

```
📋 **Claude Agent Validation Results**

✅ **Ty Lee Precision Tester** - 150ms
   ⚠️ **High Priority:**
   • Missing tests for authentication pressure points
   💡 **Recommendations:**
   • 🎯 Focus testing on authentication boundaries

❌ **Security Inquisitor** - 200ms
   🚨 **Critical Issues:**
   • SQL injection vulnerability in user input handling
   💡 **Recommendations:**
   • 🛡️ Implement defense in depth for update operations

✅ **Lordgenome Spiral King** - 180ms
   💡 **Recommendations:**
   • ⚔️ The architecture shows promise but beware the spiral of complexity
```

## Testing

Run the Claude validation tests:

```bash
cargo test claude_validation --lib
```

## Future Enhancements

1. **Parallel Execution** - Run all three agents concurrently to reduce validation time
2. **Custom Agent Selection** - Allow configuring which agents to run based on change type
3. **Agent Response Caching** - Cache results for unchanged files
4. **Progressive Enhancement** - Start with quick checks, escalate to deeper analysis as needed
5. **Integration with CI/CD** - Expose validation results to build pipelines
