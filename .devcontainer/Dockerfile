# Use the official Rust dev container image as base
FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

# Install GitHub CLI with verification
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg -o /tmp/gh-keyring.gpg \
    && gpg --no-default-keyring --keyring /tmp/test-keyring.gpg --import /tmp/gh-keyring.gpg \
    && dd if=/tmp/gh-keyring.gpg of=/usr/share/keyrings/githubcli-archive-keyring.gpg \
    && chmod go+r /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
    && rm -f /tmp/gh-keyring.gpg /tmp/test-keyring.gpg

# Skip Hurl installation in base image - install on first use
# This saves ~2-5 minutes of build time
RUN echo "Hurl will be installed on first use via: cargo install hurl"

# Install system dependencies  
RUN apt-get update && apt-get install -y \
    # Build essentials
    build-essential \
    pkg-config \
    libssl-dev \
    # Network tools
    curl \
    wget \
    # JSON processing
    jq \
    # Process management
    htop \
    # Git utilities
    git-extras \
    # Additional tools
    tree \
    vim \
    nano \
    && rm -rf /var/lib/apt/lists/*

# Set up Rust toolchain in single layer
RUN rustup component add clippy rustfmt rust-src \
    && rustup target add wasm32-unknown-unknown

# Skip Cargo tools in base image - install on first use to save build time
# This saves ~8-10 minutes of build time per build
RUN echo "Cargo tools available via: ./scripts/install-cargo-tools.sh"

# Switch to vscode user early
USER vscode

# Create workspace directory
WORKDIR /workspaces/spiral-core

# Set environment variables for development
ENV RUST_LOG=debug
ENV RUST_BACKTRACE=1
ENV PATH="/home/vscode/.cargo/bin:/home/vscode/.local/bin:$PATH"

# Install Claude Code CLI as vscode user
RUN curl -fsSL https://claude.ai/install.sh -o /tmp/claude-install.sh \
    && chmod +x /tmp/claude-install.sh \
    && /tmp/claude-install.sh \
    && rm -f /tmp/claude-install.sh \
    && echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc \
    && echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc

# Copy and source aliases file
COPY aliases.sh /tmp/aliases.sh
RUN echo '' >> ~/.bashrc \
    && echo '# Spiral Core Development Aliases' >> ~/.bashrc \
    && echo 'source /tmp/aliases.sh' >> ~/.bashrc \
    && echo '' >> ~/.zshrc \
    && echo '# Spiral Core Development Aliases' >> ~/.zshrc \
    && echo 'source /tmp/aliases.sh' >> ~/.zshrc