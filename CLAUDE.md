# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🚨 IMPORTANT: Documentation-First Development

**BEFORE starting ANY task, ALWAYS:**

1. **Read this CLAUDE.md file completely** to understand project context
2. **Check relevant modular documentation** listed in the "Modular Documentation Architecture" section
3. **Follow established patterns** from existing code and documented conventions
4. **Verify naming conventions** match the established CLAUDE-\* pattern for docs
5. **Apply colocation patterns** from [COLOCATION_PATTERNS.md](docs/COLOCATION_PATTERNS.md)

**Key Documentation to Reference:**

- **Coding Standards**: [CODING_STANDARDS.md](docs/CODING_STANDARDS.md)
- **Colocation Patterns**: [COLOCATION_PATTERNS.md](docs/COLOCATION_PATTERNS.md)
- **Architecture Decisions**: This file and module-specific guides

## Project Overview

**Spiral Core** is a Rust-based AI agent orchestration system built by Anti Spiral Interactive. The system creates specialized AI agents that collaborate through Claude Code integration to build tools and manage complex workflows. The architecture emphasizes simplicity, compile-time safety, and efficient resource management for deployment on 8GB VPS or Apple Silicon hardware.

**Key Simplification**: Agents serve as intelligent orchestrators of Claude Code capabilities rather than managing local LLM inference, dramatically reducing complexity while maintaining sophisticated functionality.

## 🚨 CRITICAL: Time Estimates vs Risk/Complexity Metrics

**NEVER provide time estimates** for any task, feature, or implementation. Time is irrelevant to quality.

**NEVER hardcode time estimates** in user-facing messages or documentation. If showing duration, calculate it dynamically from actual elapsed time.

**ALWAYS provide:**

- **Risk Level**: Using Fibonacci scale (?, 1, 2, 3, 5, 8, 13, ∞)
- **Complexity Rating**: Using Fibonacci scale (?, 1, 2, 3, 5, 8, 13, ∞)
- **Specific Risk Factors**: What could go wrong and why

**Why**: Time estimates create artificial pressure and lead to rushed, poor-quality code. Risk and complexity metrics help make informed decisions about implementation approach and testing requirements. Hardcoded time estimates become inaccurate and misleading.

**📊 See [Fibonacci Scale Documentation](docs/FIBONACCI_SCALE.md)** for detailed scale definitions, usage guidelines, and examples.

## Architecture

For complete system architecture, see [Architecture Guide](docs/ARCHITECTURE.md).

Key components:

- **Rust Backend**: Agent orchestration with Claude Code integration
- **Discord Bot Service**: Node.js/TypeScript for human interaction
- **Specialized Agents**: Developer, Project Manager, QA, and more

## Development Commands

