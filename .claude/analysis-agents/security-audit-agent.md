# Security Audit Agent

## Purpose

Analyze commits for security vulnerabilities and unsafe patterns.

## Trigger

Post-commit hook (runs on every commit affecting .rs, .toml, or .env files)

## Analysis Tasks

1. **Credential Scanning**
   - Hardcoded passwords, tokens, API keys
   - Sensitive data in logs
   - Exposed connection strings

2. **Dependency Audit**
   - Check for known vulnerabilities in Cargo.toml
   - Unsafe dependency patterns
   - Outdated security-critical crates

3. **Code Security Patterns**
   - SQL injection risks
   - Command injection possibilities
   - Path traversal vulnerabilities
   - Unsafe unwrap() usage in critical paths

4. **Permission Issues**
   - Overly permissive file operations
   - Unsafe network bindings (0.0.0.0)
   - Missing authentication checks

## Output Format

Generate `reports/security-audit-report.md` with:

- Security score (0-100)
- Vulnerability list by severity
- CVSS scores where applicable
- Remediation steps
- Compliance check results

## Discord Notification Triggers

**ALWAYS notify if:**

- Credentials found in code
- Critical vulnerability (CVSS 9.0+)
- Security score drops below 80

## Report Template

```markdown
# Security Audit Report

**Generated**: [DATE]
**Commit**: [HASH]
**Security Score**: X/100

## 🚨 Critical Findings
[Any immediate security risks]

## Vulnerabilities by Severity
### Critical
### High  
### Medium
### Low

## Remediation Priority
1. [Most critical fix]
2. [Next priority]

## Compliance Status
- [ ] No hardcoded secrets
- [ ] Dependencies up to date
- [ ] Safe error handling
- [ ] Input validation present
```
