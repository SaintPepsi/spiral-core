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
- ✅ **Universal Discord Authorization**: ALL `!spiral` commands and spiral mentions require authorization
- ✅ **Whitelist-based Access**: Only pre-configured Discord user IDs can access spiral functionality
- ✅ **Deny-by-Default**: Unauthorized users receive Lordgenome denial quotes, no access granted

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

### Discord Security Architecture

- ✅ **Universal Authorization Check**: All spiral interactions require pre-authorization
- ✅ **Multi-layer Validation**: Message security validation before authorization check
- ✅ **Protected Command Scope**:
  - All `!spiral` commands (admin, security, debug, etc.)
  - All spiral agent mentions (`@SpiralDev`, `@SpiralPM`, etc.)
  - All spiral role mentions
- ✅ **Bot Self-Exception**: Bot's own messages bypass authorization to prevent self-blocking
- ✅ **Secure Configuration**: Authorized users loaded from environment variables only
- ✅ **Audit Logging**: All authorization attempts logged with structured events
- ✅ **Rate Limiting**: Integrated with Discord message security validation
- ✅ **Intent Classification**: Security-aware message classification before processing

#### Discord Authorization Setup

Configure authorized Discord user IDs in environment variables:

```bash
# Required: Comma-separated list of Discord user IDs
DISCORD_AUTHORIZED_USERS=123456789012345678,987654321098765432

# Optional: Discord bot configuration
DISCORD_TOKEN=your_discord_bot_token
DISCORD_PREFIX=!spiral
```

**Security Note**: Never commit Discord user IDs to version control. Use environment variables or secure secret management.

### Error Handling

- ✅ No internal error details exposed to clients
- ✅ Server-side logging only for debugging
- ✅ Generic error messages for API responses
- ✅ **Contextual Denial Messages**: Unauthorized access returns thematic Lordgenome quotes

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
