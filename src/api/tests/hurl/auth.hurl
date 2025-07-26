# Authentication and Authorization Tests
# Tests authentication mechanisms across all endpoints

# Test 1: No authentication should fail across all endpoints
GET {{base_url}}/health
HTTP 401

GET {{base_url}}/system/status
HTTP 401

GET {{base_url}}/agents
HTTP 401

POST {{base_url}}/tasks
Content-Type: application/json
{}
HTTP 401

# Test 2: Invalid API key should fail
GET {{base_url}}/health
x-api-key: invalid-short-key
HTTP 401

GET {{base_url}}/health
x-api-key: {{invalid_key}}
HTTP 401

# Test 3: Empty API key should fail
GET {{base_url}}/health
x-api-key: 
HTTP 401

# Test 4: Valid API key should work across all GET endpoints
GET {{base_url}}/health
x-api-key: {{api_key}}
HTTP 200

GET {{base_url}}/system/status
x-api-key: {{api_key}}
HTTP 200

GET {{base_url}}/agents
x-api-key: {{api_key}}
HTTP 200

# Test 5: Bearer token should work across all GET endpoints
GET {{base_url}}/health
Authorization: Bearer {{api_key}}
HTTP 200

GET {{base_url}}/system/status
Authorization: Bearer {{api_key}}
HTTP 200

GET {{base_url}}/agents
Authorization: Bearer {{api_key}}
HTTP 200

# Test 6: Malformed Bearer token should fail
GET {{base_url}}/health
Authorization: Bearer
HTTP 401

GET {{base_url}}/health
Authorization: Basic {{api_key}}
HTTP 401

GET {{base_url}}/health
Authorization: {{api_key}}
HTTP 401

# Test 7: Case sensitivity - header names should be case insensitive
GET {{base_url}}/health
X-API-KEY: {{api_key}}
HTTP 200

GET {{base_url}}/health
x-Api-Key: {{api_key}}
HTTP 200

GET {{base_url}}/health
authorization: Bearer {{api_key}}
HTTP 200

# Test 8: Multiple auth headers - should use the first valid one
GET {{base_url}}/health
x-api-key: {{api_key}}
Authorization: Bearer invalid-token
HTTP 200