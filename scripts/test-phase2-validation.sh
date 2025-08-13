#!/bin/bash
# Test script for Phase 2 validation pipeline (Core Rust Compliance Checks)

set -e  # Exit on error

echo "🦀 Running Phase 2 Validation Test Suite (CRCC)"
echo "================================================"
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track overall success
ALL_PASSED=true

# Step 1: Cargo Check (Compilation)
echo -e "${BLUE}📦 Step 1: Cargo Check (Compilation)${NC}"
echo "----------------------------------------"
cargo check --all-targets

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Compilation check passed${NC}"
else
    echo -e "${RED}❌ Compilation check failed${NC}"
    ALL_PASSED=false
fi
echo ""

# Step 2: Cargo Test (Test Execution)
echo -e "${BLUE}🧪 Step 2: Cargo Test (Test Execution)${NC}"
echo "----------------------------------------"
cargo test --quiet

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed${NC}"
else
    echo -e "${RED}❌ Some tests failed${NC}"
    ALL_PASSED=false
fi
echo ""

# Step 3: Cargo Format (Code Formatting)
echo -e "${BLUE}🎨 Step 3: Cargo Format (Code Formatting)${NC}"
echo "----------------------------------------"
cargo fmt -- --check

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Code formatting is correct${NC}"
else
    echo -e "${YELLOW}⚠️  Code needs formatting${NC}"
    echo "Run 'cargo fmt' to fix formatting issues"
    ALL_PASSED=false
fi
echo ""

# Step 4: Cargo Clippy (Linting)
echo -e "${BLUE}🔍 Step 4: Cargo Clippy (Linting)${NC}"
echo "----------------------------------------"
cargo clippy --all-targets -- -D warnings

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ No clippy warnings${NC}"
else
    echo -e "${RED}❌ Clippy found issues${NC}"
    ALL_PASSED=false
fi
echo ""

# Step 5: Cargo Doc (Documentation)
echo -e "${BLUE}📚 Step 5: Cargo Doc (Documentation)${NC}"
echo "----------------------------------------"
cargo doc --no-deps --quiet

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Documentation builds successfully${NC}"
else
    echo -e "${RED}❌ Documentation build failed${NC}"
    ALL_PASSED=false
fi
echo ""

# Optional: Run specific Phase 2 validation tests
echo -e "${BLUE}🔧 Step 6: Phase 2 Validation Unit Tests${NC}"
echo "----------------------------------------"
cargo test --package spiral-core --lib discord::self_update::validation_pipeline::phase2 -- --nocapture 2>/dev/null || {
    echo -e "${YELLOW}ℹ️  No specific Phase 2 unit tests found (this is OK)${NC}"
}
echo ""

# Summary
echo "================================================"
echo -e "${BLUE}📊 Phase 2 Validation Summary (CRCC)${NC}"
echo "================================================"

if [ "$ALL_PASSED" = true ]; then
    echo -e "${GREEN}✅ ALL PHASE 2 CHECKS PASSED!${NC}"
    echo ""
    echo "The codebase meets all Core Rust Compliance Checks:"
    echo "  ✓ Compilation successful"
    echo "  ✓ All tests passing"
    echo "  ✓ Code properly formatted"
    echo "  ✓ No clippy warnings"
    echo "  ✓ Documentation builds"
    exit 0
else
    echo -e "${RED}❌ SOME PHASE 2 CHECKS FAILED${NC}"
    echo ""
    echo "Please fix the issues above before proceeding."
    echo ""
    echo "According to the validation pipeline:"
    echo "  • If ANY Phase 2 check fails, fixes must be applied"
    echo "  • After fixes, the ENTIRE pipeline (Phase 1 + Phase 2) must be re-run"
    echo "  • Maximum 3 complete iterations allowed"
    echo ""
    echo "Quick fix commands:"
    echo "  cargo fmt           # Fix formatting"
    echo "  cargo fix           # Apply suggested fixes"
    echo "  cargo clippy --fix  # Apply clippy suggestions"
    exit 1
fi