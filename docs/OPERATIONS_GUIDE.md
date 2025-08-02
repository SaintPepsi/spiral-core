# Spiral Core Operations Guide

**Purpose**: Comprehensive guide for operating and maintaining Spiral Core in production  
**Context**: Startup/shutdown procedures, monitoring, and troubleshooting  
**Updated**: 2025-07-27

## 🚀 Startup Procedures

### Pre-Startup Checklist

Before starting Spiral Core, ensure:

- [ ] **Environment Variables**: All required environment variables are set
- [ ] **Claude API Key**: Valid API key in `.env` or environment
- [ ] **Disk Space**: At least 5GB free for workspaces
- [ ] **Network**: Internet connectivity for Claude Code API
- [ ] **Permissions**: Write access to workspace directory

### Startup Sequence

The system follows a structured 5-phase startup process:

#### Phase 1: Logging Initialization

```bash
# System initializes tracing/logging
# Logs version, PID, and startup timestamp
INFO Starting Spiral Core Agent Orchestration System
INFO Version: 0.1.0
INFO PID: 12345
```

#### Phase 2: Configuration Loading

```bash
# Loads configuration from environment and files
INFO Loading configuration...
INFO Configuration loaded successfully
```

Common issues:

- Missing `.env` file → Create from `.env.example`
- Invalid TOML syntax → Check `config.toml` formatting
- Missing required fields → Review error message for specifics

#### Phase 3: Startup Validations

```bash
INFO Performing startup validations...
```

The system validates:

1. **Claude Code CLI**: Binary exists and is executable
2. **API Configuration**: API key present if API enabled
3. **Workspace Permissions**: Write access to workspace directory
4. **Discord Token**: Valid token if Discord enabled

Validation failures will abort startup with clear error messages.

#### Phase 4: Component Initialization

```bash
INFO Initializing agent orchestrator...
INFO Agent orchestrator initialized successfully
INFO Initializing API server...
INFO API server initialized successfully
```

Components initialize in order:

1. Agent Orchestrator (manages all agents)
2. API Server (HTTP endpoints)
3. Discord Bot (if enabled)

#### Phase 5: Signal Handler Setup

```bash
INFO Spiral Core startup complete - all systems operational
```

System is now ready to accept requests and handle shutdown signals.

### Normal Startup Command

```bash
# Development
cargo run --bin spiral-core

# Production (with release optimizations)
cargo run --release --bin spiral-core

# Docker container
docker run -p 3000:3000 spiral-core

# With custom config
SPIRAL_CONFIG_PATH=/etc/spiral/config.toml cargo run --bin spiral-core
```

### Startup Logs Analysis

Healthy startup should complete in <5 seconds and show:

- No ERROR level messages
- All validations passed
- All components initialized
- "all systems operational" message

Warning signs:

- Startup taking >30 seconds
- Multiple WARN messages
- Failed validations
- Component initialization errors

## 🛑 Shutdown Procedures

### Graceful Shutdown

The system supports graceful shutdown via multiple signals:

#### Supported Signals

- **SIGINT (Ctrl+C)**: Interactive shutdown
- **SIGTERM**: Process manager shutdown (systemd, Docker)
- **API Endpoint**: POST `/admin/shutdown` (if enabled)

#### Shutdown Sequence

1. **Signal Reception**

```bash
INFO Shutdown signal received, initiating graceful shutdown...
```

2. **Task Completion Grace Period** (30 seconds)

```bash
INFO Waiting for in-flight requests to complete (max 30s)...
INFO Waiting for 3 active tasks to complete (5s elapsed)
INFO Waiting for 1 active tasks to complete (12s elapsed)
INFO All tasks completed
```

3. **Resource Cleanup**

```bash
INFO Cleaning up Claude Code workspaces...
INFO Cleaned up 5 old Claude Code workspaces
```

4. **Final Shutdown**

```bash
INFO Flushing logs...
INFO Spiral Core shutdown complete
```

### Emergency Shutdown

If graceful shutdown hangs or takes too long:

```bash
# Force kill (data loss possible)
kill -9 <PID>

# Docker force stop
docker kill <container_id>
```

⚠️ **Warning**: Force shutdown may result in:

- Incomplete tasks
- Orphaned workspaces
- Corrupted state files

### Shutdown Monitoring

Monitor shutdown progress via:

- Log output (most detailed)
- Process exit code (0 = clean, non-zero = issues)
- Workspace cleanup metrics

## 📊 Health Monitoring

### Health Check Endpoint

```bash
# Basic health check
curl http://localhost:3000/health

# Expected response
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

### System Status Endpoint

```bash
# Detailed system status (requires API key)
curl -H "x-api-key: $API_KEY" http://localhost:3000/system/status

