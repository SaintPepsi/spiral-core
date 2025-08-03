//! 📝 DISCORD MESSAGE CONSTANTS
//! Purpose: Centralized location for all Discord bot messages to ensure consistency
//! and avoid duplication (DRY principle)

/// Bot response types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseType {
    SecurityBlock,
    ValidationFailed,
    RateLimited,
    Unauthorized,
    Error,
    Success,
    Help,
    Debug,
}

/// Security-related messages
pub mod security {
    pub const COMMAND_BLOCKED: &str = "⚠️ Command blocked for security reasons";
    pub const COMMAND_BLOCKED_HINT: &str = "Reply with `!spiral debug` for details";
    pub const MESSAGE_FLAGGED: &str = "🚫 Message flagged by security validation. Please ensure your message follows community guidelines.";
    pub const UNAUTHORIZED: &str =
        "🚫 This command requires authorization. Contact an administrator.";
    pub const RATE_LIMITED: &str = "⏸️ Rate limited (wait a moment)";
    pub const VALIDATION_FAILED: &str = "⚠️ Security validation failed. Message blocked.";
    pub const VALIDATION_ERROR: &str = "⚠️ Unable to process message securely. Please try again.";
    pub const SECURE_PROCESSING_BLOCKED: &str = "🚫 Message blocked by security validation.";
}

/// Error messages
pub mod errors {
    pub const MESSAGE_TOO_LONG: &str =
        "❌ Message too long for processing. Please keep requests under 4000 characters.";
    pub const UNKNOWN_AGENT: &str = "❓ I'm not sure which agent you'd like to talk to. Try mentioning @SpiralDev, @SpiralPM, @SpiralQA, @SpiralKing, or use a role mention!";
    pub const SEND_FAILED: &str = "Failed to send message";
    pub const ROLE_CREATION_FAILED: &str = "❌ Failed to create roles";
    pub const ROLE_ASSIGNMENT_FAILED: &str = "❌ Failed to assign role";
    pub const NOT_IN_SERVER: &str = "❌ This command only works in servers, not DMs.";
    pub const NOT_IN_SERVER_ROLES: &str = "❌ Role creation only works in servers, not DMs.";
    pub const NOT_IN_SERVER_ASSIGNMENT: &str = "❌ Role assignment only works in servers, not DMs.";
    pub const INVALID_USER_FORMAT: &str = "❌ Invalid user ID or mention format.";
    pub const ROLE_NOT_FOUND: &str = "❓ Unknown role";
    pub const ROLE_NOT_FOUND_DETAILED: &str = "❓ Unknown role: `{}`. Available: SpiralDev, SpiralPM, SpiralQA, SpiralDecide, SpiralCreate, SpiralCoach, SpiralKing";
}

/// Success messages
pub mod success {
    pub const ROLES_CREATED: &str = "🌌 **SpiralConstellation Setup Complete!**";
    pub const ROLE_ASSIGNED: &str = "**Welcome to {}!**";
    pub const METRICS_RESET: &str = "✅ Security metrics have been reset.";
    pub const RATE_LIMIT_RESET: &str =
        "✅ Rate limit reset for <@{}>\nThey can now send messages again.";
}

/// Help and info messages
pub mod info {
    pub const HELP_HEADER: &str = "🌌 **SpiralConstellation Bot Help**";
    pub const COMMANDS_HEADER: &str = "📋 **Available Commands**";
    pub const GENERAL_COMMANDS: &str = "**🌟 General Commands:**";
    pub const AGENT_INTERACTIONS: &str = "**🤖 Agent Interactions:**";
    pub const ADMIN_COMMANDS: &str = "**🔐 Admin Commands";
    pub const ADMIN_COMMANDS_AVAILABLE: &str = " (You have access):**";
    pub const ADMIN_COMMANDS_RESTRICTED: &str = " (Authorized users only):**";
}

