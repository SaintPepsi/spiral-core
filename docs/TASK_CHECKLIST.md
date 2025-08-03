# Task Execution Checklist

Check these items before coding to avoid common mistakes.

## 📋 Pre-Task Documentation Review

### Required Reading (Every Time)

- [ ] **CLAUDE.md** - Project overview, architecture, and current phase
- [ ] **[COLOCATION_PATTERNS.md](COLOCATION_PATTERNS.md)** - File organization and test structure
- [ ] **Relevant module documentation** for the area you're working on

### Pattern Verification

- [ ] **Naming**: Does this follow the ANACONDA_CASE pattern for docs?
- [ ] **Structure**: Am I using the correct colocation pattern (single file vs submodule)?
- [ ] **Tests**: Am I putting tests in the right location (unit.rs, integration.rs, hurl/)?
- [ ] **Documentation**: Does this need module-specific documentation?

### Code Standards Check

- [ ] **SOLID Principles**: Single Responsibility, Open-Closed, etc.
- [ ] **DRY Principle**: Not duplicating logic or information
- [ ] **SID Naming**: Short, Intuitive, Descriptive names
- [ ] **Early Return Pattern**: Using early returns with negative conditions ([details](CODING_STANDARDS.md#early-return-pattern-required))
- [ ] **Flow Comments**: Does this method have a clear flow of action? Then it needs flow comments at the top
- [ ] **Clutter Prevention**: Am I breaking up large functions/files? Are responsibilities clear?
- [ ] **No Bullshit Code**: Am I faking any status, metrics, or functionality?
- [ ] **Security**: Following established security patterns
- [ ] **Error Handling**: Using project error handling patterns

### Module-Specific Checks

- [ ] **API Module**: Are HTTP tests in `src/api/tests/hurl/`?
- [ ] **Config Module**: Are validation tests colocated?
- [ ] **Agent Module**: Following orchestrator patterns?

## 🎯 Task Execution Guidelines

### Development Flow

1. **Plan** → Use TodoWrite to track multi-step tasks
2. **Read** → Check existing implementations for patterns
3. **Implement** → Follow established conventions
4. **Test** → Use appropriate test type (unit/integration/hurl)
5. **Document** → Update relevant docs if needed

### When to Create New Modules

- File > 500 lines
- Multiple test types needed
- Requires dedicated documentation
- Multiple related concerns

### When to Create Documentation

- User-facing modules (API, Config)
- Complex implementation patterns
- Cross-team coordination needed
- Architecture decision explanations

### Decluttering Checkpoints

**Before implementing, ask:**

- [ ] **Function Size**: Is this function >50 lines? Should it be broken down?
- [ ] **Single Responsibility**: Does this function/module do one thing well?
- [ ] **Repetition**: Am I copy-pasting similar logic instead of creating a helper?
- [ ] **Clear Purpose**: Can I explain what this does in one sentence?

**Stop immediately if you see these problems:**

- Functions >100 lines (break into smaller functions)
- Files >500 lines (consider module separation)
- Copy-paste code (extract common functionality)
- Mixed concerns (UI logic + business logic)
- Unclear naming (variables like `data`, `temp`, `result`)

**Red flags for bullshit code:**

- Hardcoded status messages ("🟢 Active" without checks)
- Fake metrics ("Memory: Efficient" without measurement)
- Placeholder data (returning static values as if they're dynamic)
- Mock implementations in production code
- Features that look functional but do nothing

## 🚨 Red Flags (Stop and Check Docs)

- Creating files outside established patterns
- Naming that doesn't match conventions
- Tests that don't fit the hybrid strategy
- Documentation that duplicates existing info
- Security implementations that bypass established patterns

## 📚 Documentation Quick Reference

**Read this when working on:**

- **Any task**: `CLAUDE.md`
- **File organization**: `COLOCATION_PATTERNS.md`
- **Writing code**: `CODING_STANDARDS.md`
- **API work**: `src/api/API_REFERENCE.md`
- **Agent features**: `docs/CLAUDE-agents-*.md`
- **Security work**: `SECURITY_POLICY.md`

Internalize these patterns - they prevent most common problems.
