# Colocation Patterns for Spiral Core

This document outlines the organizational patterns used in the Spiral Core project to keep related code, tests, and documentation together for better maintainability and discoverability.

## Philosophy

**Colocation Principle**: Keep related code, tests, and documentation as close as possible to reduce cognitive load and improve maintainability.

## Directory Structure Patterns

### 1. Submodule Pattern

Convert single files into submodules when they grow complex or need multiple types of related assets.

**Before:**

```
src/
├── api.rs
├── config.rs
└── agents/
    └── orchestrator.rs
```

**After:**

```
src/
├── api/
│   ├── mod.rs                    # Core implementation
│   ├── tests/
│   │   ├── unit.rs              # Rust unit/integration tests
│   │   └── hurl/                # HTTP API tests
│   │       ├── health.hurl
│   │       ├── tasks.hurl
│   │       ├── agents.hurl
│   │       ├── system.hurl
│   │       ├── auth.hurl
│   │       └── cors.hurl
│   └── API_REFERENCE.md         # Module-specific documentation
├── config/
│   ├── mod.rs                   # Core implementation
│   ├── tests.rs                 # Configuration validation tests
│   └── CONFIG_GUIDE.md          # Configuration documentation
└── agents/
    └── orchestrator/
        ├── mod.rs               # Core implementation
        ├── tests.rs             # Orchestrator-specific tests
        └── ORCHESTRATOR_GUIDE.md # Orchestrator documentation
```

### 2. Test Organization Hierarchy

```
module/
├── mod.rs                       # Implementation
├── tests/                       # Test directory
│   ├── unit.rs                 # Rust unit tests (fast, isolated)
│   ├── integration.rs          # Rust integration tests (slower, realistic)
│   └── hurl/                   # HTTP/API tests (external, contract-based)
│       ├── endpoint1.hurl
│       ├── endpoint2.hurl
│       └── README.md
└── MODULE_GUIDE.md             # Module documentation
```

## Test Type Guidelines

### Rust Unit Tests (`tests/unit.rs`)

- **Purpose**: Fast, isolated component testing
- **Scope**: Individual functions, struct methods, internal logic
- **Environment**: Mock dependencies, controlled inputs
- **Speed**: Milliseconds
- **Control**: Full environment control, dependency injection

```rust
// Example: src/validation/tests/unit.rs
#[test]
fn test_email_validation() {
    let validator = EmailValidator::new();
    assert!(validator.is_valid("test@example.com"));
    assert!(!validator.is_valid("invalid-email"));
}
```

### Rust Integration Tests (`tests/integration.rs`)

- **Purpose**: Component interaction testing
- **Scope**: Multiple modules working together
- **Environment**: Real dependencies, test configurations
- **Speed**: Seconds
- **Control**: Environment variables, test databases, mock services

```rust
// Example: src/api/tests/integration.rs
#[tokio::test]
async fn test_full_task_lifecycle() {
    let config = create_test_config();
    let orchestrator = AgentOrchestrator::new(config).await.unwrap();
    // Test complete workflow...
}
```

### Hurl HTTP Tests (`tests/hurl/`)

- **Purpose**: API contract and behavior validation
- **Scope**: External HTTP interface testing
- **Environment**: Running server instance
- **Speed**: Seconds to minutes
- **Control**: Limited to HTTP requests/responses

```hurl
# Example: src/api/tests/hurl/tasks.hurl
POST {{base_url}}/tasks
x-api-key: {{api_key}}
Content-Type: application/json

{
  "agent_type": "SoftwareDeveloper",
  "content": "Create hello world function"
}

HTTP 200
[Asserts]
jsonpath "$.task_id" exists
```

## Documentation Colocation

### Module Documentation

- **Location**: `module/MODULE_GUIDE.md`
- **Purpose**: Implementation details, usage examples, architecture decisions
- **Audience**: Developers working on the module

### API Documentation

- **Location**: `api/API_REFERENCE.md`
- **Purpose**: HTTP endpoint documentation, request/response schemas
- **Audience**: API consumers, integration developers

### Configuration Documentation

- **Location**: `config/CONFIG_GUIDE.md`
- **Purpose**: Environment variables, configuration options, deployment guides
- **Audience**: DevOps, deployment engineers

