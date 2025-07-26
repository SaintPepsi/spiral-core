#!/bin/bash

# Spiral Core Hurl Test Runner
# Runs HTTP API tests using Hurl against a running server instance

set -euo pipefail

# Configuration
DEFAULT_BASE_URL="http://127.0.0.1:3000"
DEFAULT_API_KEY="test-api-key-1234567890123456789012345678901234567890"
INVALID_API_KEY="invalid-key-short"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Usage information
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -u, --url URL        Base URL for API server (default: $DEFAULT_BASE_URL)"
    echo "  -k, --key KEY        API key for authentication (default: test key)"
    echo "  -t, --test FILE      Run specific test file"
    echo "  -v, --verbose        Run with verbose output"
    echo "  -h, --help          Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  BASE_URL            Override default base URL"
    echo "  API_KEY             Override default API key"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Run all tests with defaults"
    echo "  $0 -u http://localhost:8080          # Run against different port"
    echo "  $0 -t health.hurl                    # Run only health tests"
    echo "  $0 -v                                # Run with verbose output"
}

# Parse command line arguments
BASE_URL="${BASE_URL:-$DEFAULT_BASE_URL}"
API_KEY="${API_KEY:-$DEFAULT_API_KEY}"
VERBOSE=""
SPECIFIC_TEST=""
HURL_OPTIONS=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -u|--url)
            BASE_URL="$2"
            shift 2
            ;;
        -k|--key)
            API_KEY="$2"
            shift 2
            ;;
        -t|--test)
            SPECIFIC_TEST="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE="--verbose"
            HURL_OPTIONS="$HURL_OPTIONS --verbose"
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Check if hurl is installed
if ! command -v hurl &> /dev/null; then
    echo -e "${RED}Error: hurl is not installed${NC}"
    echo "Install hurl from: https://hurl.dev/docs/installation.html"
    exit 1
fi

# Find the project root (directory containing Cargo.toml)
PROJECT_ROOT=""
CURRENT_DIR="$(pwd)"
while [[ "$CURRENT_DIR" != "/" ]]; do
    if [[ -f "$CURRENT_DIR/Cargo.toml" ]]; then
        PROJECT_ROOT="$CURRENT_DIR"
        break
    fi
    CURRENT_DIR="$(dirname "$CURRENT_DIR")"
done

if [[ -z "$PROJECT_ROOT" ]]; then
    echo -e "${RED}Error: Could not find project root (no Cargo.toml found)${NC}"
    exit 1
fi

# Set test directory
TEST_DIR="$PROJECT_ROOT/src/api/tests/hurl"

if [[ ! -d "$TEST_DIR" ]]; then
    echo -e "${RED}Error: Test directory not found: $TEST_DIR${NC}"
    exit 1
fi

echo -e "${GREEN}Spiral Core Hurl Test Runner${NC}"
echo "Project Root: $PROJECT_ROOT"
echo "Test Directory: $TEST_DIR"
echo "Base URL: $BASE_URL"
echo "API Key: ${API_KEY:0:10}..." # Show only first 10 chars for security
echo ""

# Check if server is running
echo -e "${YELLOW}Checking server connectivity...${NC}"
if ! curl -s --max-time 5 "$BASE_URL/health" > /dev/null 2>&1; then
    echo -e "${RED}Error: Server is not responding at $BASE_URL${NC}"
    echo "Make sure the Spiral Core server is running:"
    echo "  cargo run --bin spiral-core"
    echo "Or start it in the background:"
    echo "  cargo run --bin spiral-core &"
    exit 1
fi
echo -e "${GREEN}✓ Server is responding${NC}"
echo ""

# Prepare hurl variables
HURL_VARS="--variable base_url=$BASE_URL"
HURL_VARS="$HURL_VARS --variable api_key=$API_KEY"
HURL_VARS="$HURL_VARS --variable invalid_key=$INVALID_API_KEY"

# Function to run a single test file
run_test() {
    local test_file="$1"
    local test_name="$(basename "$test_file" .hurl)"
    
    echo -e "${YELLOW}Running $test_name tests...${NC}"
    
    if hurl $HURL_OPTIONS $HURL_VARS "$test_file"; then
        echo -e "${GREEN}✓ $test_name tests passed${NC}"
        return 0
    else
        echo -e "${RED}✗ $test_name tests failed${NC}"
        return 1
    fi
}

# Run tests
cd "$TEST_DIR"

if [[ -n "$SPECIFIC_TEST" ]]; then
    # Run specific test file
    if [[ -f "$SPECIFIC_TEST" ]]; then
        run_test "$SPECIFIC_TEST"
        exit $?
    elif [[ -f "$SPECIFIC_TEST.hurl" ]]; then
        run_test "$SPECIFIC_TEST.hurl"
        exit $?
    else
        echo -e "${RED}Error: Test file not found: $SPECIFIC_TEST${NC}"
        exit 1
    fi
else
    # Run all test files in recommended order
    TEST_FILES=(
        "auth.hurl"      # Authentication first
        "health.hurl"    # Basic health check
        "system.hurl"    # System status
        "agents.hurl"    # Agent endpoints  
        "tasks.hurl"     # Task endpoints (most complex)
        "cors.hurl"      # CORS policy
    )
    
    FAILED_TESTS=()
    PASSED_TESTS=()
    
    for test_file in "${TEST_FILES[@]}"; do
        if [[ -f "$test_file" ]]; then
            if run_test "$test_file"; then
                PASSED_TESTS+=("$test_file")
            else
                FAILED_TESTS+=("$test_file")
            fi
            echo ""
        else
            echo -e "${YELLOW}Warning: Test file not found: $test_file${NC}"
        fi
    done
    
    # Summary
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}Test Summary${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo "Passed: ${#PASSED_TESTS[@]}"
    echo "Failed: ${#FAILED_TESTS[@]}"
    echo ""
    
    if [[ ${#PASSED_TESTS[@]} -gt 0 ]]; then
        echo -e "${GREEN}Passed tests:${NC}"
        for test in "${PASSED_TESTS[@]}"; do
            echo -e "  ${GREEN}✓${NC} $test"
        done
    fi
    
    if [[ ${#FAILED_TESTS[@]} -gt 0 ]]; then
        echo ""
        echo -e "${RED}Failed tests:${NC}"
        for test in "${FAILED_TESTS[@]}"; do
            echo -e "  ${RED}✗${NC} $test"
        done
        echo ""
        echo -e "${RED}Some tests failed. Check the output above for details.${NC}"
        exit 1
    fi
    
    echo ""
    echo -e "${GREEN}All tests passed! 🎉${NC}"
fi