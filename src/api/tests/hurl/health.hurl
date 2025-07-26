# Health Endpoint Tests
# Tests the /health endpoint for availability and authentication

# Test 1: Health check without authentication should fail (security hardened)
GET {{BASE_URL}}/health

HTTP 401
[Asserts]
jsonpath "$.error" == "Unauthorized"

# Test 2: Health check with valid API key should succeed
GET {{BASE_URL}}/health
x-api-key: {{API_KEY}}

HTTP 200
[Asserts]
jsonpath "$.status" == "healthy"
jsonpath "$.service" == "spiral-core"
jsonpath "$.version" exists
header "Content-Type" contains "application/json"

# Test 3: Health check with Bearer token should succeed
GET {{BASE_URL}}/health
Authorization: Bearer {{API_KEY}}

HTTP 200
[Asserts]
jsonpath "$.status" == "healthy"
jsonpath "$.service" == "spiral-core"

# Test 4: Health check with invalid API key should fail
GET {{BASE_URL}}/health
x-api-key: invalid-key-too-short

HTTP 401
[Asserts]
jsonpath "$.error" == "Unauthorized"

# Test 5: Health check with empty API key should fail
GET {{BASE_URL}}/health
x-api-key: 

HTTP 401
[Asserts]
jsonpath "$.error" == "Unauthorized"