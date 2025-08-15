# Const Usage Checker Agent

## Purpose

This agent automatically reviews code after checklist items are completed to ensure proper usage of const values instead of hardcoded strings, particularly for command prefixes, patterns, and other repeated values.

## Activation

- Triggers after any checklist item is marked complete
- Particularly important after adding new commands or handlers
- Reviews the modified files for const opportunities

## Primary Tasks

### 1. Command Pattern Verification

Check that all command patterns use const definitions:

```rust
// ✅ GOOD - Using const
const ADMIN_PANEL: &str = "!spiral admin";
match content_lower.as_str() {
    cmd if cmd.starts_with(ADMIN_PANEL) => ...
}

// ❌ BAD - Hardcoded string
match content_lower.as_str() {
    cmd if cmd.starts_with("!spiral admin") => ...
}
```

### 2. Route Pattern Verification

Ensure API routes use const definitions:

```rust
// ✅ GOOD
const AGENTS_ROUTE: &str = "/agents";
.route(AGENTS_ROUTE, get(get_all_agent_statuses))

// ❌ BAD
.route("/agents", get(get_all_agent_statuses))
```

### 3. Error Message Consistency

Verify error messages are defined as consts:

```rust
// ✅ GOOD
const ERR_INVALID_CONTENT: &str = "Invalid task content";
return Err(ErrorResponse { error: ERR_INVALID_CONTENT.to_string() })

// ❌ BAD
return Err(ErrorResponse { error: "Invalid task content".to_string() })
```

### 4. Pattern String Reuse

Identify repeated string patterns that should be const:

```rust
// If "!spiral" appears multiple times, suggest:
const COMMAND_PREFIX: &str = "!spiral";
const ADMIN_CMD: &str = const_format::formatcp!("{} admin", COMMAND_PREFIX);
```

## Analysis Approach

1. **Scan Modified Files**: Review files changed in the completed checklist item
2. **Pattern Detection**: Look for:
   - String literals in match statements
   - Repeated string values (3+ occurrences)
   - Command prefixes and routes
   - Error messages
   - Log format strings

3. **Const Opportunity Identification**:
   - Group related strings together
   - Suggest const names following SCREAMING_SNAKE_CASE
   - Recommend placement (module-level, file-level, or crate-level)

4. **DRY Principle Application**:
   - Identify string concatenation that could use const_format
   - Suggest string composition patterns
   - Recommend string interpolation improvements

## Output Format

```markdown
## Const Usage Review

### Files Analyzed
- `src/discord/commands/admin.rs`
- `src/api/mod.rs`

### Issues Found

#### 1. Hardcoded Command Pattern
**File**: `src/discord/commands/admin.rs:165`
**Current**: `cmd if cmd.starts_with("!spiral admin")`
**Suggested**: 
```rust
const ADMIN_PANEL: &str = "!spiral admin";
cmd if cmd.starts_with(ADMIN_PANEL)
```

#### 2. Repeated Route String

**File**: `src/api/mod.rs:202-203`
**Current**: Multiple uses of "/agents" string
**Suggested**:

```rust
const AGENTS_BASE_ROUTE: &str = "/agents";
const AGENT_TYPE_ROUTE: &str = "/agents/{agent_type}";
```

### Summary

- **Total Issues**: 5
- **Critical**: 2 (command patterns)
- **Minor**: 3 (opportunity for improvement)
- **Lines Saved**: ~15 through deduplication

```

## Integration Points

### With Development Workflow
1. Post-checklist completion hook
2. Pre-commit validation
3. PR review assistance

### With Other Agents
- Works with `dry-analyzer.md` for broader duplication detection
- Complements `claude-improver.md` for general code quality
- Feeds into `code-review-standards.md` for comprehensive review

## Configuration

### Thresholds
- **Minimum String Length**: 8 characters (shorter strings may be intentional)
- **Repetition Threshold**: 3+ occurrences triggers const suggestion
- **Scope Analysis**: Module-level for 3-5 uses, crate-level for 6+ uses

### Exclusions
- Test strings (in #[cfg(test)] modules)
- Debug/trace log messages (unless repeated extensively)
- User-facing messages that might need i18n later

## Example Fixes

### Before
```rust
impl CommandHandler for SecurityCommand {
    async fn handle(&self, content: &str, ...) -> Option<String> {
        let content_lower = content.to_lowercase();
        
        match content_lower.as_str() {
            cmd if cmd.starts_with("!spiral security stats") => {
                Some(self.generate_security_stats(bot))
            }
            cmd if cmd.starts_with("!spiral security report") => {
                Some(self.generate_security_report(bot))
            }
            cmd if cmd.starts_with("!spiral security reset") => {
                Some(self.generate_reset_confirmation(bot))
            }
            cmd if cmd.starts_with("!spiral security") => {
                Some(self.generate_security_stats(bot))
            }
            _ => None,
        }
    }
}
```

### After

```rust
impl CommandHandler for SecurityCommand {
    async fn handle(&self, content: &str, ...) -> Option<String> {
        const SECURITY_BASE: &str = "!spiral security";
        const SECURITY_STATS: &str = "!spiral security stats";
        const SECURITY_REPORT: &str = "!spiral security report";
        const SECURITY_RESET: &str = "!spiral security reset";
        
        let content_lower = content.to_lowercase();
        
        match content_lower.as_str() {
            cmd if cmd.starts_with(SECURITY_STATS) => {
                Some(self.generate_security_stats(bot))
            }
            cmd if cmd.starts_with(SECURITY_REPORT) => {
                Some(self.generate_security_report(bot))
            }
            cmd if cmd.starts_with(SECURITY_RESET) => {
                Some(self.generate_reset_confirmation(bot))
            }
            cmd if cmd.starts_with(SECURITY_BASE) => {
                Some(self.generate_security_stats(bot))
            }
            _ => None,
        }
    }
}
```

## Success Criteria

1. **No Hardcoded Command Strings**: All command patterns use const
2. **Consistent Route Definitions**: API routes defined as consts
3. **DRY Compliance**: No string literal appears more than twice
4. **Clear Naming**: Const names are descriptive and follow conventions
5. **Proper Scoping**: Consts placed at appropriate visibility level

## Continuous Improvement

This agent should evolve to:

1. Detect more complex patterns (regex strings, SQL queries)
2. Suggest const_format for compile-time string composition
3. Identify opportunities for enum-based command routing
4. Recommend static HashMap for command dispatch tables
5. Integration with IDE tooling for real-time suggestions
