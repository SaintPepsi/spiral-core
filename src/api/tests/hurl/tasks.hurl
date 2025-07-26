# Task Endpoints Tests
# Tests /tasks endpoints for creation, retrieval, and analysis

# Test 1: Create task without authentication should fail
POST {{BASE_URL}}/tasks
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Create a hello world function",
  "priority": "Medium"
}

HTTP 401
[Asserts]
jsonpath "$.error" == "Unauthorized"

# Test 2: Create valid task should succeed
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper", 
  "content": "Create a hello world function in Rust",
  "priority": "Medium"
}

HTTP 200
[Captures]
task_id: jsonpath "$.task_id"
[Asserts]
jsonpath "$.task_id" exists
jsonpath "$.status" == "submitted"
header "Content-Type" contains "application/json"

# Test 3: Create task with malicious content should fail (XSS protection)
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "<script>alert('xss')</script>",
  "priority": "Medium"
}

HTTP 400
[Asserts]
jsonpath "$.error" == "Invalid task content"
jsonpath "$.details" not exists

# Test 4: Create task with SQL injection attempt should fail
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "DROP TABLE users; --",
  "priority": "Medium"
}

HTTP 400
[Asserts]
jsonpath "$.error" == "Invalid task content"

# Test 5: Create task with command injection attempt should fail
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "test && rm -rf /",
  "priority": "Medium"
}

HTTP 400
[Asserts]
jsonpath "$.error" == "Invalid task content"

# Test 6: Create task with context should succeed
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Implement user authentication",
  "priority": "High",
  "context": {
    "language": "rust",
    "framework": "axum"
  }
}

HTTP 200
[Captures]
task_with_context_id: jsonpath "$.task_id"
[Asserts]
jsonpath "$.task_id" exists
jsonpath "$.status" == "submitted"

# Test 7: Get task status should succeed
GET {{BASE_URL}}/tasks/{{task_id}}
x-api-key: {{API_KEY}}

HTTP 200
[Asserts]
jsonpath "$.task_id" == "{{task_id}}"
jsonpath "$.agent_type" == "SoftwareDeveloper"
jsonpath "$.status" exists
jsonpath "$.created_at" exists
jsonpath "$.updated_at" exists

# Test 8: Get task status without authentication should fail
GET {{BASE_URL}}/tasks/{{task_id}}

HTTP 401
[Asserts]
jsonpath "$.error" == "Unauthorized"

# Test 9: Get non-existent task should return 404
GET {{BASE_URL}}/tasks/non-existent-task-id
x-api-key: {{API_KEY}}

HTTP 404
[Asserts]
jsonpath "$.error" == "Task not found"
jsonpath "$.details" exists

# Test 10: Analyze task should succeed
POST {{BASE_URL}}/tasks/{{task_id}}/analyze
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Create a hello world function in Rust",
  "priority": "Medium"
}

HTTP 200
[Asserts]
jsonpath "$.complexity" exists
jsonpath "$.estimated_minutes" exists
jsonpath "$.required_skills" exists
jsonpath "$.challenges" exists
jsonpath "$.approach" exists

# Test 11: Analyze task without authentication should fail
POST {{BASE_URL}}/tasks/{{task_id}}/analyze
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Create a hello world function",
  "priority": "Medium"
}

HTTP 401
[Asserts]
jsonpath "$.error" == "Unauthorized"

# Test 12: Create task with missing required fields should fail
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "priority": "Medium"
}

HTTP 400
[Asserts]
jsonpath "$.error" exists

# Test 13: Create task with invalid agent type should fail
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "InvalidAgent",
  "content": "Test task",
  "priority": "Medium"
}

HTTP 400
[Asserts]
jsonpath "$.error" exists

# Test 14: Create task with invalid priority should fail
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Test task",
  "priority": "InvalidPriority"
}

HTTP 400
[Asserts]
jsonpath "$.error" exists