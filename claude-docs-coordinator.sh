#!/bin/bash
# claude-docs-coordinator.sh
# Coordination system for Claude documentation operations using file locks
# Key Features Implemented:

#   🔒 File Locking System - Prevents conflicts between concurrent operations
#   📊 Status Monitoring - Real-time visibility into coordinator state
#   🚀 Safe Execution - Wraps all operations with proper lock management
#   ⚡ Watcher Management - Start/stop/restart documentation file watcher
#   🛠️ Manual Operations - Coordinate manual Claude tasks with temporary watcher pausing

#   Available Commands:
#   ./claude-docs-coordinator.sh start-watcher    # Start automated watcher
#   ./claude-docs-coordinator.sh stop-watcher     # Stop watcher
#   ./claude-docs-coordinator.sh status           # Show current status
#   ./claude-docs-coordinator.sh manual "task-name" "claude-code --prompt '...'"
#   ./claude-docs-coordinator.sh cleanup          # Emergency cleanup

set -euo pipefail

# Configuration
LOCK_FILE="/tmp/claude-docs.lock"
PID_FILE="/tmp/claude-docs.pid"
STATE_FILE="/tmp/claude-docs-state"
LOG_FILE="/tmp/claude-docs.log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] [$level] $message" | tee -a "$LOG_FILE"
}

# Lock management functions
acquire_lock() {
    local operation="$1"
    local timeout="${2:-30}"
    
    exec 200>"$LOCK_FILE"
    
    if flock -w "$timeout" 200; then
        echo $$ > "$PID_FILE"
        echo "$operation:$(date +%s):$$" > "$STATE_FILE"
        log "INFO" "🔒 Acquired lock for: $operation (PID: $$)"
        return 0
    else
        log "ERROR" "❌ Failed to acquire lock within ${timeout}s for: $operation"
        return 1
    fi
}

release_lock() {
    if [ -f "$PID_FILE" ] && [ "$(cat "$PID_FILE")" = "$$" ]; then
        echo "idle:$(date +%s):0" > "$STATE_FILE"
        rm -f "$PID_FILE"
        flock -u 200 2>/dev/null || true
        log "INFO" "🔓 Released lock (PID: $$)"
    fi
}

# State management functions
get_current_state() {
    if [ -f "$STATE_FILE" ]; then
        cut -d: -f1 "$STATE_FILE"
    else
        echo "idle"
    fi
}

get_lock_owner_pid() {
    if [ -f "$STATE_FILE" ]; then
        cut -d: -f3 "$STATE_FILE"
    else
        echo "0"
    fi
}

is_process_running() {
    local pid=$(get_lock_owner_pid)
    [ "$pid" != "0" ] && kill -0 "$pid" 2>/dev/null
}

# Cleanup function
cleanup() {
    log "INFO" "🧹 Cleaning up coordinator (PID: $$)"
    release_lock
    exit 0
}

# Signal handlers
trap cleanup EXIT
trap cleanup SIGTERM
trap cleanup SIGINT

# Check if another coordinator is already running
check_existing_coordinator() {
    if [ -f "$PID_FILE" ]; then
        local existing_pid=$(cat "$PID_FILE")
        if kill -0 "$existing_pid" 2>/dev/null; then
            local state=$(get_current_state)
            log "WARN" "⚠️  Another coordinator is running (PID: $existing_pid, State: $state)"
            return 1
        else
            log "INFO" "🧹 Cleaning up stale lock files"
            rm -f "$PID_FILE" "$LOCK_FILE" "$STATE_FILE"
        fi
    fi
    return 0
}

# Safe execution wrapper
safe_execute() {
    local operation="$1"
    local command="$2"
    local timeout="${3:-30}"
    
    if acquire_lock "$operation" "$timeout"; then
        log "INFO" "🚀 Executing: $operation"
        
        # Execute the command and capture exit code
        local exit_code=0
        eval "$command" || exit_code=$?
        
        if [ $exit_code -eq 0 ]; then
            log "INFO" "✅ Completed successfully: $operation"
        else
            log "ERROR" "❌ Failed with exit code $exit_code: $operation"
        fi
        
        release_lock
        return $exit_code
    else
        log "ERROR" "❌ Could not acquire lock for: $operation"
        return 1
    fi
}

