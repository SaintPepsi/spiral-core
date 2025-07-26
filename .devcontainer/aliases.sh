#!/bin/bash
# Spiral Core Development Aliases
# This file is sourced by the Dockerfile to set up development aliases

# System aliases
alias ll="ls -la"
alias la="ls -A"
alias l="ls -CF"
alias ..="cd .."
alias ...="cd ../.."

# Git aliases
alias gs="git status"
alias gd="git diff"
alias gl="git log --oneline -10"

# Cargo shortcuts
alias cb="cargo build"
alias cr="cargo run"
alias ct="cargo test"
alias cc="cargo check"
alias cf="cargo fmt"
alias ccl="cargo clippy"

# Spiral Core specific aliases
alias spiral-run="cargo run --bin spiral-core"
alias spiral-test="hurl --variables-file .env.hurl --test src/api/tests/hurl/tasks.hurl"
alias spiral-health="hurl --variables-file .env.hurl --test src/api/tests/hurl/health.hurl"

# API Testing shortcuts with Hurl (test env)
alias api-test="hurl --variables-file .env.hurl --test src/api/tests/hurl/"
alias api-health="hurl --variables-file .env.hurl --test src/api/tests/hurl/health.hurl"
alias api-status="hurl --variables-file .env.hurl --test src/api/tests/hurl/system.hurl"
alias api-all="hurl --variables-file .env.hurl --test src/api/tests/hurl/*.hurl"
alias hurl-test="./scripts/test-api-hurl.sh"

# API Testing shortcuts with real env
alias api-test-real="hurl --variables-file .env --test src/api/tests/hurl/"
alias api-health-real="hurl --variables-file .env --test src/api/tests/hurl/health.hurl"
alias api-status-real="hurl --variables-file .env --test src/api/tests/hurl/system.hurl"
alias api-all-real="hurl --variables-file .env --test src/api/tests/hurl/*.hurl"

# Tool installation shortcuts
alias install-hurl="cargo install hurl --locked"
alias install-cargo-watch="cargo install cargo-watch --locked"
alias install-tools="./scripts/install-cargo-tools.sh"

# Utility aliases
alias aliases="alias | grep -E '^(spiral|api|hurl|install)' | sort"