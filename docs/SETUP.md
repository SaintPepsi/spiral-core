# Setup Guide - Spiral Core

This guide covers all setup options for Spiral Core, from quick start to production deployment.

## Prerequisites

### Required

- **Rust**: 1.70+ with Cargo
- **Git**: For cloning the repository
- **Claude API Key**: From [Anthropic Console](https://console.anthropic.com)

### Optional

- **Docker**: For containerized development
- **Discord**: Bot token for Discord integration
- **Node.js**: 18+ for Discord bot TypeScript components

## Quick Start (5 Minutes)

### 1. Clone and Build

```bash
# Clone repository
git clone <repository-url>
cd spiral-core

# Build the project
cargo build --release
```

### 2. Configure Environment

Create `.env` file in project root:

```bash
# Required
CLAUDE_API_KEY=sk-ant-api03-your-api-key-here

# Optional - Discord Integration
DISCORD_TOKEN=your-discord-bot-token
DISCORD_AUTHORIZED_USERS=123456789012345678,987654321098765432

# Optional - API Configuration
API_HOST=127.0.0.1
API_PORT=3000
API_KEY=your-secret-api-key
```

### 3. Run

```bash
# Start Spiral Core
cargo run --bin spiral-core

# Verify it's working
curl http://localhost:3000/health
```

## Development Setup Options

### Option A: Dev Container (Recommended)

Best for consistent development environment across teams.

1. **Prerequisites**

   - Docker Desktop
   - VS Code with Dev Containers extension

2. **Open in Container**

   ```bash
   code .
   # Then: Cmd+Shift+P → "Dev Containers: Reopen in Container"
   ```

3. **Configure**

   - Update `.env` with your keys
   - Container includes all tools pre-installed

4. **Run**

   ```bash
   spiral-start  # Alias for starting the system
   ```

**Performance Notes:**

- Fast build (default): 2-3 min initial, 30 sec rebuilds
- Full build (optional): 10-15 min initial, immediate startup

### Option B: Local Development

Best for maximum control and performance.

1. **Install Rust**

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Install Development Tools**

   ```bash
   # Required
   cargo install cargo-watch cargo-edit

   # Optional but recommended
   cargo install cargo-audit cargo-outdated
   brew install hurl  # macOS, or see hurl.dev for other platforms
   ```

3. **Build and Run**

   ```bash
   cargo build
   cargo run --bin spiral-core
   ```

## Discord Bot Setup

### 1. Create Discord Application

1. Visit [Discord Developer Portal](https://discord.com/developers/applications)
2. Click "New Application" → Name it "SpiralConstellation"
3. Go to "Bot" section → "Add Bot"
4. Copy the **Bot Token**

### 2. Configure Bot Permissions

**Required Intents:**

- ✅ Message Content Intent (CRITICAL)
- ✅ Server Members Intent (for roles)

**Required Permissions:**

- View Channels
- Send Messages
- Manage Roles
- Read Message History

### 3. Generate Invite URL

1. Go to OAuth2 → URL Generator
2. Select scopes: `bot`
3. Select required permissions
4. Copy generated URL and invite bot to server

### 4. Configure Environment

Add to `.env`:

```bash
DISCORD_TOKEN=your_bot_token_here
DISCORD_AUTHORIZED_USERS=user_id_1,user_id_2  # Required for security
```

### 5. Run Discord Bot

```bash
# Option A: Discord bot only (commands & chat)
cargo run --bin discord-bot

# Option B: Full system (Discord + task execution)
cargo run  # Runs everything including Discord bot
```

### 6. Test in Discord

```bash
# Set up roles (admin only)
!spiral setup roles

# Join a role
!spiral join SpiralDev

# Test agent interaction
@SpiralDev create a hello world function in Rust
```

## Available Tools and Aliases

### Development Aliases

```bash
# Core Operations
spiral-run          # Start Spiral Core server
spiral-health       # Test health endpoint
spiral-test         # Run API task tests

# API Testing
api-health          # Test health endpoint
api-test            # Run all API tests
api-status          # Test system status

# Tool Management
install-hurl        # Install Hurl API testing tool
install-tools       # Install all development tools
```

### Verification Commands

```bash
# Full system verification
npm run verify

# Quick verification (skips time-consuming checks)
npm run verify:quick

# Individual checks
cargo test                    # Run tests
cargo fmt -- --check          # Check formatting
cargo clippy --all-targets    # Run linter
```

## Configuration Reference

### Environment Variables

| Variable                   | Required | Default                     | Description                                       |
| -------------------------- | -------- | --------------------------- | ------------------------------------------------- |
| `CLAUDE_API_KEY`           | ✅       | -                           | Claude API key from Anthropic                     |
| `CLAUDE_BASE_URL`          | ❌       | <https://api.anthropic.com> | Claude API endpoint                               |
| `CLAUDE_MODEL`             | ❌       | claude-3-5-sonnet-20241022  | Model to use                                      |
| `DISCORD_TOKEN`            | ❌\*     | -                           | Discord bot token (\*required for Discord)        |
| `DISCORD_AUTHORIZED_USERS` | ❌\*     | -                           | Comma-separated user IDs (\*required for Discord) |
| `API_HOST`                 | ❌       | 127.0.0.1                   | API server host                                   |
| `API_PORT`                 | ❌       | 3000                        | API server port                                   |
| `API_KEY`                  | ❌       | -                           | API authentication key                            |
| `RUST_LOG`                 | ❌       | info                        | Log level (debug, info, warn, error)              |

### API Endpoints

```bash
# Health Check
GET http://localhost:3000/health

# Submit Task
POST http://localhost:3000/tasks
Content-Type: application/json
{
  "agent_type": "SoftwareDeveloper",
  "content": "Create hello world in Rust"
}

# System Status (requires API key)
GET http://localhost:3000/system/status
x-api-key: your-api-key
```

## Production Deployment

### Docker Deployment

1. **Build Image**

   ```bash
   docker build -t spiral-core:latest .
   ```

2. **Run Container**

   ```yaml
   # docker-compose.yml
   services:
     spiral-core:
       image: spiral-core:latest
       environment:
         - CLAUDE_API_KEY=${CLAUDE_API_KEY}
         - DISCORD_TOKEN=${DISCORD_TOKEN}
       ports:
         - "3000:3000"
       resources:
         limits:
           memory: 2.5G
   ```

3. **Start Services**

   ```bash
   docker-compose up -d
   ```

### Systemd Service

1. **Create Service File**

   ```ini
   # /etc/systemd/system/spiral-core.service
   [Unit]
   Description=Spiral Core Agent Orchestration
   After=network.target

   [Service]
   Type=simple
   User=spiral
   WorkingDirectory=/opt/spiral-core
   ExecStart=/opt/spiral-core/target/release/spiral-core
   Restart=always
   EnvironmentFile=/opt/spiral-core/.env

   [Install]
   WantedBy=multi-user.target
   ```

2. **Enable and Start**

   ```bash
   sudo systemctl enable spiral-core
   sudo systemctl start spiral-core
   ```

## Troubleshooting

### Common Issues

#### Claude API Errors

- **Solution**: Verify API key is valid
- Check rate limits in Anthropic Console
- Ensure network connectivity to api.anthropic.com

#### Discord Bot Not Responding

- **Solution**: Check Message Content Intent is enabled
- Verify bot has required permissions in channel
- Check DISCORD_AUTHORIZED_USERS includes your ID

#### Build Failures

- **Solution**: Ensure Rust 1.70+ installed
- Run `cargo clean` and rebuild
- Check disk space for build artifacts

#### Container Issues

- **Solution**: Ensure Docker is running
- Check port 3000 isn't already in use
- Verify .env file is properly configured

### Debug Mode

Enable detailed logging:

```bash
RUST_LOG=debug cargo run
```

### Getting Help

1. Check error messages in logs
2. Run verification script: `npm run verify`
3. Review [Architecture Documentation](ARCHITECTURE.md)
4. File issues on project repository

## Security Considerations

### API Keys

- Store in environment variables, never in code
- Use `.env` file for local development
- Use secrets management in production

### Discord Security

- Always configure DISCORD_AUTHORIZED_USERS
- Limit bot permissions to minimum required
- Monitor bot activity logs

### Network Security

- Use HTTPS in production
- Configure firewall rules
- Implement rate limiting

## Next Steps

1. **Explore Agent Capabilities**

   - Try different agent personas
   - Test complex task orchestration
   - Monitor agent collaboration

2. **Customize Configuration**

   - Adjust agent response patterns
   - Configure custom workflows
   - Set up monitoring

3. **Scale Deployment**
   - Deploy to cloud infrastructure
   - Set up load balancing
   - Configure automated backups

## Support Resources

- [Architecture Guide](ARCHITECTURE.md) - System design details
- [Development Guide](DEVELOPMENT.md) - Coding standards and practices
- [API Reference](API.md) - Complete endpoint documentation
- [Operations Guide](OPERATIONS.md) - Production deployment and monitoring
