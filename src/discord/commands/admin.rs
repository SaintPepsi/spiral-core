use super::CommandHandler;
use crate::discord::{
    agent_registry::get_agent_registry, spiral_constellation_bot::SpiralConstellationBot,
};
use serenity::{model::channel::Message, prelude::Context};
use std::time::Instant;

pub struct AdminCommand {
    // Admin command doesn't need state for now
}

impl Default for AdminCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl AdminCommand {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate comprehensive admin dashboard
    async fn generate_admin_panel(&self, bot: &SpiralConstellationBot) -> String {
        let start_time = Instant::now();

        // System overview
        let mut panel = format!(
            "{}\n\n",
            crate::discord::messages::patterns::ADMIN_DASHBOARD_TITLE
        );

        // Bot status and stats
        let stats = bot.stats.lock().await;
        panel.push_str("**📊 System Status**\n");
        panel.push_str("• Bot Status: 🟢 Online\n");
        panel.push_str("• Uptime: Running\n");
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
            "• Classification Confidence: {avg_confidence:.2}\n"
        ));
        panel.push_str(&format!(
            "• Threat Detections: {}\n",
            metrics.malicious_attempts + metrics.xss_attempts + metrics.injection_attempts
        ));
        panel.push_str(&format!(
            "• Rate Limited Users: {}\n\n",
            metrics.rate_limited
        ));

        // Agent status - Dynamic from registry
        // 🏗️ ARCHITECTURE DECISION: Use registry for agent list
        // Why: Single source of truth for available agents
        // Alternative: Hardcoded list (rejected: violates DRY)
        panel.push_str("**🤖 Agent Status**\n");

        let all_agents = get_agent_registry().get_all_agents().await;

        // Debug: Also show active agents from bot
        let active_from_bot = bot.get_active_agents().await;

        if all_agents.is_empty() {
            panel.push_str("• No agents in registry\n");
            if !active_from_bot.is_empty() {
                panel.push_str(&format!("• Active in bot: {active_from_bot:?}\n"));
            }
            panel.push('\n');
        } else {
            for agent in all_agents {
                // 🏗️ ARCHITECTURE DECISION: Generic active check
                // Why: No hardcoding of specific agent types
                // Alternative: Type-specific checks (rejected: doesn't scale)
                let is_active = bot.is_agent_active(&agent.name).await;

                let status = if is_active {
                    "🟢 Available"
                } else if agent.available {
                    "🟡 Registered"
                } else {
                    "🔴 Not Available"
                };

                panel.push_str(&format!("• {}: {}\n", agent.name, status));
            }
            panel.push('\n');
        }

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
                    panel.push_str(&format!("• Memory Usage: {rss_mb}MB RSS\n"));
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
        panel.push_str("• `!spiral ratelimit @user` - Check user rate limits\n");

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