For standard Rust development commands and practices, see [Coding Standards](docs/CODING_STANDARDS.md#standard-development-commands).

### Self-Update System

The system can update itself through Discord mentions. See [Self-Update Guide](docs/SELF_UPDATE_GUIDE.md) for details.

**Important**: Self-updates are only triggered via Discord mentions by authorized users, not through commands.

## Key Files and Structure

- `docs/ARCHITECTURE.md` - Comprehensive system architecture
- `docs/SETUP.md` - Complete setup and configuration guide
- `docs/CODING_STANDARDS.md` - Development standards and practices
- `src/agents/docs/` - Agent-specific implementation guides
- `src/integrations/docs/` - Integration patterns and examples

## Agent System Design

The system is designed around a sophisticated multi-agent architecture with:

### Core Agent Types

1. **Software Developer Agent** - Code generation and implementation
2. **Project Manager Agent** - Strategic analysis and coordination
3. **Quality Assurance Agent** - Risk assessment and testing
4. **Decision Maker Agent** - Priority scoring and conflict resolution
5. **Creative Innovator Agent** - Alternative approaches and innovation
6. **Process Coach Agent** - Performance optimization and facilitation

### Key Features

- **Claude Code Orchestration**: Agents specialize in coordinating Claude Code for different tasks
- **Simplified Resource Management**: Track Claude Code API usage rather than complex prompt allocation
- **Self-Update System**: Autonomous improvement capability with safety checks and rollback
- **Tool Building System**: Agents request and coordinate tool creation via Claude Code
- **Message Queue System**: Redis-based async communication between agents
- **GitHub Integration**: Automated repository management, PR creation, and code review
- **Human Integration**: Discord-based approval system for tool requests
- **Compile-time Safety**: Rust's type system prevents coordination bugs and race conditions

## Implementation Priority (Updated)

The simplified architecture focuses on Claude Code orchestration with Discord as the primary interface:

1. **Critical** (Build Immediately): Claude Code Client, Software Developer Agent, **Discord Bot Integration**
2. **High Priority**: Project Manager Agent, GitHub Integration, Agent Communication
3. **Medium Priority**: Additional agent types (QA, Decision Maker), conversation management
4. **Lower Priority**: Advanced coordination features, performance optimization

## Development Notes

- **Active Development**: Core Discord bot and self-update system implemented
- **Simplified Architecture**: Removed local LLM complexity, focusing on Claude Code orchestration
- **Resource Efficient**: ~2.1GB memory usage (vs 8GB+ with local models)
- **M2 Optimized**: Native Apple Silicon compilation without GPU model management
- Heavy emphasis on agent coordination via Claude Code
- Discord serves as the primary human interface for the system
- GitHub integration provides automated repository management

### Self-Update Philosophy

Following the principle: "A system that can improve itself is like tea that gets better with each steeping." The self-update system embodies careful, incremental improvement with robust safety mechanisms.

### Two-Phase Validation Pipeline

The self-update system uses a sophisticated two-phase validation pipeline to ensure code quality:

#### Phase 1: Advanced Quality Assurance (AQA)

- **Code Review & Standards**: Comprehensive review against project standards
- **Comprehensive Testing**: Focus on pressure points and critical scenarios
- **Security Audit**: Identify vulnerabilities and unsafe patterns
- **System Integration**: Verify no regressions or breaking changes

#### Phase 2: Core Rust Compliance Checks (CRCC)

- **Compilation**: `cargo check --all-targets`
- **Test Execution**: `cargo test`
- **Formatting**: `cargo fmt`
- **Linting**: `cargo clippy`
- **Documentation**: `cargo doc`

**Pipeline Flow Logic**: If ANY check in Phase 2 requires a retry, the entire pipeline loops back to Phase 1. Maximum 3 complete iterations before failure analysis.

**Validation Agents**: The pipeline uses 15+ specialized Claude Code agents located in `.claude/validation-agents/` for different validation aspects, including analysis agents that provide detailed reports on success, partial success, or failure scenarios.

See [Self-Update Pipeline](docs/SELF_UPDATE_PIPELINE_IMPROVEMENT.md) for detailed specifications.

## Coding Standards and Architecture Principles

The Spiral Core system follows strict architectural principles to ensure maintainability and extensibility. All development must adhere to:

- **SOLID Principles**: Single Responsibility, Open-Closed, Liskov Substitution, Interface Segregation, Dependency Inversion
- **DRY Principle**: Don't Repeat Yourself - single source of truth for all knowledge
- **Deliberate Decoupling**: Code naturally couples - fight this with inline logic, behavior passing, and explicit dependencies
- **SID Naming**: Short, Intuitive, Descriptive naming conventions
- **Early Return Pattern**: Use negative conditions with early returns for all validation and error handling
- **Clutter Prevention**: Maintain clean, organized code by preventing complexity accumulation through modular design and consistent patterns
- **No Bullshit Code**: Never fake status, metrics, or functionality - if it's not implemented, don't pretend it is
- **No Deadline Compromise**: Deadlines are the anti-spiral abyss - we work by priority: Quality > Urgency > Importance to Business. Never compromise code quality for artificial time constraints
- **Consensus-Driven Continuous Improvement**: Incremental, organic evolution of the codebase, agents, and system through collaborative consensus rather than disruptive changes or competing proposals
- **Aggressive Proximity Audit Documentation**: Critical decisions, security controls, and audit points documented directly in code where they matter

### Deliberate Decoupling Standard

**Philosophy**: Code naturally tends toward coupling - it's the path of least resistance. Decoupling requires deliberate, conscious effort.

**Key Principles**:

- **Inline Logic Over Hidden Abstractions**: Put fix logic where it's used, not in factory methods
- **Pass Behavior, Not Data**: Generic functions should accept closures/functions, not implementation details
- **Explicit Dependencies**: Make all dependencies visible at the call site
- **Composition Over Configuration**: Use closures to compose behavior, not configuration objects

**Example**: Instead of hiding fix logic in `create_fix_handler()`, put it inline where you can see exactly what agent is used and how fixes are attempted.

For complete patterns and examples, see [Decoupling Patterns](docs/DECOUPLING_PATTERNS.md).

### Conditional Logic Standard

**Required**: Use early returns with negative conditions for all validation and error handling. This pattern reduces nesting, improves readability, and optimizes the happy path.

### Clutter Prevention Standard

**Definition**: Clutter prevention is the practice of maintaining clean, organized, and minimal code structure by actively preventing the accumulation of unnecessary complexity, redundant patterns, or scattered responsibilities.

**Core Principles**:

- **Modular Design**: Break large functions into focused, single-purpose modules (e.g., command separation)
- **Logical Grouping**: Related functionality is co-located and properly separated from unrelated concerns
- **Consistent Patterns**: Similar problems solved in similar ways across the codebase
- **Clear Interfaces**: Well-defined boundaries between components with minimal surface area
- **Preventive Refactoring**: Address complexity before it becomes unwieldy

**Examples of Clutter to Avoid**:

- Monolithic functions (300+ lines like the old `handle_special_commands`)
- Mixed responsibilities (UI logic mixed with business logic)
- Scattered similar code (copy-paste programming)
- Unclear naming (variables like `data`, `temp`, `thing`)
- Dead code (unused imports, functions, variables)

**Implementation**: Follow the modular command structure in `src/discord/commands/` as the standard pattern.

### No Bullshit Code Standard

**Definition**: Never implement fake functionality, hardcoded status messages, or placeholder values that pretend to show real data when no actual implementation exists.

**Violations to Eliminate**:

- **Fake Status**: `"SpiralDev: 🟢 Active"` without checking if agent is actually running
- **Hardcoded Metrics**: `"Memory Usage: Efficient"` without measuring actual memory usage
- **Placeholder Values**: `"Response Time: Fast"` without timing real operations
- **Mock Data**: Creating fake Phase1Results or any mock data just to satisfy structure requirements
- **Pretend Features**: UI elements that suggest functionality that doesn't exist
- **Unnecessary Dependencies**: Requiring data that isn't actually needed (e.g., Phase 2 requiring Phase 1)

**Rule**: NEVER use mock data. If you can't implement it properly, refactor to allow independent execution using SOLID principles. Users prefer "Not yet implemented" over fake functionality.

**Examples**:

- Replace "🟢 Active" with "❓ Status check not implemented" when there's no actual status checking
- Replace "Memory: Efficient" with "❓ Monitoring not implemented" when there's no memory monitoring
- Create independent methods like `run_phase2_independent()` instead of mocking Phase 1 data
- Apply Interface Segregation: Don't force clients to provide unnecessary data

### No Deadline Compromise Standard

**Philosophy**: Deadlines are the anti-spiral abyss that leads to technical debt, shortcuts, and system degradation. We reject deadline-driven development in favor of priority-driven excellence.

**Priority Hierarchy**:

1. **Quality** - Code correctness, security, maintainability, and user experience
2. **Urgency** - Time-sensitive user needs and system stability issues
3. **Importance to Business** - Strategic value and long-term impact

**Principles**:

- **Sustainable Pace**: Work efficiently toward goals without artificial time pressure
- **Quality Gates**: Never bypass security, testing, or code review for speed
- **Technical Debt Prevention**: Address complexity and maintenance issues immediately
- **User-Centric Readiness**: Ship when quality serves users, independent of time
- **Vision-Focused Communication**: Share goals, quality status, and blockers - never estimates or timelines

**Anti-Patterns to Reject**:

- "Ship it now, fix it later" mentality
- Skipping tests or security reviews for deadlines
- Technical debt accumulation under time pressure
- Compromising architecture for artificial urgency
- "Good enough" solutions that create future problems

**Game Theory Connection**: This aligns with the "nice" and "forgiving" principles - we cooperate for long-term success rather than compete in zero-sum deadline games that ultimately harm everyone.

### Consensus-Driven Continuous Improvement Standard

**Philosophy**: The system evolves organically through incremental improvements where agents naturally converge on better solutions. No proposals, no voting, no democratic processes - just continuous collaborative refinement.

**Core Principles**:

- **Incremental Evolution**: Small, continuous improvements rather than large disruptive changes
- **Organic Consensus**: Agents naturally align on improvements through ongoing collaboration
- **Collaborative Refinement**: Build on each other's work rather than competing alternatives
- **Continuous Integration**: Improvements flow seamlessly into the system without ceremony
- **Emergent Excellence**: Quality emerges from the process, not from planning or proposals

**How It Works**:

- **Observe**: Agents notice opportunities for improvement during normal work
- **Suggest**: Casual suggestions in context rather than formal proposals
- **Iterate**: Small improvements that others can build upon
- **Converge**: Natural alignment as agents see what works
- **Integrate**: Seamless adoption without formal approval processes

**Anti-Patterns to Avoid**:

- Formal proposal and voting systems
- Large, disruptive "improvement projects"
- Competing solutions that require choosing sides
- Committee-driven decision making
- Waiting for permission to improve

**Game Theory Alignment**: Embodies all four successful principles - nice (collaborative), forgiving (adaptive), retaliatory (maintains quality), and clear (transparent process).

### Aggressive Proximity Audit Documentation Standard

**Definition**: Documentation lives where decisions are made - aggressive (highly visible), proximate (next to code), and auditable (traceable).

**Required Documentation Points**:

```rust
/// 🏗️ ARCHITECTURE DECISION: Two-phase validation pipeline
/// Why: Separates quality checks from compliance for targeted fixes
/// Alternative: Single pass (rejected: mixes concerns)
/// Audit: Verify Phase 2 doesn't depend on Phase 1
/// Trade-off: More complex but more maintainable
pub struct ValidationPipeline { ... }
```

**Documentation Patterns**:

- **🏗️ ARCHITECTURE DECISION**: Design choices and trade-offs
- **🛡️ SECURITY DECISION**: Security controls and threat mitigation
- **⚡ PERFORMANCE DECISION**: Optimizations and benchmarks
- **🔍 AUDIT CHECKPOINT**: Critical review points
- **🔄 DRY PATTERN**: Reusable patterns and abstractions
- **📐 SOLID**: Principle applications
- **🔧 ERROR RECOVERY**: Failure handling strategies

**Rule**: If a reviewer would ask "why?", the answer must already be in a comment.

For complete documentation patterns, examples, and enforcement strategies, see [Audit Documentation Standard](docs/AUDIT_DOCUMENTATION_STANDARD.md).

For detailed implementation guidance, code examples, and best practices, see [Development Guide](docs/DEVELOPMENT.md).

## Modular Documentation Architecture

This CLAUDE.md file serves as the orchestrator for specialized documentation modules. For detailed implementation guidance, refer to the modular documentation:

### Code Quality Resources

- **[Code Patterns](docs/PATTERNS.md)** - Reusable patterns for DRY code and consistent implementation
- **[Claude Improver Agent](.claude/utility-agents/claude-improver.md)** - Automated code quality analysis and refactoring

### Core System Modules

- **[Coding Standards](docs/CODING_STANDARDS.md)** - SOLID, DRY, SID principles, development practices, and Rust patterns
- **[Decoupling Patterns](docs/DECOUPLING_PATTERNS.md)** - Deliberate decoupling strategies to prevent natural coupling tendencies
- **[Audit Documentation Standard](docs/AUDIT_DOCUMENTATION_STANDARD.md)** - Aggressive proximity audit documentation patterns and enforcement
- **[Colocation Patterns](docs/COLOCATION_PATTERNS.md)** - Code organization, test colocation, and modular structure patterns
- **[Task Checklist](docs/TASK_CHECKLIST.md)** - Pre-task documentation review and execution guidelines
- **[Markdown Standards](docs/MARKDOWN_STANDARDS.md)** - Documentation formatting and style guidelines
- **[Development Guide](docs/DEVELOPMENT.md)** - Complete development practices and standards
- **[Security Policy](docs/SECURITY_POLICY.md)** - Security hardening measures and vulnerability reporting
- **[Self-Update Guide](docs/SELF_UPDATE_GUIDE.md)** - Self-update system usage
- **[Architecture](docs/ARCHITECTURE.md)** - Complete system architecture
- **[Setup Guide](docs/SETUP.md)** - Installation and configuration
- **[Engineering Principles](docs/ENGINEERING_PRINCIPLES.md)** - Practical engineering guidelines and quality standards
- **[Dutch Agent Communication](docs/DUTCH_AGENT_COMMUNICATION.md)** - Direct, pragmatic agent interaction patterns based on Dutch cultural communication

### Agent System Modules

- **[Developer Agent](src/agents/docs/AGENTS_DEVELOPER.md)** - Code generation, language detection, and Claude Code integration
- **[Project Manager Agent](src/agents/docs/AGENTS_PM.md)** - Strategic analysis and coordination patterns

### Integration Modules

- **[Discord Integration](src/integrations/docs/INTEGRATIONS_DISCORD.md)** - Conversational agent mentions and Discord bot patterns
- **[GitHub Integration](src/integrations/docs/INTEGRATIONS_GITHUB.md)** - Automated repository management and PR creation
- **[Claude Code Integration](src/integrations/docs/INTEGRATIONS_CLAUDE_CODE.md)** - Primary intelligence engine patterns

### Implementation Modules

- **[Phase 1 Implementation](src/implementation/docs/IMPLEMENTATION_PHASE1.md)** - Foundation setup and core systems

## Implementation Roadmap Summary

### Phase 1: Foundation (Critical Priority)

1. **Discord Bot Integration** - Primary user interface with conversational agent mentions
2. **Developer Agent** - Autonomous code generation with language detection
3. **Claude Code Client** - Primary intelligence engine integration
4. **Minimal HTTP API** - Agent communication endpoints

### Phase 2: Enhanced Coordination (High Priority)

1. **Project Manager Agent** - Strategic analysis and task coordination
2. **GitHub Integration** - Automated repository management
3. **Agent Status Monitoring** - Resource management and performance tracking

### Phase 3: Advanced Features (Medium Priority)

1. **QA Agent** - Code review and validation
2. **Enhanced Discord Commands** - Task queuing and agent assignment
3. **Database Persistence** - Agent state and task history

### Phase 4: Tool Building System (Lower Priority)

1. **Human Approval Workflows** - Tool request management
2. **Custom Tool Creation** - Dynamic capability expansion
3. **Self-Improvement Mechanisms** - Agent learning and adaptation

For detailed implementation steps, database schemas, security frameworks, and code examples, see the respective modular documentation files.

## 📚 Common Implementation Patterns

### Check-Fix-Retry Pattern

When implementing validation checks that may need fixes, use this generic pattern:

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

### Error Handling Pattern

Use early returns with negative conditions:

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

## 🔧 Code Reuse Guidelines

### When to Extract a Helper Method

- **Rule of Three**: If code appears 3+ times with minor variations
- **Complex Logic**: If the logic is complex but the pattern is common
- **Testability**: If extraction improves unit testing
- **Single Responsibility**: If it helps maintain SRP from SOLID

### Generic vs Specific

```rust
// ✅ GOOD - Generic reusable method
async fn run_cargo_command(&self, subcommand: &str, args: &[&str]) -> Result<Output>

// ❌ BAD - Multiple specific methods
async fn run_cargo_test(&self) -> Result<Output>
async fn run_cargo_check(&self) -> Result<Output>
async fn run_cargo_clippy(&self) -> Result<Output>
```

## 🦀 Rust-Specific Patterns

### Result Chaining

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

### Builder Pattern for Complex Configs

```rust
let config = ConfigBuilder::new()
    .with_timeout(Duration::from_secs(30))
    .with_retries(3)
    .with_agent("claude-agent.md")
    .build()?;
```

## 📝 Code Templates

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

## 🚨 CRITICAL: Task Completion Requirements

**NEVER declare a task "complete" or "done" without running these validations:**

1. **Run Tests** - `cargo test` (ALL tests MUST pass)
2. **Check Compilation** - `cargo check --all-targets` (MUST compile)
3. **Check Formatting** - `cargo fmt -- --check` (MUST be formatted)
4. **Run Clippy** - `cargo clippy --all-targets` (NO errors allowed)
5. **Verify Changes** - Manually verify your changes work as intended

## 📚 Discord Command Documentation

**When adding new Discord commands, ALWAYS update:**

1. **AVAILABLE_COMMANDS** array in `src/discord/commands/mod.rs`
   - Add CommandInfo entry with name, prefix, description, category, and auth requirement
2. **CommandRouter struct** in `src/discord/commands/mod.rs`
   - Add the command handler field
3. **CommandRouter::new()** method
   - Initialize the command handler
4. **CommandRouter::route_command()** match statement
   - Add routing case for the command name
5. **Help documentation** if the command should be user-visible

See [Claude Completion Checklist](docs/CLAUDE_COMPLETION_CHECKLIST.md) for detailed requirements.

**If ANY validation fails, you are NOT done - fix the issues first!**
