# Double-Check Items for Production Readiness

This document lists important items to verify before deploying Spiral Core to production.

## 🔐 Security Configuration

### Rate Limiting

- [ ] Review rate limit constants in `src/rate_limit.rs`
  - `REQUESTS_PER_MINUTE` - Currently set for development
  - `TASK_REQUESTS_PER_MINUTE` - May need adjustment based on expected load
  - `CLEANUP_INTERVAL` - Consider impact on memory usage

### Discord Bot Permissions

- [ ] Verify Discord bot token has appropriate permissions
  - Read Messages
  - Send Messages
  - Manage Messages (for reactions)
  - Create roles (if using role-based features)
- [ ] Check authorized user list in configuration
  - Ensure only trusted users are authorized for self-updates
  - Review `AUTHORIZED_USERS` environment variable

## 🔧 Environment Configuration

### Required Environment Variables

- [ ] Verify all required environment variables are set:
  - `DISCORD_TOKEN` - Bot authentication
  - `CLAUDE_API_KEY` - Claude Code API access
  - `AUTHORIZED_USERS` - Comma-separated list of Discord user IDs
  - `DATABASE_URL` (if using persistence)
  - `REDIS_URL` (if using Redis)

### Configuration Files

- [ ] Update `.env.example` with all required variables
- [ ] Document default values and acceptable ranges
- [ ] Ensure sensitive values are not committed to repository

## 🛡️ API Security

### CORS Configuration

- [ ] Review CORS settings in API module
  - Allowed origins should be restricted
  - Consider environment-specific CORS policies
  - Verify preflight request handling

### Authentication Middleware

- [ ] Ensure authentication is applied to all protected endpoints
- [ ] Verify API key validation is constant-time
- [ ] Check session timeout values are appropriate

## 📊 Resource Limits

### Queue Management

- [ ] Review queue size limits:
  - `MAX_QUEUE_SIZE` in `constants.rs`
  - Self-update queue limit (`MAX_QUEUE_SIZE` in self_update module)
  - Consider memory implications of queue sizes

### Timeout Values

- [ ] Verify timeout configurations:
  - Claude Code API timeout
  - Discord message handling timeout
  - Task execution timeout
  - Shutdown grace period

### Memory Management

- [ ] Check cache sizes and eviction policies:
  - Session cache limits
  - Message state retention
  - Circuit breaker state storage

## 🚀 Performance Considerations

### Concurrent Operations

- [ ] Review thread pool sizes
- [ ] Check concurrent request limits
- [ ] Verify database connection pool settings

### Monitoring

- [ ] Ensure monitoring endpoints are accessible
- [ ] Verify health check includes all critical components
- [ ] Set up alerts for circuit breaker trips

## 📝 Operational Readiness

### Logging

- [ ] Verify log levels are appropriate for production
- [ ] Ensure sensitive data is not logged
- [ ] Check log rotation is configured

### Deployment

- [ ] Review Dockerfile for production optimizations
- [ ] Verify graceful shutdown handling
- [ ] Check rollback procedures for self-updates

### Documentation

- [ ] Ensure README has deployment instructions
- [ ] Document all configuration options
- [ ] Include troubleshooting guide

## 🧪 Testing

### Load Testing

- [ ] Test rate limiting under load
- [ ] Verify queue behavior at capacity
- [ ] Check memory usage under sustained load

### Security Testing

- [ ] Run security scanner on API endpoints
- [ ] Test authorization bypass attempts
- [ ] Verify input validation on all user inputs

### Integration Testing

- [ ] Test Discord bot in real server environment
- [ ] Verify Claude Code integration with rate limits
- [ ] Test self-update rollback procedures

## Notes

- Review this checklist before each major release
- Update items as system requirements change
- Consider automating some checks in CI/CD pipeline
