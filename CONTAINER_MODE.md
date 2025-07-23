# VS Code Agent - Containerized Mode

## Problem Solved: Concurrent VS Code Access

The VS Code Agent now supports **containerized mode** to solve the concurrency issue you identified. When multiple VS Code instances try to use the chat agent simultaneously, they can interfere with each other.

## Solutions Provided

### 1. Automatic Detection Mode (Default)

```bash
# The agent automatically detects conflicts and suggests solutions
vscode-agent check
```

**What it detects:**

- ✅ Docker availability
- ⚠️ Running VS Code instances that could cause conflicts
- 💡 Recommendations for isolation

### 2. Containerized Mode (Recommended for Production)

```bash
# Create isolated VS Code container
vscode-agent container create

# Setup GitHub Copilot in container
vscode-agent container setup

# Enable container mode
export VSCODE_AGENT_USE_CONTAINER=true

# Now all operations use the isolated container
vscode-agent dev "Create a REST API"
```

### 3. Direct Mode (Development/Testing)

```bash
# Force direct mode (may conflict with active VS Code)
export VSCODE_AGENT_USE_CONTAINER=false
vscode-agent dev "Simple task"
```

## Container Management Commands

### Create and Setup

```bash
# Create the container (one-time setup)
vscode-agent container create

# Install and setup GitHub Copilot
vscode-agent container setup

# Manually configure Copilot (if needed)
docker exec -it vscode-agent-container code
```

### Daily Operations

```bash
# Check container status
vscode-agent container status

# Start container
vscode-agent container start

# Stop container
vscode-agent container stop

# Remove container (clean slate)
vscode-agent container remove
```

## Environment Variables

```bash
# Container mode control
export VSCODE_AGENT_USE_CONTAINER=true    # Force container mode
export VSCODE_AGENT_USE_CONTAINER=false   # Force direct mode
export VSCODE_AGENT_USE_CONTAINER=auto    # Auto-detect (default)

# Workspace location
export WORKSPACE_DIR=/path/to/workspaces
```

## Workflow Examples

### Development Workflow (You actively using VS Code)

```bash
# Setup once
vscode-agent container create
vscode-agent container setup
export VSCODE_AGENT_USE_CONTAINER=true

# Daily usage - no conflicts with your active VS Code
vscode-agent dev "Create user authentication system"
vscode-agent dev "Add database migrations"
vscode-agent dev "Implement rate limiting"
```

### CI/CD Pipeline

```bash
# In your CI pipeline
export VSCODE_AGENT_USE_CONTAINER=true
vscode-agent container create
vscode-agent container setup
vscode-agent test  # Run integration tests
```

### Multiple Agents (Enterprise)

```bash
# Each agent gets its own container
docker run -d --name agent-1 vscode-agent:latest
docker run -d --name agent-2 vscode-agent:latest
# etc.
```

## Benefits of Containerized Mode

### ✅ **Complete Isolation**

- No interference with your active VS Code session
- Multiple agents can run simultaneously
- Clean, reproducible environment

### ✅ **Production Ready**

- Consistent GitHub Copilot configuration
- Isolated from host system changes
- Easy deployment and scaling

### ✅ **Developer Friendly**

- Work normally in VS Code while agents run
- No "chat cancelled" interruptions
- Multiple projects can generate code simultaneously

## Architecture Comparison

### Before (Direct Mode)

```
Your VS Code ←→ GitHub Copilot
     ↑
   Conflict!
     ↑
Agent calls `code chat` ←→ Same Copilot instance
```

### After (Container Mode)

```
Your VS Code ←→ GitHub Copilot (Instance 1)

Agent Container ←→ GitHub Copilot (Instance 2)
```

This solves exactly the problem you identified - now you can have your "mundane AI tasks" running in containers while you actively use VS Code for normal work, with zero interference!

## Quick Start

```bash
# One-time setup (5 minutes)
vscode-agent container create
vscode-agent container setup

# Enable container mode
echo 'export VSCODE_AGENT_USE_CONTAINER=true' >> ~/.bashrc
source ~/.bashrc

# Start generating code without conflicts!
vscode-agent dev "Create a web scraper"
```

Your VS Code sessions will never be interrupted again! 🎉
