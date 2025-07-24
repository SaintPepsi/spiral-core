# Use official Rust image as base
FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

# Install additional tools needed for development and agents
RUN apt-get update && apt-get install -y \
    # Basic development tools
    git \
    curl \
    wget \
    unzip \
    jq \
    # Tools for potential agent dependencies
    python3 \
    python3-pip \
    nodejs \
    npm \
    # Network tools for debugging
    netcat \
    telnet \
    # Process monitoring
    htop \
    procps \
    # SSL/TLS support
    ca-certificates \
    openssl \
    # Clean up
    && rm -rf /var/lib/apt/lists/*

# Install additional Rust tools for development
RUN rustup component add clippy rustfmt rust-analyzer rust-src

# Install useful cargo tools
RUN cargo install cargo-watch cargo-edit cargo-audit cargo-tree

# Install process management tools for running multiple agents
RUN npm install -g pm2

# Set up workspace
WORKDIR /workspace

# Create directories for agent workspaces and logs
RUN mkdir -p /workspace/workspaces /workspace/logs /workspace/tmp && \
    chown -R vscode:vscode /workspace

# Switch to vscode user (already exists in devcontainers base image)
USER vscode

# Set environment variables for development and agent execution
ENV RUST_LOG=debug
ENV CARGO_TARGET_DIR=/workspace/target
ENV SPIRAL_WORKSPACE_DIR=/workspace/workspaces
ENV SPIRAL_LOG_DIR=/workspace/logs

# Create a simple agent runner script
USER root
RUN echo '#!/bin/bash\n\
# Simple script to run agents in background\n\
AGENT_NAME=${1:-"default"}\n\
TASK=${2:-"help"}\n\
cd /workspace\n\
nohup cargo run -- dev "$TASK" > /workspace/logs/${AGENT_NAME}.log 2>&1 &\n\
echo $! > /workspace/tmp/${AGENT_NAME}.pid\n\
echo "Agent '\''$AGENT_NAME'\'' started with PID $(cat /workspace/tmp/${AGENT_NAME}.pid)"\n\
echo "View logs: tail -f /workspace/logs/${AGENT_NAME}.log"' > /usr/local/bin/run-agent && \
    chmod +x /usr/local/bin/run-agent

# Create agent management script
RUN echo '#!/bin/bash\n\
echo "Active agents:"\n\
for pidfile in /workspace/tmp/*.pid; do\n\
    if [ -f "$pidfile" ]; then\n\
        agent=$(basename "$pidfile" .pid)\n\
        pid=$(cat "$pidfile")\n\
        if ps -p "$pid" > /dev/null 2>&1; then\n\
            echo "  $agent (PID: $pid) - RUNNING"\n\
        else\n\
            echo "  $agent (PID: $pid) - STOPPED"\n\
            rm -f "$pidfile"\n\
        fi\n\
    fi\n\
done' > /usr/local/bin/list-agents && \
    chmod +x /usr/local/bin/list-agents

USER vscode

# Expose common ports for development and agent communication
EXPOSE 3000 8000 8080 9000

# Default command
CMD ["sleep", "infinity"]