# Response includes:
{
  "agents": {
    "SoftwareDeveloper": {
      "is_busy": false,
      "tasks_completed": 42,
      "tasks_failed": 2,
      "average_execution_time": 15.3
    }
  },
  "queue_length": 3,
  "system_uptime": 3600.5
}
```

### Key Metrics to Monitor

1. **Task Queue Length**: Should stay <10 under normal load
2. **Agent Busy Status**: No agent stuck busy >30 minutes
3. **Task Failure Rate**: <5% failure rate expected
4. **Workspace Count**: Should stay <100 active workspaces
5. **Memory Usage**: Should stay <2GB under normal load

## 🔧 Troubleshooting

### Common Startup Issues

#### Claude Code CLI Not Found

```
ERROR Claude Code CLI not found
```

**Solution**: Install Claude Code CLI

```bash
npm install -g @anthropic-ai/claude-code
```

#### API Key Missing

```
ERROR Failed to load configuration: Missing CLAUDE_API_KEY
```

**Solution**: Set environment variable

```bash
export CLAUDE_API_KEY=sk-ant-api03-...
```

#### Workspace Permission Denied

```
ERROR Workspace directory not writable: /var/spiral/workspaces
```

**Solution**: Fix directory permissions

```bash
sudo chown $USER:$USER /var/spiral/workspaces
chmod 755 /var/spiral/workspaces
```

### Common Runtime Issues

#### High Memory Usage

**Symptoms**: Memory >4GB, system slowdown  
**Causes**: Too many workspaces, memory leak  
**Solution**:

1. Check workspace count
2. Trigger manual cleanup
3. Restart if necessary

#### Stuck Tasks

**Symptoms**: Tasks in "in_progress" >1 hour  
**Causes**: Claude API timeout, agent crash  
**Solution**:

1. Check agent status endpoint
2. Review task logs
3. Restart affected agent

#### API Rate Limiting

**Symptoms**: 429 errors, "rate limit" in logs  
**Causes**: Too many Claude API calls  
**Solution**:

1. Reduce concurrent tasks
2. Implement backoff strategy
3. Check API usage dashboard

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=debug cargo run --bin spiral-core
```

Enable trace logging (very verbose):

```bash
RUST_LOG=trace cargo run --bin spiral-core
```

## 🔄 Maintenance Tasks

### Daily Maintenance

1. **Check Logs**: Review for errors/warnings
2. **Monitor Metrics**: Queue length, failure rate
3. **Workspace Cleanup**: Ensure old workspaces deleted

### Weekly Maintenance

1. **Performance Review**: Analyze task execution times
2. **Storage Audit**: Check disk usage trends
3. **Security Review**: Check for suspicious activity

### Monthly Maintenance

1. **Update Dependencies**: Check for security updates
2. **Backup Configuration**: Save config files
3. **Capacity Planning**: Review growth trends

## 📈 Performance Tuning

### Configuration Optimization

```toml
# Adjust in config.toml or environment

# Increase concurrent agents
SPIRAL_MAX_CONCURRENT_TASKS=10

# Reduce workspace retention
SPIRAL_WORKSPACE_CLEANUP_HOURS=6

# Increase API timeout
SPIRAL_CLAUDE_TIMEOUT_SECONDS=300
```

### Resource Limits

Recommended limits for production:

- CPU: 2-4 cores
- Memory: 2-4GB
- Disk: 20GB (for workspaces)
- Network: 10Mbps minimum

### Scaling Considerations

For high load:

1. Run multiple instances behind load balancer
2. Use shared Redis for task queue (future)
3. Implement workspace storage on NFS/S3
4. Add monitoring/alerting (Prometheus/Grafana)

## 🚨 Emergency Procedures

### System Unresponsive

1. Check process status: `ps aux | grep spiral`
2. Check system resources: `top` or `htop`
3. Review recent logs: `tail -f spiral.log`
4. Attempt graceful shutdown: `kill -TERM <PID>`
5. Force shutdown if needed: `kill -9 <PID>`

### Data Recovery

If system crashes with active tasks:

1. Check `task_storage` for incomplete tasks
2. Review workspace directories for partial work
3. Check logs for last known state
4. Manually resubmit critical tasks

### Rollback Procedure

If new version causes issues:

1. Stop current version
2. Restore previous binary
3. Restore previous configuration
4. Clear workspace directory
5. Start previous version
6. Verify functionality

## 📝 Logging and Audit

### Log Locations

- **Console Output**: Default, follows systemd journal
- **File Logging**: Configure with `SPIRAL_LOG_FILE=/var/log/spiral.log`
- **Structured Logs**: JSON format available with config

### Log Rotation

Configure logrotate:

```bash
/var/log/spiral.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
}
```

### Audit Events

Key events logged for audit:

- Startup/shutdown sequences
- Task creation and completion
- Authentication attempts
- Configuration changes
- Error conditions

## 🔐 Security Operations

### API Key Rotation

1. Generate new API key from Anthropic dashboard
2. Update configuration with new key
3. Restart Spiral Core
4. Verify connectivity
5. Revoke old key

### Access Control

- API endpoints require authentication
- Rate limiting prevents abuse
- Input validation prevents injection
- Workspace isolation prevents cross-contamination

### Security Monitoring

Watch for:

- Failed authentication attempts
- Unusual task patterns
- Unexpected file access
- High error rates
- Permission changes

---

This operations guide provides comprehensive procedures for running Spiral Core in production. For development-specific guidance, see [CODING_STANDARDS.md](CODING_STANDARDS.md#development-practices).
