# Claude Agent Templates Index

This directory contains specialized agent templates for various tasks in the Spiral Core system.

## Directory Structure

```
.claude/
├── validation-agents/    # Two-phase validation pipeline agents
│   ├── phase1/          # Advanced Quality Assurance (AQA) agents
│   ├── phase2/          # Core Rust Compliance Checks (CRCC) agents
│   └── analysis/        # Pipeline outcome analysis agents
├── utility-agents/      # General-purpose utility agents
└── settings.local.json  # Local Claude settings
```

## Validation Pipeline Agents

### Phase 1: Advanced Quality Assurance (AQA)

These agents perform comprehensive quality checks before code changes are accepted.

| Agent | File | Purpose |
|-------|------|---------|
| Code Review | `validation-agents/phase1/code-review-standards.md` | Reviews code against SOLID, DRY, and project standards |
| Testing | `validation-agents/phase1/comprehensive-testing.md` | Analyzes test coverage and pressure points |
| Security | `validation-agents/phase1/security-audit.md` | Identifies vulnerabilities and unsafe patterns |
| Integration | `validation-agents/phase1/system-integration.md` | Verifies no regressions or breaking changes |

### Phase 2: Core Rust Compliance Checks (CRCC)

These agents handle Rust-specific compilation and compliance issues.

| Agent | File | Purpose |
|-------|------|---------|
| Compilation | `validation-agents/phase2/compilation-fixer.md` | Fixes `cargo check` errors |
| Test Failures | `validation-agents/phase2/test-failure-analyzer.md` | Analyzes and fixes `cargo test` failures |
| Formatting | `validation-agents/phase2/formatting-fixer.md` | Applies `cargo fmt` standards |
| Linting | `validation-agents/phase2/clippy-resolver.md` | Resolves `cargo clippy` warnings |
| Documentation | `validation-agents/phase2/doc-builder.md` | Fixes `cargo doc` issues |

### Analysis Agents

These agents analyze the overall pipeline execution results.

| Agent | File | Purpose |
|-------|------|---------|
| Success | `validation-agents/analysis/success-analyzer.md` | Analyzes clean passes (no retries) |
| Partial Success | `validation-agents/analysis/success-with-issues-analyzer.md` | Analyzes passes that needed fixes |
| Failure | `validation-agents/analysis/failure-analyzer.md` | Analyzes failed runs after 3 iterations |

## Utility Agents

General-purpose agents for various development tasks.

| Agent | File | Purpose |
|-------|------|---------|
| Agent Validator | `utility-agents/agent-validator.md` | Validates agent prompt quality and structure |
| Claude Improver | `utility-agents/claude-improver.md` | Improves code quality and refactoring |
| Const Usage Checker | `utility-agents/const-usage-checker.md` | Ensures proper const usage after checklist completion |
| Documentation Analyzer | `utility-agents/documentation-lean-analyzer.md` | Analyzes documentation for clarity and completeness |
| DRY Analyzer | `utility-agents/dry-analyzer.md` | Detection and elimination of code duplication |
| Package Updater | `utility-agents/package-updater-rust.md` | Rust dependency management and updates |

## Usage in Self-Update System

The validation agents are automatically invoked during self-updates:

1. **Planning Phase**: UpdatePlanner uses Claude Code to create implementation plan
2. **Implementation Phase**: UpdateExecutor uses Claude Code to implement changes
3. **Validation Phase**: ValidationPipeline invokes agents in sequence:
   - Phase 1 agents run quality checks
   - Phase 2 agents run compliance checks
   - If Phase 2 requires fixes, loop back to Phase 1 (max 3 iterations)
   - Analysis agents provide final report

## Creating New Agents

When creating new agent templates:

1. **Focus**: Each agent should have ONE specific job
2. **Structure**: Use consistent markdown format with clear sections
3. **Context**: Define what input the agent expects
4. **Output**: Specify exact output format required
5. **Examples**: Include concrete examples when possible

## Agent Prompt Best Practices

1. **Clear Role Definition**: Start with "You are a..."
2. **Specific Task**: State exactly what the agent should do
3. **Success Criteria**: Define measurable pass/fail conditions
4. **Output Format**: Specify JSON, markdown, or structured text
5. **Error Handling**: Include what to do when things go wrong
6. **No Assumptions**: Be explicit about all requirements

## Pipeline Context Structure

All validation agents receive a standardized `PipelineContext` object. See `validation-agents/README.md` for the complete structure.

## Integration Points

- **UpdateExecutor**: Spawns agents via Claude Code integration
- **ValidationPipeline**: Orchestrates two-phase validation
- **ProgressReporter**: Reports agent execution status to Discord
- **SystemLock**: Ensures only one validation runs at a time

## Future Enhancements

Planned agent templates:

- Performance profiling agent
- Dependency update agent
- Breaking change detector
- Migration script generator
- Test coverage analyzer

---

For detailed information about each agent category, see the README.md files in their respective directories.
