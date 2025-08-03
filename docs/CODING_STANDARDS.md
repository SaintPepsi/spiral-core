# Coding Standards and Development Practices

Development patterns that work well for this project  
**Updated**: 2024-08-01

## Quick Reference

Write code that compiles fast, tests easily, and reads clearly.

## Core Principles

### SOLID Principles

#### Single Responsibility Principle (SRP)

Each component has one clear responsibility and one reason to change.

```rust
// ✅ Good - Single responsibility
pub struct DeveloperAgent {
    llm_client: Box<dyn LLMClient>,
    prompts_remaining: u32,
}

impl DeveloperAgent {
    pub async fn generate_code(&mut self, requirements: &str) -> Result<CodeResult, AgentError> {
        // Only handles code generation
    }
}

// ❌ Bad - Mixed responsibilities
pub struct BadAgentManager {
    pub async fn generate_code(&mut self, requirements: &str) -> Result<CodeResult, AgentError> {
        // Handles code generation
    }

    pub async fn send_discord_message(&self, msg: &str) -> Result<(), DiscordError> {
        // Also handles Discord communication
    }
}
```

#### Open-Closed Principle (OCP)

Open for extension, closed for modification.

```rust
// ✅ Good - Extensible via trait implementation
pub trait Agent: Send + Sync {
    async fn execute(&mut self, task: Task) -> Result<TaskResult, AgentError>;
    async fn can_handle(&self, task: &Task) -> bool;
}

// New agents extend without modifying existing code
pub struct QAAgent { /* ... */ }
impl Agent for QAAgent { /* ... */ }
```

#### Liskov Substitution Principle (LSP)

Subtypes must be substitutable for their base types.

```rust
// ✅ Good - All agents respect the same contract
fn execute_with_any_agent(agent: &mut dyn Agent, task: Task) -> Result<TaskResult, AgentError> {
    agent.execute(task) // Works with any Agent implementation
}
```

#### Interface Segregation Principle (ISP)

Depend on narrow, focused interfaces.

```rust
// ✅ Good - Focused traits
pub trait CodeGenerator {
    async fn generate_code(&mut self, spec: &str) -> Result<Code, Error>;
}

pub trait CodeReviewer {
    async fn review_code(&self, code: &Code) -> Result<Review, Error>;
}

// ❌ Bad - Fat interface
pub trait BadAgent {
    async fn generate_code(&mut self, spec: &str) -> Result<Code, Error>;
    async fn review_code(&self, code: &Code) -> Result<Review, Error>;
    async fn deploy_code(&self, code: &Code) -> Result<(), Error>;
    async fn monitor_code(&self, code: &Code) -> Result<Metrics, Error>;
}
```

#### Dependency Inversion Principle (DIP)

Depend on abstractions, not concretions.

```rust
// ✅ Good - Depends on trait abstraction
pub struct AgentOrchestrator {
    agents: Vec<Box<dyn Agent>>,
    llm_client: Box<dyn LLMClient>,
}

// ❌ Bad - Depends on concrete types
pub struct BadOrchestrator {
    developer_agent: DeveloperAgent,
    claude_client: ClaudeClient, // Concrete implementation
}
```

### DRY Principle (Don't Repeat Yourself)

Single source of truth for all knowledge.

```rust
// ✅ Good - Centralized configuration
pub const MAX_RETRIES: u32 = 3;
pub const TIMEOUT_SECONDS: u64 = 30;

pub struct RetryConfig {
    pub max_retries: u32,
    pub timeout: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: MAX_RETRIES,
            timeout: Duration::from_secs(TIMEOUT_SECONDS),
        }
    }
}

// ❌ Bad - Duplicated values
impl DeveloperAgent {
    async fn execute(&mut self) -> Result<()> {
        let max_retries = 3; // Duplicated
        let timeout = Duration::from_secs(30); // Duplicated
        // ...
    }
}
```

### SID Naming Convention (Short, Intuitive, Descriptive)

```rust
// ✅ Good SID naming
pub struct TaskQueue { }           // Short, clear purpose
pub fn validate_input() { }        // Intuitive action
pub const MAX_QUEUE_SIZE: usize = 1000; // Descriptive constant

// ❌ Bad naming
pub struct TQ { }                  // Too short, unclear
pub fn do_validation_on_input_data() { } // Too verbose
pub const SIZE: usize = 1000;      // Not descriptive
```

## Code Style Requirements

### Flow Comments (Required)

Complex methods must have flow comments at the top explaining the action sequence.

```rust
/// Handle Discord message processing with security validation
async fn handle_message(&self, msg: &Message, ctx: &Context) {
    // FLOW: Check relevance → Authorize → Validate → Process → Respond
    // 1. Check if message needs processing (mentions/commands)
    // 2. Verify user authorization immediately
    // 3. Validate message content for security
    // 4. Route to appropriate handler
    // 5. Send response with error handling

    if !self.needs_processing(&msg.content) {
        return;
    }

    if !self.is_authorized_user(msg.author.id.get()) {
        self.send_denial_quote(&msg).await;
        return;
    }

    // Rest of implementation...
}
```

**When to add flow comments:**

- Methods with >30 lines or >3 logical steps
- Security-critical code paths
- Complex business logic
- Methods with multiple early returns
- Integration points between modules

### Early Return Pattern (Required)

All validation and error handling must use early returns with negative conditions.

