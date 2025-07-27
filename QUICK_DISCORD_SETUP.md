# 🚀 Quick Discord Bot Setup

Get your SpiralConstellation Discord bot running in 5 minutes!

## 1. Prerequisites

- Rust installed (`cargo --version` should work)
- Discord server with admin permissions
- Basic `.env` file setup

## 2. Create Discord Bot

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Click "New Application" → name it "SpiralConstellation"
3. Go to "Bot" section → "Add Bot"
4. Copy the **Bot Token** (keep it secret!)
5. **CRITICAL:** Enable "Privileged Gateway Intents":
   - ✅ **Message Content Intent** (REQUIRED)
   - ✅ **Server Members Intent** (for role management)
   - Click "Save Changes"

## 3. Get Bot Permissions

1. Go to "OAuth2" → "URL Generator"
2. Select scopes: `bot` ✅
3. Select permissions:
   - View Channels ✅
   - Send Messages ✅
   - Manage Roles ✅ (for role features)
   - Read Message History ✅
4. Copy the generated URL and open it to invite bot to your server

## 4. Configure Environment

Create `.env` file in project root:

```bash
# Discord Configuration
DISCORD_TOKEN=your_bot_token_here

# Claude Code Configuration (optional for basic testing)
CLAUDE_API_KEY=your_claude_api_key_here
CLAUDE_BASE_URL=https://api.anthropic.com
CLAUDE_MODEL=claude-3-5-sonnet-20241022

# API Configuration
API_HOST=127.0.0.1
API_PORT=3000
API_KEY=your-secret-api-key-here
```

## 5. Run the Bot

### **Option A: Discord Bot Only (Commands & Chat)**

```bash
# Run just the Discord bot (no task execution)
cargo run --bin discord-bot

# You should see:
# 🌌 Starting SpiralConstellation Discord Bot...
# 🌌 SpiralConstellation bot is connected and ready!
```

### **Option B: Full System (Discord + Task Execution)**

```bash
# Run complete system (Discord + Claude Code execution)
cargo run

# Includes: API server, Discord bot, and Claude Code integration
```

**💡 Difference:**

- **Option A:** Bot responds to commands but can't execute code tasks
- **Option B:** Bot can actually write code and perform tasks

## 6. Test in Discord

1. **Set up roles** (admin only):

   ```
   !spiral setup roles
   ```

2. **Join a role**:

   ```
   !spiral join SpiralDev
   ```

3. **Test agent interaction**:

   ```
   @SpiralDev create a hello world function in Rust
   ```

4. **Get help**:

   ```
   !spiral help
   ```

## 🎭 Available Agent Personas

- 🚀 **SpiralDev** - Software development & coding
- 📋 **SpiralPM** - Project management & coordination  
- 🔍 **SpiralQA** - Quality assurance & testing
- 🎯 **SpiralDecide** - Decision making & analysis
- ✨ **SpiralCreate** - Creative solutions & innovation
- 🧘 **SpiralCoach** - Process optimization & guidance

## 🛠️ Commands Reference

| Command | Description |
|---------|-------------|
| `!spiral help` | Show help message |
| `!spiral setup roles` | Create agent roles (admin) |
| `!spiral join <role>` | Join an agent role |
| `@SpiralDev <message>` | Talk to developer agent |
| `<@&role_id> <message>` | Use role mentions |

## 🔧 Troubleshooting

**Error: "Disallowed intent(s)" / Code 4014?**

- Go to Discord Developer Portal → Your App → Bot
- Enable "Message Content Intent" ✅
- Enable "Server Members Intent" ✅  
- Click "Save Changes" and restart bot

**Bot offline?**

- Check token in `.env`
- Verify intents are enabled in Developer Portal

**No responses?**

- Check bot has Send Messages permission
- Try `!spiral help` first
- Ensure Message Content Intent is enabled

**Role commands fail?**

- Ensure bot has Manage Roles permission
- Bot role must be higher than created roles
- Server Members Intent must be enabled

**Need Claude integration?**

- Add `CLAUDE_API_KEY` to `.env`
- Get key from [Anthropic Console](https://console.anthropic.com)

## 🌟 That's it

Your SpiralConstellation bot is ready! Each agent persona responds with unique personality and expertise.

For advanced configuration, see [DISCORD_BOT_SETUP.md](docs/DISCORD_BOT_SETUP.md).
