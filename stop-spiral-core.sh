#!/bin/bash

# 🛑 Comprehensive Spiral Core Stop Script
# 🏗️ ARCHITECTURE DECISION: Multiple search patterns for resilience
# Why: Bot can be started in various ways (cargo run, direct binary, etc)
# Alternative: Single pattern (rejected: misses edge cases)
# Trade-off: Slightly slower but more reliable

# 🎨 UX DECISION: Color-coded output for clarity
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo ""
echo "🛑 Stopping Spiral Core..."
echo ""

# Track if we found any processes
FOUND_PROCESS=false

# 🔍 AUDIT CHECKPOINT: Check multiple process patterns
echo -e "${BLUE}Searching for Spiral Core processes...${NC}"

# Pattern 1: Direct binary execution (debug or release)
if pgrep -f "target/(debug|release)/spiral-core" > /dev/null 2>&1; then
    echo -e "${YELLOW}Found spiral-core binary processes${NC}"
    pkill -TERM -f "target/(debug|release)/spiral-core" 2>/dev/null || true
    FOUND_PROCESS=true
fi

# Pattern 2: Simplified name match
if pgrep -f "spiral-core" > /dev/null 2>&1; then
    echo -e "${YELLOW}Found spiral-core processes${NC}"
    pkill -TERM -f "spiral-core" 2>/dev/null || true
    FOUND_PROCESS=true
fi

# Pattern 3: Cargo run commands
if pgrep -f "cargo.*run.*spiral" > /dev/null 2>&1; then
    echo -e "${YELLOW}Found cargo run processes${NC}"
    pkill -TERM -f "cargo.*run.*spiral" 2>/dev/null || true
    FOUND_PROCESS=true
fi

# Pattern 4: Check by binary name only (in case it's in PATH)
if pgrep -x "spiral-core" > /dev/null 2>&1; then
    echo -e "${YELLOW}Found spiral-core by exact name${NC}"
    pkill -TERM -x "spiral-core" 2>/dev/null || true
    FOUND_PROCESS=true
fi

# Pattern 5: Check for processes listening on port 8080 (API port)
# 🏗️ ARCHITECTURE DECISION: Also check by port binding
# Why: Process might have unexpected name but we know it uses port 8080
SPIRAL_PID=$(lsof -ti :8080 2>/dev/null)
if [ ! -z "$SPIRAL_PID" ]; then
    echo -e "${YELLOW}Found process on port 8080 (PID: $SPIRAL_PID)${NC}"
    # Check if it's actually our process
    PROCESS_NAME=$(ps -p $SPIRAL_PID -o comm= 2>/dev/null || echo "")
    if [[ "$PROCESS_NAME" == *"spiral"* ]] || [[ "$PROCESS_NAME" == *"cargo"* ]]; then
        echo -e "${YELLOW}Stopping Spiral Core on port 8080${NC}"
        kill -TERM $SPIRAL_PID 2>/dev/null || true
        FOUND_PROCESS=true
    else
        echo -e "${BLUE}Process on port 8080 is not Spiral Core: $PROCESS_NAME${NC}"
    fi
fi

# Give processes time to terminate gracefully
if [ "$FOUND_PROCESS" = true ]; then
    echo -e "${BLUE}Waiting for graceful shutdown...${NC}"
    sleep 2
    
    # 🔧 ERROR RECOVERY: Force kill if still running
    if pgrep -f "spiral-core" > /dev/null 2>&1 || \
       pgrep -f "cargo.*run.*spiral" > /dev/null 2>&1 || \
       pgrep -f "target/(debug|release)/spiral-core" > /dev/null 2>&1; then
        echo -e "${YELLOW}Force stopping remaining processes...${NC}"
        pkill -9 -f "spiral-core" 2>/dev/null || true
        pkill -9 -f "cargo.*run.*spiral" 2>/dev/null || true
        pkill -9 -f "target/(debug|release)/spiral-core" 2>/dev/null || true
        sleep 1
    fi
fi

# 🧹 CLEANUP: Remove stale PID files
if [ -d "pids" ]; then
    rm -f pids/*.pid 2>/dev/null || true
    echo -e "${GREEN}Cleaned up PID files${NC}"
fi

# 📊 Final verification
echo ""
echo -e "${BLUE}Verifying shutdown...${NC}"

# Check if anything is still running
STILL_RUNNING=false
if pgrep -f "spiral-core" > /dev/null 2>&1; then
    STILL_RUNNING=true
fi
if lsof -ti :8080 > /dev/null 2>&1; then
    PORT_PROC=$(lsof -ti :8080)
    PROC_NAME=$(ps -p $PORT_PROC -o comm= 2>/dev/null || echo "unknown")
    if [[ "$PROC_NAME" == *"spiral"* ]]; then
        STILL_RUNNING=true
    fi
fi

# Final status
echo ""
if [ "$STILL_RUNNING" = true ]; then
    echo -e "${RED}⚠️  Some Spiral Core processes may still be running${NC}"
    echo -e "${YELLOW}Try running: pkill -9 -f spiral${NC}"
elif [ "$FOUND_PROCESS" = true ]; then
    echo -e "${GREEN}✓ All Spiral Core processes stopped${NC}"
else
    echo -e "${GREEN}✓ No Spiral Core processes were running${NC}"
fi

echo ""
echo "To start Spiral Core, run:"
echo "  ./start-spiral-core.sh"
echo ""