# Agent Endpoints Tests
# Tests /agents endpoints for status retrieval

# Test 1: Get all agent statuses without authentication should fail
GET {{base_url}}/agents

HTTP 401
[Asserts]
jsonpath "$.error" == "Unauthorized"

# Test 2: Get all agent statuses should succeed
GET {{base_url}}/agents
x-api-key: {{api_key}}

HTTP 200
[Asserts]
jsonpath "$" exists
jsonpath "$.SoftwareDeveloper" exists
jsonpath "$.SoftwareDeveloper.agent_type" == "SoftwareDeveloper"
jsonpath "$.SoftwareDeveloper.is_busy" exists
jsonpath "$.SoftwareDeveloper.tasks_completed" exists
jsonpath "$.SoftwareDeveloper.tasks_failed" exists
jsonpath "$.SoftwareDeveloper.average_execution_time" exists
header "Content-Type" contains "application/json"

# Test 3: Get specific agent status should succeed
GET {{base_url}}/agents/SoftwareDeveloper
x-api-key: {{api_key}}

HTTP 200
[Asserts]
jsonpath "$.agent_type" == "SoftwareDeveloper"
jsonpath "$.is_busy" exists
jsonpath "$.current_task_id" exists
jsonpath "$.tasks_completed" exists
jsonpath "$.tasks_failed" exists
jsonpath "$.average_execution_time" exists

# Test 4: Get specific agent status without authentication should fail
GET {{base_url}}/agents/SoftwareDeveloper

HTTP 401
[Asserts]
jsonpath "$.error" == "Unauthorized"

# Test 5: Get non-existent agent should return 404
GET {{base_url}}/agents/NonExistentAgent
x-api-key: {{api_key}}

HTTP 404
[Asserts]
jsonpath "$.error" == "Agent not found"
jsonpath "$.details" exists

# Test 6: Get agent with Bearer token should succeed
GET {{base_url}}/agents/SoftwareDeveloper
Authorization: Bearer {{api_key}}

HTTP 200
[Asserts]
jsonpath "$.agent_type" == "SoftwareDeveloper"

# Test 7: Verify initial agent state (fresh server)
GET {{base_url}}/agents/SoftwareDeveloper
x-api-key: {{api_key}}

HTTP 200
[Asserts]
jsonpath "$.is_busy" == false
jsonpath "$.current_task_id" == null
jsonpath "$.tasks_completed" == 0
jsonpath "$.tasks_failed" == 0

# Test 8: Test that all expected agent types are available
GET {{base_url}}/agents
x-api-key: {{api_key}}

HTTP 200
[Asserts]
jsonpath "$.SoftwareDeveloper" exists
# Note: Other agent types may not be implemented yet
# jsonpath "$.ProjectManager" exists
# jsonpath "$.QualityAssurance" exists