# Status reporting
show_status() {
    echo -e "${BLUE}📊 Claude Documentation Coordinator Status${NC}"
    echo "----------------------------------------"
    
    local state=$(get_current_state)
    local pid=$(get_lock_owner_pid)
    
    echo -e "Current State: ${GREEN}$state${NC}"
    
    if [ "$state" != "idle" ] && is_process_running; then
        echo -e "Lock Owner PID: ${YELLOW}$pid${NC}"
        echo -e "Status: ${YELLOW}ACTIVE${NC}"
    else
        echo -e "Status: ${GREEN}IDLE${NC}"
    fi
    
    if [ -f "$LOCK_FILE" ]; then
        local lock_time=$(stat -f "%Sm" -t "%Y-%m-%d %H:%M:%S" "$LOCK_FILE" 2>/dev/null || stat -c "%y" "$LOCK_FILE" 2>/dev/null || echo "Unknown")
        echo -e "Lock File Created: ${BLUE}$lock_time${NC}"
    fi
    
    echo ""
    echo -e "${BLUE}Recent Log Entries:${NC}"
    if [ -f "$LOG_FILE" ]; then
        tail -5 "$LOG_FILE" | while read -r line; do
            echo "  $line"
        done
    else
        echo "  No log entries found"
    fi
}

