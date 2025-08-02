# Claude Completion Checklist

**CRITICAL**: Never declare "Done" or "Complete" without running these checks!

## 🚨 MANDATORY Pre-Completion Checklist

Before EVER saying a task is complete, you MUST:

### 1. **Run Tests** (ALWAYS REQUIRED)

```bash
cargo test
```

- ✅ All tests must pass
- ❌ If ANY test fails, you are NOT done

### 2. **Check Compilation** (ALWAYS REQUIRED)

```bash
cargo check --all-targets
```

- ✅ Must compile without errors
- ⚠️ Warnings should be addressed

### 3. **Run Clippy** (REQUIRED for code changes)

```bash
cargo clippy --all-targets
```

- ✅ No errors allowed
- ⚠️ Address warnings when possible

### 4. **Check Formatting** (REQUIRED for code changes)

```bash
cargo fmt -- --check
```

- ✅ Code must be properly formatted

### 5. **Verify Documentation** (REQUIRED for public API changes)

```bash
cargo doc --no-deps
```

- ✅ Public APIs must have documentation

## 🎯 Task-Specific Validation

### For Bug Fixes

- ✅ Original failing test now passes
- ✅ No new tests are broken
- ✅ Added regression test if applicable

### For New Features

- ✅ Feature has tests
- ✅ All existing tests still pass
- ✅ Documentation updated

### For Refactoring

- ✅ All tests pass (same as before)
- ✅ No behavior changes
- ✅ Performance not degraded

## 📝 Completion Statement Template

When declaring completion, use this format:

```
✅ Task Complete - All Validations Passed

**Test Results:**
- `cargo test`: ✅ All tests passing
- `cargo check`: ✅ No compilation errors
- `cargo clippy`: ✅ No errors (X warnings)
- `cargo fmt`: ✅ Code formatted

**Changes Made:**
1. [List specific changes]
2. [...]

**Tests Added/Modified:**
- [List any test changes]
```

## ❌ NEVER Say "Done" If

1. You haven't run `cargo test`
2. Any test is failing
3. Code doesn't compile
4. You made changes but didn't verify them

## 🔄 If Tests Fail After Changes

1. **STOP** - Do not declare completion
2. **INVESTIGATE** - Read the error messages
3. **FIX** - Address the root cause
4. **RETEST** - Run tests again
5. **ITERATE** - Repeat until all pass

## 💡 Common Pitfalls

### "I fixed the code but didn't run tests"

- **Problem**: Changes often have unintended side effects
- **Solution**: ALWAYS run `cargo test` after ANY change

### "I assumed the tests would pass"

- **Problem**: Assumptions lead to broken builds
- **Solution**: Verify with actual test runs

### "The change was trivial"

- **Problem**: Even small changes can break things
- **Solution**: No change is too small to test

### "I only changed documentation"

- **Problem**: Doc tests can fail too
- **Solution**: Run tests even for doc changes

## 🚀 Quick Validation Command

Run all checks at once:

```bash
cargo test && cargo check --all-targets && cargo clippy --all-targets && cargo fmt -- --check
```

Only declare "complete" if ALL commands succeed!
