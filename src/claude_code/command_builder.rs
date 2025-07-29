use std::path::PathBuf;
use tokio::process::Command;

/// 🏗️ CLAUDE CODE COMMAND BUILDER: Fluent interface for CLI command construction
/// ARCHITECTURAL PATTERN: Builder Pattern for complex command construction
/// Why Builder Pattern: Claude Code CLI has many flags and complex option combinations
/// Alternative: Direct command construction (rejected: error-prone, hard to maintain)
/// Benefits: Type safety, validation, reusable configurations, testable
///
/// # Example Usage
/// ```rust
/// use spiral_core::claude_code::ClaudeCommandBuilder;
/// let command = ClaudeCommandBuilder::new("/usr/bin/claude")
///     .with_json_output()
///     .with_permission_mode("bypassPermissions")
///     .with_session_id("abc123")
///     .with_allowed_tools(vec!["Read", "Write", "Edit"])
///     .with_workspace("/tmp/workspace")
///     .with_timeout(300)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct ClaudeCommandBuilder {
    binary_path: String,
    output_format: OutputFormat,
    permission_mode: PermissionMode,
    session_mode: SessionMode,
    allowed_tools: Vec<String>,
    workspace: Option<PathBuf>,
    additional_dirs: Vec<PathBuf>,
    timeout_seconds: Option<u32>,
    environment_vars: Vec<(String, String)>,
}

/// 📊 OUTPUT FORMAT: How Claude Code returns results
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Json,     // Machine-readable JSON format
    Text,     // Human-readable text format
    Markdown, // Rich markdown format
}

/// 🔒 PERMISSION MODE: Security level for Claude Code operations
/// DECISION: Enum over strings for compile-time safety
/// Why: Prevents typos and invalid permission modes
#[derive(Debug, Clone, PartialEq)]
pub enum PermissionMode {
    Standard,          // Default permissions
    BypassPermissions, // Elevated permissions (use with caution)
    Restricted,        // Limited permissions for untrusted input
}

/// 🔄 SESSION MODE: How to handle conversation context
#[derive(Debug, Clone, PartialEq)]
pub enum SessionMode {
    NewSession,     // Start fresh
    Resume(String), // Resume specific session ID
    Continue,       // Continue most recent session
}

impl ClaudeCommandBuilder {
    /// 🚀 BUILDER INITIALIZATION: Start with binary path
    /// DECISION: Require binary path upfront for fail-fast behavior
    /// Why: Can't build valid command without knowing the executable
    pub fn new(binary_path: impl Into<String>) -> Self {
        Self {
            binary_path: binary_path.into(),
            output_format: OutputFormat::Json, // Default to JSON for parsing
            permission_mode: PermissionMode::Standard,
            session_mode: SessionMode::NewSession,
            allowed_tools: Vec::new(),
            workspace: None,
            additional_dirs: Vec::new(),
            timeout_seconds: None,
            environment_vars: Vec::new(),
        }
    }

    /// 📋 OUTPUT FORMAT CONFIGURATION
    pub fn with_json_output(mut self) -> Self {
        self.output_format = OutputFormat::Json;
        self
    }

    pub fn with_text_output(mut self) -> Self {
        self.output_format = OutputFormat::Text;
        self
    }

    pub fn with_markdown_output(mut self) -> Self {
        self.output_format = OutputFormat::Markdown;
        self
    }

    /// 🔐 PERMISSION MODE CONFIGURATION
    /// 🛡️ SECURITY AUDIT CHECKPOINT: Permission elevation point
    pub fn with_permission_mode(mut self, mode: impl Into<PermissionMode>) -> Self {
        self.permission_mode = mode.into();
        self
    }

    pub fn with_standard_permissions(mut self) -> Self {
        self.permission_mode = PermissionMode::Standard;
        self
    }

    pub fn with_bypass_permissions(mut self) -> Self {
        // 🚨 SECURITY WARNING: This bypasses Claude Code safety checks
        // AUDIT CHECKPOINT: Critical security boundary - log when this is used for security monitoring
        // DECISION: Explicit permission bypass method for controlled usage
        // Why: Provides clear audit trail when security boundaries are crossed
        // Risk: May execute with elevated privileges, potential for abuse
        // Mitigation: Comprehensive logging with context for security analysis
        tracing::warn!(
            "SECURITY EVENT: bypassPermissions mode activated in command builder - timestamp: {}, workspace: {:?}",
            chrono::Utc::now().to_rfc3339(),
            self.workspace
        );
        self.permission_mode = PermissionMode::BypassPermissions;
        self
    }

    /// 💬 SESSION MANAGEMENT
    pub fn with_session_mode(mut self, mode: SessionMode) -> Self {
        self.session_mode = mode;
        self
    }

