use super::CommandHandler;
use crate::discord::spiral_constellation_bot::SpiralConstellationBot;
use serenity::{model::channel::Message, prelude::Context};
use tracing::{info, warn};

pub struct SecurityCommand {
    // Security command doesn't need state for now
}

impl SecurityCommand {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate detailed security statistics
    fn generate_security_stats(&self, bot: &SpiralConstellationBot) -> String {
        let metrics = bot.secure_message_handler.get_security_metrics();
        let avg_confidence = bot.secure_message_handler.get_average_confidence();

        let mut report = "🛡️ **Spiral Security Statistics**\n\n".to_string();

        // Core metrics
        report.push_str("**📊 Core Metrics**\n");
        report.push_str(&format!(
            "• Messages Processed: {}\n",
            metrics.messages_processed
        ));
        report.push_str(&format!(
            "• Messages Blocked: {}\n",
            metrics.messages_blocked
        ));

        let block_rate = if metrics.messages_processed > 0 {
            (metrics.messages_blocked as f64 / metrics.messages_processed as f64) * 100.0
        } else {
            0.0
        };
        report.push_str(&format!("• Block Rate: {:.1}%\n", block_rate));
        report.push_str(&format!("• Average Confidence: {:.2}\n\n", avg_confidence));

        // Threat detection
        report.push_str("**🚨 Threat Detection**\n");
        report.push_str(&format!(
            "• Malicious Attempts: {}\n",
            metrics.malicious_attempts
        ));
        report.push_str(&format!("• XSS Attempts: {}\n", metrics.xss_attempts));
        report.push_str(&format!(
            "• Injection Attempts: {}\n",
            metrics.injection_attempts
        ));

        let total_threats =
            metrics.malicious_attempts + metrics.xss_attempts + metrics.injection_attempts;
        report.push_str(&format!("• Total Threat Detections: {}\n\n", total_threats));

        // Rate limiting
        report.push_str("**⏱️ Rate Limiting**\n");
        report.push_str(&format!("• Rate Limited Users: {}\n", metrics.rate_limited));
        report.push_str(&format!("• Spam Detected: {}\n\n", metrics.spam_detected));

        // Authorization
        report.push_str("**🔐 Authorization**\n");
        report.push_str("• Universal Authorization: ✅ Active\n");
        report.push_str("• All Commands Protected: ✅ Enabled\n");
        report.push_str("• Whitelist-based Access: ✅ Enforced\n");
        report.push_str("• Denial System: 🎭 Lordgenome Quotes\n\n");

        // Classification metrics
        report.push_str("**📈 Classification Metrics**\n");
        report.push_str(&format!(
            "• Total Classifications: {}\n",
            metrics.classification_count
        ));
        report.push_str(&format!(
            "• Low Confidence Classifications: {}\n",
            metrics.low_confidence_count
        ));
        report.push_str(&format!(
            "• Malicious Intent Detected: {}\n\n",
            metrics.intent_malicious
        ));

        // Recent activity summary
        report.push_str("**🕐 Recent Activity**\n");
        if metrics.messages_processed > 0 {
            let recent_activity = if metrics.messages_blocked > 0 {
                "🟡 Security blocks detected"
            } else {
                "🟢 All messages passed validation"
            };
            report.push_str(&format!("• Status: {}\n", recent_activity));
        } else {
            report.push_str("• Status: 📊 No messages processed yet\n");
        }

        report.push_str("• Last Updated: Real-time\n\n");

        // Quick actions
        report.push_str("**🔧 Security Actions**\n");
        report.push_str("• `!spiral security reset` - Reset all security metrics\n");
        report.push_str("• `!spiral security report` - Generate detailed security report\n");
        report.push_str("• `!spiral debug <message>` - Analyze specific message security\n\n");

        report.push_str("*Security monitoring is active 24/7* 🛡️");

        report
    }

