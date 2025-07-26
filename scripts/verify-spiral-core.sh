#!/bin/bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track overall status
FAILED=0

echo -e "${BLUE}=== Spiral Core Verification Suite ===${NC}"
echo ""

# Function to run a check
run_check() {
    local name="$1"
    local cmd="$2"
    
    echo -n "Checking $name... "
    if eval "$cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        FAILED=1
    fi
}

# Function to run a test suite
run_test() {
    local name="$1"
    local cmd="$2"
    
    echo -e "\n${YELLOW}Running $name...${NC}"
    if eval "$cmd"; then
        echo -e "${GREEN}✓ $name passed${NC}"
    else
        echo -e "${RED}✗ $name failed${NC}"
        FAILED=1
    fi
}

# 1. Environment checks
echo -e "${BLUE}1. Environment Verification${NC}"
run_check "Rust installation" "rustc --version"
run_check "Cargo installation" "cargo --version"
run_check "Node.js installation" "node --version"
run_check "npm installation" "npm --version"

# 2. Dependency checks
echo -e "\n${BLUE}2. Dependency Verification${NC}"
run_check "Rust dependencies" "cargo check"
run_check "Node dependencies" "npm list --depth=0"

# 3. Build verification
echo -e "\n${BLUE}3. Build Verification${NC}"
run_test "Rust build" "cargo build --release"

# 4. Linting and formatting
echo -e "\n${BLUE}4. Code Quality Checks${NC}"
run_test "Rust formatting" "cargo fmt -- --check"
run_test "Rust linting" "cargo clippy -- -D warnings"
run_test "Markdown linting" "npm run lint:md"

# 5. Test suites
echo -e "\n${BLUE}5. Test Suites${NC}"
run_test "Rust unit tests" "cargo test"
run_test "Doc tests" "npm run test:docs"

# 6. Documentation verification
echo -e "\n${BLUE}6. Documentation Verification${NC}"
run_test "Rust docs" "cargo doc --no-deps"

# 7. Security checks
echo -e "\n${BLUE}7. Security Verification${NC}"
if command -v cargo-audit &> /dev/null; then
    run_test "Dependency audit" "cargo audit"
else
    echo -e "${YELLOW}⚠ cargo-audit not installed, skipping dependency audit${NC}"
fi

if command -v cargo-deny &> /dev/null && [ -f deny.toml ]; then
    run_test "License check" "cargo deny check licenses"
else
    echo -e "${YELLOW}⚠ cargo-deny not installed or deny.toml missing, skipping license check${NC}"
fi

# 8. API verification (if server is running)
echo -e "\n${BLUE}8. API Verification${NC}"
if curl -s http://localhost:3033/health > /dev/null 2>&1; then
    run_test "API health check" "curl -sf http://localhost:3033/health"
    run_test "API tests" "./scripts/test-api-hurl.sh"
else
    echo -e "${YELLOW}⚠ Server not running, skipping API tests${NC}"
    echo -e "${YELLOW}  Start with: cargo run --release${NC}"
fi

# 9. Integration verification
echo -e "\n${BLUE}9. Integration Verification${NC}"
run_check "Claude Code config" "[ -f src/integrations/docs/INTEGRATIONS_CLAUDE_CODE.md ]"
run_check "Discord config" "[ -f src/integrations/docs/INTEGRATIONS_DISCORD.md ]"
run_check "GitHub config" "[ -f src/integrations/docs/INTEGRATIONS_GITHUB.md ]"

# 10. Container build verification
echo -e "\n${BLUE}10. Container Build Verification${NC}"
if command -v docker &> /dev/null; then
    echo -n "Checking Docker daemon... "
    if docker info > /dev/null 2>&1; then
        echo -e "${GREEN}✓${NC}"
        
        # Check if Dockerfile exists
        if [ -f Dockerfile ]; then
            run_test "Container build" "docker build -t spiral-core:verify ."
            run_test "Container cleanup" "docker rmi spiral-core:verify"
        else
            echo -e "${YELLOW}⚠ No Dockerfile found, skipping container build${NC}"
        fi
        
        # Check devcontainer
        if [ -d .devcontainer ] && [ -f .devcontainer/devcontainer.json ]; then
            echo -e "${GREEN}✓ Dev container configuration found${NC}"
            # Build dev container
            if [ -f .devcontainer/Dockerfile ]; then
                run_test "Dev container build" "docker build -t spiral-core-devcontainer:verify -f .devcontainer/Dockerfile .devcontainer/"
                run_test "Dev container cleanup" "docker rmi spiral-core-devcontainer:verify"
            else
                echo -e "${YELLOW}⚠ No .devcontainer/Dockerfile found, skipping dev container build${NC}"
            fi
        else
            echo -e "${YELLOW}⚠ No dev container configuration found${NC}"
        fi
    else
        echo -e "${YELLOW}✗ Docker daemon not running${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Docker not installed, skipping container verification${NC}"
fi

# Summary
echo -e "\n${BLUE}=== Verification Summary ===${NC}"
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed! Spiral Core is ready.${NC}"
    echo -e "\nNext steps:"
    echo -e "  1. Start the server: ${YELLOW}cargo run --release${NC}"
    echo -e "  2. Run Discord bot: ${YELLOW}npm run discord${NC}"
    echo -e "  3. View API docs: ${YELLOW}http://localhost:3033/docs${NC}"
else
    echo -e "${RED}✗ Some checks failed. Please review the output above.${NC}"
    exit 1
fi