```rust
// ✅ REQUIRED: Early return pattern
fn process_task(task: &Task) -> Result<()> {
    // Validate inputs first (negative conditions)
    if task.description.is_empty() {
        return Err(Error::InvalidInput("Task description cannot be empty"));
    }

    if task.priority > Priority::MAX {
        return Err(Error::InvalidInput("Priority exceeds maximum"));
    }

    if !self.can_handle_task(task) {
        return Err(Error::Unsupported("Agent cannot handle this task type"));
    }

    // Happy path - unindented and clear
    let result = self.execute_task(task)?;
    self.record_result(result)?;
    Ok(())
}

// ❌ AVOID: Nested conditionals
fn process_task(task: &Task) -> Result<()> {
    if !task.description.is_empty() {
        if task.priority <= Priority::MAX {
            if self.can_handle_task(task) {
                let result = self.execute_task(task)?;
                self.record_result(result)?;
                Ok(())
            } else {
                Err(Error::Unsupported("Agent cannot handle this task type"))
            }
        } else {
            Err(Error::InvalidInput("Priority exceeds maximum"))
        }
    } else {
        Err(Error::InvalidInput("Task description cannot be empty"))
    }
}
```

### Rust-Specific Patterns

#### Error Handling

```rust
// ✅ Use Result types with descriptive errors
pub enum AgentError {
    RateLimitExceeded { retry_after: Duration },
    InvalidTask { reason: String },
    ExecutionFailed { task_id: String, cause: String },
}

// ✅ Use ? operator for propagation
async fn execute_task(&mut self, task: Task) -> Result<TaskResult, AgentError> {
    let validated = self.validate_task(&task)?;
    let prepared = self.prepare_execution(validated)?;
    self.run_execution(prepared).await
}
```

#### Type Safety

```rust
// ✅ Use NewType pattern for domain types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentId(String);

// Prevents mixing up IDs
fn get_session(session_id: SessionId) -> Option<Session> { }
```

## Development Practices

### Environment Setup

1. **Clone repository**
2. **Install dependencies**: `npm install` (no -g flag)
3. **Use npx**: All Node commands run through `npx`
4. **Cargo commands**: Use standard Rust toolchain

### Package Management

#### ✅ Approved Practices

```bash
# Local packages only
npm install --save-dev <package>
npx <command>

# Rust development
cargo build
cargo test
cargo clippy
cargo fmt
```

#### ❌ Prohibited Commands

Never run:

- Global npm installs (`npm install -g`)
- Destructive system commands (`rm -rf /`)
- System-wide permission changes
- Direct package manager modifications

### Standard Development Commands

```bash
# Rust commands
cargo build              # Build the project
cargo test              # Run all tests
cargo test -- --nocapture    # Tests with output
cargo clippy            # Linting
cargo fmt               # Format code

# Documentation
cargo doc --open        # Generate and view docs

# Node/npm commands
npm run lint:md         # Check markdown
npm run format:md       # Format markdown
```

## Testing Standards

### Test Organization

- Unit tests in `#[cfg(test)] mod tests` blocks
- Integration tests in `tests/` directory
- Doctests for usage examples

### Test Naming

```rust
#[test]
fn test_validate_session_extends_expiry() { }  // Clear behavior

#[test]
fn test_queue_rejects_when_full() { }         // Negative case
```

### Test Coverage

- Public APIs must have tests
- Error paths must be tested
- Edge cases documented in tests

## Documentation Requirements

### Inline Documentation

````rust
/// Validates and extends an active session.
///
/// # Arguments
/// * `session_id` - The ID of the session to validate
///
/// # Returns
/// * `Ok(Session)` - The validated session with extended expiry
/// * `Err(Error::NotFound)` - If session doesn't exist
/// * `Err(Error::Expired)` - If session has expired
///
/// # Example
/// ```
/// let session = manager.validate_session(&id).await?;
/// assert!(session.expires_at > Utc::now());
/// ```
pub async fn validate_session(&self, session_id: &SessionId) -> Result<Session> {
````

### Module Documentation

````rust
//! # Session Management
//!
//! This module provides secure session management with:
//! - Automatic expiry handling
//! - Concurrent session limits
//! - Activity tracking
//!
//! ## Usage
//! ```
//! let manager = SessionManager::new(config);
//! let session = manager.create_session(user_id).await?;
//! ```
````

## Code Review Checklist

Before merging, verify these basics work as expected:

- [ ] Follows SOLID principles
- [ ] No code duplication (DRY)
- [ ] SID naming convention used
- [ ] Early return pattern for validation
- [ ] Flow comments for complex methods
- [ ] Proper error handling (no unwrap in production)
- [ ] Tests included for new functionality
- [ ] Public APIs documented

## Related Documentation

- [Colocation Patterns](COLOCATION_PATTERNS.md) - File organization
- [Task Checklist](TASK_CHECKLIST.md) - Pre-task verification
- [Security Policy](SECURITY_POLICY.md) - Security requirements
- [Self-Update Guide](SELF_UPDATE_GUIDE.md) - Update procedures

## Migration Notes

This document consolidates the former `CODING_STANDARDS.md` and `DEVELOPMENT_PRACTICES.md` files:

- Merged architectural principles and development workflow
- Removed duplication (e.g., early return pattern)
- Created single source of truth for all development standards

All teams should reference this document for coding standards and development practices.
