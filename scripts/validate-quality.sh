#!/bin/bash
# 🏗️ ARCHITECTURE DECISION: Code quality validation script
# Why: Single source of truth for build, test, lint, and format validation
# Alternative: Run commands individually (rejected: error-prone, inconsistent)
# Usage: ./scripts/validate-quality.sh or make validate

set -e  # Exit on first error

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          🔍 Spiral Core - Quality Validation                 ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Track failures
FAILED_CHECKS=""

# Function to run a check and capture result
run_check() {
    local name="$1"
    local command="$2"
    
    echo "▶ $name..."
    if eval "$command" > /dev/null 2>&1; then
        echo "  ✅ $name passed"
    else
        echo "  ❌ $name FAILED"
        FAILED_CHECKS="$FAILED_CHECKS\n  - $name"
        return 1
    fi
}

# Run all checks (don't exit on failure, collect all results)
set +e

echo "📦 Build Checks"
echo "─────────────────"
run_check "Library build" "cargo build --lib"
run_check "Binary build" "cargo build --bin spiral-core"
echo ""

echo "🧪 Test Suite"
echo "─────────────"
run_check "Unit tests" "cargo test --lib"
run_check "Integration tests" "cargo test --test '*'"
echo ""

echo "🎨 Code Quality"
echo "───────────────"
run_check "Format check" "cargo fmt -- --check"
run_check "Clippy lints" "cargo clippy --all-targets -- -D warnings"
echo ""

echo "📋 Documentation"
echo "────────────────"
run_check "Doc generation" "cargo doc --no-deps"
echo ""

# Summary
echo "╔══════════════════════════════════════════════════════════════╗"
if [ -z "$FAILED_CHECKS" ]; then
    echo "║                    ✅ ALL CHECKS PASSED!                     ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    exit 0
else
    echo "║                    ❌ SOME CHECKS FAILED                     ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "Failed checks:$FAILED_CHECKS"
    exit 1
fi