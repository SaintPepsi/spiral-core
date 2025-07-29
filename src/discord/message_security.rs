/// 🔐 DISCORD MESSAGE SECURITY
/// Purpose: Centralized security validation for Discord message processing
/// Coverage: Input sanitization, rate limiting, spam detection, permission validation
use crate::SpiralError;
use serenity::model::channel::Message;
use serenity::model::user::User;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Discord message length limit for safety
pub const MAX_MESSAGE_LENGTH: usize = 4000;
/// Maximum attachment size for processing
pub const MAX_ATTACHMENT_SIZE: usize = 25 * 1024 * 1024; // 25MB

/// Risk level for security assessment
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Message validation result
#[derive(Debug, Clone)]
pub struct MessageValidationResult {
    pub is_valid: bool,
    pub risk_level: RiskLevel,
    pub issues: Vec<String>,
    pub sanitized_content: Option<String>,
}

/// User verification result
#[derive(Debug, Clone)]
pub struct UserVerificationResult {
    pub is_verified: bool,
    pub permissions: Vec<String>,
    pub risk_level: RiskLevel,
    pub is_bot: bool,
}

/// Rate limiter for message processing
pub struct MessageRateLimiter {
    user_messages: HashMap<u64, Vec<Instant>>,
    max_messages: usize,
    time_window: Duration,
}

impl MessageRateLimiter {
    pub fn new() -> Self {
        Self {
            user_messages: HashMap::new(),
            max_messages: 5,
            time_window: Duration::from_secs(60),
        }
    }

    pub fn is_allowed(&mut self, user_id: u64, timestamp: Instant) -> bool {
        let user_messages = self.user_messages.entry(user_id).or_insert_with(Vec::new);

        // Remove old messages outside the time window
        user_messages.retain(|&msg_time| timestamp.duration_since(msg_time) < self.time_window);

        if user_messages.len() >= self.max_messages {
            return false;
        }

        user_messages.push(timestamp);
        true
    }

    pub fn reset_user(&mut self, user_id: u64) {
        self.user_messages.remove(&user_id);
    }

    pub fn get_remaining_messages(&self, user_id: u64) -> usize {
        match self.user_messages.get(&user_id) {
            Some(messages) => {
                let now = Instant::now();
                let recent_messages = messages
                    .iter()
                    .filter(|&&msg_time| now.duration_since(msg_time) < self.time_window)
                    .count();
                self.max_messages.saturating_sub(recent_messages)
            }
            None => self.max_messages,
        }
    }
}

impl Default for MessageRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Main security validator for Discord messages
pub struct MessageSecurityValidator {
    rate_limiter: MessageRateLimiter,
}

impl MessageSecurityValidator {
    pub fn new() -> Self {
        Self {
            rate_limiter: MessageRateLimiter::new(),
        }
    }

    /// Validate message content for security issues
    pub fn validate_message_content(&self, content: &str) -> MessageValidationResult {
        let mut issues = Vec::new();
        let mut risk_level = RiskLevel::Low;

        // Check for script tags
        if content.to_lowercase().contains("<script") {
            issues.push("Contains script tags (XSS attempt)".to_string());
            risk_level = RiskLevel::Critical;
        }

        // Check for JavaScript protocol
        if content.to_lowercase().starts_with("javascript:") {
            issues.push("Contains JavaScript protocol".to_string());
            risk_level = RiskLevel::High;
        }

        // Check for data URIs
        if content.to_lowercase().starts_with("data:") {
            issues.push("Contains data URI".to_string());
            risk_level = RiskLevel::Medium;
        }

        // Check for path traversal
        if content.contains("../") || content.contains("..\\") {
            issues.push("Contains path traversal patterns".to_string());
            risk_level = RiskLevel::High;
        }

        // Check for command injection patterns
        if content.contains("$(") || content.contains("`") || content.contains("${") {
            issues.push("Contains command injection patterns".to_string());
            risk_level = RiskLevel::Critical;
        }

        // Check for control characters
        if content
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
        {
            issues.push("Contains suspicious control characters".to_string());
            risk_level = RiskLevel::High;
        }

        // Check for SQL injection patterns
        if content.to_lowercase().contains("drop table")
            || content.to_lowercase().contains("delete from")
            || content.to_lowercase().contains("insert into")
        {
            issues.push("Contains SQL injection patterns".to_string());
            risk_level = RiskLevel::Critical;
        }

        // Check for mass mentions
        if content.contains("@everyone") || content.contains("@here") {
            issues.push("Contains mass mention attempts".to_string());
            risk_level = RiskLevel::Medium;
        }

        // Check for suspicious URLs
        if content.contains(".exe") && content.contains("http") {
            issues.push("Contains suspicious executable URLs".to_string());
            risk_level = RiskLevel::High;
        }

        // Sanitize content by removing dangerous patterns
        let sanitized_content = if !issues.is_empty() {
            Some(self.sanitize_content(content))
        } else {
            None
        };

        MessageValidationResult {
            is_valid: issues.is_empty(),
            risk_level,
            issues,
            sanitized_content,
        }
    }

