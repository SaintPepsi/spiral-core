# Rust Package Update Process

## Overview
This document defines the standard process for updating Rust dependencies in the Spiral Core project. Package updates require careful planning to avoid breaking changes and ensure system stability.

## Risk and Complexity Assessment

All package updates must be assessed using the Fibonacci scale (1, 2, 3, 5, 8, 13, ∞):

### Risk Levels
- **1**: Patch version, no API changes
- **2**: Minor version, backward compatible
- **3**: Minor version with new features to adopt  
- **5**: Major version with some breaking changes
- **8**: Major version with significant breaking changes
- **13**: Core dependency requiring architectural changes
- **∞**: Complete rewrite needed

### Complexity Levels
- **1**: Drop-in replacement
- **2**: Configuration changes only
- **3**: Minor code adjustments
- **5**: Multiple file changes required
- **8**: Significant refactoring needed
- **13**: Architecture adjustments required
- **∞**: Fundamental redesign needed

## Standard Update Process

### Phase 1: Assessment and Planning

1. **Identify Update Candidates**
   ```bash
   cargo outdated
   ```

2. **Create Update Plan Document**
   - File: `docs/PACKAGE_UPDATE_[PACKAGE_NAME].md`
   - Include risk and complexity assessment
   - Document all breaking changes
   - List affected files
   - Define rollback strategy

3. **Review Dependencies**
   ```bash
   cargo tree -i [package_name]
   ```

### Phase 2: Pre-Update Validation

1. **Ensure Clean State**
   ```bash
   cargo check --all-targets
   cargo test
   cargo clippy --all-targets
   ```

2. **Create Git Snapshot**
   ```bash
   git checkout -b update/[package_name]-[version]
   git add -A
   git commit -m "chore: Pre-update snapshot before [package] update"
   ```

### Phase 3: Implementation

1. **Update Package**
   ```bash
   cargo update -p [package_name]
   # OR for major version
   # Edit Cargo.toml manually
   ```

2. **Initial Compilation Check**
   ```bash
   cargo check --all-targets 2>&1 | tee update_errors.log
   ```

3. **Fix Compilation Errors**
   - Address errors systematically
   - Document non-obvious changes
   - Follow existing code patterns

4. **Update Tests**
   - Fix broken tests
   - Add tests for new functionality
   - Ensure coverage maintained

### Phase 4: Validation

1. **Run Full Test Suite**
   ```bash
   cargo test
   cargo test --doc
   cargo test --examples
   ```

2. **Check Code Quality**
   ```bash
   cargo fmt -- --check
   cargo clippy --all-targets -- -D warnings
   ```

3. **Manual Testing**
   - Test primary functionality
   - Verify external integrations
   - Check performance metrics

### Phase 5: Documentation

1. **Update Documentation**
   - Update README if needed
   - Update API documentation
   - Document migration steps

2. **Create Migration Notes**
   - File: `docs/migrations/[package_name]_[old_version]_to_[new_version].md`
   - Include code examples
   - List common pitfalls

### Phase 6: Review and Merge

1. **Create Pull Request**
   - Reference update plan document
   - Include test results
   - Document any behavior changes

2. **Post-Merge Monitoring**
   - Monitor for issues
   - Be ready to rollback
   - Document any discoveries

## Automated Update Agent Specification

### Purpose
A specialized Claude Code agent for automating package update analysis and implementation.

### Agent Location
`.claude/utility-agents/package-updater.md`

### Capabilities

1. **Analysis Phase**
   - Parse `cargo outdated` output
   - Fetch changelogs from crates.io
   - Identify breaking changes
   - Assess risk and complexity

2. **Planning Phase**
   - Generate update plan documents
   - Identify affected files
   - Create implementation strategy
   - Define test requirements

3. **Implementation Phase**
   - Execute cargo update commands
   - Parse compilation errors
   - Suggest code fixes
   - Update import statements

4. **Validation Phase**
   - Run test suites
   - Check for regressions
   - Verify functionality
   - Generate report

### Agent Prompt Template
```markdown
You are a Rust package update specialist for the Spiral Core project.

## Your Task
Analyze and assist with updating the [PACKAGE_NAME] package from [OLD_VERSION] to [NEW_VERSION].

## Process
1. Analyze breaking changes from changelog
2. Identify all affected files using `cargo tree` and grep
3. Create detailed update plan following docs/PACKAGE_UPDATE_PROCESS.md
4. Implement necessary code changes
5. Ensure all tests pass

## Constraints
- Follow existing code patterns
- Maintain backward compatibility where possible
- Document all non-obvious changes
- Never skip tests or validation

## Risk Assessment
Use Fibonacci scale (1,2,3,5,8,13,∞) for risk and complexity.

## Output Format
Provide structured update plan with:
- Risk and complexity scores
- List of breaking changes
- Required code modifications
- Test requirements
- Rollback procedure
```

## Common Package Update Patterns

### 1. Async Runtime Updates (tokio)
- Check async syntax changes
- Review spawn patterns
- Validate timer APIs

### 2. Web Framework Updates (axum, actix-web)
- Review handler signatures
- Check middleware APIs
- Validate routing patterns

### 3. Serialization Updates (serde)
- Check derive macro changes
- Review custom implementations
- Validate JSON output

### 4. Database Updates (sqlx, diesel)
- Review query syntax
- Check migration compatibility
- Validate connection pools

## Rollback Procedures

### Quick Rollback
```bash
git checkout main
git branch -D update/[package_name]-[version]
```

### Partial Rollback
```bash
cargo update -p [package_name] --precise [old_version]
git checkout -- Cargo.lock
```

### Emergency Rollback
```bash
git revert [commit_hash]
cargo update
cargo test
```

## Best Practices

1. **One Package at a Time**
   - Don't update multiple major dependencies simultaneously
   - Exception: Tightly coupled packages (e.g., tokio ecosystem)

2. **Test in Isolation**
   - Create minimal reproduction cases
   - Test edge cases explicitly

3. **Document Everything**
   - Why the update is needed
   - What changed
   - How to migrate

4. **Incremental Updates**
   - Don't skip major versions
   - Update through each major version

5. **Monitor After Deployment**
   - Watch for performance changes
   - Monitor error rates
   - Check memory usage

## Package-Specific Guides

- [Reqwest Update Guide](PACKAGE_UPDATE_REQWEST.md)
- [Axum Update Guide](PACKAGE_UPDATE_AXUM.md)
- [Tokio Update Guide](PACKAGE_UPDATE_TOKIO.md) (TODO)
- [Serde Update Guide](PACKAGE_UPDATE_SERDE.md) (TODO)

## Validation Checklist Template

For each package update:

- [ ] Risk and complexity assessed
- [ ] Update plan documented
- [ ] All compilation errors resolved
- [ ] All tests pass
- [ ] Clippy warnings addressed
- [ ] Documentation updated
- [ ] Manual testing completed
- [ ] Performance verified
- [ ] Rollback plan tested
- [ ] PR created with detailed description