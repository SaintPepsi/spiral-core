/// 🛡️ INPUT VALIDATION: Critical security boundary for user content
/// DECISION ARCHAEOLOGY: Every validation rule includes security reasoning
/// AUDIT CHECKPOINT: These patterns prevent XSS, injection, and DoS attacks
use crate::SpiralError;
use html_escape::encode_text;
use regex::Regex;
use std::collections::HashSet;

/// 📏 MAX TASK CONTENT LENGTH: DoS protection via size limits
/// Why: 10KB allows full feature descriptions while preventing memory exhaustion
/// Calculation: 10K chars ≈ 2-3 pages of text, sufficient for detailed requirements
/// Alternative: 50KB (rejected: DoS risk), 1KB (rejected: insufficient for complex tasks)
pub const MAX_TASK_CONTENT_LENGTH: usize = 10000;

/// 🔑 MAX CONTEXT KEY LENGTH: Prevent malformed or malicious key names
/// Why: 100 chars allows descriptive keys while preventing abuse
/// Reasoning: Longest reasonable key: "project_framework_configuration_override" = 43 chars
/// Alternative: 50 chars (rejected: too restrictive), 500 chars (rejected: abuse potential)
pub const MAX_CONTEXT_KEY_LENGTH: usize = 100;

/// 📝 MAX CONTEXT VALUE LENGTH: Balance utility with resource protection
/// Why: 1KB allows rich context while preventing memory attacks
/// Use case: File paths, configuration snippets, requirement descriptions
/// Alternative: 10KB (rejected: DoS risk), 100 chars (rejected: insufficient context)
pub const MAX_CONTEXT_VALUE_LENGTH: usize = 1000;

/// 🔤 SAFE CHARACTER REGEX: Comprehensive allowlist for secure content
/// DECISION: Allowlist approach more secure than blocklist (can't anticipate all attacks)
/// Why: Includes all necessary chars for code requests: A-Z, 0-9, punctuation, whitespace
/// Security: Blocks control chars, unicode exploits, binary data
/// Alternative: Blocklist (rejected: whack-a-mole security), Plain text only (rejected: too restrictive)
static SAFE_TASK_CONTENT_REGEX: &str =
    r"^[a-zA-Z0-9\s\.,!?:;()\[\]{}\-_+=@#$%^&*|\\/<>'`~\n\r\t]+$";

/// 🚨 DANGEROUS PATTERNS: Known attack vectors and exploitation attempts
/// SECURITY PHILOSOPHY: Multi-layer defense with pattern recognition
/// Why these patterns: Based on OWASP top 10 and real-world attack data
/// Updates: Review quarterly and add new patterns from security advisories
static DANGEROUS_PATTERNS: &[&str] = &[
    // 📜 SCRIPT INJECTION: Prevent XSS and code execution
    "<script",        // HTML script tags
    "javascript:",    // JavaScript URLs
    "data:text/html", // Data URLs with HTML
    "vbscript:",      // VBScript URLs (legacy but still dangerous)
    // 💻 COMMAND INJECTION: Prevent shell command execution
    "&&",    // Command chaining
    "||",    // Command OR logic
    ";rm",   // Unix file deletion
    ";del",  // Windows file deletion
    "`rm",   // Backtick command substitution
    "`del",  // Backtick Windows deletion
    "$(rm",  // Command substitution
    "$(del", // Command substitution Windows
    // 🗄️ SQL INJECTION: Prevent database attacks
    "';",           // SQL statement termination
    "\";",          // Double-quote SQL termination
    "union select", // SQL UNION attacks
    "drop table",   // Table deletion
    "delete from",  // Record deletion
    // 📁 PATH TRAVERSAL: Prevent file system access
    "../",  // Unix directory traversal
    "..\\", // Windows directory traversal
    // 🗂️ SENSITIVE FILE ACCESS: Prevent system file exposure
    "file://",               // File protocol URLs
    "/etc/passwd",           // Unix password file
    "/etc/shadow",           // Unix shadow passwords
    "C:\\Windows\\System32", // Windows system directory
];

#[derive(Debug, Clone)]
pub struct TaskContentValidator {
    safe_content_regex: Regex,
    dangerous_patterns: HashSet<String>,
}

impl TaskContentValidator {
    pub fn new() -> Result<Self, SpiralError> {
        let safe_content_regex = Regex::new(SAFE_TASK_CONTENT_REGEX)
            .map_err(|e| SpiralError::ConfigurationError(format!("Invalid regex pattern: {e}")))?;

        let dangerous_patterns: HashSet<String> = DANGEROUS_PATTERNS
            .iter()
            .map(|s| s.to_lowercase())
            .collect();

        Ok(Self {
            safe_content_regex,
            dangerous_patterns,
        })
    }

