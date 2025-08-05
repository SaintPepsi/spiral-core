# Spiral Core Agent Orchestration System

A Rust-based AI agent orchestration system built by Anti Spiral Interactive. The system creates specialized AI agents that collaborate through Claude Code integration to build tools and manage complex workflows.

## Quick Links

📋 **[Architecture](docs/ARCHITECTURE.md)** - Complete system design and components
🚀 **[Setup Guide](docs/SETUP.md)** - All installation and configuration options
🔧 **[Development](docs/DEVELOPMENT.md)** - Coding standards and practices
📖 **[API Reference](docs/API.md)** - HTTP endpoint documentation

## Architecture Overview

Spiral Core uses Claude Code as the primary intelligence engine, reducing complexity while maintaining sophisticated functionality:

- **Rust Backend**: Agent orchestration with Claude Code integration
- **Discord Bot Service**: Conversational agent mentions (@SpiralDev, @SpiralPM)
- **HTTP API**: RESTful endpoints for agent communication
- **Specialized Agents**: Developer, Project Manager, QA (planned)

## Quick Start

### Getting Started

**Prerequisites:**

- Rust 1.70+ with Cargo
- Claude API key from Anthropic
- Discord bot token (optional)

### Installation & Build

```bash
# Clone and build
git clone <repository-url>
cd spiral-core
cargo build --release

# Configure environment (copy .env.example to .env)
CLAUDE_API_KEY=sk-ant-api03-your-api-key-here
DISCORD_TOKEN=your-discord-bot-token  # optional
DISCORD_AUTHORIZED_USERS=123456789012345678,987654321098765432  # required for Discord

# Run
cargo run --bin spiral-core
```

See [Setup Guide](docs/SETUP.md) for all installation options including Docker, local development, and production deployment.

### Available Aliases

The development environment includes helpful aliases for common tasks:

```bash
# Show all available Spiral Core aliases
aliases

# Core operations
spiral-run          # Start the Spiral Core server
spiral-health       # Test health endpoint
spiral-test         # Run API task tests

# API Testing (with test environment)
api-health          # Test health endpoint
api-test            # Run all API tests
api-status          # Test system status
api-all             # Run all test files

# API Testing (with real environment)
api-health-real     # Test with real API keys
api-test-real       # Run tests with real environment
api-status-real     # Test status with real config
api-all-real        # Run all tests with real env

# Tool management
hurl-test           # Run comprehensive test script
install-hurl        # Install Hurl API testing tool
install-tools       # Install all development tools
```

**Note**: The `-real` aliases use your actual `.env` configuration, while regular aliases use test configuration from `.env.hurl`.

### Verify Installation

Run the comprehensive verification suite to ensure everything is working:

```bash
# Full system verification
npm run verify

# Quick verification (skips some time-consuming checks)
npm run verify:quick
```

The verification script checks:

- Environment setup (Rust, Node.js, dependencies)
- Build processes (Cargo build, TypeScript)
- Code quality (formatting, linting, clippy)
- Test suites (unit and integration tests)
- Documentation validity
- Security audits (if tools installed)
- API health (if server running)

## Usage

### Discord Integration

**🔐 Security First**: All Discord interactions require authorization. Configure authorized user IDs:

```bash
# In your .env file
DISCORD_AUTHORIZED_USERS=123456789012345678,987654321098765432
```

Conversational agent mentions for natural interaction:

```
@SpiralDev create a Python FastAPI todo application
@SpiralPM what's the best architecture for this microservice?
!spiral admin    # Access admin dashboard (authorized users only)
```

**Protected Commands & Mentions:**

- All `!spiral` commands (admin, security, debug, etc.)
- All spiral agent mentions (`@SpiralDev`, `@SpiralPM`, etc.)
- All spiral role mentions

**Unauthorized Access:** Returns contextual denial messages for security.

### HTTP API

RESTful endpoints for programmatic access:

