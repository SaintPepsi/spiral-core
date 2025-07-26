#!/bin/bash

# Spiral Core API Testing with Hurl
# Usage: ./test-api-hurl.sh [command]
# Available commands: health, status, agents, task, all

set -e  # Exit on any error

# Configuration
TESTS_DIR="src/api/tests/hurl"
ENV_FILE=".env.hurl"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper function to print colored headers
print_header() {
    echo -e "\n${BLUE}=== $1 ===${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Check if hurl is available
check_hurl() {
    if ! command -v hurl &> /dev/null; then
        print_error "Hurl is not installed. Install it from: https://hurl.dev"
        exit 1
    fi
}

# Check if server is running
check_server() {
    print_header "Checking if Spiral Core server is running"
    if curl -s -H "x-api-key: test-api-key-1234567890123456789012345678901234567890" http://localhost:3000/health | grep -q "healthy" > /dev/null 2>&1; then
        print_success "Server is running at http://localhost:3000"
    else
        print_error "Server is not running at http://localhost:3000"
        print_warning "Start the server with: cargo run --bin spiral-core"
        exit 1
    fi
}

# Test health endpoint
test_health() {
    print_header "Testing Health Endpoint"
    if hurl --variables-file "$ENV_FILE" --test "$TESTS_DIR/health.hurl"; then
        print_success "Health tests passed"
    else
        print_error "Health tests failed"
    fi
}

# Test system status
test_status() {
    print_header "Testing System Status"
    if hurl --variables-file "$ENV_FILE" --test "$TESTS_DIR/system.hurl"; then
        print_success "Status tests passed"
    else
        print_error "Status tests failed"
    fi
}

# Test all endpoints
test_all_endpoints() {
    print_header "Testing All API Endpoints"
    if hurl --variables-file "$ENV_FILE" --test "$TESTS_DIR/tasks.hurl"; then
        print_success "All API tests passed"
    else
        print_error "Some API tests failed"
    fi
}

# Run all hurl files
test_all_files() {
    print_header "Running All Hurl Test Files"
    local failed=0
    
    for hurl_file in "$TESTS_DIR"/*.hurl; do
        if [ -f "$hurl_file" ]; then
            echo "Running $(basename "$hurl_file")..."
            if hurl --variables-file "$ENV_FILE" --test "$hurl_file"; then
                print_success "$(basename "$hurl_file") passed"
            else
                print_error "$(basename "$hurl_file") failed"
                failed=1
            fi
        fi
    done
    
    if [ $failed -eq 0 ]; then
        print_success "All test files passed!"
    else
        print_error "Some test files failed"
        exit 1
    fi
}

# Show usage
show_usage() {
    echo "Spiral Core API Testing Script (Hurl Edition)"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  health      Test health endpoint"
    echo "  status      Test system status endpoint"
    echo "  api         Test all endpoints in api.hurl"
    echo "  all         Run all .hurl files (default)"
    echo ""
    echo "Environment Files:"
    echo "  tests/api/hurl.env         - Local development"
    echo "  tests/api/hurl-staging.env - Staging environment"
    echo ""
    echo "Examples:"
    echo "  $0            # Run all tests"
    echo "  $0 health     # Test only health endpoint"
    echo "  $0 api        # Test all endpoints"
    echo ""
    echo "Custom environment:"
    echo "  ENV_FILE=tests/api/hurl-staging.env $0 all"
}

# Main execution
check_hurl

case "${1:-all}" in
    "health")
        check_server
        test_health
        ;;
    "status")
        check_server
        test_status
        ;;
    "api")
        check_server
        test_all_endpoints
        ;;
    "all")
        check_server
        test_all_files
        print_header "All Tests Completed"
        print_success "API testing finished successfully!"
        ;;
    "help"|"-h"|"--help")
        show_usage
        ;;
    *)
        print_error "Unknown command: $1"
        show_usage
        exit 1
        ;;
esac