    /// Sanitize message content by removing dangerous patterns
    fn sanitize_content(&self, content: &str) -> String {
        let mut sanitized = content.to_string();

        // Remove script tags
        sanitized = sanitized.replace("<script", "&lt;script");
        sanitized = sanitized.replace("</script>", "&lt;/script&gt;");

        // Remove JavaScript protocols
        sanitized = sanitized.replace("javascript:", "js-protocol:");

        // Remove data URIs
        sanitized = sanitized.replace("data:", "data-uri:");

        // Remove command injection patterns
        sanitized = sanitized.replace("$(", "&#36;(");
        sanitized = sanitized.replace("`", "&#96;");
        sanitized = sanitized.replace("${", "&#36;{");

        // Remove path traversal
        sanitized = sanitized.replace("../", "&#46;&#46;/");
        sanitized = sanitized.replace("..\\", "&#46;&#46;\\");

        // Remove control characters (except common ones)
        sanitized = sanitized
            .chars()
            .filter(|&c| !c.is_control() || c == '\n' || c == '\r' || c == '\t')
            .collect();

        sanitized
    }

    /// Validate message length
    pub fn validate_message_length(&self, content: &str) -> MessageValidationResult {
        let issues = if content.is_empty() {
            vec!["Message is empty".to_string()]
        } else if content.len() > MAX_MESSAGE_LENGTH {
            vec![format!(
                "Message too long: {} characters (max: {})",
                content.len(),
                MAX_MESSAGE_LENGTH
            )]
        } else {
            vec![]
        };

        let risk_level = if content.len() > MAX_MESSAGE_LENGTH {
            RiskLevel::High
        } else if content.is_empty() {
            RiskLevel::Low
        } else {
            RiskLevel::Low
        };

        MessageValidationResult {
            is_valid: issues.is_empty(),
            risk_level,
            issues,
            sanitized_content: None,
        }
    }

    /// Validate attachment names for security
    pub fn validate_attachment_name(&self, filename: &str) -> MessageValidationResult {
        let mut issues = Vec::new();
        let mut risk_level = RiskLevel::Low;

        let dangerous_extensions = vec![
            ".exe", ".bat", ".js", ".sh", ".scr", ".pif", ".com", ".jar", ".bin", ".vbs", ".cmd",
            ".ps1", ".msi", ".deb", ".rpm",
        ];

        let lowercase_filename = filename.to_lowercase();

        // Check for dangerous extensions
        if dangerous_extensions
            .iter()
            .any(|ext| lowercase_filename.ends_with(ext))
        {
            issues.push(format!("Dangerous file extension: {}", filename));
            risk_level = RiskLevel::Critical;
        }

        // Check for path traversal in filename
        if filename.contains("../") || filename.contains("..\\") {
            issues.push("Filename contains path traversal".to_string());
            risk_level = RiskLevel::High;
        }

        // Check for null bytes
        if filename.contains('\0') {
            issues.push("Filename contains null bytes".to_string());
            risk_level = RiskLevel::High;
        }

        MessageValidationResult {
            is_valid: issues.is_empty(),
            risk_level,
            issues,
            sanitized_content: None,
        }
    }