# Force cleanup (emergency use)
force_cleanup() {
    echo -e "${RED}🚨 Force cleaning up all lock files...${NC}"
    rm -f "$LOCK_FILE" "$PID_FILE" "$STATE_FILE"
    log "WARN" "🚨 Force cleanup executed"
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Main coordination functions
start_watcher() {
    if ! check_existing_coordinator; then
        echo -e "${RED}❌ Cannot start watcher - another coordinator is running${NC}"
        return 1
    fi
    
    log "INFO" "🎯 Starting Claude documentation watcher"
    
    # Enhanced chokidar command with feedback loop prevention
    chokidar "CLAUDE*.md" --ignore-initial --debounce 10000 -c "
        source '$(pwd)/claude-docs-coordinator.sh'
        
        # Check if we're in a feedback loop cooldown
        COOLDOWN_FILE=\"/tmp/claude-docs-cooldown\"
        if [ -f \"\$COOLDOWN_FILE\" ]; then
            COOLDOWN_END=\$(cat \"\$COOLDOWN_FILE\")
            CURRENT_TIME=\$(date +%s)
            if [ \$CURRENT_TIME -lt \$COOLDOWN_END ]; then
                echo \"🔄 In cooldown period, skipping validation to prevent feedback loop\"
                exit 0
            else
                rm -f \"\$COOLDOWN_FILE\"
            fi
        fi
        
        if safe_execute 'documentation-validation' '
            echo \"🔍 Validating documentation consistency (READ-ONLY)...\"
            
            # Set cooldown period BEFORE running Claude to prevent feedback loops
            COOLDOWN_END=\$(($(date +%s) + 300))  # 5 minute cooldown
            echo \$COOLDOWN_END > \"/tmp/claude-docs-cooldown\"
            
            claude-code --prompt \"CLAUDE Documentation Consistency Validator - READ-ONLY ANALYSIS

⚠️  CRITICAL: This is a READ-ONLY validation task. DO NOT modify any files. DO NOT use Edit, Write, or MultiEdit tools.

TRIGGER: A CLAUDE*.md file was just modified
TASK: Analyze documentation ecosystem for consistency issues

DISCOVERY PHASE:
1. Use Read tool to scan ALL files matching CLAUDE*.md pattern
2. Use Bash tool to identify the most recently changed file (git status or ls -lt)
3. Map the relationships between all CLAUDE documentation files
4. Note the last modified timestamps of each file

ANALYSIS PHASE (READ-ONLY):
5. Compare content across all CLAUDE*.md files for:
   - Architecture descriptions and diagrams
   - API references and method signatures
   - Installation/setup instructions
   - Configuration examples and parameters
   - Process workflows and sequences
   - Terminology and naming conventions
   - Version numbers and compatibility info
   - Cross-references and internal links

VALIDATION CHECKS (READ-ONLY):
6. Verify all internal links between CLAUDE files still work
7. Check for orphaned references to moved/renamed sections
8. Identify outdated information that conflicts between files
9. Spot missing updates (e.g., new features in one doc but not others)
10. Flag inconsistent code examples or API usage patterns

OUTPUT REQUIREMENTS:
- If ALL files are consistent: Return exactly \\\"✅ CLAUDE DOCS CONSISTENT\\\"
- If issues found: Provide a REPORT ONLY with:
  * Which files have inconsistencies
  * What sections need attention
  * What the inconsistencies are
  * Priority level (critical/minor) for each issue

🚫 DO NOT FIX ANYTHING - ONLY REPORT WHAT YOU FIND
🚫 DO NOT USE Edit, Write, MultiEdit, or any modification tools
🚫 This prevents infinite feedback loops with the file watcher

CONTEXT: Automated validation to detect documentation drift across a multi-file CLAUDE documentation system.\"
        ' 120; then
            echo \"✅ Documentation validation completed successfully\"
            # Remove cooldown if validation succeeds without modifications
            rm -f \"/tmp/claude-docs-cooldown\" 2>/dev/null || true
        else
            echo \"⚠️ Documentation validation failed or timed out\"
            # Keep cooldown in place if there were issues
        fi
    " &
    
    local watcher_pid=$!
    echo "$watcher_pid" > "/tmp/claude-docs-watcher.pid"
    
    echo -e "${GREEN}✅ Documentation watcher started (PID: $watcher_pid)${NC}"
    log "INFO" "📡 Watcher started with PID: $watcher_pid"
}

stop_watcher() {
    if [ -f "/tmp/claude-docs-watcher.pid" ]; then
        local watcher_pid=$(cat "/tmp/claude-docs-watcher.pid")
        if kill -0 "$watcher_pid" 2>/dev/null; then
            echo -e "${YELLOW}⏹️  Stopping documentation watcher (PID: $watcher_pid)${NC}"
            kill "$watcher_pid"
            rm -f "/tmp/claude-docs-watcher.pid"
            log "INFO" "⏹️ Watcher stopped"
            echo -e "${GREEN}✅ Watcher stopped${NC}"
        else
            echo -e "${YELLOW}⚠️  Watcher PID file exists but process not running${NC}"
            rm -f "/tmp/claude-docs-watcher.pid"
        fi
    else
        echo -e "${YELLOW}⚠️  No watcher PID file found${NC}"
    fi
    
    # Also cleanup any lingering chokidar processes
    pkill -f "chokidar.*CLAUDE" 2>/dev/null || true
}

restart_watcher() {
    echo -e "${BLUE}🔄 Restarting documentation watcher${NC}"
    stop_watcher
    sleep 2
    start_watcher
}

# Manual operation wrapper
manual_operation() {
    local operation_name="$1"
    shift
    local command="$*"
    
    echo -e "${BLUE}🚀 Starting manual operation: $operation_name${NC}"
    
    # Check if watcher is running and stop it temporarily
    local watcher_was_running=false
    if [ -f "/tmp/claude-docs-watcher.pid" ]; then
        local watcher_pid=$(cat "/tmp/claude-docs-watcher.pid")
        if kill -0 "$watcher_pid" 2>/dev/null; then
            watcher_was_running=true
            echo -e "${YELLOW}⏸️  Temporarily stopping watcher for manual operation${NC}"
            stop_watcher
        fi
    fi
    
    # Execute the manual operation
    if safe_execute "$operation_name" "$command" 60; then
        echo -e "${GREEN}✅ Manual operation completed successfully${NC}"
        local exit_code=0
    else
        echo -e "${RED}❌ Manual operation failed${NC}"
        local exit_code=1
    fi
    
    # Restart watcher if it was running
    if [ "$watcher_was_running" = true ]; then
        echo -e "${BLUE}▶️  Restarting documentation watcher${NC}"
        sleep 2
        start_watcher
    fi
    
    return $exit_code
}

# Help function
show_help() {
    cat << EOF
${BLUE}Claude Documentation Coordinator${NC}
==================================

USAGE:
    $0 <command> [arguments]

COMMANDS:
    start-watcher       Start the documentation file watcher
    stop-watcher        Stop the documentation file watcher  
    restart-watcher     Restart the documentation file watcher
    status             Show current coordinator status
    manual <name> <cmd> Execute a manual operation with coordination
    cleanup            Force cleanup of all lock files (emergency use)
    help               Show this help message

EXAMPLES:
    $0 start-watcher
    $0 status
    $0 manual "fix-links" "claude-code --prompt 'Fix broken links in docs'"
    $0 stop-watcher

COORDINATION:
    This script ensures safe coordination between:
    - Automated documentation validation (chokidar watcher)
    - Manual Claude operations
    - Multiple concurrent processes

LOCK FILES:
    - Lock: $LOCK_FILE
    - PID:  $PID_FILE  
    - State: $STATE_FILE
    - Log:  $LOG_FILE

EOF
}

# Main command dispatcher
main() {
    # Create log file if it doesn't exist
    touch "$LOG_FILE"
    
    case "${1:-help}" in
        "start-watcher"|"start")
            start_watcher
            ;;
        "stop-watcher"|"stop")
            stop_watcher
            ;;
        "restart-watcher"|"restart")
            restart_watcher
            ;;
        "status")
            show_status
            ;;
        "manual")
            if [ $# -lt 3 ]; then
                echo -e "${RED}❌ Usage: $0 manual <operation_name> <command>${NC}"
                exit 1
            fi
            manual_operation "$2" "${@:3}"
            ;;
        "cleanup")
            force_cleanup
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            echo -e "${RED}❌ Unknown command: $1${NC}"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# Only run main if script is executed directly (not sourced)
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    main "$@"
fi