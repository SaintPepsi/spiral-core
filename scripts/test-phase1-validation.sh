#!/bin/bash
# Simple test script for Phase 1 validation

echo "🧪 Running Phase 1 Validation Tests"
echo "===================================="
echo ""

# Just compile the tests first to check for errors
echo "📦 Compiling tests..."
cargo test --package spiral-core --lib discord::self_update::pre_validation_tests --no-run

if [ $? -ne 0 ]; then
    echo "❌ Compilation failed. Please fix the errors above."
    exit 1
fi

echo "✅ Tests compiled successfully!"
echo ""

# Run the actual tests
echo "🏃 Running tests..."
cargo test --package spiral-core --lib discord::self_update::pre_validation_tests -- --nocapture

echo ""
echo "📊 Test Summary"
echo "---------------"
echo "Phase 1 validation tests have been executed."
echo ""
echo "To test with real Claude integration:"
echo "1. Ensure Claude binary is installed"
echo "2. Run: cargo test --package spiral-core --lib discord::self_update::pre_validation_tests::integration_tests::test_real_claude_integration -- --ignored --nocapture"