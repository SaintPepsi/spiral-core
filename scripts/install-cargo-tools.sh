#!/bin/bash

# Install additional Cargo tools on-demand
# Usage: ./scripts/install-cargo-tools.sh [tool1] [tool2] ...

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_header() {
    echo -e "\n${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Available tools with descriptions
declare -A TOOLS=(
    ["cargo-audit"]="Security audit tool for Rust dependencies"
    ["cargo-outdated"]="Check for outdated dependencies"
    ["cargo-expand"]="Show macro expansions"
    ["cargo-tree"]="Display dependency tree"
    ["cargo-machete"]="Remove unused dependencies"
    ["cargo-udeps"]="Find unused dependencies"
    ["cargo-deny"]="Cargo plugin for linting dependencies"
    ["cargo-feature"]="Manage feature flags"
)

show_available_tools() {
    print_header "Available Cargo Tools"
    for tool in "${!TOOLS[@]}"; do
        echo "  $tool - ${TOOLS[$tool]}"
    done
    echo ""
    echo "Usage: $0 [tool1] [tool2] ..."
    echo "Example: $0 cargo-audit cargo-outdated"
}

install_tool() {
    local tool=$1
    if command -v "$tool" &> /dev/null; then
        print_warning "$tool is already installed"
        return 0
    fi
    
    echo "Installing $tool..."
    if cargo install --locked "$tool"; then
        print_success "$tool installed successfully"
    else
        echo "Failed to install $tool"
        return 1
    fi
}

# Main execution
if [ $# -eq 0 ]; then
    show_available_tools
    exit 0
fi

print_header "Installing Cargo Tools"

for tool in "$@"; do
    install_tool "$tool"
done

print_success "Tool installation complete!"