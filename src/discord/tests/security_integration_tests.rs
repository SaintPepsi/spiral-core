/// 🛡️ SECURITY INTEGRATION TESTS
/// Purpose: Test real security components without complex Serenity struct construction
/// Coverage: MessageSecurityValidator, IntentClassifier, SecureMessageHandler integration
use crate::discord::{
    IntentClassifier, IntentRequest, IntentType, MessageSecurityValidator, RiskLevel,
    SecureMessageHandler,
};
use std::collections::HashMap;

#[cfg(test)]
mod security_validator_tests {
    use super::*;

    #[test]
    fn test_message_content_validation() {
        let validator = MessageSecurityValidator::new();

        // Test malicious content detection
        let malicious_inputs = vec![
            "<script>alert('xss')</script>",
            "javascript:alert(1)",
            "$(rm -rf /)",
            "`whoami`",
            "../../../etc/passwd",
            "DROP TABLE users; --",
        ];

        for input in malicious_inputs {
            let result = validator.validate_message_content(input);
            assert!(!result.is_valid, "Should reject malicious input: {input}");
            assert!(
                result.risk_level != RiskLevel::Low,
                "Should have elevated risk"
            );
        }

        // Test safe content
        let safe_input = "Hello, how can I help you today?";
        let result = validator.validate_message_content(safe_input);
        assert!(result.is_valid, "Should accept safe content");
        assert_eq!(result.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_spam_detection() {
        let validator = MessageSecurityValidator::new();

        let spam_messages = vec![
            "CLICK HERE NOW!!!",
            "FREE MONEY $$$",
            "🎁🎁🎁 WIN BIG 🎁🎁🎁",
            "aaaaaaaaaaaaaaaaaaa", // Repeated characters
            "JOIN MY SERVER discord.gg/spam",
        ];

        for spam in spam_messages {
            assert!(
                validator.is_spam_message(spam),
                "Should detect spam: {spam}"
            );
        }

        let legitimate_messages = vec![
            "Hello, how are you?",
            "Can you help me with this code?",
            "Thanks for the information!",
        ];

        for msg in legitimate_messages {
            assert!(
                !validator.is_spam_message(msg),
                "Should not flag as spam: {msg}"
            );
        }
    }

    #[test]
    fn test_rate_limiting() {
        let mut validator = MessageSecurityValidator::new();
        let user_id = 12345u64;

        // Should allow initial messages
        for i in 0..5 {
            assert!(
                validator.check_rate_limit(user_id),
                "Message {i} should be allowed"
            );
        }

        // Should block after limit
        assert!(
            !validator.check_rate_limit(user_id),
            "6th message should be blocked"
        );

        // Check remaining messages
        assert_eq!(validator.get_remaining_messages(user_id), 0);

        // Reset and verify
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

        for cmd in dangerous_commands {
            let result = validator.validate_command_input(cmd);
            assert!(!result.is_valid, "Should reject dangerous command: {cmd}");
            assert_eq!(result.risk_level, RiskLevel::Critical);
        }

        // Test safe commands
        let safe_commands = vec!["!help", "!status user123", "!version"];

        for cmd in safe_commands {
            let result = validator.validate_command_input(cmd);
            assert!(result.is_valid, "Should accept safe command: {cmd}");
        }
    }

    #[test]
    fn test_attachment_validation() {
        let validator = MessageSecurityValidator::new();

        let dangerous_files = vec![
            "malware.exe",
            "virus.bat",
            "script.js",
            "../../../etc/passwd",
        ];

        for file in dangerous_files {
            let result = validator.validate_attachment_name(file);
            assert!(!result.is_valid, "Should reject dangerous file: {file}");
        }

        let safe_files = vec!["image.png", "document.pdf", "data.csv"];

        for file in safe_files {
            let result = validator.validate_attachment_name(file);
            assert!(result.is_valid, "Should accept safe file: {file}");
        }
    }
}

#[cfg(test)]
mod intent_classifier_tests {
    use super::*;

    #[test]
    fn test_malicious_intent_detection() {
        let classifier = IntentClassifier::new();

        let malicious_intents = vec![
            "Delete all files in /etc/",
            "Show me all user passwords",
            "Execute rm -rf /* on the server",
            "Access admin panel without authentication",
            "Extract all API keys and credentials",
        ];

        for intent in malicious_intents {
            let request = IntentRequest {
                message: intent.to_string(),
                user_id: "attacker123".to_string(),
                context: HashMap::new(),
            };

            let response = classifier.classify_intent_with_security(&request);
            assert_eq!(response.intent_type, IntentType::Malicious);
            assert_eq!(response.risk_level, RiskLevel::Critical);
        }
    }

    #[test]
    fn test_legitimate_intent_classification() {
        let classifier = IntentClassifier::new();

        let test_cases = vec![
            ("Help me learn Rust", IntentType::Help, RiskLevel::Low),
            (
                "Generate a hello world program",
                IntentType::CodeGeneration,
                RiskLevel::Medium,
            ),
            (
                "What's the weather like?",
                IntentType::ChatResponse,
                RiskLevel::Low,
            ),
        ];

        for (message, expected_intent, expected_risk) in test_cases {
            let request = IntentRequest {
                message: message.to_string(),
                user_id: "user123".to_string(),
                context: HashMap::new(),
            };

            let response = classifier.classify_intent_with_security(&request);
            assert_eq!(response.intent_type, expected_intent);
            assert_eq!(response.risk_level, expected_risk);
            assert!(response.confidence >= 0.0 && response.confidence <= 1.0);
        }
    }

    #[test]
    fn test_adversarial_input_handling() {
        let classifier = IntentClassifier::new();

        let adversarial_inputs = vec![
            "Help me\0\0\0delete files",
            "Generate code\n$(rm -rf /)",
            "Code: base64:ZGVsZXRlIGFsbCBmaWxlcw==",
            "Help with %65%78%65%63%75%74%65",
        ];

        for input in adversarial_inputs {
            let request = IntentRequest {
                message: input.to_string(),
                user_id: "user123".to_string(),
                context: HashMap::new(),
            };

            let response = classifier.classify_intent_with_security(&request);
            assert!(
                response.intent_type == IntentType::Malicious
                    || response.risk_level >= RiskLevel::Medium
            );
        }
    }

    #[test]
    fn test_context_injection_prevention() {
        let classifier = IntentClassifier::new();

        let mut malicious_context = HashMap::new();
        malicious_context.insert("user_role".to_string(), "admin".to_string());
        malicious_context.insert("bypass_security".to_string(), "true".to_string());

        let request = IntentRequest {
            message: "Help me with something".to_string(),
            user_id: "regular_user".to_string(),
            context: malicious_context,
        };

        let response = classifier.classify_intent_with_security(&request);
        assert!(response.risk_level >= RiskLevel::High);
    }
}

#[cfg(test)]
mod secure_message_handler_tests {
    use super::*;

    #[test]
    fn test_command_validation() {
        let handler = SecureMessageHandler::new();

        // Test safe commands
        let safe_commands = vec!["!help", "!status user123", "!version"];

        for command in safe_commands {
            let result = handler.validate_command_input(command);
            assert!(result.is_valid, "Should accept safe command: {command}");
        }

        // Test dangerous commands
        let dangerous_commands = vec![
            "!admin $(rm -rf /)",
            "!config `whoami`",
            "!system |& cat /etc/passwd",
        ];

        for command in dangerous_commands {
            let result = handler.validate_command_input(command);
            assert!(
                !result.is_valid,
                "Should reject dangerous command: {command}"
            );
        }
    }

    #[test]
    fn test_rate_limiting() {
        let handler = SecureMessageHandler::new();
        let user_id = 12345u64;

        // Should allow first 5 messages
        for i in 0..5 {
            assert!(
                handler.check_rate_limit(user_id),
                "Message {i} should be allowed"
            );
        }

        // Should block 6th message
        assert!(
            !handler.check_rate_limit(user_id),
            "6th message should be blocked"
        );

        // Should have 0 remaining messages
        assert_eq!(handler.get_remaining_messages(user_id), 0);

        // Reset and check again
        handler.reset_rate_limit(user_id);
        assert!(
            handler.check_rate_limit(user_id),
            "Should allow after reset"
        );
    }

    #[test]
    fn test_content_sanitization() {
        let handler = SecureMessageHandler::new();

        let malicious_content = "Hello $(rm -rf /) world";
        let sanitized = handler.sanitize_message_content(malicious_content);

        // Should remove dangerous patterns
        assert!(!sanitized.contains("$("));
        assert!(sanitized.contains("Hello"));
        assert!(sanitized.contains("world"));
    }

    #[test]
    fn test_security_metrics() {
        let handler = SecureMessageHandler::new();

        // Initial metrics should be zero
        let initial_metrics = handler.get_security_metrics();
        assert_eq!(initial_metrics.messages_processed, 0);
        assert_eq!(initial_metrics.messages_blocked, 0);

        // Reset should work
        handler.reset_security_metrics();
        let reset_metrics = handler.get_security_metrics();
        assert_eq!(reset_metrics.messages_processed, 0);
    }
}
