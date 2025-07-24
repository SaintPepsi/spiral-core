# Spiral Core Dev Container

This dev container provides a complete development environment for the Spiral Core project, supporting both development and agent execution within the same container.

## Features

- **Rust Development Environment**: Complete Rust toolchain with useful cargo tools
- **VS Code Integration**: Pre-configured extensions and settings for Rust development
- **Agent Execution**: Run multiple agents within the dev container
- **Process Management**: Built-in scripts for managing agent processes
- **Port Forwarding**: Automatic forwarding of common development ports

## Getting Started

1. **Open in Dev Container**:

   - Open VS Code in the project root
   - Press `F1` and select "Dev Containers: Reopen in Container"
   - Wait for the container to build and VS Code to connect

2. **Build the Project**:

   ```bash
   cargo build
   ```

3. **Run the Main Application**:
   ```bash
   cargo run -- --help
   ```

## Agent Management

### Running Agents

You can run agents in several ways:

1. **Using VS Code Tasks**:

   - Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on Mac)
   - Type "Tasks: Run Task"
   - Select "Start Agent"
   - Enter agent name and task

2. **Using the Command Line**:

   ```bash
   # Start an agent in background
   run-agent myagent "create a simple rust function"

   # List active agents
   list-agents

   # Watch agent logs
   tail -f /workspace/logs/myagent.log
   ```

3. **Direct Execution**:
   ```bash
   # Run agent in foreground
   cargo run -- dev "create a simple rust function"
   ```

### Agent Logs and Monitoring

- Agent logs are stored in `/workspace/logs/`
- Process IDs are tracked in `/workspace/tmp/`
- Use `list-agents` to see running agents
- Use VS Code task "Watch Agent Logs" to monitor specific agents

## Development Workflow

### Building and Testing

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run with watch mode (auto-rebuild on changes)
cargo watch -x build

# Check code quality
cargo clippy
```

### Debugging

- Use VS Code's built-in debugger with the provided launch configurations
- Set breakpoints in your Rust code
- Use "Debug Main" or "Debug Dev Command" configurations

### VS Code Tasks

Available tasks (access via `Ctrl+Shift+P` → "Tasks: Run Task"):

- **Build Project**: Build the Rust project
- **Run Project**: Run with help flag
- **Test Project**: Run all tests
- **Start Agent**: Start a new agent (interactive)
- **List Active Agents**: Show running agents
- **Watch Agent Logs**: Monitor specific agent logs
- **Clean Workspaces**: Clean up generated workspaces

## Environment Variables

- `RUST_LOG=debug`: Enable debug logging
- `CARGO_TARGET_DIR=/workspace/target`: Shared build directory
- `SPIRAL_WORKSPACE_DIR=/workspace/workspaces`: Agent workspace directory
- `SPIRAL_LOG_DIR=/workspace/logs`: Agent log directory

## Port Forwarding

The following ports are automatically forwarded:

- `3000`: Development Server
- `8000`: Agent API
- `8080`: Agent Management
- `9000`: Debug/Monitoring

## File Structure

```
/workspace/
├── src/                 # Source code
├── target/              # Build artifacts (cached)
├── workspaces/          # Agent-generated workspaces
├── logs/                # Agent logs
├── tmp/                 # Process ID files
└── .devcontainer/       # Dev container configuration
```

## Tips

1. **Performance**: The container uses volume mounts for cargo cache and workspaces to improve performance
2. **Multiple Agents**: You can run multiple agents simultaneously - they'll each get unique log files
3. **Persistence**: Agent workspaces and logs persist between container restarts
4. **Docker Access**: The container has docker-in-docker support if you need to run containers from within

## Troubleshooting

### Agent Won't Start

- Check if the build succeeded: `cargo build`
- Verify the agent command syntax: `cargo run -- --help`
- Check agent logs: `tail -f /workspace/logs/<agent-name>.log`

### Port Conflicts

- VS Code will automatically forward ports, but you can manually configure them in the dev container settings
- Use `netstat -tlnp` to check what's listening on ports

### Performance Issues

- Make sure you're using the volume mounts for target directory
- Consider increasing Docker resources if builds are slow

For more help, check the main project documentation or create an issue.