    pub fn validate_and_sanitize_task_content(&self, content: &str) -> Result<String, SpiralError> {
        // SECURITY: Length validation
        if content.len() > MAX_TASK_CONTENT_LENGTH {
            return Err(SpiralError::Agent {
                message: format!(
                    "Task content exceeds maximum length of {MAX_TASK_CONTENT_LENGTH} characters"
                ),
            });
        }

        if content.trim().is_empty() {
            return Err(SpiralError::Agent {
                message: "Task content cannot be empty".to_string(),
            });
        }

        // SECURITY: Check for dangerous patterns
        let content_lower = content.to_lowercase();
        for pattern in &self.dangerous_patterns {
            if content_lower.contains(pattern) {
                return Err(SpiralError::Agent {
                    message: "Task content contains potentially dangerous patterns".to_string(),
                });
            }
        }

        // SECURITY: Character whitelist validation
        if !self.safe_content_regex.is_match(content) {
            return Err(SpiralError::Agent {
                message: "Task content contains invalid characters".to_string(),
            });
        }

        // SECURITY: HTML escape the content
        let sanitized = encode_text(content).to_string();

        Ok(sanitized)
    }

    pub fn validate_context_key(&self, key: &str) -> Result<(), SpiralError> {
        if key.is_empty() || key.len() > MAX_CONTEXT_KEY_LENGTH {
            return Err(SpiralError::Agent {
                message: format!(
                    "Context key must be non-empty and under {MAX_CONTEXT_KEY_LENGTH} characters"
                ),
            });
        }

        // SECURITY: Only allow alphanumeric and underscore for keys
        if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(SpiralError::Agent {
                message: "Context keys can only contain alphanumeric characters and underscores"
                    .to_string(),
            });
        }

        Ok(())
    }

    pub fn validate_and_sanitize_context_value(&self, value: &str) -> Result<String, SpiralError> {
        if value.len() > MAX_CONTEXT_VALUE_LENGTH {
            return Err(SpiralError::Agent {
                message: format!(
                    "Context value exceeds maximum length of {MAX_CONTEXT_VALUE_LENGTH} characters"
                ),
            });
        }

        // SECURITY: Check for dangerous patterns in context values
        let value_lower = value.to_lowercase();
        for pattern in &self.dangerous_patterns {
            if value_lower.contains(pattern) {
                return Err(SpiralError::Agent {
                    message: "Context value contains potentially dangerous patterns".to_string(),
                });
            }
        }

        // SECURITY: HTML escape context values
        let sanitized = encode_text(value).to_string();

        Ok(sanitized)
    }
}

