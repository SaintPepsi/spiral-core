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
- **Code Examples**: [CODING_EXAMPLES.md](docs/CODING_EXAMPLES.md)

## Project Overview

**Spiral Core** is a Rust-based AI agent orchestration system built by Anti Spiral Interactive. The system creates specialized AI agents that collaborate through Claude Code integration to build tools and manage complex workflows. The architecture emphasizes simplicity, compile-time safety, and efficient resource management.

**Key Simplification**: Agents serve as intelligent orchestrators of Claude Code capabilities rather than managing local LLM inference.

## 🚨 CRITICAL: Architecture Awareness Before Implementation

**ALWAYS understand the actual system architecture before implementing any functionality.**

**Before writing ANY code or scripts:**

1. **Verify Process Architecture**: Understand what runs as separate processes vs integrated components
2. **Check Existing Integration**: Search for how components are actually connected
3. **Avoid Pattern Assumptions**: Don't assume common patterns apply without verification

**Key Principle**: Pattern matching from other projects is dangerous. This codebase's specific architecture always overrides common patterns.

For verification examples, see [CODING_EXAMPLES.md](docs/CODING_EXAMPLES.md#architecture-verification-example).

## 🚨 CRITICAL: Time Estimates vs Risk/Complexity Metrics

**NEVER provide time estimates**. Use Fibonacci scale (?, 1, 2, 3, 5, 8, 13, ∞) for Risk Level and Complexity Rating instead.

**📊 See [Fibonacci Scale Documentation](docs/FIBONACCI_SCALE.md)** for details.

## Architecture

See [Architecture Guide](docs/ARCHITECTURE.md). Key: Rust backend, Discord bot, specialized agents.

## 🚨 MANDATORY Development Practices

**These practices are NON-NEGOTIABLE for all code changes:**

### 1. 🏗️ Aggressive Proximity Audit Documentation

- **EVERY significant decision must be documented WHERE IT HAPPENS**
- Add audit markers directly in code: 🏗️ ARCHITECTURE, 🛡️ SECURITY, ⚡ PERFORMANCE
- Document alternatives considered and trade-offs made
- If someone would ask "why?", the answer must be in the code

### 2. 🔧 Const Usage Requirements

- **NO hardcoded strings that appear 3+ times**
- Command patterns MUST use const definitions
- API routes MUST use const definitions
- Run const-usage-checker after EVERY change

### 3. 📋 Pre-Completion Checklist

- Run ALL validation steps (cargo test, check, fmt, clippy)
- Verify proximity audit comments are present
- Verify const usage for repeated strings
- No placeholder/fake implementations

**Failure to follow these practices = Task NOT complete**

## Post-Commit Analysis System

**🛡️ CRITICAL**: Analysis Agents are READ-ONLY - they NEVER modify code.

Automated agents in `.claude/analysis-agents/` run after commits to generate reports. Use `make analyze` manually or `SKIP_POST_COMMIT_ANALYSIS=1` to skip.

## Development Commands

See [Coding Standards](docs/CODING_STANDARDS.md#standard-development-commands).

**Script Creation**: NEVER use complex inline bash commands. Create reusable scripts in `scripts/` instead.

For examples, see [CODING_EXAMPLES.md](docs/CODING_EXAMPLES.md#script-creation-examples).

## Rust String Formatting

**ALWAYS use inline variables in format strings** (modern Rust idiom, prevents Clippy warnings).

See [CODING_EXAMPLES.md](docs/CODING_EXAMPLES.md#rust-string-formatting-examples) for examples.

## Self-Update System

See [Self-Update Guide](docs/SELF_UPDATE_GUIDE.md). Updates only via Discord mentions by authorized users.

## Agent System

Multi-agent architecture with Developer, Project Manager, QA, and other specialized agents. Features Claude Code orchestration, self-update capability, GitHub integration, and Discord-based human interaction.

For details, see agent documentation in `docs/agents/`.

## Implementation Priority

1. **Critical**: Claude Code Client, Developer Agent, Discord Bot
2. **High**: Project Manager Agent, GitHub Integration
3. **Medium**: Additional agents, conversation management
4. **Lower**: Advanced coordination, optimization

## Two-Phase Validation Pipeline

**Phase 1**: Quality Assurance (code review, testing, security, integration)
**Phase 2**: Rust Compliance (`cargo check`, `test`, `fmt`, `clippy`, `doc`)

See [Self-Update Pipeline](docs/SELF_UPDATE_PIPELINE_IMPROVEMENT.md) for details.

## Coding Standards

**Core Principles**: SOLID, DRY, YAGNI, Deliberate Decoupling, SID Naming, Early Return Pattern, Clutter Prevention, No Bullshit Code, No Deadline Compromise, Consensus-Driven Improvement, Aggressive Proximity Audit Documentation.

For detailed standards, see [CODING_STANDARDS.md](docs/CODING_STANDARDS.md).

### YAGNI Standard

**Philosophy**: Only implement what's actually needed now. Delete unused code. Avoid premature abstraction.

For examples, see [CODING_EXAMPLES.md](docs/CODING_EXAMPLES.md#yagni-examples).

### Deliberate Decoupling Standard

Code naturally couples. Fight with inline logic, behavior passing, explicit dependencies. See [Decoupling Patterns](docs/DECOUPLING_PATTERNS.md).

### Early Return Pattern

Use negative conditions with early returns for validation/error handling.

### Clutter Prevention

Prevent complexity accumulation through modular design, logical grouping, consistent patterns.

### Problem Space Boundaries

Check existing dependencies before suggesting solutions. Never add new libraries without approval. See [CODING_EXAMPLES.md](docs/CODING_EXAMPLES.md#problem-space-boundary-examples).

### File-Struct Naming

Main struct must match filename: `structured_logger.rs` → `StructuredLogger`

### No Half-Broken State

Every change must result in a fully functional system. Complete refactors, integrate new modules, ensure compilation/tests pass. See [CODING_EXAMPLES.md](docs/CODING_EXAMPLES.md#no-half-broken-state-examples).

### No Bullshit Code

Never fake functionality or placeholder values. If not implemented, don't pretend. See [CODING_EXAMPLES.md](docs/CODING_EXAMPLES.md#no-bullshit-code-examples).

### No Deadline Compromise

Priority: Quality > Urgency > Business. Never compromise for artificial time constraints.

### Consensus-Driven Improvement

Organic evolution through incremental improvements. No proposals or voting - continuous collaborative refinement.

### Proximity Audit Documentation

Document decisions WHERE they happen using markers: 🏗️ ARCHITECTURE, 🛡️ SECURITY, ⚡ PERFORMANCE, etc.

See [Audit Documentation Standard](docs/AUDIT_DOCUMENTATION_STANDARD.md) and [CODING_EXAMPLES.md](docs/CODING_EXAMPLES.md#proximity-audit-documentation-example).

### God Object Prevention

Avoid structs with 8+ fields or 500+ line files. Use service decomposition and single responsibility. See [CODING_EXAMPLES.md](docs/CODING_EXAMPLES.md#god-object-prevention-example).

## Modular Documentation Architecture

This CLAUDE.md file serves as the orchestrator for specialized documentation modules. For detailed implementation guidance, refer to the modular documentation:

### Code Quality Resources

- **[Code Patterns](docs/PATTERNS.md)** - Reusable patterns for DRY code and consistent implementation
- **[Claude Improver Agent](.claude/utility-agents/claude-improver.md)** - Automated code quality analysis and refactoring
- **[DRY Analyzer Agent](.claude/utility-agents/dry-analyzer.md)** - Detection and elimination of code duplication

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

- **[Developer Agent](docs/agents/DEVELOPER.md)** - Code generation, language detection, and Claude Code integration
- **[Project Manager Agent](docs/agents/PROJECT_MANAGER.md)** - Strategic analysis and coordination patterns

### Integration Modules

- **[Discord Integration](docs/integrations/DISCORD.md)** - Conversational agent mentions and Discord bot patterns
- **[GitHub Integration](docs/integrations/GITHUB.md)** - Automated repository management and PR creation
- **[Claude Code Integration](docs/integrations/CLAUDE_CODE.md)** - Primary intelligence engine patterns

### Implementation Modules

- **[Phase 1 Implementation](docs/implementation/PHASE1.md)** - Foundation setup and core systems

## 📚 Common Implementation Patterns

For Check-Fix-Retry, Error Handling, and other patterns, see [CODING_EXAMPLES.md](docs/CODING_EXAMPLES.md#check-fix-retry-pattern).

## 🚨 Task Completion Requirements

### Quality Checklist

**Before marking tasks complete:**

1. Changes match request exactly
2. No ambiguous variable names
3. SOLID principles followed
4. 🏗️ Proximity audit comments present
5. 🔧 Const usage for repeated strings (3+)
6. No bullshit code (fake functions/status)

### Technical Validation (MANDATORY)

```bash
cargo build --lib      # Must build
cargo test            # Must pass
cargo check --all-targets
cargo fmt -- --check
cargo clippy --all-targets
```

## Discord Commands

When adding commands, update: AVAILABLE_COMMANDS array, CommandRouter struct, new() method, route_command() match.

See [CLAUDE_COMPLETION_CHECKLIST.md](docs/CLAUDE_COMPLETION_CHECKLIST.md).
