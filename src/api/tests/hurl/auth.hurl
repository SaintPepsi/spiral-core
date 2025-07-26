# Authentication and Authorization Tests
# Tests authentication mechanisms across all endpoints

# Test 1: No authentication should fail across all endpoints
GET {{BASE_URL}}/health
HTTP 401

GET {{BASE_URL}}/system/status
HTTP 401

GET {{BASE_URL}}/agents
HTTP 401

POST {{BASE_URL}}/tasks
Content-Type: application/json
{}
HTTP 401

# Test 2: Invalid API key should fail
GET {{BASE_URL}}/health
x-api-key: invalid-short-key
HTTP 401

GET {{BASE_URL}}/health
x-api-key: {{invalid_key}}
HTTP 401

# Test 3: Empty API key should fail
GET {{BASE_URL}}/health
x-api-key: 
HTTP 401

# Test 4: Valid API key should work across all GET endpoints
GET {{BASE_URL}}/health
x-api-key: {{API_KEY}}
HTTP 200

GET {{BASE_URL}}/system/status
x-api-key: {{API_KEY}}
HTTP 200

GET {{BASE_URL}}/agents
x-api-key: {{API_KEY}}
HTTP 200

# Test 5: Bearer token should work across all GET endpoints
GET {{BASE_URL}}/health
Authorization: Bearer {{API_KEY}}
HTTP 200

GET {{BASE_URL}}/system/status
Authorization: Bearer {{API_KEY}}
HTTP 200

GET {{BASE_URL}}/agents
Authorization: Bearer {{API_KEY}}
HTTP 200

# Test 6: Malformed Bearer token should fail
GET {{BASE_URL}}/health
Authorization: Bearer
HTTP 401

GET {{BASE_URL}}/health
Authorization: Basic {{API_KEY}}
HTTP 401

GET {{BASE_URL}}/health
Authorization: {{API_KEY}}
HTTP 401

# Test 7: Case sensitivity - header names should be case insensitive
GET {{BASE_URL}}/health
X-API-KEY: {{API_KEY}}
HTTP 200

GET {{BASE_URL}}/health
x-Api-Key: {{API_KEY}}
HTTP 200

GET {{BASE_URL}}/health
authorization: Bearer {{API_KEY}}
HTTP 200

# Test 8: Multiple auth headers - should use the first valid one
GET {{BASE_URL}}/health
x-api-key: {{API_KEY}}
Authorization: Bearer invalid-token
HTTP 200