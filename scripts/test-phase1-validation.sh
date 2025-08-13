#!/bin/bash
# Test script for Phase 1 validation (Advanced Quality Assurance)

echo "🧪 Running Phase 1 Validation Tests (AQA)"
echo "========================================="
echo ""
echo "Phase 1: Advanced Quality Assurance"
echo "  • Code Review & Standards"
echo "  • Comprehensive Testing"
echo "  • Security Audit"
echo "  • System Integration"
echo ""

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

ALL_PASSED=true

# Test 1: Run pre-validation tests
echo -e "${BLUE}📝 Test 1: Pre-Validation Module Tests${NC}"
echo "----------------------------------------"
cargo test --package spiral-core --lib discord::self_update::pre_validation::tests -- --nocapture

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Pre-validation tests passed${NC}"
else
    echo -e "${RED}❌ Pre-validation tests failed${NC}"
    ALL_PASSED=false
fi
echo ""

# Test 2: Run pipeline tests (includes Phase 1 components)
echo -e "${BLUE}🔄 Test 2: Pipeline Tests${NC}"
echo "----------------------------------------"
cargo test --package spiral-core --lib discord::self_update::tests::pipeline_tests -- --nocapture

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Pipeline tests passed${NC}"
else
    echo -e "${RED}❌ Pipeline tests failed${NC}"
    ALL_PASSED=false
fi
echo ""

# Test 3: Check if validation agents exist
echo -e "${BLUE}🤖 Test 3: Validation Agent Check${NC}"
echo "----------------------------------------"
if [ -d ".claude/validation-agents/phase1" ]; then
    echo "Found Phase 1 validation agents:"
    ls -1 .claude/validation-agents/phase1/*.md 2>/dev/null | while read agent; do
        echo "  ✓ $(basename $agent)"
    done
    echo -e "${GREEN}✅ Phase 1 agents present${NC}"
else
    echo -e "${RED}❌ Phase 1 validation agents directory not found${NC}"
    ALL_PASSED=false
fi

echo ""
echo "💡 To run Phase 1 validation with real Claude integration:"
echo ""
echo "  cargo test --ignored -- --test-threads=1"
echo ""
echo "================================================"
echo -e "${BLUE}📊 Phase 1 Validation Summary (AQA)${NC}"
echo "================================================"

if [ "$ALL_PASSED" = true ]; then
    echo -e "${GREEN}✅ ALL PHASE 1 TESTS PASSED!${NC}"
    echo ""
    echo "Phase 1 (Advanced Quality Assurance) validated:"
    echo "  ✓ Pre-validation module tests"
    echo "  ✓ Pipeline tests"
    echo "  ✓ Validation agents present"
    exit 0
else
    echo -e "${RED}❌ SOME PHASE 1 TESTS FAILED${NC}"
    echo ""
    echo "Please fix the issues above before proceeding."
    echo ""
    echo "Note: Phase 1 focuses on:"
    echo "  • Code review and standards compliance"
    echo "  • Comprehensive testing coverage"
    echo "  • Security audit checks"
    echo "  • System integration validation"
    exit 1
fi