# CORS Policy Tests
# Tests Cross-Origin Resource Sharing configuration

# Test 1: Preflight request for allowed origin should succeed
OPTIONS {{BASE_URL}}/tasks
Origin: http://localhost:3000
Access-Control-Request-Method: POST
Access-Control-Request-Headers: x-api-key, content-type

HTTP 200
[Asserts]
header "Access-Control-Allow-Origin" == "http://localhost:3000"
header "Access-Control-Allow-Methods" contains "POST"
header "Access-Control-Allow-Headers" contains "x-api-key"
header "Access-Control-Allow-Headers" contains "content-type"
header "Access-Control-Max-Age" exists

# Test 2: Preflight request for disallowed origin should not get CORS headers
OPTIONS {{BASE_URL}}/tasks
Origin: http://evil.com
Access-Control-Request-Method: POST
Access-Control-Request-Headers: x-api-key

HTTP 200
[Asserts]
header "Access-Control-Allow-Origin" not exists

# Test 3: Simple request with allowed origin should get CORS headers
GET {{BASE_URL}}/health
Origin: http://localhost:3000
x-api-key: {{API_KEY}}

HTTP 200
[Asserts]
header "Access-Control-Allow-Origin" == "http://localhost:3000"
jsonpath "$.status" == "healthy"

# Test 4: Simple request with disallowed origin should not get CORS headers
GET {{BASE_URL}}/health
Origin: http://evil.com
x-api-key: {{API_KEY}}

HTTP 200
[Asserts]
header "Access-Control-Allow-Origin" not exists
jsonpath "$.status" == "healthy"

# Test 5: Request without Origin header should succeed (same-origin)
GET {{BASE_URL}}/health
x-api-key: {{API_KEY}}

HTTP 200
[Asserts]
jsonpath "$.status" == "healthy"

# Test 6: Verify allowed methods are restrictive
OPTIONS {{BASE_URL}}/tasks
Origin: http://localhost:3000
Access-Control-Request-Method: DELETE

HTTP 200
[Asserts]
header "Access-Control-Allow-Methods" not contains "DELETE"
header "Access-Control-Allow-Methods" contains "GET"
header "Access-Control-Allow-Methods" contains "POST"

# Test 7: Verify cache age is set properly
OPTIONS {{BASE_URL}}/health
Origin: http://localhost:3000
Access-Control-Request-Method: GET

HTTP 200
[Asserts]
header "Access-Control-Max-Age" == "3600"