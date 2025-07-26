# Security Policy

## Supported Versions

We release patches for security vulnerabilities. Currently supported versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

If you discover a security vulnerability within Spiral Core, please send an email to <security@antispiral.dev> with:

- Type of issue (e.g., buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

## Preferred Languages

We prefer all communications to be in English.

## Disclosure Policy

When we receive a security bug report, we will:

1. Confirm the problem and determine the affected versions
2. Audit code to find any similar problems
3. Prepare fixes for all supported releases
4. Release security advisory and patches

## Security Hardening

This project implements several security measures:

### Authentication & Authorization

- ✅ Mandatory API key authentication (32+ character minimum)
- ✅ No authentication bypass options
- ✅ All endpoints require authentication (including health checks)

### Input Validation

- ✅ Comprehensive input sanitization
- ✅ Character whitelisting for task content
- ✅ Length limits on all user inputs
- ✅ HTML escaping to prevent XSS

### Network Security

- ✅ HTTPS-only for external API calls
- ✅ Claude API endpoint validation (whitelist only)
- ✅ Restrictive CORS policy
- ✅ Rate limiting on all endpoints

### Configuration Security

- ✅ No hardcoded credentials
- ✅ Mandatory environment variables for secrets
- ✅ Secure defaults (localhost only by default)
- ✅ Credential format validation

### Error Handling

- ✅ No internal error details exposed to clients
- ✅ Server-side logging only for debugging
- ✅ Generic error messages for API responses

## Security Tools

Our CI/CD pipeline includes:

- **cargo-audit**: Vulnerability scanning for dependencies
- **cargo-deny**: License compliance and security policy enforcement
- **Semgrep**: Static application security testing (SAST)
- **TruffleHog**: Secret scanning
- **Clippy**: Security-focused linting

## Dependencies

We regularly update dependencies and monitor for security advisories. To check for vulnerabilities locally:

```bash
cargo install cargo-audit
cargo audit
```

## Security Headers

When deploying, ensure these security headers are configured:

```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000; includeSubDomains
Content-Security-Policy: default-src 'self'
```

## Contact

For security concerns, contact: <security@antispiral.dev>

For general issues, use GitHub Issues.
