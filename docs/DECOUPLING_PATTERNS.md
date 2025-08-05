# Decoupling Patterns

## Philosophy: Decoupling by Default

**Core Principle**: Code naturally tends toward coupling. Deliberate effort is required to maintain decoupling.

**The Coupling Trap**: Most developers instinctively create coupling because it feels "organized" - factory methods, shared handlers, inheritance hierarchies. This creates hidden dependencies and reduces flexibility.

**The Decoupling Mindset**: Always ask "what does this code need to know?" - the answer should be "as little as possible."

## Pattern 1: Inline Logic Over Hidden Abstractions

### ❌ Anti-Pattern: Hidden Factory Methods

```rust
// BAD: Logic hidden in factory method
async fn run_compilation_check(&mut self) -> Result<ComplianceCheck> {
    let check = create_check("compilation");
    let handler = self.create_fix_handler("compilation"); // What does this do?
    ValidationRunner::run_with_retry(&check, handler.as_ref(), 3).await
}

fn create_fix_handler(&self, check_name: &str) -> Box<dyn FixHandler> {
    // Complex logic hidden here - requires jumping to understand
    match check_name {
        "compilation" => // ... 50 lines of logic
        "tests" => // ... more hidden complexity
    }
}
```

### ✅ Pattern: Inline Visibility

```rust
// GOOD: Logic is visible at the call site
async fn run_compilation_check(&mut self) -> Result<ComplianceCheck> {
    let check = CargoCheck::new(
        "compilation",
        "cargo",
        vec!["check".to_string(), "--all-targets".to_string()],
    );
    
    // Fix logic is RIGHT HERE - no jumping required
    let claude_client = self.claude_client.clone();
    let fix_handler = move |error_msg: &str, _retries: u8| {
        let client = claude_client.clone();
        let error = error_msg.to_string();
        async move {
            if let Some(client) = client {
                // I can see EXACTLY what agent is used
                let fix_handler = ClaudeFixHandler::new(client, CLAUDE_AGENT_COMPILATION_FIX);
                fix_handler.attempt_fix("compilation", &error).await
            } else {
                Ok(false)
            }
        }
    };
    
    ValidationRunner::run_with_retry(&check, fix_handler, 3).await
}
```

**Benefits**:
- **Immediate understanding** - no need to jump between files
- **Easy customization** - modify behavior without affecting other checks
- **Clear dependencies** - see exactly what's used
- **No hidden coupling** - each check is independent

## Pattern 2: Generic Functions Accept Behavior, Not Data

### ❌ Anti-Pattern: Passing Implementation Details

```rust
// BAD: Generic function knows about specific implementations
pub async fn run_check_with_retry(
    handler: &dyn FixHandler,
    check_name: &str,
    command: &str,
    args: &[&str],
    agent_path: Option<&str>,    // Why does generic code know about agents?
    auto_fix: Option<(&str, &[&str])>, // Why does it know about auto-fix?
) -> Result<ComplianceCheck>
```

### ✅ Pattern: Pass Behavior as Functions

```rust
// GOOD: Generic function only knows about behavior contracts
pub async fn run_with_retry<C, F, Fut>(
    check: &C,           // Something that can check
    fix_handler: F,      // Something that can fix
    max_attempts: u8,
) -> Result<ComplianceCheck>
where
    C: CheckRunner,      // Contract: can run a check
    F: Fn(&str, u8) -> Fut,  // Contract: can attempt a fix
    Fut: Future<Output = Result<bool>>,
```

**Benefits**:
- **True genericity** - works with ANY implementation
- **No leaky abstractions** - implementation details stay local
- **Testable** - easy to pass mock functions
- **Extensible** - add new fix strategies without changing core

## Pattern 3: Composition Over Configuration

### ❌ Anti-Pattern: Configuration-Driven Behavior

```rust
// BAD: Behavior determined by configuration
struct FixHandler {
    use_claude: bool,
    use_auto_fix: bool,
    claude_agent_path: Option<String>,
    auto_fix_command: Option<String>,
    // Endless configuration options...
}

impl FixHandler {
    fn attempt_fix(&self) {
        if self.use_claude && self.claude_agent_path.is_some() {
            // Complex branching logic
        } else if self.use_auto_fix {
            // More branching
        }
    }
}
```

### ✅ Pattern: Compose Behavior with Closures