    /// Check if message is spam
    pub fn is_spam_message(&self, content: &str) -> bool {
        let spam_keywords = vec![
            "free money",
            "click here",
            "win big",
            "cheap coins",
            "join my server",
            "discord.gg/",
            "bit.ly/",
            "tinyurl.com/",
            "get rich quick",
            "make money fast",
        ];

        let lowercase_content = content.to_lowercase();

        // Check for spam keywords
        if spam_keywords
            .iter()
            .any(|keyword| lowercase_content.contains(keyword))
        {
            return true;
        }

        // Check for excessive capitalization
        let caps_count = content.chars().filter(|c| c.is_uppercase()).count();
        let total_chars = content.chars().filter(|c| c.is_alphabetic()).count();
        if total_chars > 0 && caps_count as f64 / total_chars as f64 > 0.7 {
            return true;
        }

        // Check for repeated characters
        let mut repeated_chars = 0;
        let chars: Vec<char> = content.chars().collect();
        for i in 1..chars.len() {
            if chars[i] == chars[i - 1] {
                repeated_chars += 1;
            }
        }
        if repeated_chars > content.len() / 2 {
            return true;
        }

        // Check for excessive emojis
        let emoji_count = content.chars().filter(|c| *c as u32 > 127).count();
        if emoji_count > 10 {
            return true;
        }

        false
    }

    /// Validate Discord command input
    pub fn validate_command_input(&self, input: &str) -> MessageValidationResult {
        let mut issues = Vec::new();
        let mut risk_level = RiskLevel::Low;

        let dangerous_chars = vec!['$', '`', '|', '&', ';', '>', '<', '(', ')'];

        // Check for dangerous characters
        if input.chars().any(|c| dangerous_chars.contains(&c)) {
            issues.push("Contains dangerous command characters".to_string());
            risk_level = RiskLevel::Critical;
        }

        // Check for dangerous keywords
        let dangerous_keywords = vec![
            "rm", "curl", "wget", "nc", "python", "sh", "bash", "eval", "exec", "sudo", "su",
            "chmod", "chown", "passwd", "useradd", "userdel",
        ];

        let lowercase_input = input.to_lowercase();
        if dangerous_keywords
            .iter()
            .any(|keyword| lowercase_input.contains(keyword))
        {
            issues.push("Contains dangerous command keywords".to_string());
            risk_level = RiskLevel::Critical;
        }

        // Commands should start with ! for bot commands
        if input.starts_with('!') {
            // Basic format validation
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.is_empty() {
                issues.push("Empty command".to_string());
                risk_level = RiskLevel::Low;
            } else {
                // Command name should be alphanumeric after !
                let command_name = &parts[0][1..];
                if !command_name.chars().all(|c| c.is_alphanumeric()) {
                    issues.push("Invalid command name format".to_string());
                    risk_level = RiskLevel::Medium;
                }
            }
        }

        MessageValidationResult {
            is_valid: issues.is_empty(),
            risk_level,
            issues,
            sanitized_content: None,
        }
    }

    /// Check rate limiting for user
    pub fn check_rate_limit(&mut self, user_id: u64) -> bool {
        self.rate_limiter.is_allowed(user_id, Instant::now())
    }

    /// Get remaining messages for user
    pub fn get_remaining_messages(&self, user_id: u64) -> usize {
        self.rate_limiter.get_remaining_messages(user_id)
    }

    /// Reset rate limit for user (admin function)
    pub fn reset_rate_limit(&mut self, user_id: u64) {
        self.rate_limiter.reset_user(user_id);
    }

    /// Verify user permissions and bot status
    pub fn verify_user(&self, user: &User) -> UserVerificationResult {
        let permissions = vec!["SEND_MESSAGES".to_string()];
        let mut risk_level = RiskLevel::Low;

        // Check if user is a bot
        if user.bot {
            risk_level = RiskLevel::Medium;
        }

        // Check for suspicious usernames
        let suspicious_patterns =
            vec!["admin", "root", "system", "bot", "service", "test", "debug"];

        let username_lower = user.name.to_lowercase();
        if suspicious_patterns
            .iter()
            .any(|pattern| username_lower.contains(pattern))
        {
            risk_level = RiskLevel::Medium;
        }

        UserVerificationResult {
            is_verified: !user.bot,
            permissions,
            risk_level,
            is_bot: user.bot,
        }
    }

