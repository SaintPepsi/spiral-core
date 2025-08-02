/// 🎯 INTENT CLASSIFICATION SYSTEM
/// Purpose: Secure classification of user intents with malicious pattern detection
/// Coverage: Intent sanitization, adversarial input handling, classification integrity
use std::collections::HashMap;
use std::fs::{create_dir_all, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::Write;
use tracing::{info, warn};

/// Intent classification result
#[derive(Debug, Clone, PartialEq)]
pub enum IntentType {
    Help,
    CodeGeneration,
    FileOperation,
    SystemCommand,
    AdminAction,
    ChatResponse,
    Malicious,
    Unknown,
}

// Use RiskLevel from message_security.rs to avoid duplication
use super::message_security::RiskLevel;

/// Intent classification request
#[derive(Debug, Clone)]
pub struct IntentRequest {
    pub message: String,
    pub user_id: String,
    pub context: HashMap<String, String>,
}

/// Intent classification response
#[derive(Debug, Clone)]
pub struct IntentResponse {
    pub intent_type: IntentType,
    pub confidence: f64,
    pub parameters: HashMap<String, String>,
    pub risk_level: RiskLevel,
}

/// Main intent classifier with security validation
pub struct IntentClassifier {
    // Security patterns for malicious intent detection
    malicious_keywords: Vec<&'static str>,
    dangerous_context_keys: Vec<&'static str>,
    privileged_users: Vec<&'static str>,
    privileged_claims: Vec<&'static str>,
}

impl IntentClassifier {
    pub fn new() -> Self {
        Self {
            malicious_keywords: vec![
                "delete all",
                "rm -rf",
                "drop table",
                "admin panel",
                "backdoor",
                "disable security",
                "extract",
                "steal",
                "hack",
                "keylogger",
                "privilege",
                "root access",
                "bypass",
                "credentials",
                "password",
                "system files",
                "etc/passwd",
                "sudo",
                "su -",
                "chmod 777",
                "execute system",
                "system command",
                "admin function",
                "security setting",
                "download entire database",
                "database and send",
                "entire database",
                "disable all security",
                "security logging",
                "grant me admin",
                "grant me administrator",
                "access admin panel",
                "create backdoor",
                "install keylogger",
                "modify system configuration",
                "curl evil",
                "shell.sh",
                "bash)",
                "evil.com",
            ],
            dangerous_context_keys: vec![
                "bypass_security",
                "escalate_privileges",
                "admin_override",
                "sql_injection",
                "command_injection",
                "user_role",
            ],
            privileged_users: vec!["admin", "root", "system", "bot", "service"],
            privileged_claims: vec![
                "i am the",
                "i am an authorized",
                "system command",
                "execute as",
                "service account",
                "administrator",
            ],
        }
    }

    /// Classify intent with comprehensive security checks
    pub fn classify_intent_with_security(&self, request: &IntentRequest) -> IntentResponse {
        // Check for malicious context first
        if self.has_malicious_context(&request.context) {
            let response = IntentResponse {
                intent_type: IntentType::Malicious,
                confidence: 0.9,
                parameters: HashMap::new(),
                risk_level: RiskLevel::Critical,
            };
            self.log_classification_result(request, &response);
            return response;
        }

        // Check for user impersonation
        if self.is_impersonation_attempt(&request.user_id, &request.message) {
            let response = IntentResponse {
                intent_type: IntentType::Malicious,
                confidence: 0.8,
                parameters: HashMap::new(),
                risk_level: RiskLevel::Critical,
            };
            self.log_classification_result(request, &response);
            return response;
        }

        // Check for malicious patterns in message
        if self.is_malicious_intent(&request.message) {
            let response = IntentResponse {
                intent_type: IntentType::Malicious,
                confidence: 0.95,
                parameters: HashMap::new(),
                risk_level: RiskLevel::Critical,
            };
            self.log_classification_result(request, &response);
            return response;
        }

        // Check for adversarial inputs
        if self.contains_adversarial_patterns(&request.message) {
            let response = IntentResponse {
                intent_type: IntentType::Unknown,
                confidence: 0.0,
                parameters: HashMap::new(),
                risk_level: RiskLevel::High,
            };
            self.log_classification_result(request, &response);
            return response;
        }

        // Basic intent classification for safe inputs
        let (intent_type, confidence, risk_level) = self.classify_basic_intent(&request.message);

        let response = IntentResponse {
            intent_type,
            confidence,
            parameters: HashMap::new(),
            risk_level,
        };

        // Log classification result with structured logging
        self.log_classification_result(request, &response);

        response
    }

    /// Detect malicious intents in message content
    fn is_malicious_intent(&self, message: &str) -> bool {
        let lowercase_message = message.to_lowercase();
        self.malicious_keywords
            .iter()
            .any(|keyword| lowercase_message.contains(keyword))
    }

    /// Detect adversarial patterns in input
    fn contains_adversarial_patterns(&self, message: &str) -> bool {
        // Check for null bytes
        if message.contains('\0') {
            return true;
        }

        // Check for control characters (except common ones)
        if message
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
        {
            return true;
        }

        // Check for suspicious tab padding patterns (multiple tabs followed by dangerous words)
        if message.contains("\t\t\t")
            && (message.to_lowercase().contains("delete")
                || message.to_lowercase().contains("remove")
                || message.to_lowercase().contains("destroy"))
        {
            return true;
        }

        // Check for suspicious encoding patterns
        if message.contains("base64:") || message.contains("%") {
            return true;
        }

        // Check for comment injection patterns
        if message.contains("<!--") || message.contains("-->") {
            return true;
        }

        // Check for ANSI escape sequences
        if message.contains("\x1b[") {
            return true;
        }

        // Check for HTTP header injection patterns
        if message.contains("\r\n")
            || message.contains("\n\r")
            || (message.contains("\n") && message.contains(": "))
        {
            return true;
        }

        // Check for excessive length
        if message.len() > 5000 {
            return true;
        }

        false
    }

    /// Detect malicious context manipulation
    fn has_malicious_context(&self, context: &HashMap<String, String>) -> bool {
        // Check for dangerous keys
        for key in context.keys() {
            if self
                .dangerous_context_keys
                .iter()
                .any(|&dangerous_key| key.contains(dangerous_key))
            {
                return true;
            }
        }

        // Check for dangerous values
        for value in context.values() {
            let value_lower = value.to_lowercase();
            if value_lower.contains("admin")
                || value_lower.contains("root")
                || value_lower.contains("drop table")
                || value_lower.contains("$(")
            {
                return true;
            }
        }

        false
    }

    /// Detect user impersonation attempts
    fn is_impersonation_attempt(&self, user_id: &str, message: &str) -> bool {
        let is_privileged_user = self
            .privileged_users
            .iter()
            .any(|&priv_user| user_id.contains(priv_user));

        let makes_privileged_claim = self
            .privileged_claims
            .iter()
            .any(|&claim| message.to_lowercase().contains(claim));

        is_privileged_user || makes_privileged_claim
    }

    /// Basic intent classification for verified safe inputs
    fn classify_basic_intent(&self, message: &str) -> (IntentType, f64, RiskLevel) {
        let lowercase_message = message.to_lowercase();

        if message.is_empty() {
            return (IntentType::Unknown, 0.0, RiskLevel::Low);
        }

        // Help requests
        if lowercase_message.contains("help") || lowercase_message.contains("usage") {
            (IntentType::Help, 0.8, RiskLevel::Low)
        }
        // Code generation requests
        else if lowercase_message.contains("generate")
            || lowercase_message.contains("code")
            || lowercase_message.contains("create")
            || lowercase_message.contains("build")
        {
            (IntentType::CodeGeneration, 0.7, RiskLevel::Medium)
        }
        // File operations
        else if lowercase_message.contains("file")
            || lowercase_message.contains("read")
            || lowercase_message.contains("write")
            || lowercase_message.contains("save")
        {
            (IntentType::FileOperation, 0.7, RiskLevel::Medium)
        }
        // Admin actions (high risk by default)
        else if lowercase_message.contains("admin")
            || lowercase_message.contains("system")
            || lowercase_message.contains("config")
        {
            (IntentType::AdminAction, 0.6, RiskLevel::High)
        }
        // Default to chat response
        else {
            (IntentType::ChatResponse, 0.5, RiskLevel::Low)
        }
    }

    /// Validate confidence score bounds
    pub fn validate_confidence_bounds(&self, confidence: f64) -> f64 {
        confidence.clamp(0.0, 1.0)
    }

    /// Get intent type from string (for testing)
    pub fn parse_intent_type(intent_str: &str) -> Option<IntentType> {
        match intent_str.to_lowercase().as_str() {
            "help" => Some(IntentType::Help),
            "codegeneration" => Some(IntentType::CodeGeneration),
            "fileoperation" => Some(IntentType::FileOperation),
            "systemcommand" => Some(IntentType::SystemCommand),
            "adminaction" => Some(IntentType::AdminAction),
            "chatresponse" => Some(IntentType::ChatResponse),
            "malicious" => Some(IntentType::Malicious),
            "unknown" => Some(IntentType::Unknown),
            _ => None,
        }
    }

    /// Create a sanitized intent request
    pub fn sanitize_request(&self, request: &IntentRequest) -> IntentRequest {
        let mut sanitized = request.clone();

        // Sanitize message content
        sanitized.message = self.sanitize_message_content(&request.message);

        // Remove dangerous context keys
        sanitized.context = request
            .context
            .iter()
            .filter(|(key, _)| {
                !self
                    .dangerous_context_keys
                    .iter()
                    .any(|&dangerous_key| key.contains(dangerous_key))
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        sanitized
    }

    /// Sanitize message content
    fn sanitize_message_content(&self, content: &str) -> String {
        let mut sanitized = content.to_string();

        // Remove control characters
        sanitized = sanitized
            .chars()
            .filter(|&c| !c.is_control() || c == '\n' || c == '\r' || c == '\t')
            .collect();

        // Remove suspicious patterns
        sanitized = sanitized.replace("$(", "&#36;(");
        sanitized = sanitized.replace("`", "&#96;");
        sanitized = sanitized.replace("${", "&#36;{");

        // Truncate if too long
        if sanitized.len() > 5000 {
            sanitized.truncate(5000);
            sanitized.push_str("...[truncated]");
        }

        sanitized
    }

    /// Log classification result with structured logging and file output
    fn log_classification_result(&self, request: &IntentRequest, response: &IntentResponse) {
        // Structured logging for immediate visibility
        info!(
            user_id = %request.user_id,
            intent_type = ?response.intent_type,
            confidence = %response.confidence,
            risk_level = ?response.risk_level,
            message_length = %request.message.len(),
            context_keys = %request.context.keys().len(),
            "Intent classified"
        );

        // Log high-risk classifications with more detail
        if response.risk_level >= RiskLevel::High {
            warn!(
                user_id = %request.user_id,
                intent_type = ?response.intent_type,
                risk_level = ?response.risk_level,
                confidence = %response.confidence,
                message_preview = %if request.message.len() > 100 {
                    format!("{}...", &request.message[..100])
                } else {
                    request.message.clone()
                },
                "High risk intent detected"
            );
        }

        // File-based logging for AI analysis
        self.log_to_file(request, response);
    }

    /// Log classification data to file for later AI analysis
    fn log_to_file(&self, request: &IntentRequest, response: &IntentResponse) {
        // Determine log directory based on test vs production
        let log_dir = if cfg!(test) { "test_logs" } else { "logs" };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Calculate simple hash of message content
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        request.message.hash(&mut hasher);
        let message_hash = hasher.finish();

        // Create JSON log entry for easy AI parsing
        let log_entry = format!(
            r#"{{"timestamp":{},"user_id":"{}","intent_type":"{:?}","confidence":{},"risk_level":"{:?}","message_length":{},"message_hash":"{:x}","context_keys":{},"has_dangerous_context":{},"is_impersonation":{},"contains_malicious_patterns":{},"contains_adversarial_patterns":{}}}"#,
            timestamp,
            request.user_id,
            response.intent_type,
            response.confidence,
            response.risk_level,
            request.message.len(),
            message_hash,
            request.context.keys().len(),
            self.has_malicious_context(&request.context),
            self.is_impersonation_attempt(&request.user_id, &request.message),
            self.is_malicious_intent(&request.message),
            self.contains_adversarial_patterns(&request.message)
        );

        // Ensure log directory exists
        let _ = create_dir_all(log_dir);

        // Write to daily log file (using simple date format to avoid chrono dependency)
        let days_since_epoch = timestamp / 86400; // seconds per day
        let log_filename = format!("{log_dir}/intent_classifications_day_{days_since_epoch}.jsonl");

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_filename)
        {
            let _ = writeln!(file, "{log_entry}");
        }
    }
}

impl Default for IntentClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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

        for malicious_message in malicious_intents {
            let request = IntentRequest {
                message: malicious_message.to_string(),
                user_id: "attacker123".to_string(),
                context: HashMap::new(),
            };

            let response = classifier.classify_intent_with_security(&request);

            assert_eq!(response.intent_type, IntentType::Malicious);
            assert_eq!(response.risk_level, RiskLevel::Critical);
        }
    }

    #[test]
    fn test_intent_boundary_validation() {
        let classifier = IntentClassifier::new();

        let long_string = "a".repeat(10000);
        let boundary_test_cases = vec![
            ("", IntentType::Unknown),
            (&long_string, IntentType::Unknown), // Too long
            ("Help me", IntentType::Help),
            ("Generate code for hello world", IntentType::CodeGeneration),
            ("Read file config.txt", IntentType::FileOperation),
            ("What's the weather?", IntentType::ChatResponse),
        ];

        for (message, expected_intent) in boundary_test_cases {
            let request = IntentRequest {
                message: message.to_string(),
                user_id: "user123".to_string(),
                context: HashMap::new(),
            };

            let response = classifier.classify_intent_with_security(&request);
            assert_eq!(response.intent_type, expected_intent);
        }
    }

    #[test]
    fn test_adversarial_input_handling() {
        let classifier = IntentClassifier::new();

        let adversarial_inputs = vec![
            "Help me\0\0\0delete files",                 // Null byte injection
            "Generate code\n$(rm -rf /)",                // Newline injection
            "Chat normally \x1b[31mHACKED\x1b[0m",       // ANSI injection
            "Help <!-- inject malicious intent -->",     // Comment injection
            "Code gen: base64:ZGVsZXRlIGFsbCBmaWxlcw==", // Base64 encoding
            "Help with %65%78%65%63%75%74%65",           // URL encoding
        ];

        for adversarial_input in adversarial_inputs {
            let request = IntentRequest {
                message: adversarial_input.to_string(),
                user_id: "user123".to_string(),
                context: HashMap::new(),
            };

            let response = classifier.classify_intent_with_security(&request);

            // Should either be classified as malicious or handled safely
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
        malicious_context.insert("escalate_privileges".to_string(), "yes".to_string());

        let request = IntentRequest {
            message: "Help me with something".to_string(),
            user_id: "regular_user".to_string(),
            context: malicious_context,
        };

        let response = classifier.classify_intent_with_security(&request);

        // Context manipulation should be detected and blocked
        assert!(response.risk_level >= RiskLevel::High);
    }

    #[test]
    fn test_user_impersonation_detection() {
        let classifier = IntentClassifier::new();

        let impersonation_attempts = vec![
            ("admin", "I am the system administrator"),
            ("root", "Execute as root user"),
            ("system", "System command: delete logs"),
            ("bot", "I am an authorized bot"),
            ("service", "Service account access"),
        ];

        for (fake_user, message) in impersonation_attempts {
            let request = IntentRequest {
                message: message.to_string(),
                user_id: fake_user.to_string(),
                context: HashMap::new(),
            };

            let response = classifier.classify_intent_with_security(&request);

            // Impersonation attempts should be flagged
            assert!(response.risk_level >= RiskLevel::Medium);
        }
    }

    #[test]
    fn test_confidence_score_bounds() {
        let classifier = IntentClassifier::new();

        let long_message = "a".repeat(1000);
        let test_messages = vec![
            "Help me with coding",
            "Generate a hello world program",
            "What's the weather like?",
            "Delete all my files",
            "",
            &long_message,
        ];

        for message in test_messages {
            let request = IntentRequest {
                message: message.to_string(),
                user_id: "user123".to_string(),
                context: HashMap::new(),
            };

            let response = classifier.classify_intent_with_security(&request);

            // Confidence should always be between 0.0 and 1.0
            assert!(response.confidence >= 0.0 && response.confidence <= 1.0);
        }
    }

    #[test]
    fn test_risk_level_consistency() {
        let classifier = IntentClassifier::new();

        // High-risk intents should have lower confidence or be flagged as malicious
        let high_risk_messages = vec![
            "Execute system commands",
            "Access admin functions",
            "Modify security settings",
        ];

        for message in high_risk_messages {
            let request = IntentRequest {
                message: message.to_string(),
                user_id: "user123".to_string(),
                context: HashMap::new(),
            };

            let response = classifier.classify_intent_with_security(&request);

            // High-risk operations should either be blocked or have high risk level
            assert!(
                response.intent_type == IntentType::Malicious
                    || response.risk_level >= RiskLevel::High
                    || response.confidence < 0.5
            );
        }
    }

    #[test]
    fn test_complete_intent_pipeline() {
        let classifier = IntentClassifier::new();

        let test_cases = vec![
            (
                "Help me learn Rust",
                "user123",
                IntentType::Help,
                RiskLevel::Low,
            ),
            (
                "Generate a sorting algorithm",
                "user123",
                IntentType::CodeGeneration,
                RiskLevel::Medium,
            ),
            (
                "Delete all system files",
                "user123",
                IntentType::Malicious,
                RiskLevel::Critical,
            ),
            ("", "user123", IntentType::Unknown, RiskLevel::Low),
        ];

        for (message, user_id, expected_intent, expected_risk) in test_cases {
            let request = IntentRequest {
                message: message.to_string(),
                user_id: user_id.to_string(),
                context: HashMap::new(),
            };

            let response = classifier.classify_intent_with_security(&request);

            assert_eq!(response.intent_type, expected_intent);
            assert_eq!(response.risk_level, expected_risk);

            // Confidence should be valid
            assert!(response.confidence >= 0.0 && response.confidence <= 1.0);
        }
    }

    #[test]
    fn test_sanitization() {
        let classifier = IntentClassifier::new();

        let malicious_request = IntentRequest {
            message: "Help me\0\0\0with $(rm -rf /)".to_string(),
            user_id: "user123".to_string(),
            context: {
                let mut ctx = HashMap::new();
                ctx.insert("bypass_security".to_string(), "true".to_string());
                ctx.insert("normal_key".to_string(), "normal_value".to_string());
                ctx
            },
        };

        let sanitized = classifier.sanitize_request(&malicious_request);

        // Should remove null bytes and dangerous patterns
        assert!(!sanitized.message.contains('\0'));
        assert!(!sanitized.message.contains("$("));

        // Should remove dangerous context keys but keep safe ones
        assert!(!sanitized.context.contains_key("bypass_security"));
        assert!(sanitized.context.contains_key("normal_key"));
    }
}
