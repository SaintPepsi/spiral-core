# System Status Endpoint Tests
# Tests /system/status endpoint for overall system health

# Test 1: Get system status without authentication should fail
GET {{base_url}}/system/status

HTTP 401
[Asserts]
jsonpath "$.error" == "Unauthorized"

# Test 2: Get system status should succeed
GET {{base_url}}/system/status
x-api-key: {{api_key}}

HTTP 200
[Asserts]
jsonpath "$.agents" exists
jsonpath "$.queue_length" exists
jsonpath "$.system_uptime" exists
jsonpath "$.agents.SoftwareDeveloper" exists
jsonpath "$.agents.SoftwareDeveloper.agent_type" == "SoftwareDeveloper"
jsonpath "$.agents.SoftwareDeveloper.is_busy" exists
jsonpath "$.agents.SoftwareDeveloper.tasks_completed" exists
jsonpath "$.agents.SoftwareDeveloper.tasks_failed" exists
jsonpath "$.agents.SoftwareDeveloper.average_execution_time" exists
header "Content-Type" contains "application/json"

# Test 3: Verify initial system state (fresh server)
GET {{base_url}}/system/status
x-api-key: {{api_key}}

HTTP 200
[Asserts]
jsonpath "$.queue_length" == 0
jsonpath "$.system_uptime" >= 0
jsonpath "$.agents.SoftwareDeveloper.is_busy" == false
jsonpath "$.agents.SoftwareDeveloper.tasks_completed" == 0

# Test 4: Get system status with Bearer token should succeed
GET {{base_url}}/system/status
Authorization: Bearer {{api_key}}

HTTP 200
[Asserts]
jsonpath "$.agents" exists
jsonpath "$.queue_length" exists
jsonpath "$.system_uptime" exists

# Test 5: Verify queue length is numeric
GET {{base_url}}/system/status
x-api-key: {{api_key}}

HTTP 200
[Asserts]
jsonpath "$.queue_length" isInteger
jsonpath "$.system_uptime" isFloat

# Test 6: Verify agents object structure
GET {{base_url}}/system/status
x-api-key: {{api_key}}

HTTP 200
[Asserts]
jsonpath "$.agents" exists
jsonpath "$.agents.SoftwareDeveloper.agent_type" isString
jsonpath "$.agents.SoftwareDeveloper.is_busy" isBoolean
jsonpath "$.agents.SoftwareDeveloper.tasks_completed" isInteger
jsonpath "$.agents.SoftwareDeveloper.tasks_failed" isInteger
jsonpath "$.agents.SoftwareDeveloper.average_execution_time" isFloat