# Operations Guide - Spiral Core

This guide covers deployment, monitoring, maintenance, and troubleshooting for production Spiral Core deployments.

## Deployment Options

### Docker Deployment

#### Single Container

```bash
# Build image
docker build -t spiral-core:latest .

# Run container
docker run -d \
  --name spiral-core \
  -p 3000:3000 \
  -e CLAUDE_API_KEY=$CLAUDE_API_KEY \
  -e DISCORD_TOKEN=$DISCORD_TOKEN \
  --restart unless-stopped \
  spiral-core:latest
```

#### Docker Compose

```yaml
# docker-compose.yml
version: "3.8"

services:
  spiral-core:
    image: spiral-core:latest
    container_name: spiral-core
    ports:
      - "3000:3000"
    environment:
      - CLAUDE_API_KEY=${CLAUDE_API_KEY}
      - DISCORD_TOKEN=${DISCORD_TOKEN}
      - DISCORD_AUTHORIZED_USERS=${DISCORD_AUTHORIZED_USERS}
      - RUST_LOG=${RUST_LOG:-info}
    volumes:
      - ./logs:/app/logs
      - ./data:/app/data
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
    deploy:
      resources:
        limits:
          memory: 2.5G
          cpus: "2"
        reservations:
          memory: 2G
          cpus: "1"

  redis:
    image: redis:7-alpine
    container_name: spiral-redis
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    restart: unless-stopped

  postgres:
    image: postgres:15-alpine
    container_name: spiral-postgres
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_DB=spiral_core
      - POSTGRES_USER=spiral
      - POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
    volumes:
      - postgres-data:/var/lib/postgresql/data
    restart: unless-stopped

volumes:
  redis-data:
  postgres-data:
```

### Kubernetes Deployment

```yaml
# spiral-core-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: spiral-core
  namespace: spiral
spec:
  replicas: 2
  selector:
    matchLabels:
      app: spiral-core
  template:
    metadata:
      labels:
        app: spiral-core
    spec:
      containers:
        - name: spiral-core
          image: spiral-core:latest
          ports:
            - containerPort: 3000
          env:
            - name: CLAUDE_API_KEY
              valueFrom:
                secretKeyRef:
                  name: spiral-secrets
                  key: claude-api-key
            - name: DISCORD_TOKEN
              valueFrom:
                secretKeyRef:
                  name: spiral-secrets
                  key: discord-token
          resources:
            requests:
              memory: "2Gi"
              cpu: "1"
            limits:
              memory: "2.5Gi"
              cpu: "2"
          livenessProbe:
            httpGet:
              path: /health
              port: 3000
            initialDelaySeconds: 30
            periodSeconds: 30
          readinessProbe:
            httpGet:
              path: /ready
              port: 3000
            initialDelaySeconds: 10
            periodSeconds: 10
```

### Systemd Service

```ini
# /etc/systemd/system/spiral-core.service
[Unit]
Description=Spiral Core Agent Orchestration System
After=network.target
Requires=network.target

[Service]
Type=simple
User=spiral
Group=spiral
WorkingDirectory=/opt/spiral-core
Environment=RUST_LOG=info
EnvironmentFile=/opt/spiral-core/.env
ExecStart=/opt/spiral-core/target/release/spiral-core
Restart=always
RestartSec=10
StandardOutput=append:/var/log/spiral-core/spiral.log
StandardError=append:/var/log/spiral-core/spiral.error.log

# Resource limits
MemoryLimit=2.5G
CPUQuota=200%

[Install]
WantedBy=multi-user.target
```

## Monitoring

### Health Checks

```bash
# Basic health check
curl http://localhost:3000/health

# Detailed readiness check
curl http://localhost:3000/ready

# System status with metrics
curl -H "x-api-key: $API_KEY" http://localhost:3000/system/status
```

### Prometheus Metrics

```yaml
# prometheus.yml
scrape_configs:
  - job_name: "spiral-core"
    static_configs:
      - targets: ["localhost:3000"]
    metrics_path: "/metrics"
```

Key metrics to monitor:

- `spiral_core_requests_total` - API request count
- `spiral_core_task_duration_seconds` - Task completion times
- `spiral_core_agent_utilization` - Agent usage percentage
- `spiral_core_claude_api_calls` - Claude API usage
- `spiral_core_error_rate` - Error rate per endpoint

### Logging

#### Log Levels

```bash
# Set log level via environment variable
RUST_LOG=debug  # debug, info, warn, error
```

#### Log Aggregation

```yaml
# filebeat.yml for ELK stack
filebeat.inputs:
  - type: log
    enabled: true
    paths:
      - /var/log/spiral-core/*.log
    fields:
      service: spiral-core
      environment: production
    multiline.pattern: '^\d{4}-\d{2}-\d{2}'
    multiline.negate: true
    multiline.match: after

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
```

### Alerts

```yaml
# alertmanager rules
groups:
  - name: spiral-core
    rules:
      - alert: HighErrorRate
        expr: rate(spiral_core_errors_total[5m]) > 0.05
        for: 5m
        annotations:
          summary: "High error rate detected"

      - alert: HighMemoryUsage
        expr: spiral_core_memory_usage_bytes > 2147483648 # 2GB
        for: 10m
        annotations:
          summary: "Memory usage exceeding threshold"

      - alert: ClaudeAPIQuotaExhausted
        expr: spiral_core_claude_quota_remaining < 100
        annotations:
          summary: "Claude API quota running low"
```