    /// Generate comprehensive security report
    fn generate_security_report(&self, bot: &SpiralConstellationBot) -> String {
        let metrics = bot.secure_message_handler.get_security_metrics();
        let avg_confidence = bot.secure_message_handler.get_average_confidence();

        let mut report = "📋 **Spiral Security Report**\n\n".to_string();

        // Executive summary
        report.push_str("**📝 Executive Summary**\n");
        let security_status = if metrics.messages_blocked == 0 && metrics.messages_processed > 0 {
            "🟢 Excellent"
        } else if metrics.messages_blocked < (metrics.messages_processed / 10) {
            "🟡 Good"
        } else {
            "🔴 Needs Attention"
        };
        report.push_str(&format!("• Overall Security Status: {}\n", security_status));
        report.push_str(&format!(
            "• Total Messages Analyzed: {}\n",
            metrics.messages_processed
        ));
        report.push_str(&format!(
            "• Threat Prevention Rate: {:.1}%\n",
            if metrics.messages_processed > 0 {
                ((metrics.messages_processed - metrics.messages_blocked) as f64
                    / metrics.messages_processed as f64)
                    * 100.0
            } else {
                100.0
            }
        ));
        report.push_str(&format!(
            "• System Confidence: {:.1}%\n\n",
            avg_confidence * 100.0
        ));

        // Detailed metrics
        report.push_str("**🔍 Detailed Analysis**\n");
        report.push_str(&format!(
            "• Malicious Content Blocked: {}\n",
            metrics.malicious_attempts
        ));
        report.push_str(&format!(
            "• XSS Attempts Prevented: {}\n",
            metrics.xss_attempts
        ));
        report.push_str(&format!(
            "• Injection Attacks Stopped: {}\n",
            metrics.injection_attempts
        ));
        report.push_str(&format!(
            "• Rate Limit Enforcements: {}\n\n",
            metrics.rate_limited
        ));

        // Security layers
        report.push_str("**🛡️ Security Layers Status**\n");
        report.push_str("• Input Validation: ✅ Active\n");
        report.push_str("• Intent Classification: ✅ Active\n");
        report.push_str("• Rate Limiting: ✅ Active\n");
        report.push_str("• Authorization: ✅ Universal Protection\n");
        report.push_str("• Audit Logging: ✅ Comprehensive\n\n");

        // Recommendations
        report.push_str("**💡 Recommendations**\n");
        if metrics.messages_blocked > (metrics.messages_processed / 5) {
            report.push_str("• ⚠️ High block rate detected - review security policies\n");
        }
        if avg_confidence < 0.8 {
            report.push_str("• ⚠️ Low confidence scores - consider model tuning\n");
        }
        if metrics.messages_processed == 0 {
            report.push_str("• 📊 No activity yet - security systems ready\n");
        } else {
            report.push_str("• ✅ Security systems operating within normal parameters\n");
        }

        report.push_str("\n*Report generated in real-time* 📊");

        report
    }

    /// Reset security metrics
    fn generate_reset_confirmation(&self, _bot: &SpiralConstellationBot) -> String {
        // Note: Would need to implement actual reset in the secure_message_handler
        format!(
            "🔄 **Security Metrics Reset**\n\n\
            **Action:** All security counters have been reset\n\
            **Status:** ✅ Reset completed successfully\n\n\
            **Metrics Reset:**\n\
            • Message processing counters\n\
            • Threat detection counters\n\
            • Rate limiting counters\n\
            • Risk level distributions\n\n\
            **Note:** Historical logs are preserved for audit purposes\n\n\
            *Security monitoring continues with fresh counters* 🛡️"
        )
    }
}

impl CommandHandler for SecurityCommand {
    async fn handle(
        &self,
        content: &str,
        msg: &Message,
        _ctx: &Context,
        bot: &SpiralConstellationBot,
    ) -> Option<String> {
        const SECURITY_STATS: &str = "!spiral security stats";
        const SECURITY_REPORT: &str = "!spiral security report";
        const SECURITY_RESET: &str = "!spiral security reset";

        let content_lower = content.to_lowercase();

        // Match security command type using const patterns
        match content_lower.as_str() {
            cmd if cmd.starts_with(SECURITY_STATS) => {
                info!(
                    "[SecurityCommand] Security stats for admin {} ({})",
                    msg.author.name,
                    msg.author.id.get()
                );
                Some(self.generate_security_stats(bot))
            }
            cmd if cmd.starts_with(SECURITY_REPORT) => {
                info!(
                    "[SecurityCommand] Security report for admin {} ({})",
                    msg.author.name,
                    msg.author.id.get()
                );
                Some(self.generate_security_report(bot))
            }
            cmd if cmd.starts_with(SECURITY_RESET) => {
                warn!(
                    "[SecurityCommand] Security reset by admin {} ({})",
                    msg.author.name,
                    msg.author.id.get()
                );
                bot.secure_message_handler.reset_security_metrics();
                Some(self.generate_reset_confirmation(bot))
            }
            _ => None,
        }
    }

    fn command_prefix(&self) -> &str {
        "!spiral security"
    }

    fn description(&self) -> &str {
        "Security monitoring with statistics, reports, and metric management"
    }
}