## When to Apply Patterns

### Single File → Submodule Migration Triggers

1. **File Size**: > 500 lines
2. **Test Complexity**: Multiple test types needed (unit + integration + HTTP)
3. **Documentation Needs**: Requires dedicated documentation
4. **Multiple Concerns**: File handles multiple related but distinct responsibilities

### Test Type Selection

| Test Type       | Use When                                              | Don't Use When                                |
| --------------- | ----------------------------------------------------- | --------------------------------------------- |
| **Unit**        | Testing individual functions, algorithms, validators  | Testing HTTP endpoints, database interactions |
| **Integration** | Testing component interactions, configuration loading | Testing external API contracts                |
| **Hurl**        | Testing HTTP API behavior, authentication flows       | Testing internal business logic               |

## Benefits of This Pattern

### 1. **Discoverability**

- Related code, tests, and docs are in the same location
- New developers can understand a module completely by exploring one directory

### 2. **Maintainability**

- Changes to implementation naturally prompt updates to colocated tests and docs
- Refactoring is contained within module boundaries

### 3. **Testing Strategy Clarity**

- Different test types have clear purposes and locations
- Easy to run specific test categories (unit vs integration vs HTTP)

### 4. **Documentation Freshness**

- Module-specific docs stay close to implementation
- Higher likelihood of keeping documentation current

## Implementation Guidelines

### Step 1: Identify Migration Candidates

```bash
# Find large files that might benefit from submodule pattern
find src -name "*.rs" -exec wc -l {} + | sort -n | tail -10
```

### Step 2: Create Submodule Structure

```bash
mkdir -p src/module/{tests/hurl}
mv src/module.rs src/module/mod.rs
```

### Step 3: Migrate Tests

- Move existing tests to appropriate test type directories
- Convert HTTP-testable functionality to Hurl tests
- Keep complex setup/teardown in Rust integration tests

### Step 4: Add Documentation

- Create module-specific documentation
- Document test organization and running procedures

## Anti-Patterns to Avoid

### ❌ Over-Colocation

Don't create submodules for simple, stable modules that don't need multiple test types or extensive documentation.

### ❌ Test Duplication

Don't test the same functionality in both Rust and Hurl tests. Choose the appropriate test type for each scenario.

### ❌ Documentation Sprawl

Don't create documentation files for every small module. Reserve dedicated docs for complex or user-facing modules.

### ❌ Inconsistent Patterns

Once you establish a pattern in your project, apply it consistently. Mixed organizational patterns create cognitive overhead.

## Tool Integration

### Cargo Test Integration

```toml
# Cargo.toml
[workspace]
members = ["src/*/"]

# Run specific test types
cargo test --package spiral-core-api unit
cargo test --package spiral-core-config
```

### Hurl Integration

```bash
# Run all Hurl tests for a module
hurl src/api/tests/hurl/*.hurl

# Run specific endpoint tests
hurl src/api/tests/hurl/tasks.hurl --variable base_url="$API_URL"
```

### CI/CD Integration

```yaml
# .github/workflows/test.yml
- name: Run Unit Tests
  run: cargo test unit

- name: Run Integration Tests
  run: cargo test integration

- name: Run API Tests
  run: |
    cargo run --bin spiral-core &
    sleep 5
    hurl src/api/tests/hurl/*.hurl --variable base_url="http://localhost:3000"
```

## Examples in Spiral Core

### API Module (`src/api/`)

- **Implementation**: `mod.rs` - HTTP server, routing, handlers
- **Unit Tests**: `tests/unit.rs` - Handler logic, request parsing
- **HTTP Tests**: `tests/hurl/` - Authentication, CORS, endpoint behavior
- **Docs**: `API_REFERENCE.md` - OpenAPI-style endpoint documentation

### Config Module (`src/config/`)

- **Implementation**: `mod.rs` - Environment variable loading, validation
- **Unit Tests**: `tests.rs` - Validation logic, default values
- **Docs**: `CONFIG_GUIDE.md` - Environment variable reference

This pattern scales from simple modules (config) to complex ones (api) while maintaining consistency and discoverability.
