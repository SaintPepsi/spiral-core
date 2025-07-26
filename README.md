# Spiral Core Agent Orchestration System

A Rust-based AI agent orchestration system built by Anti Spiral Interactive. The system creates specialized AI agents that collaborate through Claude Code integration to build tools and manage complex workflows.

## Quick Links

📋 **[Architecture Overview](docs/ARCHITECTURE_OVERVIEW.md)** - System design and component relationships  
🏗️ **[Development Setup](docs/DEVELOPMENT_PRACTICES.md)** - Local development and container setup  
🔧 **[Coding Standards](docs/CODING_STANDARDS.md)** - SOLID, DRY, SID principles and best practices  
🚀 **[Phase 1 Implementation](src/implementation/docs/IMPLEMENTATION_PHASE1.md)** - Current development phase  

## Architecture Overview

Spiral Core uses Claude Code as the primary intelligence engine, reducing complexity while maintaining sophisticated functionality:

- **Rust Backend**: Agent orchestration with Claude Code integration
- **Discord Bot Service**: Conversational agent mentions (@SpiralDev, @SpiralPM)
- **HTTP API**: RESTful endpoints for agent communication
- **Specialized Agents**: Developer, Project Manager, QA (planned)

## Quick Start

### Development Container (Recommended)

1. **Prerequisites:** Docker Desktop + VS Code with Dev Containers extension
2. **Open in container:** `code .` → "Dev Containers: Reopen in Container"
3. **Configure:** Update `.env` with your Claude API key
4. **Run:** `spiral-start` or `./test-api.sh`

See [.devcontainer/DEVCONTAINER-GUIDE.md](.devcontainer/DEVCONTAINER-GUIDE.md) for complete setup.

### Local Development

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

# Run
cargo run --bin spiral-core
```

See [Development Practices](docs/DEVELOPMENT_PRACTICES.md) for complete environment setup.

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

Conversational agent mentions for natural interaction:

```
@SpiralDev create a Python FastAPI todo application
@SpiralPM what's the best architecture for this microservice?
```

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

See [API Reference](src/api/API_REFERENCE.md) for complete endpoint documentation.

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

### Development Guidelines

The project follows strict architectural principles documented in [Coding Standards](docs/CODING_STANDARDS.md):

- **SOLID Principles**: Single responsibility, open-closed, etc.
- **DRY Principle**: Single source of truth for all knowledge
- **SID Naming**: Short, Intuitive, Descriptive conventions

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

See [Architecture Overview](docs/ARCHITECTURE_OVERVIEW.md) for complete system design.

## Troubleshooting

Common issues and solutions:

1. **Claude API Errors**: Verify API key and rate limits
2. **Discord Connection**: Check bot token and permissions  
3. **Compilation**: Ensure Rust 1.70+ and dependencies

Debug mode: `RUST_LOG=debug cargo run`

See [Development Practices](docs/DEVELOPMENT_PRACTICES.md) for detailed troubleshooting.

## Documentation Structure

As the project grows, we're transitioning to a wiki-style documentation model:

```
docs/                           # Core project documentation
├── ARCHITECTURE_*.md          # System architecture guides
├── CODING_STANDARDS.md        # Development standards
└── DEVELOPMENT_PRACTICES.md   # Setup and workflow guides

src/                           # Implementation-specific docs
├── agents/docs/              # Agent implementation guides
├── integrations/docs/        # Integration patterns and examples
└── implementation/docs/      # Phase-based implementation plans
```

## Contributing

1. Follow [Coding Standards](docs/CODING_STANDARDS.md) and architectural principles
2. Ensure all tests pass before submitting PRs  
3. Update relevant documentation for changes
4. See [Development Practices](docs/DEVELOPMENT_PRACTICES.md) for workflow

## License

MIT License - see LICENSE file for details.

---

💡 **Moving to Wiki Model**: As documentation grows, we're transitioning to a modular, wiki-style approach. Links above will eventually point to dedicated wiki pages for better organization and searchability.
