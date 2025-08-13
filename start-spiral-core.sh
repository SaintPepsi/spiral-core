#!/bin/bash

# 🚀 Spiral Core System Startup Script
# This script starts all Spiral Core services in the correct order

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
LOG_DIR="${SCRIPT_DIR}/logs"
PID_DIR="${SCRIPT_DIR}/pids"

# Create necessary directories
mkdir -p "$LOG_DIR" "$PID_DIR"

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if a service is already running
check_service() {
    local service_name=$1
    local pid_file="${PID_DIR}/${service_name}.pid"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if ps -p "$pid" > /dev/null 2>&1; then
            return 0  # Service is running
        else
            rm "$pid_file"  # Clean up stale PID file
        fi
    fi
    return 1  # Service is not running
}

# Kill existing services
cleanup_services() {
    log_info "Checking for existing services..."
    
    # Kill any existing spiral-core process
    if check_service "spiral-core"; then
        log_warning "Stopping existing Spiral Core service..."
        kill $(cat "${PID_DIR}/spiral-core.pid") 2>/dev/null || true
        rm -f "${PID_DIR}/spiral-core.pid"
        sleep 2
    fi
    
    # Kill any existing Discord bot process
    if check_service "discord-bot"; then
        log_warning "Stopping existing Discord bot..."
        kill $(cat "${PID_DIR}/discord-bot.pid") 2>/dev/null || true
        rm -f "${PID_DIR}/discord-bot.pid"
        sleep 2
    fi
}

# Environment checks
check_environment() {
    log_info "Performing environment checks..."
    
    # Check Rust installation
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is not installed. Please install Rust."
        exit 1
    fi
    
    # Check Node.js installation (for Discord bot if separate)
    if ! command -v node &> /dev/null; then
        log_warning "Node.js is not installed. Discord bot features may be limited."
    fi
    
    # Check for .env file
    if [ ! -f "${SCRIPT_DIR}/.env" ]; then
        log_error ".env file not found. Please create one with required configuration."
        log_info "Required variables: DISCORD_TOKEN, CLAUDE_API_KEY"
        exit 1
    fi
    
    # Source environment variables
    source "${SCRIPT_DIR}/.env"
    
    # Verify critical environment variables
    if [ -z "$DISCORD_TOKEN" ]; then
        log_error "DISCORD_TOKEN not set in .env file"
        exit 1
    fi
    
    log_success "Environment checks passed"
}

# Build the project
build_project() {
    log_info "Building Spiral Core..."
    
    cd "$SCRIPT_DIR"
    
    # Run cargo build in release mode
    if cargo build --release 2>&1 | tee "${LOG_DIR}/build.log"; then
        log_success "Build completed successfully"
    else
        log_error "Build failed. Check ${LOG_DIR}/build.log for details"
        exit 1
    fi
}

# Start the Spiral Core server
start_spiral_core() {
    log_info "Starting Spiral Core server..."
    
    cd "$SCRIPT_DIR"
    
    # Set Rust log level
    export RUST_LOG=${RUST_LOG:-info}
    export RUST_BACKTRACE=${RUST_BACKTRACE:-1}
    
    # Start the server in background
    # Use the built binary directly for better PID tracking
    if [ -f "${SCRIPT_DIR}/target/release/spiral-core" ]; then
        nohup "${SCRIPT_DIR}/target/release/spiral-core" \
            > "${LOG_DIR}/spiral-core.log" 2>&1 &
    else
        # Fallback to cargo run if binary doesn't exist
        nohup cargo run --release --bin spiral-core \
            > "${LOG_DIR}/spiral-core.log" 2>&1 &
    fi
    
    local pid=$!
    echo $pid > "${PID_DIR}/spiral-core.pid"
    
    # Also save the actual process name for verification
    echo "spiral-core" > "${PID_DIR}/spiral-core.name"
    
    # Wait for server to start
    sleep 3
    
    # Check if server started successfully
    if ps -p $pid > /dev/null; then
        log_success "Spiral Core server started (PID: $pid)"
        log_info "Server log: ${LOG_DIR}/spiral-core.log"
    else
        log_error "Failed to start Spiral Core server"
        log_error "Check ${LOG_DIR}/spiral-core.log for details"
        exit 1
    fi
}

