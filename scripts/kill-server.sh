#!/bin/bash

# Kill Spiral Core Server Script
# This script finds and terminates any running spiral-core processes

echo "🔍 Looking for running spiral-core processes..."

# Find processes matching spiral-core
PIDS=$(pgrep -f "spiral-core" 2>/dev/null)

if [ -z "$PIDS" ]; then
    echo "✅ No spiral-core processes found running"
else
    echo "📋 Found spiral-core processes:"
    ps aux | grep -E "spiral-core" | grep -v grep
    
    echo ""
    echo "🛑 Killing processes..."
    
    # Kill each process
    for pid in $PIDS; do
        echo "   Killing PID: $pid"
        kill -9 $pid 2>/dev/null
    done
    
    echo "✅ All spiral-core processes terminated"
fi

# Also check if port 3000 is in use
echo ""
echo "🔍 Checking port 3000..."
PORT_PID=$(lsof -ti:3000 2>/dev/null)

if [ -z "$PORT_PID" ]; then
    echo "✅ Port 3000 is free"
else
    echo "⚠️  Port 3000 is in use by PID: $PORT_PID"
    echo "🛑 Killing process using port 3000..."
    kill -9 $PORT_PID 2>/dev/null
    echo "✅ Port 3000 is now free"
fi

echo ""
echo "🎉 Server cleanup complete!"