use super::CommandHandler;
use crate::discord::spiral_constellation_bot::SpiralConstellationBot;
use serenity::{model::channel::Message, prelude::Context};
use std::time::Instant;

pub struct AdminCommand {
    // Admin command doesn't need state for now
}

impl AdminCommand {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate comprehensive admin dashboard
    async fn generate_admin_panel(&self, bot: &SpiralConstellationBot) -> String {
        let start_time = Instant::now();

        // System overview
        let mut panel = "🔐 **Spiral Core Admin Dashboard**\n\n".to_string();

        // Bot status and stats
        let stats = bot.stats.lock().await;
        panel.push_str("**📊 System Status**\n");
        panel.push_str(&format!("• Bot Status: 🟢 Online\n"));
        panel.push_str(&format!("• Uptime: Running\n"));
        panel.push_str(&format!(
            "• Dev Tasks Completed: {}\n",
            stats.dev_tasks_completed
        ));
        panel.push_str(&format!(
            "• PM Tasks Completed: {}\n",
            stats.pm_tasks_completed
        ));
        panel.push_str(&format!(
            "• QA Tasks Completed: {}\n",
            stats.qa_tasks_completed
        ));
        panel.push_str(&format!(
            "• Total Agent Interactions: {}\n\n",
            stats.dev_tasks_completed + stats.pm_tasks_completed + stats.qa_tasks_completed
        ));

        // Security metrics
        let metrics = bot.secure_message_handler.get_security_metrics();
        let avg_confidence = bot.secure_message_handler.get_average_confidence();
        panel.push_str("**🛡️ Security Overview**\n");
        panel.push_str(&format!(
            "• Messages Processed: {}\n",
            metrics.messages_processed
        ));
        panel.push_str(&format!(
            "• Messages Blocked: {}\n",
            metrics.messages_blocked
        ));
        panel.push_str(&format!(
            "• Block Rate: {:.1}%\n",
            if metrics.messages_processed > 0 {
                (metrics.messages_blocked as f64 / metrics.messages_processed as f64) * 100.0
            } else {
                0.0
            }
        ));
        panel.push_str(&format!(
            "• Classification Confidence: {:.2}\n",
            avg_confidence
        ));
        panel.push_str(&format!(
            "• Threat Detections: {}\n",
            metrics.malicious_attempts + metrics.xss_attempts + metrics.injection_attempts
        ));
        panel.push_str(&format!(
            "• Rate Limited Users: {}\n\n",
            metrics.rate_limited
        ));

        // Agent status - Check actual agent availability
        panel.push_str("**🤖 Agent Status**\n");

        let dev_status = if bot.has_developer_agent() {
            "🟢 Available"
        } else {
            "🔴 Not Available"
        };
        panel.push_str(&format!("• SpiralDev: {}\n", dev_status));

        let orchestrator_status = if bot.has_orchestrator() {
            "🟢 Available"
        } else {
            "🔴 Not Available"
        };
        panel.push_str(&format!("• Orchestrator: {}\n", orchestrator_status));

        // Other agents not yet implemented
        panel.push_str("• SpiralPM: 🔴 Not Implemented\n");
        panel.push_str("• SpiralQA: 🔴 Not Implemented\n");
        panel.push_str("• SpiralDecide: 🔴 Not Implemented\n");
        panel.push_str("• SpiralCreate: 🔴 Not Implemented\n");
        panel.push_str("• SpiralCoach: 🔴 Not Implemented\n\n");

        // Performance stats - HONEST metrics only
        let generation_time = start_time.elapsed();
        panel.push_str("**⚡ Performance**\n");
        panel.push_str(&format!(
            "• Dashboard Generation: {:.2}ms\n",
            generation_time.as_millis()
        ));

        // Basic memory usage (process RSS)
        if let Ok(usage) = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
        {
            if let Ok(output) = String::from_utf8(usage.stdout) {
                if let Ok(rss_kb) = output.trim().parse::<u64>() {
                    let rss_mb = rss_kb / 1024;
                    panel.push_str(&format!("• Memory Usage: {}MB RSS\n", rss_mb));
                } else {
                    panel.push_str("• Memory Usage: ❓ Parse error\n");
                }
            } else {
                panel.push_str("• Memory Usage: ❓ Command error\n");
            }
        } else {
            panel.push_str("• Memory Usage: ❓ Monitoring not available\n");
        }

        panel.push_str("• Response Time: ❓ Historical metrics not implemented\n\n");

        // Quick actions
        panel.push_str("**🔧 Quick Actions**\n");
        panel.push_str("• `!spiral security stats` - View detailed security metrics\n");
        panel.push_str("• `!spiral security reset` - Reset security metrics\n");
        panel.push_str("• `!spiral debug <message>` - Debug specific issues\n");
        panel.push_str("• `!spiral ratelimit @user` - Check user rate limits\n\n");

        panel.push_str("*Dashboard updated in real-time* ⚡");

        panel
    }
}

impl CommandHandler for AdminCommand {
    async fn handle(
        &self,
        content: &str,
        _msg: &Message,
        _ctx: &Context,
        bot: &SpiralConstellationBot,
    ) -> Option<String> {
        const ADMIN_PANEL: &str = "!spiral admin";

        let content_lower = content.to_lowercase();

        // Match admin command using const pattern
        match content_lower.as_str() {
            cmd if cmd.starts_with(ADMIN_PANEL) => Some(self.generate_admin_panel(bot).await),
            _ => None,
        }
    }

    fn command_prefix(&self) -> &str {
        "!spiral admin"
    }

    fn description(&self) -> &str {
        "Admin dashboard with system overview and quick actions"
    }
}
