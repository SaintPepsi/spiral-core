# Hurl API Tests

This directory contains Hurl-based API endpoint tests for the Spiral Core system.

## Test Structure

- `health.hurl` - Health check endpoint tests
- `tasks.hurl` - Task management endpoint tests  
- `agents.hurl` - Agent status endpoint tests
- `system.hurl` - System status endpoint tests
- `auth.hurl` - Authentication and authorization tests
- `cors.hurl` - CORS policy tests

## Running Tests

### Prerequisites

1. **Install Hurl**: <https://hurl.dev/docs/installation.html>
2. **Start the server**: The API server must be running on the configured port
3. **Set environment variables**: Required for authentication

### Environment Setup

```bash
export API_KEY="your-test-api-key-here-32-chars-minimum"
export BASE_URL="http://127.0.0.1:3000"
```

### Run All Tests

```bash
# From project root
./scripts/run-hurl-tests.sh
```

### Run Individual Test Files

```bash
# Single test file
hurl --variable base_url="$BASE_URL" --variable api_key="$API_KEY" tests/hurl/health.hurl

# With verbose output
hurl --verbose --variable base_url="$BASE_URL" --variable api_key="$API_KEY" tests/hurl/tasks.hurl
```

## Test Organization

### Unit Tests (Rust)

- Server lifecycle management
- Environment variable configuration
- Complex mocking and setup
- Internal component integration

### Integration Tests (Hurl)

- HTTP endpoint behavior
- Authentication flows
- API contract validation
- Response structure verification
- Error handling scenarios

## Configuration

Tests use variables for flexibility:

- `{{base_url}}` - API server base URL
- `{{api_key}}` - Authentication API key
- `{{invalid_key}}` - Invalid key for negative tests

## Expected Server State

These tests assume:

1. Fresh server instance (empty queues, default agent states)
2. Valid configuration with authentication enabled
3. No pre-existing tasks or modified agent states
4. Standard test environment setup
