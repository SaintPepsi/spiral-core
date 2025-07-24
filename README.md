# Spiral Core

AI agent orchestration system with Claude Code integration and automated documentation management.

## Quick Start

### Prerequisites
- **Rust** (for core system)
- **Node.js 22+ LTS** (for documentation tooling)
- **Claude Code** CLI installed

### Installation

1. **Install Node.js dependencies:**
   ```bash
   npm install
   ```

2. **Install Rust dependencies:**
   ```bash
   cargo build
   ```

### Documentation Management

The project uses an automated documentation coordination system to maintain consistency across all CLAUDE*.md files.

#### Available Commands

```bash
# Start automated documentation watcher
npm run watch-docs
# or directly:
./claude-docs-coordinator.sh start-watcher

# Stop the watcher
npm run stop-watch

# Check coordinator status
npm run status

# Manually validate documentation
npm run validate-docs

# Emergency cleanup
./claude-docs-coordinator.sh cleanup
```

#### How It Works

1. **File Watcher**: Monitors all `CLAUDE*.md` files for changes
2. **Feedback Loop Prevention**: 5-minute cooldown + read-only validation
3. **Process Coordination**: File locking prevents conflicts
4. **Automatic Analysis**: Detects inconsistencies across documentation files

### Project Structure

```
spiral-core/
├── CLAUDE.md                           # Main documentation orchestrator
├── docs/                               # Modular documentation
│   ├── CLAUDE-agents-developer.md      # Developer agent implementation
│   ├── CLAUDE-agents-pm.md             # Project manager agent
│   ├── CLAUDE-core-coding-standards.md # Coding standards (SOLID/DRY/SID)
│   ├── CLAUDE-integrations-*.md        # Integration patterns
│   └── CLAUDE-implementation-*.md      # Implementation guides
├── claude-docs-coordinator.sh          # Documentation coordination system
├── package.json                        # Node.js dependencies
└── target/                             # Rust build artifacts
```

### Development Workflow

1. **Make changes** to any CLAUDE*.md file
2. **Automatic validation** runs within 10 seconds
3. **Review reports** for any inconsistencies detected
4. **Manual fixes** can be made using coordinator's manual mode
5. **Process coordination** ensures no conflicts

### Troubleshooting

- **Watcher not starting**: Check if chokidar is installed (`npm install`)
- **Feedback loops**: The system has built-in prevention, but check cooldown status
- **Lock conflicts**: Use `./claude-docs-coordinator.sh cleanup` for emergency reset
- **Process conflicts**: Check status with `./claude-docs-coordinator.sh status`

## Architecture

This is a hybrid Rust/Node.js project:
- **Rust**: Core AI agent orchestration system
- **Node.js**: Documentation tooling and file watching
- **Claude Code**: AI intelligence engine integration

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed system design.