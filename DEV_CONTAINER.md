# Development Container Setup

This project includes a development container configuration that allows you to develop the Spiral Core project and run agents within a consistent, isolated environment.

## Quick Start

### Option 1: VS Code Dev Containers (Recommended)

1. Install the "Dev Containers" extension in VS Code
2. Open the project in VS Code
3. When prompted, click "Reopen in Container" or run:
   - `Ctrl/Cmd + Shift + P` → "Dev Containers: Reopen in Container"

### Option 2: Docker Compose

```bash
# Start the development container
docker-compose -f docker-compose.dev.yml up -d

# Execute commands in the container
docker-compose -f docker-compose.dev.yml exec spiral-dev bash

# Stop the container
docker-compose -f docker-compose.dev.yml down
```

## Development Workflow

Once inside the container, you can use the development helper script:

```bash
# Check the setup
./dev.sh check

# Build the project
./dev.sh build

# Start development mode with auto-rebuild
./dev.sh dev

# Run an agent
./dev.sh agent "create a simple web server"

# Run tests
./dev.sh test
```

## Features

### Development Environment

- **Rust toolchain**: Latest stable Rust with clippy, rustfmt, and rust-analyzer
- **VS Code extensions**: Pre-configured with Rust support, GitHub Copilot, and Docker tools
- **Auto-build**: Cargo watch for automatic rebuilding on file changes
- **Port forwarding**: Common development ports (3000, 8000, 8080, 9000) are exposed

### Agent Runtime

- **In-container execution**: Agents run within the same container as your development environment
- **Shared workspace**: Agents can access and modify files in your workspace
- **Network access**: Agents can make network requests and communicate with external services
- **Docker support**: Docker-in-Docker capability for agents that need to manage containers

## Container Architecture

```
┌─────────────────────────────────────────┐
│           Dev Container                 │
│  ┌─────────────────┐ ┌─────────────────┐│
│  │   Development   │ │     Agents      ││
│  │   Environment   │ │   (same space)  ││
│  │                 │ │                 ││
│  │ • VS Code       │ │ • vscode-agent  ││
│  │ • Rust tools    │ │ • Custom agents ││
│  │ • File editing  │ │ • Automation    ││
│  └─────────────────┘ └─────────────────┘│
│           Shared Workspace              │
└─────────────────────────────────────────┘
```

## Benefits of This Approach

1. **Consistency**: Same environment for all developers
2. **Isolation**: Dependencies and tools don't pollute your host system
3. **Simplicity**: No need for separate agent containers
4. **Efficiency**: Shared filesystem and resources
5. **Debugging**: Easy to debug agents in the same environment where you develop

## Customization

### Adding Development Tools

Edit `.devcontainer/Dockerfile` to add additional tools:

```dockerfile
RUN apt-get update && apt-get install -y \
    your-tool-here \
    && rm -rf /var/lib/apt/lists/*
```

### Adding VS Code Extensions

Edit `.devcontainer/devcontainer.json`:

```json
{
  "customizations": {
    "vscode": {
      "extensions": ["your.extension.id"]
    }
  }
}
```

### Environment Variables

Add environment variables in `docker-compose.dev.yml`:

```yaml
environment:
  - YOUR_ENV_VAR=value
```

## Troubleshooting

### Container won't start

- Check Docker is running
- Ensure no port conflicts
- Check the logs: `docker-compose -f docker-compose.dev.yml logs`

### Build fails

- Clear Docker cache: `docker system prune -a`
- Rebuild: `docker-compose -f docker-compose.dev.yml build --no-cache`

### Agent can't access files

- Ensure you're in the `/workspace` directory
- Check file permissions: `ls -la`
