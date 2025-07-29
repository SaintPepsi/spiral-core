# Discord Admin Commands Configuration

This document explains how to configure and use admin-only commands in the SpiralConstellation Discord bot.

## Configuration

### 1. Authorized Users

Admin commands are restricted to users listed in the `DISCORD_AUTHORIZED_USERS` environment variable.

**In your `.env` file:**

```bash
# Comma-separated list of Discord user IDs
DISCORD_AUTHORIZED_USERS=123456789012345678,987654321098765432
```

**To get your Discord user ID:**

1. Enable Developer Mode in Discord (User Settings > Advanced > Developer Mode)
2. Right-click on your username
3. Select "Copy User ID"

### 2. Permission Levels

The bot checks for admin permissions using only:

- **Authorized Users List** - Users explicitly listed in the `DISCORD_AUTHORIZED_USERS` environment variable

Commands are only accessible to users in the authorized list. Discord role permissions are not considered.

## Available Commands

### For All Users

- `!spiral help` - Show detailed help information
- `!spiral commands` - Show concise command list (personalized based on your permissions)
- `!spiral ratelimit` - Check your own rate limit status

### Authorized Users Only Commands

#### Security Monitoring

- `!spiral security stats` - View comprehensive security metrics
- `!spiral security reset` - Reset all security metrics
- `!spiral security report` - Generate detailed security report for current message

#### Rate Limit Management

- `!spiral ratelimit @user` - Check another user's rate limit status
- `!spiral reset ratelimit @user` - Reset a user's rate limit

## Command Examples

### List Available Commands

```
!spiral commands
```

Response (for authorized users):

```
📋 Available Commands

🌟 General Commands:
• !spiral help - Show detailed help information
• !spiral commands - Show this command list
• !spiral join <role> - Join an agent role (SpiralDev, SpiralPM, etc.)
• !spiral ratelimit - Check your rate limit status
• !spiral setup roles - Create agent roles in server

🤖 Agent Interactions:
• @SpiralDev <request> - Software development tasks
• @SpiralPM <request> - Project management queries
• @SpiralQA <request> - Quality assurance reviews
• @SpiralKing <request> - Comprehensive code review
• Use role mentions: <@&role_id> <request>

🔐 Admin Commands (You have access):
• !spiral security stats - View security metrics
• !spiral security reset - Reset security metrics
• !spiral security report - Generate security report
• !spiral ratelimit @user - Check user's rate limit
• !spiral reset ratelimit @user - Reset user's rate limit

*Use !spiral help for detailed usage information* 💡
```

### Check Security Stats

```
!spiral security stats
```

Response:

```
🛡️ Security Metrics

📊 Message Statistics:
• Total Processed: 150
• Messages Blocked: 5
• Block Rate: 3.3%

⚠️ Threat Detection:
• Malicious Attempts: 2
• XSS Attempts: 1
• Injection Attempts: 0
• Spam Detected: 2
• Rate Limited: 3
```

### Check User's Rate Limit

```
!spiral ratelimit @username
```

Response:

```
📊 Rate Limit Status for @username
• Remaining messages: 3/5
• Status: ✅ Active
```

### Reset User's Rate Limit

```
!spiral reset ratelimit @username
```

Response:

```
✅ Rate limit reset for @username
They can now send messages again.
```

## Security Features

### Command Input Validation

All commands are validated through the security system before processing to prevent:

- Command injection attacks
- Malicious input patterns
- Unauthorized access attempts

### Permission Verification

The bot verifies permissions for every admin command:

- Validates against authorized users list only
- Logs permission failures for security monitoring
- No Discord role permissions are considered

### Error Handling

- Invalid user mentions are handled gracefully
- Permission denied messages are clear and informative
- Failed commands don't expose system information

## Troubleshooting

### Command Not Working

1. Verify your user ID is in `DISCORD_AUTHORIZED_USERS`
2. Check that your `.env` file is properly formatted
3. Restart the bot after changing environment variables

### User ID Not Recognized

1. Ensure Developer Mode is enabled in Discord
2. Copy the User ID (not username) from Discord
3. Verify the ID is correctly added to the comma-separated list

### Permission Denied

- Admin commands require being in the authorized users list only
- Regular users can only use `!spiral ratelimit` for their own status

## Best Practices

1. **Limit Authorized Users**: Only add trusted users to the authorized list
2. **Regular Monitoring**: Check security stats periodically for unusual activity
3. **Rate Limit Management**: Reset rate limits for legitimate users who hit limits
4. **Environment Security**: Never commit your `.env` file with real user IDs to version control
