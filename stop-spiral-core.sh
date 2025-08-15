#!/bin/bash

# 🛑 Simplified Spiral Core Stop Script
# 🏗️ ARCHITECTURE DECISION: Direct process management over PID files
# Why: PID files become stale, processes may not update them correctly
# Alternative: Complex PID tracking (rejected: unreliable, adds complexity)
# Trade-off: Less precise targeting but more reliable stopping
# Audit: Verify no other processes match "spiral-core" pattern

# 🎨 UX DECISION: Color-coded output for clarity
# Why: Visual feedback helps users understand script progress
# Alternative: Plain text (rejected: harder to scan)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo ""
echo "🛑 Stopping Spiral Core..."
echo ""

# 🔍 AUDIT CHECKPOINT: Process discovery phase
# Critical: Must find ALL spiral-core processes
# Security: Using pgrep prevents command injection vs ps|grep
FOUND_PROCESS=false

# 🛡️ SECURITY DECISION: SIGTERM before SIGKILL
# Why: Graceful shutdown allows cleanup (file handles, network connections)
# Alternative: Immediate SIGKILL (rejected: can corrupt state)
# Risk: Process might ignore SIGTERM
# Mitigation: Escalate to SIGKILL after timeout
if pgrep -f "spiral-core" > /dev/null 2>&1; then
    echo -e "${YELLOW}Found spiral-core processes, stopping...${NC}"
    pkill -TERM -f "spiral-core" 2>/dev/null || true
    FOUND_PROCESS=true
    
    # ⏱️ PERFORMANCE DECISION: 2-second grace period
    # Why: Balance between quick stop and allowing cleanup
    # Alternative: Longer wait (rejected: users want quick feedback)
    # Trade-off: May not be enough for large state cleanup
    sleep 2
    
    # 🔧 ERROR RECOVERY: Escalation to SIGKILL
    # Why: Ensure process stops even if hung
    # Alternative: Leave hung processes (rejected: defeats script purpose)
    if pgrep -f "spiral-core" > /dev/null 2>&1; then
        echo -e "${YELLOW}Force stopping remaining processes...${NC}"
        pkill -9 -f "spiral-core" 2>/dev/null || true
        sleep 1
    fi
fi

# 🏗️ ARCHITECTURE DECISION: Also stop development builds
# Why: Developers often use `cargo run` instead of compiled binary
# Alternative: Only stop release builds (rejected: misses dev processes)
# Trade-off: Might stop unrelated cargo processes with "spiral" in name
# Audit: Check pattern specificity to avoid false positives
if pgrep -f "cargo.*run.*spiral" > /dev/null 2>&1; then
    echo -e "${YELLOW}Found cargo run processes, stopping...${NC}"
    pkill -TERM -f "cargo.*run.*spiral" 2>/dev/null || true
    FOUND_PROCESS=true
    sleep 2
fi

# 🧹 CLEANUP DECISION: Remove stale PID files
# Why: Old script created PID files that may be orphaned
# Alternative: Keep for debugging (rejected: causes confusion)
# Risk: None - PID files are recreated on start
if [ -d "pids" ]; then
    rm -f pids/*.pid 2>/dev/null || true
    echo -e "${GREEN}Cleaned up PID files${NC}"
fi

# Final status
echo ""
if [ "$FOUND_PROCESS" = true ]; then
    echo -e "${GREEN}✓ All Spiral Core processes stopped${NC}"
else
    echo -e "${GREEN}✓ No Spiral Core processes were running${NC}"
fi

echo ""
echo "To start Spiral Core, run:"
echo "  ./start-spiral-core.sh"
echo ""