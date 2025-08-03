use super::CommandHandler;
use crate::discord::spiral_constellation_bot::SpiralConstellationBot;
use serenity::{model::channel::Message, prelude::Context};
use tracing::{info, warn};

pub struct RateLimitCommand {
    // Rate limit command doesn't need state for now
}

impl RateLimitCommand {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate rate limit status for a user  
    fn generate_rate_limit_status(&self, _user_id: u64, username: &str) -> String {
        // Rate limit data access not yet implemented in secure message handler
        format!(
            "⏱️ **Rate Limit Status for {}**\n\n\
            **Current Status:** 🟢 Good Standing\n\
            **Requests Used:** ❓ Data not available\n\
            **Reset Time:** ❓ Data not available\n\
            **Daily Limit:** ❓ Data not available\n\n\
            **Notes:**\n\
            • Rate limiting is active to prevent abuse\n\
            • Limits reset daily at midnight UTC\n\
            • Contact admin if you need higher limits\n\n\
            *Rate limit data collection not yet implemented* ⚠️",
            username
        )
    }

    /// Generate rate limit reset confirmation
    fn generate_reset_confirmation(&self, target_user: &str) -> String {
        format!(
            "🔄 **Rate Limit Reset**\n\n\
            **Target User:** {}\n\
            **Action:** Rate limit reset requested\n\
            **Status:** ❓ Reset functionality not implemented\n\n\
            **What would happen:**\n\
            • User's rate limit counters would be cleared\n\
            • Fresh daily allowance would be granted\n\
            • User would be notified of the reset\n\n\
            *Rate limit reset not yet implemented* ⚠️",
            target_user
        )
    }

    /// Parse mentioned user from command
    fn parse_mentioned_user(&self, content: &str, msg: &Message) -> Option<(u64, String)> {
        // Check for @user mention
        if !msg.mentions.is_empty() {
            let mentioned_user = &msg.mentions[0];
            return Some((mentioned_user.id.get(), mentioned_user.name.clone()));
        }

        // Check for raw user ID
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 4 {
            if let Ok(user_id) = parts[3].parse::<u64>() {
                return Some((user_id, format!("User {}", user_id)));
            }
        }

        None
    }
}

impl CommandHandler for RateLimitCommand {
    async fn handle(
        &self,
        content: &str,
        msg: &Message,
        _ctx: &Context,
        _bot: &SpiralConstellationBot,
    ) -> Option<String> {
        // FLOW: Parse action → Validate target → Execute → Respond
        // 1. Parse ratelimit action (check, reset, etc.)
        // 2. Validate target user (self vs mentioned user)
        // 3. Execute the appropriate action
        // 4. Return formatted response
        
        const RATELIMIT_RESET: &str = "!spiral ratelimit reset";
        const RATELIMIT_CHECK: &str = "!spiral ratelimit";
        
        let content_lower = content.to_lowercase();

        // Match ratelimit command type using const patterns
        match content_lower.as_str() {
            cmd if cmd.starts_with(RATELIMIT_RESET) => {
                if let Some((target_user_id, target_username)) = self.parse_mentioned_user(content, msg) {
                    info!(
                        "[RateLimitCommand] Admin {} resetting rate limit for user {} ({})",
                        msg.author.name, target_username, target_user_id
                    );
                    Some(self.generate_reset_confirmation(&target_username))
                } else {
                    warn!(
                        "[RateLimitCommand] Invalid reset command from {}: missing user target",
                        msg.author.name
                    );
                    Some("❌ Usage: `!spiral ratelimit reset @user` or `!spiral ratelimit reset <user_id>`".to_string())
                }
            }
            cmd if cmd.starts_with(RATELIMIT_CHECK) => {
                if let Some((target_user_id, target_username)) = self.parse_mentioned_user(content, msg) {
                    // Admin checking another user's rate limit
                    info!(
                        "[RateLimitCommand] Admin {} checking rate limit for user {} ({})",
                        msg.author.name, target_username, target_user_id
                    );
                    Some(self.generate_rate_limit_status(target_user_id, &target_username))
                } else {
                    // User checking their own rate limit
                    info!(
                        "[RateLimitCommand] User {} ({}) checking own rate limit",
                        msg.author.name, msg.author.id.get()
                    );
                    Some(self.generate_rate_limit_status(msg.author.id.get(), &msg.author.name))
                }
            }
            _ => None,
        }
    }

    fn command_prefix(&self) -> &str {
        "!spiral ratelimit"
    }

    fn description(&self) -> &str {
        "Check and manage user rate limits with admin controls"
    }
}
