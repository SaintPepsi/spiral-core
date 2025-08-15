# Spiral Core Makefile
# Provides consistent commands across all platforms

.PHONY: help setup build test fmt lint clean start stop pre-commit validate analyze

# Default target
help:
	@echo "Spiral Core - Available commands:"
	@echo "  make setup      - Initial setup (hooks, dependencies)"
	@echo "  make build      - Build the project in release mode"
	@echo "  make test       - Run all tests"
	@echo "  make fmt        - Format code (Rust + Markdown)"
	@echo "  make lint       - Run linting checks"
	@echo "  make validate   - Run comprehensive quality validation (build, test, lint, fmt)"
	@echo "  make analyze    - Run post-commit analysis agents"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make start      - Start Spiral Core"
	@echo "  make stop       - Stop Spiral Core"
	@echo "  make pre-commit - Run pre-commit checks manually"

# Initial setup for new developers
setup:
	@echo "🔧 Setting up Spiral Core development environment..."
	@./scripts/setup-git-hooks.sh
	@./scripts/setup-analysis-hooks.sh
	@echo "✅ Setup complete!"

# Build the project
build:
	cargo build --release

# Run tests
test:
	cargo test --all-features

# Format all code
fmt:
	cargo fmt
	@if command -v markdownlint > /dev/null 2>&1; then \
		find . -name "*.md" -not -path "./target/*" -exec markdownlint --fix {} \; ; \
	fi

# Run linting
lint:
	cargo clippy --all-targets -- -D warnings
	cargo fmt -- --check

# Run comprehensive quality validation
validate:
	@./scripts/validate-quality.sh

# Run post-commit analysis agents
analyze:
	@echo "🤖 Running post-commit analysis agents..."
	@if [ ! -f ./target/release/spiral-analyze ] && [ ! -f ./target/debug/spiral-analyze ]; then \
		echo "Building spiral-analyze binary..."; \
		cargo build --bin spiral-analyze; \
	fi
	@if [ -f ./target/release/spiral-analyze ]; then \
		./target/release/spiral-analyze; \
	elif [ -f ./target/debug/spiral-analyze ]; then \
		./target/debug/spiral-analyze; \
	else \
		echo "❌ Failed to build spiral-analyze binary"; \
		exit 1; \
	fi

# Clean build artifacts
clean:
	cargo clean
	rm -rf logs/ pids/

# Start the system
start:
	./start-spiral-core.sh

# Stop the system
stop:
	./stop-spiral-core.sh

# Run pre-commit checks manually
pre-commit:
	@echo "🔧 Running pre-commit checks..."
	@cargo fmt
	@cargo clippy --all-targets
	@cargo check --all-targets
	@echo "✅ Pre-commit checks passed!"

# Install development dependencies (optional)
install-dev-deps:
	@echo "Installing development dependencies..."
	@if ! command -v markdownlint > /dev/null 2>&1; then \
		echo "Installing markdownlint..."; \
		npm install -g markdownlint-cli; \
	fi
	@echo "✅ Development dependencies installed!"