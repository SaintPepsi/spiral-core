# Reqwest Package Update Plan: 0.11 → 0.12

## Overview

Updating reqwest from 0.11.x to 0.12.x introduces breaking changes that need careful migration. This document outlines all required changes.

## Risk Assessment

- **Risk Level**: 5 (Fibonacci scale)
- **Complexity**: 5 (Fibonacci scale)
- **Risk Factors**:
  - Breaking API changes in HTTP client configuration
  - Dependency version conflicts with other crates
  - Potential runtime behavior changes

## Breaking Changes

### 1. Minimum Rust Version

- **0.11**: Rust 1.63+
- **0.12**: Rust 1.70+
- **Action**: Ensure Rust toolchain is updated

### 2. TLS Backend Changes

- **Change**: Default TLS backend changed from native-tls to rustls
- **Impact**: May affect certificate validation behavior
- **Action**: Explicitly specify TLS backend if needed:

  ```toml
  reqwest = { version = "0.12", default-features = false, features = ["rustls-tls"] }
  # OR
  reqwest = { version = "0.12", default-features = false, features = ["native-tls"] }
  ```

### 3. Client Builder API Changes

- **Change**: Some builder methods now consume self instead of &mut self
- **Impact**: Method chaining patterns may need adjustment
- **Files Affected**:
  - `src/claude_code/client.rs` (if using ClientBuilder)
  - Any test files using mock clients

### 4. Header API Changes

- **Change**: Header value construction is more strict
- **Impact**: Raw header value construction needs review
- **Action**: Use `HeaderValue::from_static()` or proper constructors

### 5. Error Type Changes

- **Change**: Error variants have been reorganized
- **Impact**: Error matching patterns need update
- **Files to Check**:
  - Any files with `match` on `reqwest::Error`
  - Error conversion implementations

## Implementation Steps

### Phase 1: Analysis (Risk: 2, Complexity: 2)

1. Run `cargo tree -i reqwest` to identify all dependents
2. Check for version constraints in dependent crates
3. Review all `use reqwest::` statements in codebase

### Phase 2: Preparation (Risk: 3, Complexity: 3)

1. Create feature branch for update
2. Update Cargo.toml with new version
3. Run `cargo check` to identify compilation errors

### Phase 3: Code Migration (Risk: 5, Complexity: 5)

1. Fix Client builder patterns
2. Update error handling code
3. Review and update header construction
4. Adjust async runtime compatibility if needed

### Phase 4: Testing (Risk: 3, Complexity: 3)

1. Run all unit tests
2. Test HTTP client functionality manually
3. Verify TLS/SSL connections work correctly
4. Check proxy support if used

### Phase 5: Integration Testing (Risk: 5, Complexity: 3)

1. Test Claude Code API integration
2. Verify Discord webhook requests (if any)
3. Test any external API calls

## Validation Checklist

- [ ] All compilation errors resolved
- [ ] All tests pass
- [ ] TLS connections work in both dev and prod environments
- [ ] Error handling correctly migrated
- [ ] No performance regressions observed
- [ ] Headers are correctly constructed
- [ ] Timeouts and retries work as expected

## Rollback Plan

If issues are discovered:

1. Revert Cargo.toml change
2. Run `cargo update -p reqwest --precise 0.11.27`
3. Rebuild and redeploy

## Dependencies to Consider

Other crates that might need updates alongside reqwest:

- `hyper`: May need compatible version
- `tokio`: Ensure async runtime compatibility
- `tower`: If using middleware

## Testing Commands

```bash
# Check for breaking changes
cargo check --all-targets

# Run tests
cargo test

# Check specific functionality
cargo test --test http_client_tests

# Verify with clippy
cargo clippy --all-targets
```

## Notes

- Consider enabling `gzip` and `brotli` features for compression support
- The `json` feature is still required for JSON support
- Review proxy configuration if your deployment uses proxies