/// Debug messages
pub mod debug {
    pub const SECURITY_DEBUG_HEADER: &str = "🔍 **Security Debug Report**";
    pub const GENERAL_DEBUG_HEADER: &str = "🔍 **General Debug Report**";
    pub const MESSAGE_DETAILS: &str = "**Message Details:**";
    pub const SECURITY_VALIDATION: &str = "**Security Validation:**";
    pub const COMMAND_VALIDATION: &str = "**Command Validation:**";
    pub const INTENT_CLASSIFICATION: &str = "**Intent Classification:**";
    pub const RATE_LIMIT_STATUS: &str = "**Rate Limit Status:**";
    pub const CONTENT_ANALYSIS: &str = "**Content Analysis:**";
    pub const PATTERN_DETECTION: &str = "**Pattern Detection:**";
    pub const SUGGESTED_ACTIONS: &str = "**Suggested Actions:**";
    pub const CORRECTION_PROMPT_HEADER: &str = "🔨 **Security Pattern Correction**";
    pub const CORRECTION_OPTIONS: &str = "**Available Actions:**";
    pub const FALSE_POSITIVE_OPTION: &str =
        "1️⃣ **False Positive** - This message should be allowed";
    pub const PATTERN_UPDATE_OPTION: &str =
        "2️⃣ **Update Pattern** - The validation rule needs adjustment";
    pub const WHITELIST_OPTION: &str = "3️⃣ **Whitelist** - Add exception for this specific case";
    pub const VALIDATION_CONTEXT: &str = "**Context for Pattern Analysis:**";
}

/// Metrics and stats messages
pub mod metrics {
    pub const SECURITY_METRICS_HEADER: &str = "🛡️ **Security Metrics**";
    pub const MESSAGE_STATS: &str = "📊 **Message Statistics:**";
    pub const INTENT_STATS: &str = "🎯 **Intent Classification:**";
    pub const THREAT_DETECTION: &str = "⚠️ **Threat Detection:**";
    pub const RATE_LIMIT_STATUS: &str = "📊 **Rate Limit Status**";
    pub const YOUR_RATE_LIMIT: &str = "📊 **Your Rate Limit Status**";
}

/// Usage instructions
pub mod usage {
    pub const RATE_LIMIT_RESET_USAGE: &str =
        "❌ Usage: `!spiral reset ratelimit @user` or `!spiral reset ratelimit <user_id>`";
}

/// Self-update system messages
pub mod auto_core_update {
    pub const PROCESSING: &str = "🔄 Processing self-update request...";
    pub const STARTING: &str = "🚀 Starting self-update";
    pub const WORKING: &str = "⚙️ Updating Spiral Core...";
    pub const RESTARTING: &str = "🔄 Restarting Spiral Core...";
    pub const SUCCESS: &str = "✅ Spiral Core Back online";
    pub const FAILURE: &str = "❌ Update failed:";
    pub const UNAUTHORIZED: &str = "🔒 Self-updates require authorization...";
    pub const QUEUE_BLOCKED: &str = "⏳ Self-update in progress. Your request has been queued.";
    pub const QUEUE_SUCCESS: &str = "✅ Self-update request queued successfully.";
    pub const INSUFFICIENT_INFO: &str =
        "❓ Insufficient information for self-update. Please provide more details:";
    pub const PRE_FLIGHT_FAILED: &str =
        "🚫 Pre-flight checks failed. Cannot proceed with self-update.";
    pub const ROLLBACK_SUCCESS: &str = "🔄 Self-update rolled back to previous state successfully.";
    pub const ROLLBACK_FAILED: &str =
        "💥 Critical: Self-update rollback failed. Manual intervention required.";
}

/// Risk level string constants
pub mod risk_levels {
    pub const LOW: &str = "Low";
    pub const MEDIUM: &str = "Medium";
    pub const HIGH: &str = "High";
    pub const CRITICAL: &str = "Critical";
}

/// Authorization helpers
pub struct AuthHelper;

impl AuthHelper {
    /// Check if user is authorized, return error message if not
    pub fn require_authorization(is_authorized: bool) -> Option<String> {
        if !is_authorized {
            Some(security::UNAUTHORIZED.to_string())
        } else {
            None
        }
    }

    /// Check if user is authorized, return Lordgenome quote if not
    pub fn require_authorization_with_quote(
        is_authorized: bool,
        username: &str,
        command: &str,
    ) -> Option<String> {
        if !is_authorized {
            use crate::discord::lordgenome_quotes::LordgenomeQuoteGenerator;
            let generator = LordgenomeQuoteGenerator::new();
            let action_type = LordgenomeQuoteGenerator::detect_action_type(command);
            Some(generator.generate_denial(username, action_type))
        } else {
            None
        }
    }
}

/// Macro for early return on authorization failure
#[macro_export]
macro_rules! require_auth {
    ($authorized:expr) => {
        if !$authorized {
            return Some($crate::discord::messages::security::UNAUTHORIZED.to_string());
        }
    };
}