```bash
# Submit task
curl -X POST http://localhost:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{"agent_type": "SoftwareDeveloper", "content": "Create hello world in Rust"}'

# System status
curl -H "x-api-key: your-api-key" http://localhost:3000/system/status
```

See [API Reference](docs/API.md) for complete endpoint documentation.

### Current Agents

- **SpiralDev**: Autonomous code generation with language detection
- **SpiralPM**: Strategic analysis and task coordination _(planned)_
- **SpiralQA**: Code review and validation _(planned)_

See [Agent Documentation](src/agents/docs/) for implementation details.

## Development

### Project Structure

```
src/
├── main.rs              # Main orchestration binary
├── agents/              # Agent implementations and docs
├── integrations/        # Claude Code, Discord, GitHub integration docs
├── implementation/      # Phase-based implementation guides
├── claude_code.rs       # Claude Code API client
├── discord.rs           # Discord bot integration
├── api.rs               # HTTP API server
└── bin/discord_bot.rs   # Discord bot binary
```

### Running Tests

```bash
cargo test                    # All tests
cargo test --test integration # Integration tests only
RUST_LOG=debug cargo test    # With debug logging
```

### Phase 2 Validation (CRCC)

Run standalone Core Rust Compliance Checks without the full pipeline:

```bash
# Run Phase 2 validation checks in parallel
cargo run --bin run_phase2
```

This executes all Phase 2 compliance checks:

- **Compilation**: `cargo check --all-targets`
- **Tests**: `cargo test`
- **Formatting**: `cargo fmt -- --check`
- **Clippy**: `cargo clippy --all-targets`
- **Documentation**: `cargo doc --no-deps`

All checks run in parallel for optimal performance. In the full validation pipeline, failed checks would trigger Claude agents for automatic fixes.

### Development Guidelines

The project follows strict architectural principles:

- **SOLID Principles**: Single responsibility, open-closed, etc.
- **DRY Principle**: Single source of truth for all knowledge
- **SID Naming**: Short, Intuitive, Descriptive conventions

See [Development Guide](docs/DEVELOPMENT.md) for complete standards.

## System Architecture

### Key Benefits

- **Resource Efficient**: ~2.1GB RAM vs 8GB+ for local LLM approaches
- **Claude Code Integration**: Access to latest AI capabilities via API
- **Apple Silicon Optimized**: Native compilation without GPU complexity
- **Simplified Deployment**: No local model management or CUDA dependencies

### System Requirements

- **Memory**: ~2.1GB RAM
- **CPU**: 2+ cores recommended
- **Network**: Stable internet for Claude API calls

See [Architecture](docs/ARCHITECTURE.md) for complete system design.

## Troubleshooting

Common issues and solutions:

1. **Claude API Errors**: Verify API key and rate limits
2. **Discord Connection**: Check bot token and permissions
3. **Compilation**: Ensure Rust 1.70+ and dependencies

Debug mode: `RUST_LOG=debug cargo run`

See [Setup Guide](docs/SETUP.md#troubleshooting) for detailed troubleshooting.

## Documentation

### Core Documentation
- **[Architecture](docs/ARCHITECTURE.md)** - System design and components
- **[Setup Guide](docs/SETUP.md)** - Installation and configuration
- **[Development](docs/DEVELOPMENT.md)** - Standards and practices
- **[API Reference](docs/API.md)** - Endpoint documentation
- **[Operations](docs/OPERATIONS.md)** - Deployment and monitoring

### Implementation Documentation
- **[Agent Guides](src/agents/docs/)** - Agent-specific implementation
- **[Integration Patterns](src/integrations/docs/)** - Integration examples
- **[Implementation Phases](src/implementation/docs/)** - Development roadmap

## Contributing

1. Follow architectural principles in [Development Guide](docs/DEVELOPMENT.md)
2. Ensure all tests pass before submitting PRs
3. Update relevant documentation for changes
4. See [Contributing Guidelines](docs/CONTRIBUTING.md) for workflow

## License

MIT License - see LICENSE file for details.

---
