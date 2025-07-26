# Hurl Test Environment Configuration
# This file contains environment variables specifically for running Hurl API tests

# API Server Configuration
API_HOST=127.0.0.1
API_PORT=3000
base_url=http://127.0.0.1:3000
api_key=test-api-key-1234567890123456789012345678901234567890
invalid_key=invalid-test-key-123

# Authentication Configuration for Testing
API_KEY=test-api-key-1234567890123456789012345678901234567890
ENABLE_API_AUTH=true

# Claude API Configuration (minimal for testing)
CLAUDE_API_KEY=sk-ant-api03-test-claude-key-for-hurl-testing-1234567890
CLAUDE_BASE_URL=https://api.anthropic.com
CLAUDE_MODEL=claude-sonnet-4-20250514
CLAUDE_MAX_TOKENS=4096
CLAUDE_TEMPERATURE=0.7

# Discord Configuration (disabled for testing)
DISCORD_TOKEN=test-discord-token-for-hurl-testing-1234567890123456789012345678
DISCORD_PREFIX=!spiral
AGENT_MENTION_PATTERN="@(\\w+)agent"

# Redis Configuration (unused)
REDIS_URL=redis://localhost:6379
TASK_QUEUE=spiral:tasks
RESULT_QUEUE=spiral:results