/// Macro for early return on authorization failure with Lordgenome quote
#[macro_export]
macro_rules! require_auth_with_quote {
    ($authorized:expr, $username:expr, $command:expr) => {
        if !$authorized {
            use $crate::discord::lordgenome_quotes::LordgenomeQuoteGenerator;
            let generator = LordgenomeQuoteGenerator::new();
            let action_type = LordgenomeQuoteGenerator::detect_action_type($command);
            return Some(generator.generate_denial($username, &action_type));
        }
    };
}

/// Risk level utilities  
pub fn risk_level_to_str(risk_level: &crate::discord::RiskLevel) -> &'static str {
    match risk_level {
        crate::discord::RiskLevel::Low => risk_levels::LOW,
        crate::discord::RiskLevel::Medium => risk_levels::MEDIUM,
        crate::discord::RiskLevel::High => risk_levels::HIGH,
        crate::discord::RiskLevel::Critical => risk_levels::CRITICAL,
    }
}

/// Message formatting helpers
pub struct MessageFormatter;

impl MessageFormatter {
    /// Format a command blocked message
    pub fn command_blocked() -> String {
        format!(
            "{}\n\n*{}*",
            security::COMMAND_BLOCKED,
            security::COMMAND_BLOCKED_HINT
        )
    }

    /// Format a role assignment success message
    pub fn role_assigned(role_name: &str) -> String {
        format!(
            "{} {}",
            "✅",
            success::ROLE_ASSIGNED.replace("{}", role_name)
        )
    }

    /// Format rate limit reset message
    pub fn rate_limit_reset(user_id: u64) -> String {
        success::RATE_LIMIT_RESET.replace("{}", &user_id.to_string())
    }

    /// Format admin commands header
    pub fn admin_commands_header(has_access: bool) -> String {
        if has_access {
            format!("{}{}", info::ADMIN_COMMANDS, info::ADMIN_COMMANDS_AVAILABLE)
        } else {
            format!(
                "{}{}",
                info::ADMIN_COMMANDS,
                info::ADMIN_COMMANDS_RESTRICTED
            )
        }
    }

    /// Format debug report type
    pub fn debug_report_header(is_security: bool) -> &'static str {
        if is_security {
            debug::SECURITY_DEBUG_HEADER
        } else {
            debug::GENERAL_DEBUG_HEADER
        }
    }
}

/// Common message patterns
pub mod patterns {
    pub const COMMAND_BLOCKED_PATTERN: &str = "⚠️ Command blocked";
    pub const MESSAGE_FLAGGED_PATTERN: &str = "🚫 Message flagged";
}

/// Emoji constants
pub mod emojis {
    pub const TRASH_BIN: char = '🗑';
    pub const HAMMER: char = '🔨';
    pub const BUG: char = '🐛';
    pub const WRENCH: char = '🔧';
    pub const RETRY: char = '🔄';
    pub const DELETE: char = '❌';
    pub const EYES: char = '👀';
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_formatter() {
        let blocked = MessageFormatter::command_blocked();
        assert!(blocked.contains(security::COMMAND_BLOCKED));
        assert!(blocked.contains(security::COMMAND_BLOCKED_HINT));

        let role = MessageFormatter::role_assigned("SpiralDev");
        assert!(role.contains("SpiralDev"));

        let rate_limit = MessageFormatter::rate_limit_reset(123456789);
        assert!(rate_limit.contains("123456789"));
    }

    #[test]
    fn test_auth_helper() {
        // Test authorized user
        assert_eq!(AuthHelper::require_authorization(true), None);

        // Test unauthorized user
        let result = AuthHelper::require_authorization(false);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), security::UNAUTHORIZED);
    }

    #[test]
    fn test_risk_level_conversion() {
        use crate::discord::RiskLevel;

        assert_eq!(risk_level_to_str(&RiskLevel::Low), risk_levels::LOW);
        assert_eq!(risk_level_to_str(&RiskLevel::Medium), risk_levels::MEDIUM);
        assert_eq!(risk_level_to_str(&RiskLevel::High), risk_levels::HIGH);
        assert_eq!(
            risk_level_to_str(&RiskLevel::Critical),
            risk_levels::CRITICAL
        );
    }

    #[test]
    fn test_constants_defined() {
        // Ensure all constants have meaningful content
        assert!(security::COMMAND_BLOCKED.contains("blocked"));
        assert!(security::UNAUTHORIZED.contains("authorization"));
        assert!(errors::MESSAGE_TOO_LONG.contains("too long"));
        assert!(success::ROLES_CREATED.contains("Complete"));
        assert!(risk_levels::LOW.contains("Low"));
        assert!(risk_levels::CRITICAL.contains("Critical"));
    }
}
