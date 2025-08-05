# API Reference - Spiral Core

## Base URL

```
http://localhost:3000
```

## Authentication

Most endpoints require API key authentication:

```http
x-api-key: your-api-key-here
```

## Endpoints

### Health Check

Check if the API server is running.

```http
GET /health
```

**Response:**

```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### System Status

Get detailed system status and agent information.

```http
GET /system/status
x-api-key: {{api_key}}
```

**Response:**

```json
{
  "status": "operational",
  "agents": {
    "SoftwareDeveloper": "ready",
    "ProjectManager": "ready"
  },
  "resources": {
    "memory_usage": "2.1GB",
    "cpu_usage": "15%"
  }
}
```

### List Agents

Get all available agents and their capabilities.

```http
GET /agents
x-api-key: {{api_key}}
```

**Response:**

```json
{
  "agents": [
    {
      "name": "SoftwareDeveloper",
      "status": "ready",
      "capabilities": ["code_generation", "language_detection", "testing"]
    },
    {
      "name": "ProjectManager",
      "status": "ready",
      "capabilities": [
        "strategic_analysis",
        "task_coordination",
        "risk_assessment"
      ]
    }
  ]
}
```

### Get Agent Status

Get detailed status for a specific agent.

```http
GET /agents/{agent_type}
x-api-key: {{api_key}}
```

**Parameters:**

- `agent_type` - Agent type (e.g., "SoftwareDeveloper", "ProjectManager")

**Response:**

```json
{
  "name": "SoftwareDeveloper",
  "status": "ready",
  "current_task": null,
  "completed_tasks": 42,
  "average_response_time": "3.2s"
}
```

### Submit Task

Submit a new task for agent processing.

```http
POST /tasks
x-api-key: {{api_key}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Create a hello world function in Rust",
  "priority": "Medium",
  "context": {
    "file_path": "src/main.rs",
    "project_type": "rust"
  }
}
```

**Request Body:**

- `agent_type` (required) - Type of agent to handle the task
- `content` (required) - Task description
- `priority` (optional) - "Low", "Medium", "High", "Critical"
- `context` (optional) - Additional context for the task

**Response:**

```json
{
  "task_id": "task_123456",
  "status": "queued",
  "agent_type": "SoftwareDeveloper",
  "estimated_completion": "2024-01-01T12:05:00Z"
}
```

### Get Task Status

Check the status of a submitted task.

```http
GET /tasks/{task_id}
x-api-key: {{api_key}}
```

**Parameters:**

- `task_id` - The task ID returned from submit

**Response:**

```json
{
  "task_id": "task_123456",
  "status": "completed",
  "agent_type": "SoftwareDeveloper",
  "result": {
    "code": "fn hello_world() { println!(\"Hello, World!\"); }",
    "language": "rust",
    "tests_passed": true
  },
  "started_at": "2024-01-01T12:00:00Z",
  "completed_at": "2024-01-01T12:00:05Z"
}
```

### Analyze Task

Submit a task for analysis without execution.

```http
POST /tasks/analyze
x-api-key: {{api_key}}
Content-Type: application/json

{
  "content": "Create a web application with user authentication",
  "context": {
    "tech_stack": ["rust", "postgresql", "redis"]
  }
}
```

**Response:**

```json
{
  "analysis": {
    "complexity": "high",
    "estimated_effort": "2-3 weeks",
    "recommended_agents": [
      "SoftwareDeveloper",
      "ProjectManager",
      "QualityAssurance"
    ],
    "suggested_phases": [
      "Database schema design",
      "Authentication implementation",
      "API development",
      "Testing and security review"
    ]
  }
}
```

### Cancel Task

Cancel a queued or running task.

```http
DELETE /tasks/{task_id}
x-api-key: {{api_key}}
```

**Response:**

```json
{
  "task_id": "task_123456",
  "status": "cancelled",
  "message": "Task cancelled successfully"
}
```

## Error Responses

All endpoints may return error responses:

### 400 Bad Request

```json
{
  "error": "Invalid request",
  "message": "Missing required field: agent_type"
}
```

### 401 Unauthorized

```json
{
  "error": "Unauthorized",
  "message": "Invalid or missing API key"
}
```

### 404 Not Found

```json
{
  "error": "Not found",
  "message": "Task not found: task_123456"
}
```

### 429 Too Many Requests

```json
{
  "error": "Rate limit exceeded",
  "message": "Please wait 60 seconds before making another request"
}
```

### 500 Internal Server Error

```json
{
  "error": "Internal server error",
  "message": "An unexpected error occurred"
}
```

## Rate Limiting

- **Default limit**: 100 requests per minute per API key
- **Burst limit**: 10 requests per second
- Rate limit headers included in responses:
  - `X-RateLimit-Limit`: Maximum requests per minute
  - `X-RateLimit-Remaining`: Requests remaining
  - `X-RateLimit-Reset`: Unix timestamp when limit resets

## Testing with Hurl

The project includes Hurl test files for API testing:

```bash
# Install Hurl
cargo install hurl --locked

# Run all API tests
./scripts/test-api-hurl.sh

# Run specific test
hurl --env-file tests/api/hurl.env --test tests/api/health.hurl
```

## WebSocket Support (Planned)

Future versions will support WebSocket connections for real-time updates:

```javascript
const ws = new WebSocket("ws://localhost:3000/ws");
ws.send(
  JSON.stringify({
    type: "subscribe",
    task_id: "task_123456",
  })
);
```

## SDK Support

### Rust Client

```rust
use spiral_core_client::Client;

let client = Client::new("http://localhost:3000", "api_key");
let task = client.submit_task(
    AgentType::SoftwareDeveloper,
    "Create hello world",
    Priority::Medium,
).await?;
```

### Python Client

```python
from spiral_core import Client

client = Client(base_url="http://localhost:3000", api_key="api_key")
task = await client.submit_task(
    agent_type="SoftwareDeveloper",
    content="Create hello world",
    priority="Medium"
)
```

## Monitoring

The API exposes Prometheus metrics at `/metrics`:

```http
GET /metrics
```

Key metrics:

- `spiral_core_requests_total` - Total API requests
- `spiral_core_task_duration_seconds` - Task completion times
- `spiral_core_agent_utilization` - Agent utilization percentage
