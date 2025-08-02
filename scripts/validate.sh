#!/bin/bash
# 🚨 VALIDATION SCRIPT: Run all checks before declaring completion
# This script ensures code quality standards are met

set -e  # Exit on any error

echo "🔍 Running Spiral Core Validation Suite..."
echo "========================================"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track overall status
FAILED=0

# Function to run a command and check status
run_check() {
    local name=$1
    local cmd=$2
    
    echo -e "\n📋 Running: $name"
    echo "Command: $cmd"
    
    if eval $cmd; then
        echo -e "${GREEN}✅ $name passed${NC}"
    else
        echo -e "${RED}❌ $name FAILED${NC}"
        FAILED=1
    fi
}

# Run all validation checks
run_check "Cargo Check" "cargo check --all-targets"
run_check "Cargo Test" "cargo test"
run_check "Cargo Format" "cargo fmt -- --check"
run_check "Cargo Clippy" "cargo clippy --all-targets -- -D warnings"
run_check "Documentation" "cargo doc --no-deps --quiet"

echo -e "\n========================================"

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All validation checks passed!${NC}"
    echo "The code is ready for completion."
    exit 0
else
    echo -e "${RED}❌ Some checks failed!${NC}"
    echo "Please fix the issues before declaring the task complete."
    exit 1
fi