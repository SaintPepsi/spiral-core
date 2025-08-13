# Spiral Core Documentation

## 📚 Documentation Structure

This directory contains all technical documentation for the Spiral Core system.

### Core Documentation

- [Architecture](./ARCHITECTURE.md) - Complete system architecture
- [Setup Guide](./SETUP.md) - Installation and configuration  
- [Development Guide](./DEVELOPMENT.md) - Development practices and standards
- [Security Policy](./SECURITY_POLICY.md) - Security measures and vulnerability reporting
- [Coding Standards](./CODING_STANDARDS.md) - Code quality and best practices
- [Engineering Principles](./ENGINEERING_PRINCIPLES.md) - Core engineering philosophy

### Agent System

- [Developer Agent](./agents/DEVELOPER.md) - Autonomous code generation agent
- [Project Manager Agent](./agents/PROJECT_MANAGER.md) - Strategic planning and coordination

### Integrations

- [Claude Code](./integrations/CLAUDE_CODE.md) - Primary AI engine integration
- [Discord](./integrations/DISCORD.md) - Bot integration and command system
- [GitHub](./integrations/GITHUB.md) - Repository management and automation
- [Claude Code Integration Guide](./CLAUDE_CODE_INTEGRATION.md) - Implementation details

### Self-Update System

- [Self-Update Guide](./SELF_UPDATE_GUIDE.md) - Autonomous improvement system
- [Self-Update Pipeline](./SELF_UPDATE_PIPELINE_IMPROVEMENT.md) - Two-phase validation
- [Agent Interaction Model](./AGENT_INTERACTION_MODEL.md) - How agents communicate

### Implementation Phases

- [Phase 1](./implementation/PHASE1.md) - Foundation and core systems

### API Documentation

- [API Reference](./api/API_REFERENCE.md) - HTTP API endpoints and testing

### Standards & Patterns

- [Naming Conventions](./NAMING_CONVENTIONS.md) - Spiral naming philosophy
- [Markdown Standards](./MARKDOWN_STANDARDS.md) - Documentation formatting
- [Colocation Patterns](./COLOCATION_PATTERNS.md) - Code organization
- [Decoupling Patterns](./DECOUPLING_PATTERNS.md) - Architecture patterns
- [Audit Documentation](./AUDIT_DOCUMENTATION_STANDARD.md) - Decision tracking
- [Dutch Communication](./DUTCH_AGENT_COMMUNICATION.md) - Direct communication style
- [Fibonacci Scale](./FIBONACCI_SCALE.md) - Risk and complexity metrics

### Checklists & Templates

- [Task Checklist](./TASK_CHECKLIST.md) - Pre-task review guidelines
- [Claude Completion Checklist](./CLAUDE_COMPLETION_CHECKLIST.md) - Quality requirements

### Testing & Validation

- [Phase 2 Validation Script](../scripts/test-phase2-validation.sh) - CRCC test suite

## Quick Links

- **Main README**: [../README.md](../README.md)
- **CLAUDE.md**: [../CLAUDE.md](../CLAUDE.md) - AI assistant instructions
- **Source Code**: [../src/](../src/)
- **Tests**: [../src/tests/](../src/tests/)

## Documentation Guidelines

1. All documentation should be in Markdown format
2. Follow [Markdown Standards](./MARKDOWN_STANDARDS.md)
3. Keep documentation close to code when possible
4. Update documentation when changing functionality
5. Use clear, concise language
6. Include examples where helpful

## Contributing

When adding new documentation:
1. Place it in the appropriate subdirectory
2. Update this README index
3. Update CLAUDE.md if it affects AI interactions
4. Ensure all links are relative and working