```rust
// GOOD: Behavior composed at call site
let fix_handler = move |error_msg: &str, retries: u8| {
    async move {
        // Try auto-fix first for formatting
        if retries == 0 && check_type == "formatting" {
            if try_cargo_fmt().await? {
                return Ok(true);
            }
        }
        
        // Then try Claude
        if let Some(client) = claude_client {
            spawn_claude_agent(client, SPECIFIC_AGENT).await
        } else {
            Ok(false)
        }
    }
};
```

**Benefits**:
- **Explicit flow** - see the exact sequence of attempts
- **No configuration soup** - behavior is code, not config
- **Flexible composition** - mix and match strategies per use case
- **Type safe** - compiler validates behavior, not runtime config

## Pattern 4: Data and Behavior Separation

### ❌ Anti-Pattern: Objects That Do Too Much

```rust
// BAD: Object holds data AND knows how to process it
struct ValidationPipeline {
    context: PipelineContext,
    claude_client: Option<ClaudeCodeClient>,
    
    fn run_check(&mut self) { /* uses internal state */ }
    fn fix_issue(&mut self) { /* modifies internal state */ }
    fn create_report(&self) { /* reads internal state */ }
}
```

### ✅ Pattern: Separate Data from Functions

```rust
// GOOD: Data is just data
struct PipelineContext {
    iterations: u32,
    errors: Vec<String>,
}

// GOOD: Functions operate on data
async fn run_check(check: &impl CheckRunner) -> CheckOutput { }
async fn attempt_fix<F>(handler: F, error: &str) -> bool { }
fn create_report(context: &PipelineContext) -> Report { }
```

**Benefits**:
- **Reusable functions** - use with different data sources
- **Testable in isolation** - no complex object setup
- **Clear dependencies** - functions declare what they need
- **Parallel-friendly** - no shared mutable state

## Pattern 5: Explicit Over Implicit

### ❌ Anti-Pattern: Implicit Behavior

```rust
// BAD: What does this actually do?
pipeline.run().await;  // Magic happens somewhere
```

### ✅ Pattern: Explicit Steps

```rust
// GOOD: Clear sequence of operations
let check = CargoCheck::new("compilation", "cargo", vec!["check"]);
let fix = create_claude_fix_handler(CLAUDE_AGENT_COMPILATION_FIX);
let result = ValidationRunner::run_with_retry(&check, fix, 3).await?;
```

## Implementation Guidelines

### When Creating New Functions

1. **Start with behavior, not data**
   - Ask: "What action does this perform?"
   - Not: "What data does this manage?"

2. **Accept functions, not configurations**
   - Pass closures for variable behavior
   - Use traits only for stable contracts

3. **Make dependencies explicit**
   - Parameters should show all requirements
   - No hidden dependencies on object state

### When Refactoring Existing Code

1. **Identify coupling points**
   - Factory methods that hide logic
   - Configuration objects with many options
   - Methods that "know too much"

2. **Extract behavior to call site**
   - Move logic from factories to usage points
   - Replace configuration with composition

3. **Simplify interfaces**
   - Reduce parameter count by passing behaviors
   - Remove implementation details from signatures

### Code Review Checklist

- [ ] Can I understand this function without jumping to other code?
- [ ] Are implementation details visible at the call site?
- [ ] Could I test this with simple mock functions?
- [ ] Does the generic code know about specific implementations?
- [ ] Is behavior determined by code or configuration?

## Real-World Example: Validation Pipeline Refactoring

### Before: Coupled Design
- Generic `run_check_with_retry` knew about agent paths and auto-fix commands
- Fix logic hidden in `create_fix_handler` factory method  
- 114 lines of duplicate code between Phase2Executor and ValidationPipeline

### After: Decoupled Design
- Generic `run_with_retry` only knows about behavior contracts
- Fix logic inline at each call site with full visibility
- Zero duplicate code, complete reusability

### The Key Insight

**Don't hide complexity - organize it at the right level.**

Instead of hiding fix logic in a factory method (false simplicity), put it where it's used (true clarity). The code is "longer" at each call site, but it's **honest** about what it does.

## References

- [SOLID Principles](CODING_STANDARDS.md#solid-principles) - Especially Dependency Inversion
- [DRY Principle](CODING_STANDARDS.md#dry-principle) - Reuse without coupling
- [Audit Documentation](AUDIT_DOCUMENTATION_STANDARD.md) - Document decisions at usage sites