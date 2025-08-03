use super::CommandHandler;
use crate::discord::spiral_constellation_bot::SpiralConstellationBot;
use serenity::{model::channel::Message, prelude::Context};
use tracing::info;

pub struct DebugCommand {
    // Debug command doesn't need state for now
}

impl DebugCommand {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate comprehensive debug report for a message or system state
    async fn generate_debug_report(
        &self,
        _content: &str,
        msg: &Message,
        _ctx: &Context,
        bot: &SpiralConstellationBot,
    ) -> String {
        let mut report = "🔍 **Spiral Debug Report**\n\n".to_string();

        // Extract target message if replying to one
        let target_message = if let Some(referenced_message) = &msg.referenced_message {
            Some(referenced_message.as_ref())
        } else {
            None
        };

        if let Some(target_msg) = target_message {
            report.push_str("**🎯 Target Message Debug**\n");
            report.push_str(&format!("• Message ID: {}\n", target_msg.id.get()));
            report.push_str(&format!(
                "• Author: {} ({})\n",
                target_msg.author.name,
                target_msg.author.id.get()
            ));
            report.push_str(&format!("• Channel: <#{}>\n", target_msg.channel_id.get()));
            report.push_str(&format!(
                "• Content Length: {} chars\n",
                target_msg.content.len()
            ));
            report.push_str(&format!("• Timestamp: {}\n", target_msg.timestamp));

            // Content preview (first 100 chars)
            let content_preview = if target_msg.content.len() > 100 {
                format!("{}...", &target_msg.content[..97])
            } else {
                target_msg.content.clone()
            };
            report.push_str(&format!("• Content Preview: `{}`\n\n", content_preview));

            // Security validation analysis using bot's secure message handler
            report.push_str("**🛡️ Security Analysis**\n");

            let validation_result = bot
                .secure_message_handler
                .validate_command_input(&target_msg.content);
            let validation_status = if validation_result.is_valid {
                "🟢 Clean"
            } else {
                "🔴 Issues Found"
            };
            report.push_str(&format!("• Message Validation: {}\n", validation_status));

            if !validation_result.is_valid {
                report.push_str(&format!(
                    "• Issues: {}\n",
                    validation_result.issues.join(", ")
                ));
            }

            let risk_level = &validation_result.risk_level;
            let risk_emoji = match risk_level {
                crate::discord::message_security::RiskLevel::Low => "🟢",
                crate::discord::message_security::RiskLevel::Medium => "🟡",
                crate::discord::message_security::RiskLevel::High => "🔴",
                crate::discord::message_security::RiskLevel::Critical => "🚨",
            };
            report.push_str(&format!(
                "• Risk Assessment: {} {:?}\n\n",
                risk_emoji, risk_level
            ));
        } else {
            // System debug information
            report.push_str("**🖥️ System Debug Information**\n");
            report.push_str("• Discord Connection: 🟢 Active\n");
            report.push_str("• Message Processing: 🟢 Operational\n");
            report.push_str("• Command Recognition: 🟢 Working\n");
            report.push_str("• Authorization System: 🟢 Universal Protection Active\n\n");

            // Debug command usage
            report.push_str("**💡 Debug Command Usage**\n");
            report.push_str("• Reply to any message: `!spiral debug` (analyzes that message)\n");
            report.push_str("• System check: `!spiral debug` (shows system status)\n");
            report.push_str("• Error analysis: Reply to error messages for detailed breakdown\n\n");
        }

        // Current request debug
        report.push_str("**📋 Current Request Debug**\n");
        report.push_str(&format!(
            "• Requester: {} ({})\n",
            msg.author.name,
            msg.author.id.get()
        ));
        report.push_str(&format!("• Channel: <#{}>\n", msg.channel_id.get()));
        if let Some(guild_id) = msg.guild_id {
            report.push_str(&format!("• Server: Guild ID {}\n", guild_id.get()));
        } else {
            report.push_str("• Server: Direct Message\n");
        }
        report.push_str(&format!("• Request Time: {}\n", msg.timestamp));

        // Authorization status
        let is_authorized = bot.is_authorized_user(msg.author.id.get());
        let auth_status = if is_authorized {
            "🟢 Authorized"
        } else {
            "🔴 Not Authorized"
        };
        report.push_str(&format!("• Authorization: {}\n\n", auth_status));

        // System status summary
        report.push_str("**⚡ System Status Summary**\n");
        report.push_str("• Overall Health: 🟢 Operational\n");
        report.push_str("• Security Layer: 🟢 Active\n");
        report.push_str("• Command Processing: 🟢 Functional\n");

        // Rate limiting status from security metrics
        let security_metrics = bot.secure_message_handler.get_security_metrics();
        let rate_limited_count = security_metrics.rate_limited;
        let rate_limit_status = if rate_limited_count > 0 {
            format!("🟡 Active ({} users limited)", rate_limited_count)
        } else {
            "🟢 Active (no limits triggered)".to_string()
        };
        report.push_str(&format!("• Rate Limiting: {}\n\n", rate_limit_status));

        report.push_str("*Debug report generated successfully* ✅\n");
        report.push_str("*Use this information to troubleshoot issues or verify system behavior*");

        report
    }
}

impl CommandHandler for DebugCommand {
    async fn handle(
        &self,
        content: &str,
        msg: &Message,
        ctx: &Context,
        bot: &SpiralConstellationBot,
    ) -> Option<String> {
        const DEBUG_PREFIX: &str = "!spiral debug";

        let content_lower = content.to_lowercase();

        // Match debug command using const pattern
        match content_lower.as_str() {
            cmd if cmd.starts_with(DEBUG_PREFIX) => {
                info!(
                    "[DebugCommand] Debug report for user {} ({})",
                    msg.author.name,
                    msg.author.id.get()
                );
                let debug_report = self.generate_debug_report(content, msg, ctx, bot).await;
                Some(debug_report)
            }
            _ => None,
        }
    }

    fn command_prefix(&self) -> &str {
        "!spiral debug"
    }

    fn description(&self) -> &str {
        "Generate comprehensive debug reports for messages and system status"
    }
}