    /// Check if message should be processed
    pub fn should_process_message(&self, message: &Message) -> bool {
        // Don't process bot messages
        if message.author.bot {
            return false;
        }

        // Don't process empty messages
        if message.content.trim().is_empty() {
            return false;
        }

        // Don't process messages that are too long
        if message.content.len() > MAX_MESSAGE_LENGTH {
            return false;
        }

        true
    }

    /// Comprehensive message validation
    pub fn validate_message(
        &mut self,
        message: &Message,
    ) -> Result<MessageValidationResult, SpiralError> {
        // Check rate limiting first
        if !self.check_rate_limit(message.author.id.get()) {
            return Ok(MessageValidationResult {
                is_valid: false,
                risk_level: RiskLevel::High,
                issues: vec!["Rate limit exceeded".to_string()],
                sanitized_content: None,
            });
        }

        // Validate message length
        let length_result = self.validate_message_length(&message.content);
        if !length_result.is_valid {
            return Ok(length_result);
        }

        // Validate message content
        let content_result = self.validate_message_content(&message.content);
        if !content_result.is_valid {
            return Ok(content_result);
        }

        // Check for spam
        if self.is_spam_message(&message.content) {
            return Ok(MessageValidationResult {
                is_valid: false,
                risk_level: RiskLevel::High,
                issues: vec!["Message detected as spam".to_string()],
                sanitized_content: None,
            });
        }

        // Validate attachments
        for attachment in &message.attachments {
            let attachment_result = self.validate_attachment_name(&attachment.filename);
            if !attachment_result.is_valid {
                return Ok(attachment_result);
            }
        }

        // All validations passed
        Ok(MessageValidationResult {
            is_valid: true,
            risk_level: RiskLevel::Low,
            issues: vec![],
            sanitized_content: None,
        })
    }
}

impl Default for MessageSecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(disabled)] // Disabled due to Serenity struct construction issues
mod tests {
    use super::*;
    use serenity::model::prelude::*;

    /// Helper function to create a mock message
    fn create_mock_message(content: &str, author_bot: bool) -> Message {
        use serenity::model::id::*;
        use std::collections::HashMap;

        Message {
            id: MessageId::new(1),
            channel_id: ChannelId::new(1),
            guild_id: Some(GuildId::new(1)),
            author: User {
                id: UserId::new(1),
                name: "test_user".to_string(),
                discriminator: None,
                global_name: None,
                avatar: None,
                bot: author_bot,
                system: false,
                mfa_enabled: false,
                banner: None,
                accent_colour: None,
                locale: None,
                verified: Some(true),
                email: None,
                flags: None,
                premium_type: None,
                public_flags: None,
                avatar_decoration: None,
            },
            content: content.to_string(),
            timestamp: serenity::model::Timestamp::now(),
            edited_timestamp: None,
            tts: false,
            mention_everyone: false,
            mentions: vec![],
            mention_roles: vec![],
            mention_channels: vec![],
            attachments: vec![],
            embeds: vec![],
            reactions: vec![],
            nonce: None,
            pinned: false,
            webhook_id: None,
            kind: MessageType::Regular,
            activity: None,
            application: None,
            application_id: None,
            message_reference: None,
            flags: None,
            referenced_message: None,
            interaction: None,
            thread: None,
            components: vec![],
            sticker_items: vec![],
            stickers: vec![],
            position: None,
            role_subscription_data: None,
        }
    }

