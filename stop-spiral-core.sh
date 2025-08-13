#!/bin/bash

# 🛑 Spiral Core System Shutdown Script
# This script gracefully stops all Spiral Core services

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PID_DIR="${SCRIPT_DIR}/pids"

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

# Stop a service gracefully
stop_service() {
    local service_name=$1
    local pid_file="${PID_DIR}/${service_name}.pid"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if ps -p "$pid" > /dev/null 2>&1; then
            log_info "Stopping ${service_name} (PID: ${pid})..."
            
            # Send SIGTERM for graceful shutdown
            kill -TERM "$pid" 2>/dev/null || true
            
            # Wait for process to stop (max 10 seconds)
            local count=0
            while [ $count -lt 10 ] && ps -p "$pid" > /dev/null 2>&1; do
                sleep 1
                count=$((count + 1))
            done
            
            # Force kill if still running
            if ps -p "$pid" > /dev/null 2>&1; then
                log_warning "Force stopping ${service_name}..."
                kill -9 "$pid" 2>/dev/null || true
                sleep 1
            fi
            
            log_success "${service_name} stopped"
        else
            log_info "${service_name} is not running (stale PID file)"
        fi
        
        # Remove PID file
        rm -f "$pid_file"
    else
        log_info "${service_name} is not running"
    fi
}

# Show current status
show_status() {
    echo ""
    echo "Current Service Status:"
    echo "-----------------------"
    
    local any_running=false
    
    if [ -f "${PID_DIR}/spiral-core.pid" ]; then
        local pid=$(cat "${PID_DIR}/spiral-core.pid")
        if ps -p "$pid" > /dev/null 2>&1; then
            echo -e "Spiral Core Server: ${GREEN}● Running${NC} (PID: $pid)"
            any_running=true
        else
            echo -e "Spiral Core Server: ${YELLOW}○ Stale PID file${NC}"
        fi
    else
        echo -e "Spiral Core Server: ${RED}○ Stopped${NC}"
    fi
    
    echo ""
    return $([ "$any_running" = true ] && echo 0 || echo 1)
}

# Main execution
main() {
    echo ""
    echo "🛑 Stopping Spiral Core System..."
    echo "=================================="
    echo ""
    
    # Check if PID directory exists
    if [ ! -d "$PID_DIR" ]; then
        log_warning "PID directory not found. Services may not be running."
        exit 0
    fi
    
    # Show current status
    if ! show_status; then
        log_info "No services are currently running"
        exit 0
    fi
    
    # Stop all services
    stop_service "spiral-core"
    stop_service "discord-bot"  # In case it becomes separate
    
    # Clean up any stray processes (optional)
    log_info "Checking for stray processes..."
    pkill -f "spiral-core" 2>/dev/null || true
    
    # Final status
    echo ""
    log_success "All Spiral Core services have been stopped"
    echo ""
    echo "To restart the system, run:"
    echo "  ${SCRIPT_DIR}/start-spiral-core.sh"
    echo ""
}

# Handle script arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [OPTIONS]"
        echo ""
        echo "Options:"
        echo "  --force    Force kill all processes immediately"
        echo "  --help     Show this help message"
        echo ""
        echo "This script gracefully stops all Spiral Core services."
        echo ""
        exit 0
        ;;
    --force)
        log_warning "Force stopping all services..."
        pkill -9 -f "spiral-core" 2>/dev/null || true
        rm -f "${PID_DIR}"/*.pid 2>/dev/null || true
        log_success "All services force stopped"
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac