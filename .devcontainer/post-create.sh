#!/bin/bash

# Post-create script to install tools on first container startup
# This runs once when the dev container is first created

set -e

echo "🚀 Setting up Spiral Core development environment..."

# Install Hurl if not already installed
if ! command -v hurl &> /dev/null; then
    echo "📦 Installing Hurl (API testing tool)..."
    cargo install hurl --locked --quiet
    echo "✅ Hurl installed successfully"
else
    echo "✅ Hurl already installed"
fi

# Install essential Cargo tools if not already installed
if ! command -v cargo-watch &> /dev/null; then
    echo "📦 Installing cargo-watch..."
    cargo install cargo-watch --locked --quiet
    echo "✅ cargo-watch installed successfully"
else
    echo "✅ cargo-watch already installed"
fi

if ! cargo --list | grep -q edit; then
    echo "📦 Installing cargo-edit..."
    cargo install cargo-edit --locked --quiet
    echo "✅ cargo-edit installed successfully"
else
    echo "✅ cargo-edit already installed"
fi

# Create .env from template if it doesn't exist
if [ ! -f "/workspaces/spiral-core/.env" ] && [ -f "/workspaces/spiral-core/.env.example" ]; then
    echo "📄 Creating .env file from template..."
    cp /workspaces/spiral-core/.env.example /workspaces/spiral-core/.env
    echo "⚠️  Please update the .env file with your actual API keys!"
fi

# Make scripts executable
if [ -d "/workspaces/spiral-core/scripts" ]; then
    echo "🔧 Making scripts executable..."
    chmod +x /workspaces/spiral-core/scripts/*.sh
fi

echo "✅ Development environment setup complete!"
echo ""
echo "🎯 Quick start commands:"
echo "  spiral-health   - Test API health endpoint" 
echo "  api-test        - Run all API tests"
echo "  hurl-test       - Run API test script"
echo "  ./scripts/install-cargo-tools.sh - Install additional tools"