impl Default for TaskContentValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create TaskContentValidator")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_content() {
        let validator = TaskContentValidator::new().unwrap();
        let result =
            validator.validate_and_sanitize_task_content("Create a hello world function in Rust");
        assert!(result.is_ok());
    }

    #[test]
    fn test_script_injection_blocked() {
        let validator = TaskContentValidator::new().unwrap();
        let result = validator
            .validate_and_sanitize_task_content("Create a function <script>alert('xss')</script>");
        assert!(result.is_err());
    }

    #[test]
    fn test_command_injection_blocked() {
        let validator = TaskContentValidator::new().unwrap();
        let result = validator.validate_and_sanitize_task_content("Create file && rm -rf /");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_content_blocked() {
        let validator = TaskContentValidator::new().unwrap();
        let result = validator.validate_and_sanitize_task_content("");
        assert!(result.is_err());
    }

    #[test]
    fn test_content_too_long_blocked() {
        let validator = TaskContentValidator::new().unwrap();
        let long_content = "x".repeat(MAX_TASK_CONTENT_LENGTH + 1);
        let result = validator.validate_and_sanitize_task_content(&long_content);
        assert!(result.is_err());
    }

    /// 🛡️ XSS PREVENTION TEST: HTML entity encoding and script tag removal
    /// CRITICAL: Prevents cross-site scripting attacks via user input
    #[test]
    fn test_xss_prevention_comprehensive() {
        let validator = TaskContentValidator::new().unwrap();

        // 🚨 MALICIOUS INPUTS: Common XSS attack vectors that should be blocked
        let xss_payloads = vec![
            "<script>alert('xss')</script>",
            "javascript:alert('xss')",
            "vbscript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
        ];

        for payload in xss_payloads {
            let result = validator.validate_and_sanitize_task_content(payload);
            // Should be rejected due to dangerous patterns
            assert!(result.is_err(), "Should reject XSS payload: {payload}");
        }
    }

    /// 📏 CONTENT LENGTH VALIDATION: Prevent DoS via oversized inputs
    #[test]
    fn test_content_length_limits_comprehensive() {
        let validator = TaskContentValidator::new().unwrap();

        // ✅ VALID LENGTH: Should accept reasonable content
        let valid_content = "Create a simple Rust function that adds two numbers";
        let result = validator.validate_and_sanitize_task_content(valid_content);
        assert!(result.is_ok(), "Should accept reasonable length content");

        // 🎯 BOUNDARY TESTING: Test exact limits
        let boundary_content = "x".repeat(MAX_TASK_CONTENT_LENGTH);
        let result = validator.validate_and_sanitize_task_content(&boundary_content);
        assert!(result.is_ok(), "Should accept content at exact limit");

        let over_boundary = "x".repeat(MAX_TASK_CONTENT_LENGTH + 1);
        let result = validator.validate_and_sanitize_task_content(&over_boundary);
        assert!(result.is_err(), "Should reject content over limit");
    }

    /// 🔧 CONTEXT KEY VALIDATION: Prevent injection via context keys
    #[test]
    fn test_context_key_validation_comprehensive() {
        let validator = TaskContentValidator::new().unwrap();

        // ✅ VALID KEYS: Should accept standard alphanumeric keys
        let valid_keys = vec![
            "project_type",
            "file_path",
            "coding_standards",
            "language",
            "framework",
            "test_type",
            "priority_hint",
        ];

        for key in valid_keys {
            let result = validator.validate_context_key(key);
            assert!(result.is_ok(), "Should accept valid key: {key}");
        }

        // 🚨 INVALID KEYS: Should reject potentially dangerous keys
        let invalid_keys = vec![
            "", // Empty key
            "key with spaces",
            "key-with-dashes",
            "key.with.dots",
            "key/with/slashes",
            "key<with>brackets",
        ];

        for key in invalid_keys {
            let result = validator.validate_context_key(key);
            assert!(result.is_err(), "Should reject invalid key: {key}");
        }
    }

    /// 🧹 CONTEXT VALUE SANITIZATION: Clean potentially dangerous values
    #[test]
    fn test_context_value_sanitization_comprehensive() {
        let validator = TaskContentValidator::new().unwrap();

        // ✅ CLEAN VALUES: Should pass through safely
        let clean_values = vec![
            "rust",
            "web application",
            "REST API",
            "unit tests",
            "SOLID principles",
            "high priority",
        ];

        for value in clean_values {
            let result = validator.validate_and_sanitize_context_value(value);
            assert!(result.is_ok(), "Should accept clean value: {value}");
        }

        // 🚨 DANGEROUS VALUES: Should be rejected due to dangerous patterns
        let dangerous_values = vec![
            "<script>alert('xss')</script>",
            "'; DROP TABLE users; --",
            "javascript:alert(1)",
            "file://etc/passwd",
        ];

        for value in dangerous_values {
            let result = validator.validate_and_sanitize_context_value(value);
            assert!(result.is_err(), "Should reject dangerous value: {value}");
        }
    }

    /// 📊 VALIDATION PERFORMANCE: Ensure validation doesn't become bottleneck
    #[test]
    fn test_validation_performance() {
        let validator = TaskContentValidator::new().unwrap();

        let test_content = "Create a comprehensive web application with authentication".repeat(50);

        let start = std::time::Instant::now();

        // Run validation multiple times
        for _ in 0..100 {
            let _ = validator.validate_and_sanitize_task_content(&test_content);
        }

        let duration = start.elapsed();

        // ⚡ PERFORMANCE REQUIREMENT: Should complete quickly
        assert!(
            duration.as_millis() < 1000,
            "Validation should be fast: took {}ms for 100 operations",
            duration.as_millis()
        );
    }

    /// 🔄 IDEMPOTENCY TEST: Multiple validations should yield same result
    #[test]
    fn test_validation_idempotency() {
        let validator = TaskContentValidator::new().unwrap();

        let test_inputs = vec![
            "Simple clean text",
            "Text with ampersands & symbols",
            "Mixed content with valid punctuation!",
        ];

        for input in test_inputs {
            let result1 = validator.validate_and_sanitize_task_content(input);
            let result2 = validator.validate_and_sanitize_task_content(input);

            // 🔄 CONSISTENCY CHECK: Same input should yield same result
            match (result1, result2) {
                (Ok(output1), Ok(output2)) => {
                    assert_eq!(
                        output1, output2,
                        "Validation should be idempotent for: {input}"
                    );
                }
                (Err(_), Err(_)) => {
                    // Both rejections is consistent
                }
                _ => {
                    panic!("Validation should be consistent for: {input}");
                }
            }
        }
    }
}
