# Spiral Core API Reference - Hurl Commands

## Quick Setup

```bash
# Install Hurl (API testing tool)
cargo install hurl --locked

# Or use the dev container (includes Hurl pre-installed)

# Make test script executable
chmod +x scripts/test-api-hurl.sh

# Run all tests
./scripts/test-api-hurl.sh
```

## Environment Variables

Create `tests/api/hurl.env` with:

```bash
api_key=e6fc6fcdebac4437d3fc75bce66e771e
base_url=http://localhost:3000
```

## Individual Endpoints

### 1. Health Check (No Auth Required)

```bash
hurl --test tests/api/health.hurl
```

Or create a simple `.hurl` file:

```hurl
GET http://localhost:3000/health

HTTP/1.1 200
Content-Type: application/json

[Asserts]
jsonpath "$.status" == "healthy"
```

### 2. System Status (Auth Required)

```bash
hurl --env-file tests/api/hurl.env --test tests/api/status.hurl
```

### 3. Get All Agents (Auth Required)

```bash
# Create tests/api/agents.hurl:
# GET http://localhost:3000/agents
# x-api-key: {{api_key}}

hurl --env-file tests/api/hurl.env --test tests/api/agents.hurl
```

### 4. Get Specific Agent Status (Auth Required)

```bash
# GET http://localhost:3000/agents/SoftwareDeveloper
# x-api-key: {{api_key}}

hurl --env-file tests/api/hurl.env tests/api/agent-status.hurl
```

### 5. Submit Task (Auth Required)

```bash
# POST http://localhost:3000/tasks
# x-api-key: {{api_key}}
# Content-Type: application/json
# {
#   "agent_type": "SoftwareDeveloper",
#   "content": "Create a hello world function in Rust",
#   "priority": "Medium",
#   "context": {"file_path": "src/main.rs", "project_type": "rust"}
# }

hurl --env-file tests/api/hurl.env tests/api/task-submit.hurl
```

### 6. Get Task Status (Auth Required)

```bash
# GET http://localhost:3000/tasks/{{task_id}}
# x-api-key: {{api_key}}
# (task_id captured from previous request)

hurl --env-file tests/api/hurl.env tests/api/task-status.hurl
```

### 7. Analyze Task (Auth Required)

```bash
# POST http://localhost:3000/tasks/{{task_id}}/analyze
# x-api-key: {{api_key}}
# Content-Type: application/json
# {"analysis_type": "comprehensive"}

hurl --env-file tests/api/hurl.env tests/api/task-analyze.hurl
```

## Authentication Examples

### Bearer Token Authentication

```hurl
GET http://localhost:3000/system/status
Authorization: Bearer {{api_key}}
```

### API Key Header Authentication

```hurl
GET http://localhost:3000/system/status
x-api-key: {{api_key}}
```

### JSON Response Validation

```hurl
GET http://localhost:3000/system/status
x-api-key: {{api_key}}

HTTP/1.1 200
Content-Type: application/json

[Asserts]
jsonpath "$.system.status" == "operational"
jsonpath "$.agents" exists
```

## Error Testing

### Authentication Failure

```hurl
# Should return 401 Unauthorized
GET http://localhost:3000/system/status

HTTP/1.1 401
```

### Invalid Agent Type

```hurl
POST http://localhost:3000/tasks
x-api-key: {{api_key}}
Content-Type: application/json
{
  "agent_type": "InvalidAgent",
  "content": "test"
}

HTTP/1.1 400
```

## Performance Testing

```bash
# Time multiple requests
hurl --test tests/api/health.hurl --repeat 10

# With variables for load testing
hurl --variables "load_test=true" tests/api/load-test.hurl
```

## Useful Hurl Options

- `--test`: Run in test mode with assertions
- `--env-file`: Load environment variables from file
- `--variables`: Set variables for the session
- `--verbose`: Show detailed request/response info
- `--json`: Output results as JSON
- `--timeout`: Set request timeout (default 30s)
- `--retry`: Number of retry attempts
- `--repeat`: Repeat requests N times

## Test Organization

```
tests/api/
├── hurl.env              # Local environment
├── hurl-staging.env      # Staging environment
├── health.hurl           # Health checks
├── status.hurl           # System status
├── api.hurl              # Complete test suite
├── agents.hurl           # Agent endpoints
├── task-submit.hurl      # Task creation
├── task-status.hurl      # Task retrieval
└── README.md             # Documentation
```

## Running Tests

```bash
# All tests
./scripts/test-api-hurl.sh

# Specific test
./scripts/test-api-hurl.sh health

# With staging environment
ENV_FILE=tests/api/hurl-staging.env ./scripts/test-api-hurl.sh

# Individual file
hurl --env-file tests/api/hurl.env --test tests/api/health.hurl
```
