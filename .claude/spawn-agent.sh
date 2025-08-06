#!/bin/bash

# Spawn a Claude agent with the specified template
# Usage: ./spawn-agent.sh <agent-type> [context-file]
#
# Examples:
#   ./spawn-agent.sh code-review
#   ./spawn-agent.sh test-failure-analyzer /tmp/test-output.json
#   ./spawn-agent.sh success-analyzer /tmp/pipeline-context.json

set -e

AGENT_TYPE="$1"
CONTEXT_FILE="$2"
CLAUDE_DIR="$(dirname "$0")"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Find agent template file
find_agent_template() {
    local agent="$1"
    
    # Search in validation-agents and utility-agents
    for dir in validation-agents/phase1 validation-agents/phase2 validation-agents/analysis utility-agents; do
        for file in "$CLAUDE_DIR"/"$dir"/*.md; do
            if [[ -f "$file" ]]; then
                basename_file=$(basename "$file" .md)
                if [[ "$basename_file" == *"$agent"* ]] || [[ "$agent" == *"$basename_file"* ]]; then
                    echo "$file"
                    return 0
                fi
            fi
        done
    done
    
    return 1
}

# List available agents
list_agents() {
    echo -e "${GREEN}Available agents:${NC}"
    echo ""
    
    echo -e "${YELLOW}Phase 1 (Quality Assurance):${NC}"
    for file in "$CLAUDE_DIR"/validation-agents/phase1/*.md; do
        [[ -f "$file" ]] && echo "  - $(basename "$file" .md)"
    done
    
    echo ""
    echo -e "${YELLOW}Phase 2 (Compliance):${NC}"
    for file in "$CLAUDE_DIR"/validation-agents/phase2/*.md; do
        [[ -f "$file" ]] && echo "  - $(basename "$file" .md)"
    done
    
    echo ""
    echo -e "${YELLOW}Analysis:${NC}"
    for file in "$CLAUDE_DIR"/validation-agents/analysis/*.md; do
        [[ -f "$file" ]] && echo "  - $(basename "$file" .md)"
    done
    
    echo ""
    echo -e "${YELLOW}Utilities:${NC}"
    for file in "$CLAUDE_DIR"/utility-agents/*.md; do
        [[ -f "$file" ]] && echo "  - $(basename "$file" .md)"
    done
}

# Main logic
if [[ -z "$AGENT_TYPE" ]] || [[ "$AGENT_TYPE" == "--help" ]] || [[ "$AGENT_TYPE" == "-h" ]]; then
    echo "Usage: $0 <agent-type> [context-file]"
    echo ""
    list_agents
    exit 0
fi

# Find the agent template
TEMPLATE=$(find_agent_template "$AGENT_TYPE")
if [[ -z "$TEMPLATE" ]]; then
    echo -e "${RED}Error: Agent template not found for '$AGENT_TYPE'${NC}"
    echo ""
    list_agents
    exit 1
fi

echo -e "${GREEN}Found agent template:${NC} $TEMPLATE"

# Build the Claude command
CLAUDE_CMD="claude --profile claude-code"

# Add the agent template
CLAUDE_CMD="$CLAUDE_CMD --agent-file \"$TEMPLATE\""

# Add context if provided
if [[ -n "$CONTEXT_FILE" ]]; then
    if [[ ! -f "$CONTEXT_FILE" ]]; then
        echo -e "${RED}Error: Context file not found: $CONTEXT_FILE${NC}"
        exit 1
    fi
    echo -e "${GREEN}Using context:${NC} $CONTEXT_FILE"
    CLAUDE_CMD="$CLAUDE_CMD --context \"$CONTEXT_FILE\""
fi

# Spawn the agent
echo -e "${YELLOW}Spawning agent...${NC}"
echo "Command: $CLAUDE_CMD"
echo ""

# Execute the command
eval "$CLAUDE_CMD"