# Claude Code Integration Tests
# Tests the Claude Code CLI integration through API endpoints

# Test 1: Simple code generation task with session management
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Create a simple function that adds two numbers and returns the result. This will be part of a calculator project.",
  "priority": "Medium",
  "context": {
    "language": "rust",
    "file_type": "function",
    "project_name": "calculator"
  }
}

HTTP 200
[Captures]
simple_task_id: jsonpath "$.task_id"
[Asserts]
jsonpath "$.task_id" exists
jsonpath "$.status" == "submitted"

# Test 2: Analyze the simple task
POST {{BASE_URL}}/tasks/{{simple_task_id}}/analyze
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Create a simple function that adds two numbers and returns the result",
  "priority": "Medium",
  "context": {
    "language": "rust",
    "file_type": "function"
  }
}

HTTP 200
[Asserts]
jsonpath "$.complexity" exists
jsonpath "$.estimated_minutes" exists
jsonpath "$.required_skills" exists
jsonpath "$.challenges" exists
jsonpath "$.approach" exists
# Verify Claude Code detected the language correctly
jsonpath "$.required_skills" contains "rust"

# Test 3: Continue working on the calculator project (session continuity)
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Now create a Calculator struct with add, subtract, multiply, and divide methods. This should work with the addition function we just created.",
  "priority": "High",
  "context": {
    "language": "rust",
    "project_type": "library",
    "project_name": "calculator",
    "continues_from": "{{simple_task_id}}",
    "requires_file_creation": "true"
  }
}

HTTP 200
[Captures]
calculator_task_id: jsonpath "$.task_id"
[Asserts]
jsonpath "$.task_id" exists
jsonpath "$.status" == "submitted"

# Test 4: Complex task with multiple requirements
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Implement a RESTful API endpoint for user registration with input validation, password hashing, and database integration",
  "priority": "High",
  "context": {
    "language": "rust",
    "framework": "axum",
    "database": "postgresql",
    "security": "bcrypt"
  }
}

HTTP 200
[Captures]
complex_task_id: jsonpath "$.task_id"
[Asserts]
jsonpath "$.task_id" exists
jsonpath "$.status" == "submitted"

# Test 5: Analyze complex task to verify Claude Code capabilities
POST {{BASE_URL}}/tasks/{{complex_task_id}}/analyze
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Implement a RESTful API endpoint for user registration with input validation, password hashing, and database integration",
  "priority": "High",
  "context": {
    "language": "rust",
    "framework": "axum",
    "database": "postgresql",
    "security": "bcrypt"
  }
}

HTTP 200
[Asserts]
jsonpath "$.complexity" == "High"
jsonpath "$.estimated_minutes" exists
jsonpath "$.required_skills" exists
jsonpath "$.challenges" exists
jsonpath "$.approach" exists
# Verify Claude Code understands the technology stack
jsonpath "$.required_skills" contains "rust"
jsonpath "$.required_skills" contains "api"

# Test 6: Language detection task
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "def fibonacci(n): return n if n <= 1 else fibonacci(n-1) + fibonacci(n-2)",
  "priority": "Low",
  "context": {
    "task_type": "language_detection"
  }
}

HTTP 200
[Captures]
language_detection_task_id: jsonpath "$.task_id"
[Asserts]
jsonpath "$.task_id" exists
jsonpath "$.status" == "submitted"

# Test 7: Analyze language detection task
POST {{BASE_URL}}/tasks/{{language_detection_task_id}}/analyze
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "def fibonacci(n): return n if n <= 1 else fibonacci(n-1) + fibonacci(n-2)",
  "priority": "Low",
  "context": {
    "task_type": "language_detection"
  }
}

HTTP 200
[Asserts]
jsonpath "$.complexity" exists
jsonpath "$.estimated_minutes" exists
jsonpath "$.required_skills" exists
# Verify Claude Code detected Python
jsonpath "$.required_skills" contains "python"

