# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🚨 IMPORTANT: Documentation-First Development

**BEFORE starting ANY task, ALWAYS:**

1. **Read this CLAUDE.md file completely** to understand project context
2. **Check relevant modular documentation** listed in the "Modular Documentation Architecture" section
3. **Follow established patterns** from existing code and documented conventions
4. **Verify naming conventions** match the established CLAUDE-* pattern for docs
5. **Apply colocation patterns** from [COLOCATION_PATTERNS.md](docs/COLOCATION_PATTERNS.md)

**Key Documentation to Reference:**

- **Coding Standards**: [CODING_STANDARDS.md](docs/CODING_STANDARDS.md)
- **Colocation Patterns**: [COLOCATION_PATTERNS.md](docs/COLOCATION_PATTERNS.md)
- **Architecture Decisions**: This file and module-specific guides

## Project Overview

**Spiral Core** is a Rust-based AI agent orchestration system built by Anti Spiral Interactive. The system creates specialized AI agents that collaborate through Claude Code integration to build tools and manage complex workflows. The architecture emphasizes simplicity, compile-time safety, and efficient resource management for deployment on 8GB VPS or Apple Silicon hardware.

**Key Simplification**: Agents serve as intelligent orchestrators of Claude Code capabilities rather than managing local LLM inference, dramatically reducing complexity while maintaining sophisticated functionality.

## Architecture

This is a Rust-based system with Claude Code as the primary intelligence engine. The simplified architecture consists of:

- **Rust Backend**: Agent orchestration system that coordinates with Claude Code
- **Claude Code Integration**: Primary AI engine for code generation and analysis
- **Discord Bot Service**: Node.js/TypeScript service for human interaction
- **GitHub Integration**: Automated repository management and PR creation
- **Agent Types**: Specialized orchestrators (Developer, Project Manager, QA, Decision Maker, Creative Innovator, Process Coach)

The system follows a "Spiral" naming convention for all components, using cosmic/space-inspired terms (e.g., Spiral Constellation, Spiral Comet, Spiral Cluster).

## Development Commands

For standard Rust development commands and practices, see [Coding Standards](docs/CODING_STANDARDS.md#standard-development-commands).

### Self-Update System

The system can update itself through Discord mentions. See [Self-Update Guide](docs/SELF_UPDATE_GUIDE.md) for details.

**Important**: Self-updates are only triggered via Discord mentions by authorized users, not through commands.

## Key Files and Structure

- `ARCHITECTURE.md` - Comprehensive technical architecture document detailing the full agent system design
- `SPIRAL_CODE.md` - Naming conventions and branding guidelines for the Spiral ecosystem
- `plans/DISCORD_AI_AGENT_ORCHESTRATOR_ARCHITECTURE.md` - Detailed Discord integration architecture
- `target/` - Rust build artifacts directory (contains rust-analyzer metadata)

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

Following Uncle Iroh's wisdom: "A system that can improve itself is like tea that gets better with each steeping." The self-update system embodies careful, incremental improvement with robust safety mechanisms. See [Iroh's Wisdom](docs/IROH_WISDOM.md) for philosophical guidance.

## Coding Standards and Architecture Principles

The Spiral Core system follows strict architectural principles to ensure maintainability and extensibility. All development must adhere to:

- **SOLID Principles**: Single Responsibility, Open-Closed, Liskov Substitution, Interface Segregation, Dependency Inversion
- **DRY Principle**: Don't Repeat Yourself - single source of truth for all knowledge  
- **SID Naming**: Short, Intuitive, Descriptive naming conventions
- **Early Return Pattern**: Use negative conditions with early returns for all validation and error handling

### Conditional Logic Standard

**Required**: Use early returns with negative conditions for all validation and error handling. This pattern reduces nesting, improves readability, and optimizes the happy path.

For detailed implementation guidance, code examples, and best practices, see [Coding Standards](docs/CODING_STANDARDS.md).

## Modular Documentation Architecture

This CLAUDE.md file serves as the orchestrator for specialized documentation modules. For detailed implementation guidance, refer to the modular documentation:

### Core System Modules

- **[Coding Standards](docs/CODING_STANDARDS.md)** - SOLID, DRY, SID principles, development practices, and Rust patterns
- **[Colocation Patterns](docs/COLOCATION_PATTERNS.md)** - Code organization, test colocation, and modular structure patterns
- **[Task Checklist](docs/TASK_CHECKLIST.md)** - Pre-task documentation review and execution guidelines
- **[Markdown Standards](docs/MARKDOWN_STANDARDS.md)** - Documentation formatting and style guidelines
- **[Development Practices](docs/CODING_STANDARDS.md#development-practices)** - Package management and development workflow
- **[Security Policy](docs/SECURITY_POLICY.md)** - Security hardening measures and vulnerability reporting
- **[Self-Update Guide](docs/SELF_UPDATE_GUIDE.md)** - How to use the self-update system
- **[Iroh's Wisdom](docs/IROH_WISDOM.md)** - Philosophical principles guiding system design

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

## 🚨 CRITICAL: Task Completion Requirements

**NEVER declare a task "complete" or "done" without running these validations:**

1. **Run Tests** - `cargo test` (ALL tests MUST pass)
2. **Check Compilation** - `cargo check --all-targets` (MUST compile)
3. **Check Formatting** - `cargo fmt -- --check` (MUST be formatted)
4. **Run Clippy** - `cargo clippy --all-targets` (NO errors allowed)
5. **Verify Changes** - Manually verify your changes work as intended

See [Claude Completion Checklist](docs/CLAUDE_COMPLETION_CHECKLIST.md) for detailed requirements.

**If ANY validation fails, you are NOT done - fix the issues first!**