    pub fn with_new_session(mut self) -> Self {
        self.session_mode = SessionMode::NewSession;
        self
    }

    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_mode = SessionMode::Resume(session_id.into());
        self
    }

    pub fn with_continue_session(mut self) -> Self {
        self.session_mode = SessionMode::Continue;
        self
    }

    /// 🔧 TOOL CONFIGURATION
    pub fn with_allowed_tools(mut self, tools: Vec<impl Into<String>>) -> Self {
        self.allowed_tools = tools.into_iter().map(|t| t.into()).collect();
        self
    }

    pub fn add_allowed_tool(mut self, tool: impl Into<String>) -> Self {
        self.allowed_tools.push(tool.into());
        self
    }

    /// 📁 WORKSPACE AND DIRECTORY CONFIGURATION
    pub fn with_workspace(mut self, path: impl Into<PathBuf>) -> Self {
        self.workspace = Some(path.into());
        self
    }

    pub fn add_allowed_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.additional_dirs.push(path.into());
        self
    }

    /// ⏱️ TIMEOUT CONFIGURATION
    pub fn with_timeout(mut self, seconds: u32) -> Self {
        self.timeout_seconds = Some(seconds);
        self
    }

    /// 🌍 ENVIRONMENT CONFIGURATION
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment_vars.push((key.into(), value.into()));
        self
    }

    /// 🏗️ BUILD FINAL COMMAND
    /// DECISION: Return tokio::process::Command for async execution
    /// Why: Integrates seamlessly with async runtime
    /// Alternative: Return Vec<String> (rejected: loses type safety)
    pub fn build(self) -> Command {
        let mut command = Command::new(&self.binary_path);

        // Always add print flag for output
        command.arg("--print");

        // Output format
        command.args([
            "--output-format",
            match self.output_format {
                OutputFormat::Json => "json",
                OutputFormat::Text => "text",
                OutputFormat::Markdown => "markdown",
            },
        ]);

        // Permission mode
        command.args([
            "--permission-mode",
            match self.permission_mode {
                PermissionMode::Standard => "default",
                PermissionMode::BypassPermissions => "bypassPermissions",
                PermissionMode::Restricted => "restricted",
            },
        ]);

        // Session handling
        match self.session_mode {
            SessionMode::NewSession => {
                // No session flags needed
            }
            SessionMode::Resume(ref session_id) => {
                command.args(["--resume", session_id]);
            }
            SessionMode::Continue => {
                command.arg("--continue");
            }
        }

        // Allowed tools
        if !self.allowed_tools.is_empty() {
            command.args(["--allowedTools", &self.allowed_tools.join(",")]);
        }

        // Workspace directory
        if let Some(ref workspace) = self.workspace {
            command.current_dir(workspace);
            // Also add as allowed directory
            command.args(["--add-dir", &workspace.to_string_lossy()]);
        }

        // Additional allowed directories
        for dir in &self.additional_dirs {
            command.args(["--add-dir", &dir.to_string_lossy()]);
        }

        // Timeout
        if let Some(timeout) = self.timeout_seconds {
            command.args(["--timeout", &timeout.to_string()]);
        }

        // Environment variables
        for (key, value) in &self.environment_vars {
            command.env(key, value);
        }

        // Standard I/O configuration
        command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        command
    }

    /// 🧪 VALIDATION: Check if configuration is valid
    /// DECISION: Separate validation from building for better error handling
    /// Why: Allows early detection of configuration issues
    pub fn validate(&self) -> Result<(), String> {
        // Check binary path is not empty
        if self.binary_path.is_empty() {
            return Err("Binary path cannot be empty".to_string());
        }

        // Check for conflicting session modes
        if matches!(self.session_mode, SessionMode::Resume(ref id) if id.is_empty()) {
            return Err("Session ID cannot be empty when resuming".to_string());
        }

        // Validate tool names (basic check)
        for tool in &self.allowed_tools {
            if tool.is_empty() {
                return Err("Tool name cannot be empty".to_string());
            }
        }

        Ok(())
    }
}

// Implement conversions for convenience
impl From<&str> for PermissionMode {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "standard" | "default" => PermissionMode::Standard,
            "bypasspermissions" | "bypass" => PermissionMode::BypassPermissions,
            "restricted" => PermissionMode::Restricted,
            _ => PermissionMode::Standard, // Default to standard
        }
    }
}

impl From<String> for PermissionMode {
    fn from(s: String) -> Self {
        PermissionMode::from(s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_command_building() {
        let builder = ClaudeCommandBuilder::new("/usr/bin/claude")
            .with_json_output()
            .with_standard_permissions();

        // Clone to check state before consuming with build()
        let output_format = builder.output_format.clone();
        let permission_mode = builder.permission_mode.clone();
        let _command = builder.build();

        // Can't easily test Command directly, but we can verify builder state
        assert_eq!(output_format, OutputFormat::Json);
        assert_eq!(permission_mode, PermissionMode::Standard);
    }

    #[test]
    fn test_session_configuration() {
        let builder = ClaudeCommandBuilder::new("/usr/bin/claude").with_session_id("test-123");

        assert_eq!(
            builder.session_mode,
            SessionMode::Resume("test-123".to_string())
        );
    }

    #[test]
    fn test_tool_configuration() {
        let builder = ClaudeCommandBuilder::new("/usr/bin/claude")
            .with_allowed_tools(vec!["Read", "Write"])
            .add_allowed_tool("Edit");

        assert_eq!(builder.allowed_tools, vec!["Read", "Write", "Edit"]);
    }

    #[test]
    fn test_validation() {
        // Valid configuration
        let valid = ClaudeCommandBuilder::new("/usr/bin/claude");
        assert!(valid.validate().is_ok());

        // Invalid: empty binary path
        let invalid = ClaudeCommandBuilder::new("");
        assert!(invalid.validate().is_err());

        // Invalid: empty session ID
        let invalid = ClaudeCommandBuilder::new("/usr/bin/claude").with_session_id("");
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_permission_mode_conversion() {
        assert_eq!(PermissionMode::from("standard"), PermissionMode::Standard);
        assert_eq!(
            PermissionMode::from("bypass"),
            PermissionMode::BypassPermissions
        );
        assert_eq!(
            PermissionMode::from("RESTRICTED"),
            PermissionMode::Restricted
        );
        assert_eq!(PermissionMode::from("unknown"), PermissionMode::Standard);
    }
}