# Test 8: Task with existing code modification
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Add error handling and input validation to this function",
  "priority": "Medium",
  "context": {
    "language": "rust",
    "existing_code": "fn divide(a: f64, b: f64) -> f64 { a / b }",
    "modification_type": "enhancement"
  }
}

HTTP 200
[Captures]
modification_task_id: jsonpath "$.task_id"
[Asserts]
jsonpath "$.task_id" exists
jsonpath "$.status" == "submitted"

# Test 9: Test with tools and file operations context
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Create a comprehensive test suite for the calculator module including unit tests and integration tests",
  "priority": "Medium",
  "context": {
    "language": "rust",
    "test_framework": "built-in",
    "requires_file_operations": "true",
    "module_name": "calculator"
  }
}

HTTP 200
[Captures]
test_suite_task_id: jsonpath "$.task_id"
[Asserts]
jsonpath "$.task_id" exists
jsonpath "$.status" == "submitted"

# Test 10: Analyze test suite task
POST {{BASE_URL}}/tasks/{{test_suite_task_id}}/analyze
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Create a comprehensive test suite for the calculator module including unit tests and integration tests",
  "priority": "Medium",
  "context": {
    "language": "rust",
    "test_framework": "built-in",
    "requires_file_operations": "true",
    "module_name": "calculator"
  }
}

HTTP 200
[Asserts]
jsonpath "$.complexity" exists
jsonpath "$.estimated_minutes" exists
jsonpath "$.required_skills" contains "testing"
jsonpath "$.challenges" exists
jsonpath "$.approach" exists

# Test 11: Check task status for all created tasks
GET {{BASE_URL}}/tasks/{{simple_task_id}}
x-api-key: {{API_KEY}}

HTTP 200
[Asserts]
jsonpath "$.task_id" == "{{simple_task_id}}"
jsonpath "$.agent_type" == "SoftwareDeveloper"
jsonpath "$.status" exists

GET {{BASE_URL}}/tasks/{{file_creation_task_id}}
x-api-key: {{API_KEY}}

HTTP 200
[Asserts]
jsonpath "$.task_id" == "{{file_creation_task_id}}"
jsonpath "$.agent_type" == "SoftwareDeveloper"

GET {{BASE_URL}}/tasks/{{complex_task_id}}
x-api-key: {{API_KEY}}

HTTP 200
[Asserts]
jsonpath "$.task_id" == "{{complex_task_id}}"
jsonpath "$.agent_type" == "SoftwareDeveloper"

# Test 12: Permission-intensive task (to test Claude Code permissions)
POST {{BASE_URL}}/tasks
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Create a new Rust project structure with Cargo.toml, src/main.rs, src/lib.rs, tests/ directory, and README.md file. Include proper gitignore and CI/CD configuration.",
  "priority": "High",
  "context": {
    "language": "rust",
    "project_type": "full_project",
    "requires_multiple_files": "true",
    "requires_directory_creation": "true"
  }
}

HTTP 200
[Captures]
project_creation_task_id: jsonpath "$.task_id"
[Asserts]
jsonpath "$.task_id" exists
jsonpath "$.status" == "submitted"

# Test 13: Analyze project creation task
POST {{BASE_URL}}/tasks/{{project_creation_task_id}}/analyze
x-api-key: {{API_KEY}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Create a new Rust project structure with Cargo.toml, src/main.rs, src/lib.rs, tests/ directory, and README.md file. Include proper gitignore and CI/CD configuration.",
  "priority": "High",
  "context": {
    "language": "rust",
    "project_type": "full_project",
    "requires_multiple_files": "true",
    "requires_directory_creation": "true"
  }
}

HTTP 200
[Asserts]
jsonpath "$.complexity" == "High"
jsonpath "$.estimated_minutes" exists
jsonpath "$.required_skills" contains "rust"
jsonpath "$.challenges" exists
jsonpath "$.approach" exists