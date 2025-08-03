# Validation Pipeline Specialist Agents

This directory contains specialized Claude Code agents for the two-phase validation pipeline.

## Directory Structure

- `phase1/` - Advanced Quality Assurance agents
  - `code-review-standards.md` - Code review & standards compliance
  - `comprehensive-testing.md` - Testing analysis focused on pressure points
  - `security-audit.md` - Security vulnerability analysis
  - `system-integration.md` - Integration verification
  
- `phase2/` - Core Rust Compliance agents
  - `compilation-fixer.md` - Fixes cargo check errors
  - `test-failure-analyzer.md` - Analyzes and fixes failing tests
  - `formatting-fixer.md` - Applies cargo fmt
  - `clippy-resolver.md` - Fixes clippy warnings
  - `doc-builder.md` - Fixes documentation build errors
  
- `analysis/` - Pipeline outcome analysis agents
  - `success-analyzer.md` - Happy path success analysis
  - `success-with-issues-analyzer.md` - Success with retries analysis
  - `failure-analyzer.md` - Complete failure analysis

## Agent Design Principles

1. **Single Purpose** - Each agent has one specific job
2. **Clear Prompts** - Precise, actionable instructions
3. **Measurable Success** - Clear pass/fail criteria
4. **Context Aware** - Agents receive relevant context from previous phases
