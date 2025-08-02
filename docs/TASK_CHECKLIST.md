# Claude Task Execution Checklist

Use this checklist before starting any development task to ensure adherence to project standards.

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

## 🚨 Red Flags (Stop and Check Docs)

- Creating files outside established patterns
- Naming that doesn't match conventions
- Tests that don't fit the hybrid strategy
- Documentation that duplicates existing info
- Security implementations that bypass established patterns

## 📚 Quick Documentation Index

| Topic | Document | When to Read |
|-------|----------|-------------|
| Project Architecture | `CLAUDE.md` | Every task |
| Code Organization | `CLAUDE-colocation-patterns.md` | File/test creation |
| Coding Standards | `CODING_STANDARDS.md` | Writing code |
| API Patterns | `src/api/API_REFERENCE.md` | API work |
| Agent Development | `docs/CLAUDE-agents-*.md` | Agent features |
| Security Patterns | `SECURITY.md` | Security-related work |

This checklist should become second nature - internalize these patterns to maintain project consistency.