## Backup and Recovery

### Database Backup

```bash
#!/bin/bash
# backup.sh

# PostgreSQL backup
pg_dump -h localhost -U spiral spiral_core | gzip > backup_$(date +%Y%m%d_%H%M%S).sql.gz

# Redis backup
redis-cli --rdb /backup/redis_$(date +%Y%m%d_%H%M%S).rdb

# Upload to S3
aws s3 sync /backup s3://spiral-backups/$(date +%Y%m%d)/
```

### Disaster Recovery

1. **Database Recovery**

   ```bash
   gunzip < backup.sql.gz | psql -h localhost -U spiral spiral_core
   ```

2. **Redis Recovery**

   ```bash
   redis-cli --pipe < redis_backup.rdb
   ```

3. **Configuration Recovery**

   ```bash
   # Restore .env from secure storage
   aws secretsmanager get-secret-value --secret-id spiral-core-env > .env
   ```

## Performance Tuning

### Resource Optimization

```bash
# Analyze memory usage
cargo build --release --features jemalloc

# Profile CPU usage
perf record -g ./target/release/spiral-core
perf report
```

### Connection Pooling

```toml
# Database connection pool settings
[database]
max_connections = 20
min_connections = 5
connection_timeout = 30
idle_timeout = 300
```

### Rate Limiting

```nginx
# nginx.conf
limit_req_zone $binary_remote_addr zone=api:10m rate=100r/m;

server {
    location /api {
        limit_req zone=api burst=10 nodelay;
        proxy_pass http://localhost:3000;
    }
}
```

## Security Hardening

### API Key Rotation

```bash
#!/bin/bash
# rotate-keys.sh

# Generate new API key
NEW_KEY=$(openssl rand -hex 32)

# Update in secret manager
aws secretsmanager update-secret \
  --secret-id spiral-core-api-key \
  --secret-string "$NEW_KEY"

# Trigger rolling deployment
kubectl rollout restart deployment/spiral-core
```

### Network Security

```yaml
# Network policy for Kubernetes
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: spiral-core-network-policy
spec:
  podSelector:
    matchLabels:
      app: spiral-core
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: nginx-ingress
      ports:
        - protocol: TCP
          port: 3000
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: postgres
    - to:
        - podSelector:
            matchLabels:
              app: redis
    - ports:
        - protocol: TCP
          port: 443 # Claude API
```

## Troubleshooting

### Common Issues

#### High Memory Usage

```bash
# Check memory usage by component
ps aux | grep spiral
top -p $(pgrep spiral-core)

# Analyze heap dump
cargo build --features heap-profiling
HEAPPROFILE=/tmp/heap.prof ./target/release/spiral-core
```

#### API Connection Issues

```bash
# Test Claude API connectivity
curl -X POST https://api.anthropic.com/v1/messages \
  -H "x-api-key: $CLAUDE_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{"model":"claude-3-5-sonnet-20241022","messages":[{"role":"user","content":"test"}],"max_tokens":10}'

# Check Discord WebSocket
wscat -c "wss://gateway.discord.gg/?v=10&encoding=json"
```

#### Database Connection Issues

```bash
# Test PostgreSQL connection
psql -h localhost -U spiral -d spiral_core -c "SELECT 1;"

# Test Redis connection
redis-cli ping
```

### Debug Mode

```bash
# Enable verbose logging
RUST_LOG=spiral_core=trace,tokio=debug cargo run

# Enable backtrace for panics
RUST_BACKTRACE=full cargo run
```

### Performance Profiling

```bash
# CPU profiling with flamegraph
cargo install flamegraph
cargo build --release
flamegraph -o flamegraph.svg ./target/release/spiral-core

# Memory profiling
valgrind --tool=massif ./target/release/spiral-core
ms_print massif.out.<pid>
```

## Maintenance

### Regular Tasks

#### Daily

- Check error logs for anomalies
- Monitor API quota usage
- Verify backup completion

#### Weekly

- Review performance metrics
- Update dependencies: `cargo update`
- Run security audit: `cargo audit`

#### Monthly

- Rotate API keys
- Review and update rate limits
- Performance profiling
- Capacity planning review

### Upgrade Process

```bash
#!/bin/bash
# upgrade.sh

# 1. Build new version
git pull origin main
cargo build --release

# 2. Run tests
cargo test --release

# 3. Backup current version
cp target/release/spiral-core target/release/spiral-core.backup

# 4. Deploy with zero downtime
systemctl reload spiral-core  # Graceful reload

# 5. Verify health
curl http://localhost:3000/health

# 6. Rollback if needed
if [ $? -ne 0 ]; then
    cp target/release/spiral-core.backup target/release/spiral-core
    systemctl restart spiral-core
fi
```

## Support

### Log Collection for Support

```bash
#!/bin/bash
# collect-logs.sh

tar -czf spiral-logs-$(date +%Y%m%d).tar.gz \
  /var/log/spiral-core/*.log \
  /tmp/spiral-metrics.txt \
  .env.example  # Never include actual .env

# Upload securely
gpg --encrypt --recipient support@spiral-core.com spiral-logs-*.tar.gz
```

### Diagnostic Information

```bash
# System information
uname -a
free -h
df -h
cargo --version
rustc --version

# Application status
systemctl status spiral-core
curl http://localhost:3000/system/status | jq

# Recent errors
tail -n 100 /var/log/spiral-core/spiral.error.log
```
