# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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

This repository currently appears to be in early planning stages with no Cargo.toml file present. The main content consists of architecture documentation rather than implemented code.

**Note**: Since there's no Cargo.toml file, standard Rust commands are not yet available. The project structure suggests this will be a Rust project once implementation begins.

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

- This appears to be a greenfield project in the planning/design phase
- No actual Rust code has been implemented yet
- **Simplified Architecture**: Removed local LLM complexity, focusing on Claude Code orchestration
- **Resource Efficient**: ~2.1GB memory usage (vs 8GB+ with local models)  
- **M2 Optimized**: Native Apple Silicon compilation without GPU model management
- Heavy emphasis on agent coordination via Claude Code
- Discord serves as the primary human interface for the system
- GitHub integration provides automated repository management

## Coding Standards and Architecture Principles

The Spiral Core system follows strict architectural principles to ensure maintainability and extensibility. All development must adhere to:

- **SOLID Principles**: Single Responsibility, Open-Closed, Liskov Substitution, Interface Segregation, Dependency Inversion
- **DRY Principle**: Don't Repeat Yourself - single source of truth for all knowledge  
- **SID Naming**: Short, Intuitive, Descriptive naming conventions

For detailed implementation guidance and examples, see [Coding Standards](docs/CLAUDE-core-coding-standards.md).

## Modular Documentation Architecture

This CLAUDE.md file serves as the orchestrator for specialized documentation modules. For detailed implementation guidance, refer to the modular documentation:

### Core System Modules
- **[Coding Standards](docs/CLAUDE-core-coding-standards.md)** - SOLID, DRY, and SID principles with Rust-specific patterns

### Agent System Modules  
- **[Developer Agent](docs/CLAUDE-agents-developer.md)** - Code generation, language detection, and Claude Code integration
- **[Project Manager Agent](docs/CLAUDE-agents-pm.md)** - Strategic analysis and coordination patterns

### Integration Modules
- **[Discord Integration](docs/CLAUDE-integrations-discord.md)** - Conversational agent mentions and Discord bot patterns
- **[GitHub Integration](docs/CLAUDE-integrations-github.md)** - Automated repository management and PR creation  
- **[Claude Code Integration](docs/CLAUDE-integrations-claude-code.md)** - Primary intelligence engine patterns

### Implementation Modules
- **[Phase 1 Implementation](docs/CLAUDE-implementation-phase1.md)** - Foundation setup and core systems

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
