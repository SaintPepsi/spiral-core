# 🤖 Discord Bot Setup Guide

This guide walks you through setting up the Spiral Core Discord bot for your server. The bot provides conversational AI agent mentions and integrates with the Spiral Core agent orchestration system.

## Prerequisites

- Discord account
- Server admin permissions
- Spiral Core application configured and running
- Basic understanding of Discord permissions

## Step 1: Create Discord Application

1. **Visit Discord Developer Portal**

   - Go to [Discord Developer Portal](https://discord.com/developers/applications)
   - Log in with your Discord account

2. **Create New Application**

   - Click "New Application"
   - Give it a name like "Spiral Core Bot" or "Agent Orchestrator"
   - Add a description (optional): "AI agent orchestration system with Claude Code integration"
   - Click "Create"

3. **Configure Application Settings**
   - Upload an avatar/icon for your bot
   - Add a description that explains the bot's purpose
   - Save changes

## Step 2: Create Bot User

1. **Navigate to Bot Section**

   - In your application, click "Bot" in the left sidebar
   - Click "Add Bot" to create a bot user

2. **Configure Bot Settings**

   - **Username**: Set a memorable name like "SpiralCore" or "AgentOrchestrator"
   - **Avatar**: Upload a distinctive bot avatar
   - **Public Bot**: ✅ Enable if you want others to add your bot to their servers
   - **Require OAuth2 Code Grant**: ❌ Disable (not needed for this use case)

3. **Bot Permissions**
   - **Message Content Intent**: ✅ Enable (required for reading messages)
   - **Server Members Intent**: ❌ Disable (not needed)
   - **Presence Intent**: ❌ Disable (not needed)

## Step 3: Get Bot Token

1. **Copy Bot Token**

   - In the Bot section, find the "Token" field
   - Click "Copy" to copy your bot token
   - ⚠️ **IMPORTANT**: Keep this token secret! It's like a password for your bot

2. **Save Token Securely**
   - Store the token in a secure location
   - Never commit the token to version control
   - We'll use this in the environment configuration

## Step 4: Configure Bot Permissions

1. **Navigate to OAuth2 > URL Generator**

   - Click "OAuth2" in the left sidebar
   - Select "URL Generator"

2. **Select Scopes**

   - ✅ `bot` - Required for bot functionality
   - ✅ `applications.commands` - For slash commands (future feature)

3. **Select Bot Permissions**

   - **Text Permissions**:

     - ✅ `Send Messages` - Post responses
     - ✅ `Send Messages in Threads` - Respond in threads
     - ✅ `Embed Links` - Rich responses
     - ✅ `Attach Files` - File uploads (if needed)
     - ✅ `Read Message History` - Context awareness
     - ✅ `Use External Emojis` - Enhanced responses
     - ✅ `Add Reactions` - Reaction-based interactions

   - **General Permissions**:
     - ✅ `View Channels` - See channels to respond in

4. **Generate Invite URL**
   - Copy the generated URL at the bottom
   - This URL will be used to invite the bot to your server

## Step 5: Invite Bot to Server

1. **Use Generated URL**

   - Open the URL from Step 4 in a new browser tab
   - Select the server you want to add the bot to
   - Ensure you have "Manage Server" permissions

2. **Authorize Bot**

   - Review the permissions
   - Click "Authorize"
   - Complete any captcha if prompted

3. **Verify Bot Added**
   - Check your Discord server's member list
   - The bot should appear offline (until we start it)

## Step 6: Configure Environment Variables

1. **Set Discord Token**

   ```bash
   # Add to your .env file
   DISCORD_TOKEN=your_bot_token_here

   # Example:
   DISCORD_TOKEN=MTIzNDU2Nzg5MDEyMzQ1Njc4.Xab12c.d3f45g67h89i0j1k2l3m4n5o6p7q8r9s
   ```

2. **Configure Discord Settings** (Optional)

   ```bash
   # Guild (server) ID for server-specific commands
   DISCORD_GUILD_ID=123456789012345678

   # Allowed channel IDs (comma-separated)
   DISCORD_ALLOWED_CHANNELS=123456789012345678,987654321098765432

   # Agent mention pattern (regex)
   AGENT_MENTION_PATTERN=@(\w+)agent
   ```

## Step 7: Start the Bot

1. **Ensure Spiral Core is Running**

   ```bash
   # Start the main Spiral Core system
   cargo run --bin spiral-core
   ```

2. **Start Discord Bot**

   ```bash
   # In a separate terminal
   cargo run --bin discord-bot
   ```

3. **Verify Bot Online**
   - Check Discord server - bot should appear online
   - Look for startup messages in the console

## Step 8: Test Bot Functionality

1. **Basic Mention Test**

   ```
   @developeragent create a hello world function in Rust
   ```

2. **Agent Types Available**

   - `@developeragent` - Code generation and development tasks
   - `@projectmanageragent` - Project coordination and analysis
   - `@qaagent` - Quality assurance and testing
   - `@decisionmakeragent` - Priority scoring and conflict resolution
   - `@creativeinnovatoragent` - Alternative approaches and innovation
   - `@processcoachagent` - Performance optimization

3. **Expected Response**
   - Bot should respond with generated code or analysis
   - Response time: ~2-5 seconds depending on complexity
   - Bot will show typing indicator while processing

## Configuration Options

### Environment Variables

| Variable                   | Required | Default       | Description                             |
| -------------------------- | -------- | ------------- | --------------------------------------- |
| `DISCORD_TOKEN`            | ✅ Yes   | -             | Bot token from Discord Developer Portal |
| `DISCORD_GUILD_ID`         | ❌ No    | -             | Server ID for server-specific features  |
| `DISCORD_ALLOWED_CHANNELS` | ❌ No    | All channels  | Comma-separated channel IDs             |
| `AGENT_MENTION_PATTERN`    | ❌ No    | `@(\w+)agent` | Regex pattern for agent mentions        |

### Bot Permissions Summary

**Required Permissions:**

- View Channels
- Send Messages
- Send Messages in Threads
- Read Message History

**Recommended Permissions:**

- Embed Links
- Attach Files
- Use External Emojis
- Add Reactions

## Troubleshooting

### Bot Won't Start

1. **Check Token**

   ```bash
   # Verify token format (should be ~70 characters)
   echo $DISCORD_TOKEN | wc -c
   ```

2. **Check Permissions**

   - Ensure Message Content Intent is enabled
   - Verify bot has required permissions in server

3. **Check Logs**

   ```bash
   # Look for error messages
   RUST_LOG=debug cargo run --bin discord-bot
   ```

### Bot Online But Not Responding

1. **Check Channel Permissions**

   - Bot needs "View Channels" and "Send Messages" in target channels
   - Check channel-specific permission overrides

2. **Verify Mention Pattern**

   ```bash
   # Test mention format
   @developeragent help
   ```

3. **Check API Connection**
   - Ensure Spiral Core API is running
   - Verify API_KEY is configured correctly

### Common Error Messages

#### "Invalid Token"

- Solution: Regenerate token in Discord Developer Portal
- Ensure no extra spaces or characters in token

#### "Missing Permissions"

- Solution: Re-invite bot with correct permissions URL
- Check server-specific permission overrides

#### "Connection Timeout"

- Solution: Check network connectivity
- Verify Discord isn't blocked by firewall

## Advanced Configuration

### Server-Specific Commands

```bash
# Configure for specific server
DISCORD_GUILD_ID=your_server_id
```

### Channel Restrictions

```bash
# Limit bot to specific channels
DISCORD_ALLOWED_CHANNELS=channel_id_1,channel_id_2
```

### Custom Agent Patterns

```bash
# Custom mention pattern
AGENT_MENTION_PATTERN="!(\w+)"  # Use ! instead of @
```

## Security Considerations

1. **Token Security**

   - Never share your bot token
   - Use environment variables, not hardcoded values
   - Regenerate token if compromised

2. **Server Permissions**

   - Grant minimum required permissions
   - Regularly audit bot permissions
   - Monitor bot activity logs

3. **Channel Access**
   - Limit bot to appropriate channels
   - Consider using private channels for sensitive tasks
   - Set up channel-specific permission overrides

## Next Steps

1. **Customize Bot Behavior**

   - Modify agent response patterns
   - Add custom commands
   - Configure response formatting

2. **Monitor Usage**

   - Set up logging and monitoring
   - Track agent performance metrics
   - Monitor API usage costs

3. **Scale Deployment**
   - Deploy to production server
   - Set up process management (systemd, Docker)
   - Configure automated backups

## Support

For additional help:

- Check the [Spiral Core Documentation](../README.md)
- Review [Discord Bot Architecture](../plans/DISCORD_AI_AGENT_ORCHESTRATOR_ARCHITECTURE.md)
- File issues on the project repository

---

## Happy orchestrating! 🎭✨
