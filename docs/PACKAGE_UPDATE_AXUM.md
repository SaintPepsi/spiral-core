# Axum Package Update Plan: 0.7 → 0.8

## Overview

Updating axum from 0.7.x to 0.8.x involves significant breaking changes in the web framework. This document outlines all required migrations.

## Risk Assessment

- **Risk Level**: 8 (Fibonacci scale)
- **Complexity**: 8 (Fibonacci scale)
- **Risk Factors**:
  - Major API redesign in some areas
  - Extractor trait changes
  - Router composition changes
  - Middleware API updates

## Breaking Changes

### 1. Minimum Dependencies

- **axum 0.8** requires:
  - `hyper` 1.0
  - `tower` 0.5
  - `http` 1.0
  - Rust 1.75+

### 2. State Extraction Changes

- **Change**: `State` extractor API redesigned
- **Before (0.7)**:

  ```rust
  async fn handler(State(state): State<AppState>) -> impl IntoResponse
  ```

- **After (0.8)**:

  ```rust
  async fn handler(State(state): State<AppState>) -> impl IntoResponse
  // Same syntax but internal changes
  ```

- **Impact**: Review all handlers using State

### 3. Router API Changes

- **Change**: Router building and merging APIs updated
- **Impact**: Complex router compositions need review
- **Files Affected**:
  - `src/api/routes.rs` (if exists)
  - Main API setup code

### 4. Error Handling Changes

- **Change**: Error response formatting updated
- **Impact**: Custom error types may need adjustment
- **Action**: Review `IntoResponse` implementations

### 5. Middleware Changes

- **Change**: Tower middleware integration updated
- **Impact**: Custom middleware needs review
- **Files to Check**:
  - Any custom middleware implementations
  - CORS setup
  - Authentication middleware

### 6. Body Type Changes

- **Change**: `Body` type from hyper 1.0
- **Impact**: Raw body handling needs update
- **Action**: Use new body stream APIs

## Implementation Steps

### Phase 1: Dependency Analysis (Risk: 3, Complexity: 3)

1. Check all axum-related dependencies:

   ```bash
   cargo tree | grep -E "axum|tower|hyper|http"
   ```

2. Identify version conflicts
3. Plan coordinated update of all related packages

### Phase 2: Preparation (Risk: 3, Complexity: 5)

1. Update Cargo.toml:

   ```toml
   axum = "0.8"
   tower = "0.5"
   tower-http = { version = "0.6", features = ["trace", "cors"] }
   ```

2. Create migration branch
3. Document current API endpoints

### Phase 3: Core Migration (Risk: 8, Complexity: 8)

#### Router Updates

1. Update router construction
2. Fix route merging patterns
3. Update fallback handlers

#### Handler Updates

1. Review all async handlers
2. Update extractor usage
3. Fix response types

#### Middleware Migration

1. Update Tower middleware
2. Fix CORS configuration
3. Update tracing middleware

### Phase 4: Testing (Risk: 5, Complexity: 5)

1. Test all API endpoints
2. Verify middleware chain
3. Check error responses
4. Load test for performance

### Phase 5: Integration (Risk: 5, Complexity: 3)

1. Test with real Discord bot
2. Verify agent communication
3. Check health endpoints

## Code Changes Required

### 1. Main Server Setup

```rust
// Before (0.7)
let app = Router::new()
    .route("/health", get(health_handler))
    .layer(tower_http::trace::TraceLayer::new_for_http())
    .with_state(app_state);

// After (0.8)
let app = Router::new()
    .route("/health", get(health_handler))
    .layer(tower_http::trace::TraceLayer::new_for_http())
    .with_state(app_state);
// Similar but check for subtle API changes
```

### 2. Error Handling

```rust
// May need updates to error response formatting
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Review and update
    }
}
```

### 3. Extractors

Review all custom extractors for API changes.

## Validation Checklist

- [ ] All routes respond correctly
- [ ] State management works
- [ ] Error responses formatted correctly
- [ ] Middleware chain executes properly
- [ ] CORS headers present
- [ ] Tracing/logging works
- [ ] No performance degradation
- [ ] All tests pass

## Testing Commands

```bash
# Compile check
cargo check --all-targets

# Run tests
cargo test

# Test specific API functionality
cargo test --test api_tests

# Manual testing
curl -X GET http://localhost:8080/health
```

## Rollback Plan

1. Revert Cargo.toml changes
2. Run `cargo update -p axum --precise 0.7.9`
3. Revert code changes
4. Rebuild and test

## Related Updates

These packages must be updated together:

- `tower`: 0.4 → 0.5
- `tower-http`: 0.5 → 0.6
- `hyper`: May need explicit version
- `http`: Ensure version 1.0

## Migration Resources

- [Axum 0.8 Migration Guide](https://github.com/tokio-rs/axum/blob/main/MIGRATION_GUIDE.md)
- [Hyper 1.0 Changes](https://github.com/hyperium/hyper/releases)
- [Tower 0.5 Updates](https://github.com/tower-rs/tower/releases)

## Notes

- The `tokio` feature is still required
- Check if we need additional features like `ws` for websockets
- Review timeout configurations
- Consider enabling `http2` feature if needed