# Verify Discord bot integration
verify_discord_bot() {
    log_info "Verifying Discord bot integration..."
    
    # The Discord bot is integrated into spiral-core binary
    # Check the log for Discord connection confirmation
    sleep 3
    
    if grep -q "Discord bot connected" "${LOG_DIR}/spiral-core.log" 2>/dev/null; then
        log_success "Discord bot connection confirmed"
    elif grep -q "DISCORD_TOKEN" "${LOG_DIR}/spiral-core.log" 2>/dev/null; then
        log_warning "Discord bot waiting for valid token"
    else
        log_info "Discord bot status pending (check logs for details)"
    fi
}

# Health check
perform_health_check() {
    log_info "Performing health checks..."
    
    # Check if server is responding (adjust port as needed)
    if curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/health 2>/dev/null | grep -q "200"; then
        log_success "API server is healthy"
    else
        log_warning "API server health check failed or not configured"
    fi
    
    # Check process is still running
    if check_service "spiral-core"; then
        log_success "Spiral Core process is running"
    else
        log_error "Spiral Core process has stopped unexpectedly"
        exit 1
    fi
}

# Show status
show_status() {
    echo ""
    echo "========================================="
    echo "      Spiral Core System Status"
    echo "========================================="
    echo ""
    
    if check_service "spiral-core"; then
        local pid=$(cat "${PID_DIR}/spiral-core.pid")
        echo -e "Spiral Core Server: ${GREEN}● Running${NC} (PID: $pid)"
    else
        echo -e "Spiral Core Server: ${RED}○ Stopped${NC}"
    fi
    
    echo ""
    echo "Logs Directory: ${LOG_DIR}"
    echo "Main Log: ${LOG_DIR}/spiral-core.log"
    echo ""
    
    if [ "$AUTO_TAIL" = false ]; then
        echo "To monitor logs in real-time:"
        echo "  tail -f ${LOG_DIR}/spiral-core.log"
        echo ""
    fi
    
    echo "To stop all services:"
    echo "  ${SCRIPT_DIR}/stop-spiral-core.sh"
    echo ""
}

# Tail logs automatically
tail_logs() {
    echo ""
    echo "📋 Following log output (press Ctrl+C to stop)..."
    echo "========================================="
    echo ""
    
    # Trap Ctrl+C to show a clean exit message
    trap 'echo ""; echo "Log monitoring stopped. Service continues running in background."; exit 0' INT
    
    # Tail the log file
    tail -f "${LOG_DIR}/spiral-core.log"
}

# Main execution
main() {
    # Parse arguments - defaults
    AUTO_TAIL=true
    AUTO_BUILD=true
    
    for arg in "$@"; do
        case "$arg" in
            --no-build)
                AUTO_BUILD=false
                ;;
            --no-tail)
                AUTO_TAIL=false
                ;;
        esac
    done
    
    echo ""
    echo "🌀 Starting Spiral Core System..."
    echo "=================================="
    echo ""
    
    # Clean up any existing services
    cleanup_services
    
    # Run checks
    check_environment
    
    # Build unless explicitly disabled
    if [ "$AUTO_BUILD" = true ]; then
        build_project
    elif [ ! -f "${SCRIPT_DIR}/target/release/spiral-core" ]; then
        log_warning "Binary not found, building anyway..."
        build_project
    else
        log_info "Skipping build (use without --no-build to rebuild)"
    fi
    
    # Start services
    start_spiral_core
    verify_discord_bot
    
    # Perform health check
    sleep 2
    perform_health_check
    
    # Show final status
    show_status
    
    log_success "Spiral Core system started successfully!"
    
    # Auto-tail logs unless disabled
    if [ "$AUTO_TAIL" = true ]; then
        tail_logs
    fi
}

# Handle script arguments
if [[ " $* " == *" --help "* ]] || [[ " $* " == *" -h "* ]]; then
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --no-build  Skip the build step (unless binary is missing)"
    echo "  --no-tail   Don't automatically tail logs after startup"
    echo "  --help      Show this help message"
    echo ""
    echo "By default, the script will:"
    echo "  - Build the project (cargo build --release)"
    echo "  - Start the Spiral Core system"
    echo "  - Automatically tail the logs (press Ctrl+C to stop tailing)"
    echo ""
    echo "Examples:"
    echo "  $0                      # Build, start, and tail logs (default)"
    echo "  $0 --no-build           # Start without building, then tail"
    echo "  $0 --no-tail            # Build and start without tailing"
    echo "  $0 --no-build --no-tail # Start quickly without build or tail"
    echo ""
    exit 0
else
    main "$@"
fi