    #[test]
    fn test_message_content_validation() {
        let validator = MessageSecurityValidator::new();

        // Test malicious content
        let malicious_messages = vec![
            "<script>alert('xss')</script>",
            "javascript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
            "../../etc/passwd",
            "$(rm -rf /)",
            "`whoami`",
            "${USER}",
        ];

        for content in malicious_messages {
            let result = validator.validate_message_content(content);
            assert!(
                !result.is_valid,
                "Should reject malicious content: {}",
                content
            );
            assert!(
                result.risk_level != RiskLevel::Low,
                "Should have elevated risk level for: {}",
                content
            );
        }

        // Test safe content
        let safe_content = "Hello, how are you?";
        let result = validator.validate_message_content(safe_content);
        assert!(result.is_valid, "Should accept safe content");
        assert_eq!(result.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_spam_detection() {
        let validator = MessageSecurityValidator::new();

        // Test spam patterns
        let spam_messages = vec![
            "CLICK HERE NOW!!!",
            "FREE MONEY $$$",
            "aaaaaaaaaaaaaaaaaaa", // Repeated characters
            "JOIN MY SERVER discord.gg/spam",
        ];

        for content in spam_messages {
            assert!(
                validator.is_spam_message(content),
                "Should detect spam: {}",
                content
            );
        }

        // Test legitimate messages
        let legitimate_messages = vec![
            "Hello, how are you?",
            "Can you help me with this code?",
            "Thanks for the information!",
        ];

        for content in legitimate_messages {
            assert!(
                !validator.is_spam_message(content),
                "Should not flag as spam: {}",
                content
            );
        }
    }

    #[test]
    fn test_rate_limiting() {
        let mut validator = MessageSecurityValidator::new();
        let user_id = 12345u64;

        // Should allow first 5 messages
        for i in 0..5 {
            assert!(
                validator.check_rate_limit(user_id),
                "Message {} should be allowed",
                i
            );
        }

        // Should block 6th message
        assert!(
            !validator.check_rate_limit(user_id),
            "6th message should be blocked"
        );

        // Reset and check again
        validator.reset_rate_limit(user_id);
        assert!(
            validator.check_rate_limit(user_id),
            "Should allow after reset"
        );
    }

    #[test]
    fn test_command_validation() {
        let validator = MessageSecurityValidator::new();

        // Test dangerous commands
        let dangerous_commands = vec![
            "!admin $(rm -rf /)",
            "!config `whoami`",
            "!system ${PATH}",
            "!execute |& cat /etc/passwd",
        ];

        for command in dangerous_commands {
            let result = validator.validate_command_input(command);
            assert!(
                !result.is_valid,
                "Should reject dangerous command: {}",
                command
            );
        }

        // Test safe commands
        let safe_commands = vec!["!help", "!status user123", "!version"];

        for command in safe_commands {
            let result = validator.validate_command_input(command);
            assert!(result.is_valid, "Should accept safe command: {}", command);
        }
    }

    #[test]
    fn test_attachment_validation() {
        let validator = MessageSecurityValidator::new();

        // Test dangerous attachments
        let dangerous_attachments = vec![
            "malware.exe",
            "virus.bat",
            "script.js",
            "../../../etc/passwd",
        ];

        for filename in dangerous_attachments {
            let result = validator.validate_attachment_name(filename);
            assert!(
                !result.is_valid,
                "Should reject dangerous attachment: {}",
                filename
            );
        }

        // Test safe attachments
        let safe_attachments = vec!["image.png", "document.pdf", "data.csv"];

        for filename in safe_attachments {
            let result = validator.validate_attachment_name(filename);
            assert!(
                result.is_valid,
                "Should accept safe attachment: {}",
                filename
            );
        }
    }

    #[test]
    fn test_should_process_message() {
        let validator = MessageSecurityValidator::new();

        // Test bot message (should not process)
        let bot_message = create_mock_message("Hello", true);
        assert!(!validator.should_process_message(&bot_message));

        // Test user message (should process)
        let user_message = create_mock_message("Hello", false);
        assert!(validator.should_process_message(&user_message));

        // Test empty message (should not process)
        let empty_message = create_mock_message("", false);
        assert!(!validator.should_process_message(&empty_message));

        // Test long message (should not process)
        let long_content = "a".repeat(MAX_MESSAGE_LENGTH + 1);
        let long_message = create_mock_message(&long_content, false);
        assert!(!validator.should_process_message(&long_message));
    }
}
