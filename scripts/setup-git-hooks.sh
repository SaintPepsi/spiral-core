#!/bin/bash

# 🔧 Setup script for Git hooks
# Configures git to use the custom hooks in .githooks directory

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 Setting up Git hooks for Spiral Core...${NC}"
echo ""

# Configure git to use our hooks directory
git config core.hooksPath .githooks

echo -e "${GREEN}✅ Git configured to use .githooks directory${NC}"
echo ""

# Check if markdownlint is installed
if command -v markdownlint &> /dev/null; then
    echo -e "${GREEN}✅ markdownlint is installed${NC}"
elif command -v npx &> /dev/null; then
    echo -e "${YELLOW}⚠️  markdownlint not found globally, will use npx${NC}"
    echo -e "   To install globally: npm install -g markdownlint-cli"
else
    echo -e "${YELLOW}⚠️  markdownlint not available${NC}"
    echo -e "   To enable Markdown linting: npm install -g markdownlint-cli"
fi
echo ""

# List available hooks
echo -e "${BLUE}📋 Available hooks:${NC}"
for hook in .githooks/*; do
    if [ -f "$hook" ]; then
        hook_name=$(basename "$hook")
        echo -e "  • ${hook_name}"
    fi
done
echo ""

echo -e "${GREEN}✅ Git hooks setup complete!${NC}"
echo ""
echo "The pre-commit hook will automatically:"
echo "  • Format Rust code with cargo fmt"
echo "  • Fix Markdown linting issues (if markdownlint is installed)"
echo "  • Verify code compiles"
echo ""
echo "To bypass hooks temporarily, use: git commit --no-verify"