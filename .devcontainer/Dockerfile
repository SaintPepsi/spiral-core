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

# Add comprehensive aliases to bashrc
RUN echo '' >> ~/.bashrc \
    && echo '# System aliases' >> ~/.bashrc \
    && echo 'alias ll="ls -la"' >> ~/.bashrc \
    && echo 'alias la="ls -A"' >> ~/.bashrc \
    && echo 'alias l="ls -CF"' >> ~/.bashrc \
    && echo 'alias ..="cd .."' >> ~/.bashrc \
    && echo 'alias ...="cd ../.."' >> ~/.bashrc \
    && echo '' >> ~/.bashrc \
    && echo '# Git aliases' >> ~/.bashrc \
    && echo 'alias gs="git status"' >> ~/.bashrc \
    && echo 'alias gd="git diff"' >> ~/.bashrc \
    && echo 'alias gl="git log --oneline -10"' >> ~/.bashrc \
    && echo '' >> ~/.bashrc \
    && echo '# Cargo shortcuts' >> ~/.bashrc \
    && echo 'alias cb="cargo build"' >> ~/.bashrc \
    && echo 'alias cr="cargo run"' >> ~/.bashrc \
    && echo 'alias ct="cargo test"' >> ~/.bashrc \
    && echo 'alias cc="cargo check"' >> ~/.bashrc \
    && echo 'alias cw="cargo watch -x check -x test"' >> ~/.bashrc \
    && echo 'alias cf="cargo fmt"' >> ~/.bashrc \
    && echo 'alias ccl="cargo clippy"' >> ~/.bashrc \
    && echo '' >> ~/.bashrc \
    && echo '# Spiral Core specific aliases' >> ~/.bashrc \
    && echo 'alias spiral-start="cargo run --bin spiral-core"' >> ~/.bashrc \
    && echo 'alias spiral-discord="cargo run --bin discord-bot"' >> ~/.bashrc \
    && echo 'alias spiral-test="hurl --test tests/api/api.hurl"' >> ~/.bashrc \
    && echo 'alias spiral-health="hurl --test tests/api/health.hurl"' >> ~/.bashrc \
    && echo 'alias spiral-build="cargo build --release"' >> ~/.bashrc \
    && echo 'alias spiral-watch="cargo watch -x run --bin spiral-core"' >> ~/.bashrc \
    && echo '' >> ~/.bashrc \
    && echo '# API Testing shortcuts with Hurl' >> ~/.bashrc \
    && echo 'alias api-test="hurl --test tests/api/"' >> ~/.bashrc \
    && echo 'alias api-health="hurl --test tests/api/health.hurl"' >> ~/.bashrc \
    && echo 'alias api-status="hurl --test tests/api/status.hurl"' >> ~/.bashrc \
    && echo 'alias api-all="hurl --test tests/api/*.hurl"' >> ~/.bashrc \
    && echo 'alias hurl-test="./scripts/test-api-hurl.sh"' >> ~/.bashrc

# Append aliases to zsh config (preserve Oh My Zsh)
RUN cat ~/.bashrc >> ~/.zshrc