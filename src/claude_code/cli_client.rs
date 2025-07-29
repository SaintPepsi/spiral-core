use crate::{
    claude_code::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig},
    config::ClaudeCodeConfig,
    validation::TaskContentValidator,
    Result, SpiralError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// 🤖 CLAUDE CODE CLI CLIENT: Primary interface to Claude Code intelligence engine
/// ARCHITECTURE DECISION: CLI integration over API for enhanced security and tool access
/// Why: CLI provides file system access, tool execution, and session management
/// Alternative: Direct API (rejected: limited tool access, no workspace isolation)
/// Audit: Verify subprocess security in execute_claude_command methods
#[derive(Debug, Clone)]
pub struct ClaudeCodeCliClient {
    config: ClaudeCodeConfig,
    claude_binary: String,
    validator: TaskContentValidator,
    circuit_breaker: Arc<CircuitBreaker>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ClaudeCodeCliResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub subtype: String,
    pub is_error: bool,
    pub duration_ms: u64,
    pub duration_api_ms: u64,
    pub num_turns: u32,
    pub result: String,
    pub session_id: String,
    pub total_cost_usd: f64,
    pub usage: CliUsage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CliUsage {
    pub input_tokens: u32,
    pub cache_creation_input_tokens: Option<u32>,
    pub cache_read_input_tokens: Option<u32>,
    pub output_tokens: u32,
    pub server_tool_use: Option<serde_json::Value>,
    pub service_tier: String,
}

#[derive(Debug, Clone)]
pub struct CodeGenerationRequest {
    pub language: String,
    pub description: String,
    pub context: HashMap<String, String>,
    pub existing_code: Option<String>,
    pub requirements: Vec<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CodeGenerationResult {
    pub code: String,
    pub language: String,
    pub explanation: String,
    pub files_to_create: Vec<FileCreation>,
    pub files_to_modify: Vec<FileModification>,
    pub session_id: Option<String>,
    pub workspace_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCreation {
    pub path: String,
    pub content: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileModification {
    pub path: String,
    pub changes: String,
    pub description: String,
}

impl ClaudeCodeCliClient {
    pub async fn new(config: ClaudeCodeConfig) -> Result<Self> {
        let claude_binary = if let Some(path) = &config.claude_binary_path {
            path.clone()
        } else {
            Self::find_claude_binary().await?
        };

        let validator = TaskContentValidator::new()?;

        // Initialize circuit breaker with default config
        let circuit_breaker = Arc::new(CircuitBreaker::new(CircuitBreakerConfig::default()));

        Ok(Self {
            config,
            claude_binary,
            validator,
            circuit_breaker,
        })
    }

    /// 🔍 BINARY DISCOVERY: Locate Claude Code CLI in system environment
    /// DECISION: Search multiple standard locations for flexibility
    /// Why: Different installation methods place binary in different locations
    /// Alternative: Require explicit path (rejected: poor developer experience)
    /// PERFORMANCE DECISION: Use tokio::process::Command for non-blocking operation
    /// Why: Prevents blocking the async runtime during binary discovery
    /// Alternative: spawn_blocking (considered: more overhead for simple command)
    async fn find_claude_binary() -> Result<String> {
        // Try common locations for Claude Code
        let possible_paths = [
            "claude",                         // PATH search
            "/usr/local/bin/claude",          // Homebrew/standard install
            "/home/vscode/.local/bin/claude", // Dev container install
        ];

        for path in &possible_paths {
            // ✅ PERFORMANCE FIX: Use tokio::process::Command for async operation
            // Why: Prevents blocking the async runtime during binary discovery
            // AUDIT CHECKPOINT: Verify error handling maintains security posture
            match tokio::process::Command::new(path)
                .arg("--help")
                .output()
                .await
            {
                Ok(output) if output.status.success() => {
                    info!("Found Claude Code binary at: {}", path);
                    return Ok(path.to_string());
                }
                Ok(_) => {
                    // Binary exists but --help failed, continue searching
                    debug!("Binary at {} exists but --help failed", path);
                }
                Err(_) => {
                    // Binary not found at this path, continue searching
                    debug!("No binary found at {}", path);
                }
            }
        }

        Err(SpiralError::ConfigurationError(
            "Claude Code CLI not found. Please install with: npm install -g @anthropic-ai/claude-code".to_string()
        ))
    }

    /// Execute Claude Code CLI command with optional session ID for continuity
    async fn execute_claude_command_with_session(
        &self,
        prompt: &str,
        session_id: Option<&str>,
    ) -> Result<ClaudeCodeCliResponse> {
        // Check circuit breaker before making request
        if !self.circuit_breaker.should_allow_request().await {
            warn!("Circuit breaker is open - Claude Code service is unavailable");
            return Err(SpiralError::Agent {
                message: "Claude Code service is temporarily unavailable due to repeated failures"
                    .to_string(),
            });
        }
        // 🏗️ WORKSPACE ISOLATION DECISION: Each session gets isolated filesystem workspace
        // Why: Prevents cross-contamination between tasks, enables safe file operations
        // Alternative: Shared workspace (rejected: security risk, concurrent access issues)
        // AUDIT CHECKPOINT: Verify workspace creation doesn't allow directory traversal
        let (workspace, is_new_session) = self.get_or_create_session_workspace(session_id).await?;

        debug!(
            "Executing Claude Code command in session workspace: {:?} (new: {})",
            workspace, is_new_session
        );

        let mut command = Command::new(&self.claude_binary);
        command
            .args([
                "--print",
                "--output-format",
                "json",
                "--permission-mode",
                &self.config.permission_mode,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&workspace); // Always use session workspace

        // 🔄 SESSION CONTINUITY STRATEGY: Smart session management for context preservation
        // DECISION: Three-tier approach - explicit resume, new session, or continue
        // Why: Balances context preservation with clean slate operations
        // Alternative: Always new sessions (rejected: loses valuable context)
        if let Some(sid) = session_id {
            // Only use --resume if this is NOT a new session
            // A new session means no existing conversation to resume
            if !is_new_session {
                command.args(["--resume", sid]); // Explicit session continuation
                debug!("Resuming existing session: {}", sid);
            } else {
                // New session with ID - start fresh conversation
                debug!("Starting new session with ID: {}", sid);
            }
        } else if is_new_session {
            // For new sessions, don't add continue/resume flags
            // 📝 REASONING: Clean slate prevents unexpected context interference
        } else {
            // For existing workspace without specific session ID, continue most recent
            command.args(["--continue"]);
            // ⚠️ RISK: May inherit unexpected context - monitor for side effects
        }

        // Add allowed tools if any are specified
        if !self.config.allowed_tools.is_empty() {
            let tools_str = self.config.allowed_tools.join(",");
            command.args(["--allowedTools", &tools_str]);
        }

        // Add workspace directory to allowed directories
        let workspace_str = workspace.to_string_lossy();
        command.args(["--add-dir", &workspace_str]);

        let mut child = command.spawn().map_err(|e| SpiralError::Agent {
            message: format!("Failed to spawn Claude Code process: {e}"),
        })?;

        // 📝 STDIN COMMUNICATION: Direct prompt injection to CLI process
        // 🛡️ SECURITY AUDIT CHECKPOINT: Prompt injection vulnerability surface
        // CRITICAL: Ensure prompts are validated upstream before reaching this point
        // Risk: Malicious prompts could execute arbitrary commands via Claude Code
        // Mitigation: Input validation in API layer, prompt sanitization
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(prompt.as_bytes())
                .await
                .map_err(|e| SpiralError::Agent {
                    message: format!("Failed to write to Claude Code stdin: {e}"),
                })?;
            stdin.flush().await.map_err(|e| SpiralError::Agent {
                message: format!("Failed to flush Claude Code stdin: {e}"),
            })?;
        }

        // Wait for completion and read output
        let output = child
            .wait_with_output()
            .await
            .map_err(|e| SpiralError::Agent {
                message: format!("Claude Code process failed: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Claude Code process failed: {stderr}");

            // Record failure in circuit breaker
            self.circuit_breaker.record_failure().await;

            return Err(SpiralError::Agent {
                message: format!("Claude Code execution failed: {stderr}"),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("Claude Code raw output: {}", stdout);

        // Parse JSON response
        let response = match serde_json::from_str::<ClaudeCodeCliResponse>(&stdout) {
            Ok(resp) => resp,
            Err(e) => {
                // Record failure in circuit breaker for parse errors
                self.circuit_breaker.record_failure().await;

                return Err(SpiralError::Agent {
                    message: format!(
                        "Failed to parse Claude Code response: {e} - Output: {stdout}"
                    ),
                });
            }
        };

        // Check for limitation messages and log them for improvement
        if let Err(e) = self.check_for_limitations(&response) {
            // Record failure if we hit limitations
            self.circuit_breaker.record_failure().await;
            return Err(e);
        }

        // Record success in circuit breaker
        self.circuit_breaker.record_success().await;

        // Note: We intentionally don't clean up the workspace immediately
        // to allow for inspection of generated files if needed.
        // Cleanup should be handled by a background process or manual cleanup.
        info!(
            "Claude Code execution completed in workspace: {:?}",
            workspace
        );

        Ok(response)
    }

    /// Get or create a workspace for a specific session
    async fn get_or_create_session_workspace(
        &self,
        session_id: Option<&str>,
    ) -> Result<(PathBuf, bool)> {
        let current_dir = std::env::current_dir().map_err(|e| SpiralError::Agent {
            message: format!("Failed to get current directory: {e}"),
        })?;

        let base_workspace_dir = if let Some(working_dir) = &self.config.working_directory {
            let working_path = PathBuf::from(working_dir);
            if working_path.is_absolute() {
                current_dir.join("claude-workspaces")
            } else {
                current_dir.join(working_path).join("claude-workspaces")
            }
        } else {
            current_dir.join("claude-workspaces")
        };

        // Create base workspace directory if it doesn't exist
        if !base_workspace_dir.exists() {
            fs::create_dir_all(&base_workspace_dir)
                .await
                .map_err(|e| SpiralError::Agent {
                    message: format!("Failed to create workspace base directory: {e}"),
                })?;
        }

        let (workspace_path, is_new) = if let Some(sid) = session_id {
            // Use specific session ID for workspace
            let session_workspace = base_workspace_dir.join(format!("session-{}", sid));
            let is_new_session = !session_workspace.exists();

            if is_new_session {
                fs::create_dir_all(&session_workspace)
                    .await
                    .map_err(|e| SpiralError::Agent {
                        message: format!("Failed to create session workspace: {}", e),
                    })?;
                info!("Created new session workspace for session: {}", sid);
            } else {
                debug!("Reusing existing session workspace for session: {}", sid);
            }

            (session_workspace, is_new_session)
        } else {
            // Create a new unique workspace if no session ID provided
            let workspace_id = Uuid::new_v4().to_string();
            let workspace_path = base_workspace_dir.join(format!("workspace-{workspace_id}"));

            fs::create_dir_all(&workspace_path)
                .await
                .map_err(|e| SpiralError::Agent {
                    message: format!("Failed to create workspace: {}", e),
                })?;

            (workspace_path, true)
        };

        debug!("Session workspace: {:?} (new: {})", workspace_path, is_new);
        Ok((workspace_path, is_new))
    }

    /// Clean up old workspaces based on configuration
    pub async fn cleanup_old_workspaces(&self) -> Result<()> {
        let current_dir = std::env::current_dir().map_err(|e| SpiralError::Agent {
            message: format!("Failed to get current directory: {e}"),
        })?;

        let base_workspace_dir = if let Some(working_dir) = &self.config.working_directory {
            let working_path = PathBuf::from(working_dir);
            if working_path.is_absolute() {
                current_dir.join("claude-workspaces")
            } else {
                current_dir.join(working_path).join("claude-workspaces")
            }
        } else {
            current_dir.join("claude-workspaces")
        };

        if !base_workspace_dir.exists() {
            return Ok(());
        }

        let cleanup_duration =
            std::time::Duration::from_secs(self.config.workspace_cleanup_after_hours * 3600);

        let mut entries =
            fs::read_dir(&base_workspace_dir)
                .await
                .map_err(|e| SpiralError::Agent {
                    message: format!("Failed to read workspace directory: {}", e),
                })?;

        let mut cleaned_count = 0;
        let now = std::time::SystemTime::now();

        while let Some(entry) = entries.next_entry().await.map_err(|e| SpiralError::Agent {
            message: format!("Failed to read workspace entry: {}", e),
        })? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            // Check if workspace is old enough to clean up
            if let Ok(metadata) = entry.metadata().await {
                if let Ok(created) = metadata.created() {
                    if let Ok(age) = now.duration_since(created) {
                        if age > cleanup_duration {
                            info!("Cleaning up old workspace: {:?} (age: {:?})", path, age);
                            if let Err(e) = fs::remove_dir_all(&path).await {
                                warn!("Failed to remove workspace {:?}: {}", path, e);
                            } else {
                                cleaned_count += 1;
                            }
                        }
                    }
                }
            }
        }

        if cleaned_count > 0 {
            info!("Cleaned up {} old Claude Code workspaces", cleaned_count);
        }

        Ok(())
    }

    /// Get workspace usage statistics
    pub async fn get_workspace_stats(&self) -> Result<WorkspaceStats> {
        let current_dir = std::env::current_dir().map_err(|e| SpiralError::Agent {
            message: format!("Failed to get current directory: {e}"),
        })?;

        let base_workspace_dir = if let Some(working_dir) = &self.config.working_directory {
            let working_path = PathBuf::from(working_dir);
            if working_path.is_absolute() {
                current_dir.join("claude-workspaces")
            } else {
                current_dir.join(working_path).join("claude-workspaces")
            }
        } else {
            current_dir.join("claude-workspaces")
        };

        if !base_workspace_dir.exists() {
            return Ok(WorkspaceStats {
                total_workspaces: 0,
                total_size_mb: 0,
                oldest_workspace_age_hours: 0,
            });
        }

        let mut entries =
            fs::read_dir(&base_workspace_dir)
                .await
                .map_err(|e| SpiralError::Agent {
                    message: format!("Failed to read workspace directory: {}", e),
                })?;

        let mut workspace_count = 0;
        let mut total_size = 0u64;
        let mut oldest_age = std::time::Duration::from_secs(0);
        let now = std::time::SystemTime::now();

        while let Some(entry) = entries.next_entry().await.map_err(|e| SpiralError::Agent {
            message: format!("Failed to read workspace entry: {}", e),
        })? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            workspace_count += 1;

            // Calculate size
            if let Ok(size) = self.calculate_directory_size(&path).await {
                total_size += size;
            }

            // Calculate age
            if let Ok(metadata) = entry.metadata().await {
                if let Ok(created) = metadata.created() {
                    if let Ok(age) = now.duration_since(created) {
                        if age > oldest_age {
                            oldest_age = age;
                        }
                    }
                }
            }
        }

        Ok(WorkspaceStats {
            total_workspaces: workspace_count,
            total_size_mb: total_size / (1024 * 1024),
            oldest_workspace_age_hours: oldest_age.as_secs() / 3600,
        })
    }

    /// Calculate directory size recursively
    fn calculate_directory_size<'a>(
        &'a self,
        dir: &'a PathBuf,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64>> + 'a>> {
        Box::pin(async move {
            let mut total_size = 0;
            let mut entries = fs::read_dir(dir).await.map_err(|e| SpiralError::Agent {
                message: format!("Failed to read directory: {}", e),
            })?;

            while let Some(entry) = entries.next_entry().await.map_err(|e| SpiralError::Agent {
                message: format!("Failed to read directory entry: {}", e),
            })? {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = entry.metadata().await {
                        total_size += metadata.len();
                    }
                } else if path.is_dir() {
                    if let Ok(subdir_size) = self.calculate_directory_size(&path).await {
                        total_size += subdir_size;
                    }
                }
            }

            Ok(total_size)
        })
    }

    /// 🔎 LIMITATION DETECTION: Parse responses for known constraint patterns
    /// DECISION: Proactive detection and guidance for common issues
    /// Why: Improves developer experience by suggesting solutions
    /// Alternative: Silent failures (rejected: frustrating debugging experience)
    /// 🛡️ AUDIT CHECKPOINT: Monitor for security-related limitations
    fn check_for_limitations(&self, response: &ClaudeCodeCliResponse) -> Result<()> {
        let result_text = &response.result.to_lowercase();

        // Common limitation patterns that indicate we need to adjust CLI execution
        let limitation_patterns = [
            ("need permission", "Permission issue - consider using --permission-mode bypassPermissions for trusted environments"),
            ("cannot write", "Write permission issue - check --allowedTools includes Write,Edit,MultiEdit"),
            ("cannot read", "Read permission issue - check --allowedTools includes Read,Glob,Grep"),
            ("cannot execute", "Execution permission issue - check --allowedTools includes Bash"),
            ("access denied", "Access denied - verify working directory permissions and --add-dir settings"),
            ("file not found", "File access issue - ensure working directory is correctly set"),
            ("command not found", "Command execution issue - verify tool is in PATH or adjust environment"),
            ("timeout", "Command timeout - consider increasing timeout_seconds in config"),
            ("rate limit", "API rate limiting - implement retry logic or reduce request frequency"),
            ("quota exceeded", "API quota exceeded - monitor usage and implement backoff"),
            ("invalid tool", "Tool not available - check --allowedTools configuration"),
            ("unsafe operation", "Security restriction - review operation safety and permissions"),
        ];

        for (pattern, suggestion) in &limitation_patterns {
            if result_text.contains(pattern) {
                warn!(
                    "Claude Code limitation detected: '{}' - Suggestion: {}",
                    pattern, suggestion
                );

                // Log detailed information for debugging
                debug!("Full response for limitation analysis: {}", response.result);

                // For critical limitations, we might want to return an error
                if matches!(*pattern, "timeout" | "quota exceeded" | "rate limit") {
                    return Err(SpiralError::Agent {
                        message: format!("Claude Code limitation: {} - {}", pattern, suggestion),
                    });
                }
            }
        }

        // Check if Claude Code is reporting it can't complete the task
        let inability_patterns = [
            "i cannot",
            "unable to",
            "can't do",
            "not possible",
            "don't have access",
            "insufficient permissions",
            "not allowed",
        ];

        for pattern in &inability_patterns {
            if result_text.contains(pattern) {
                warn!(
                    "Claude Code reported inability: '{}' found in response. \
                     Consider adjusting permissions, tools, or request scope.",
                    pattern
                );

                // Log the full context for analysis
                info!(
                    "Inability context: {}",
                    response.result.chars().take(500).collect::<String>()
                );
            }
        }

        // Check for successful operations to log positive patterns
        let success_patterns = [
            "created",
            "modified",
            "updated",
            "generated",
            "implemented",
            "fixed",
        ];

        for pattern in &success_patterns {
            if result_text.contains(pattern) {
                debug!("Claude Code successful operation: '{}'", pattern);
                break;
            }
        }

        Ok(())
    }

    /// Execute command with fallback permission modes if initial attempt fails
    async fn execute_with_fallback(&self, prompt: &str) -> Result<ClaudeCodeCliResponse> {
        self.execute_with_fallback_and_session(prompt, None).await
    }

    /// Execute command with fallback permission modes and session support
    async fn execute_with_fallback_and_session(
        &self,
        prompt: &str,
        session_id: Option<&str>,
    ) -> Result<ClaudeCodeCliResponse> {
        let (response, _) = self
            .execute_with_fallback_and_session_info(prompt, session_id)
            .await?;
        Ok(response)
    }

    /// Execute command with fallback and return both response and workspace info
    async fn execute_with_fallback_and_session_info(
        &self,
        prompt: &str,
        session_id: Option<&str>,
    ) -> Result<(ClaudeCodeCliResponse, PathBuf)> {
        // Try with configured permissions first
        // Note: workspace will be created inside execute_claude_command_with_session
        match self
            .execute_claude_command_with_session(prompt, session_id)
            .await
        {
            Ok(response) => {
                // Get workspace path for return value
                let (workspace, _) = self.get_or_create_session_workspace(session_id).await?;
                Ok((response, workspace))
            }
            Err(e) => {
                // If it failed due to permissions, try with more permissive mode
                if e.to_string().contains("permission") || e.to_string().contains("access") {
                    // 🔓 PERMISSION ESCALATION DECISION: Auto-retry with elevated permissions
                    // 🛡️ SECURITY AUDIT CHECKPOINT: Permission bypass activation
                    // Why: User experience - avoid manual retry for common permission issues
                    // Risk: May execute with higher privileges than intended
                    // Mitigation: Log security event, monitor for abuse patterns
                    // 🚨 SECURITY AUDIT LOG: Permission bypass activation
                    // CRITICAL: This event must be monitored for abuse patterns
                    // MITIGATION: Log all relevant context for security analysis
                    warn!(
                        "SECURITY EVENT: Permission bypass activated - reason: initial_execution_failed, prompt_length: {}, session_id: {:?}, timestamp: {}",
                        prompt.len(),
                        session_id.unwrap_or("none"),
                        chrono::Utc::now().to_rfc3339()
                    );
                    warn!("Initial execution failed due to permissions, retrying with bypassPermissions mode");

                    let response = self
                        .execute_claude_command_with_permissions_and_session(
                            prompt,
                            "bypassPermissions",
                            session_id,
                        )
                        .await?;
                    // Get workspace path for return value
                    let (workspace, _) = self.get_or_create_session_workspace(session_id).await?;
                    Ok((response, workspace))
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Execute Claude Code with specific permission mode and session support
    async fn execute_claude_command_with_permissions_and_session(
        &self,
        prompt: &str,
        permission_mode: &str,
        session_id: Option<&str>,
    ) -> Result<ClaudeCodeCliResponse> {
        // Create or get workspace for this session (fallback)
        let (workspace, is_new_session) = self.get_or_create_session_workspace(session_id).await?;

        debug!("Executing Claude Code command with permission mode: {} in session workspace: {:?} (new: {})", permission_mode, workspace, is_new_session);

        let mut command = Command::new(&self.claude_binary);
        command
            .args([
                "--print",
                "--output-format",
                "json",
                "--permission-mode",
                permission_mode,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&workspace); // Always use session workspace

        // 🔄 SESSION CONTINUITY STRATEGY: Smart session management for context preservation
        // DECISION: Three-tier approach - explicit resume, new session, or continue
        // Why: Balances context preservation with clean slate operations
        // Alternative: Always new sessions (rejected: loses valuable context)
        if let Some(sid) = session_id {
            // Only use --resume if this is NOT a new session
            // A new session means no existing conversation to resume
            if !is_new_session {
                command.args(["--resume", sid]); // Explicit session continuation
                debug!("Resuming existing session: {}", sid);
            } else {
                // New session with ID - start fresh conversation
                debug!("Starting new session with ID: {}", sid);
            }
        } else if is_new_session {
            // For new sessions, don't add continue/resume flags
            // 📝 REASONING: Clean slate prevents unexpected context interference
        } else {
            // For existing workspace without specific session ID, continue most recent
            command.args(["--continue"]);
            // ⚠️ RISK: May inherit unexpected context - monitor for side effects
        }

        // Add allowed tools
        if !self.config.allowed_tools.is_empty() {
            let tools_str = self.config.allowed_tools.join(",");
            command.args(["--allowedTools", &tools_str]);
        }

        // Add workspace directory to allowed directories
        let workspace_str = workspace.to_string_lossy();
        command.args(["--add-dir", &workspace_str]);

        let mut child = command.spawn().map_err(|e| SpiralError::Agent {
            message: format!("Failed to spawn Claude Code process: {e}"),
        })?;

        // 📝 STDIN COMMUNICATION: Direct prompt injection to CLI process
        // 🛡️ SECURITY AUDIT CHECKPOINT: Prompt injection vulnerability surface
        // CRITICAL: Ensure prompts are validated upstream before reaching this point
        // Risk: Malicious prompts could execute arbitrary commands via Claude Code
        // Mitigation: Input validation in API layer, prompt sanitization
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(prompt.as_bytes())
                .await
                .map_err(|e| SpiralError::Agent {
                    message: format!("Failed to write to Claude Code stdin: {e}"),
                })?;
            stdin.flush().await.map_err(|e| SpiralError::Agent {
                message: format!("Failed to flush Claude Code stdin: {e}"),
            })?;
        }

        // Wait for completion and read output
        let output = child
            .wait_with_output()
            .await
            .map_err(|e| SpiralError::Agent {
                message: format!("Claude Code process failed: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Claude Code process failed: {stderr}");
            return Err(SpiralError::Agent {
                message: format!("Claude Code execution failed: {stderr}"),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        debug!("Claude Code raw output: {}", stdout);

        // Parse JSON response
        let response = serde_json::from_str::<ClaudeCodeCliResponse>(&stdout).map_err(|e| {
            SpiralError::Agent {
                message: format!("Failed to parse Claude Code response: {e} - Output: {stdout}"),
            }
        })?;

        // Check for limitation messages and log them for improvement
        self.check_for_limitations(&response)?;

        // Log workspace info for fallback execution
        info!(
            "Claude Code fallback execution completed in workspace: {:?}",
            workspace
        );

        Ok(response)
    }

    /// Generate code using Claude Code CLI
    pub async fn generate_code(
        &self,
        request: CodeGenerationRequest,
    ) -> Result<CodeGenerationResult> {
        let session_id = request.session_id.clone();
        self.generate_code_with_session(request, session_id.as_deref())
            .await
    }

    /// Generate code using Claude Code CLI with session management
    pub async fn generate_code_with_session(
        &self,
        request: CodeGenerationRequest,
        session_id: Option<&str>,
    ) -> Result<CodeGenerationResult> {
        info!("Generating code for language: {}", request.language);

        // 🛡️ INPUT VALIDATION: Critical security boundary
        // Validate description content for safety
        let _sanitized_description = self
            .validator
            .validate_and_sanitize_task_content(&request.description)
            .map_err(|e| SpiralError::Validation(format!("Invalid request content: {}", e)))?;

        // Validate context keys and values
        for (key, value) in &request.context {
            self.validator.validate_context_key(key).map_err(|e| {
                SpiralError::Validation(format!("Invalid context key '{}': {}", key, e))
            })?;
            let _sanitized_value = self
                .validator
                .validate_and_sanitize_context_value(value)
                .map_err(|e| {
                    SpiralError::Validation(format!("Invalid context value for '{}': {}", key, e))
                })?;
        }

        // Build comprehensive prompt
        let prompt = self.build_generation_prompt(&request);

        let request_start = std::time::Instant::now();
        let (response, workspace_path) = self
            .execute_with_fallback_and_session_info(&prompt, session_id)
            .await?;
        let duration = request_start.elapsed();

        info!(
            "Claude Code CLI call completed - Duration: {:?}ms, Cost: ${:.4}",
            duration.as_millis(),
            response.total_cost_usd
        );

        self.parse_code_generation_response(response, request.language, session_id, &workspace_path)
    }

    /// Detect programming language using Claude Code CLI
    pub async fn detect_language(&self, code_snippet: &str, context: &str) -> Result<String> {
        debug!("Detecting language for code snippet");

        let prompt = format!(
            "Analyze the following code snippet and context to determine the programming language. \
             Respond with just the language name (e.g., 'rust', 'python', 'javascript').\n\n\
             Context: {context}\n\nCode:\n```\n{code_snippet}\n```"
        );

        let response = self.execute_with_fallback(&prompt).await?;

        // Extract just the language name from the response
        let language = response
            .result
            .lines()
            .next()
            .unwrap_or("unknown")
            .trim()
            .to_lowercase();

        info!("Detected language: {}", language);
        Ok(language)
    }

    /// Analyze task using Claude Code CLI
    pub async fn analyze_task(
        &self,
        task_description: &str,
        context: HashMap<String, String>,
    ) -> Result<TaskAnalysis> {
        info!(
            "Analyzing task: {}",
            task_description.chars().take(100).collect::<String>()
        );

        let context_str = context
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Analyze the following task and provide a structured analysis:\n\n\
             Task: {task_description}\n\n\
             Context:\n{context_str}\n\n\
             Provide analysis including:\n\
             1. Task complexity (Low/Medium/High)\n\
             2. Estimated time (in minutes)\n\
             3. Required skills/technologies\n\
             4. Potential challenges\n\
             5. Suggested approach"
        );

        let response = self.execute_with_fallback(&prompt).await?;

        Ok(TaskAnalysis {
            complexity: self.extract_complexity(&response.result),
            estimated_minutes: self.extract_time_estimate(&response.result),
            required_skills: self.extract_required_skills(&response.result),
            challenges: self.extract_challenges(&response.result),
            approach: self.extract_approach(&response.result),
            raw_analysis: response.result,
        })
    }

    fn build_generation_prompt(&self, request: &CodeGenerationRequest) -> String {
        let mut prompt = format!(
            "You are a Software Developer Agent in the Spiral Core orchestration system. \
             Generate high-quality {} code following these principles:\n\n\
             1. Follow SOLID principles\n\
             2. Apply DRY principle\n\
             3. Use SID naming (Short, Intuitive, Descriptive)\n\
             4. Ensure compile-time safety and error handling\n\
             5. Include comprehensive documentation\n\
             6. Follow language-specific best practices\n\
             7. Implement security best practices\n\n\
             Task: {}\n\n",
            request.language, request.description
        );

        // Add context
        if !request.context.is_empty() {
            prompt.push_str("Context:\n");
            for (key, value) in &request.context {
                prompt.push_str(&format!("- {key}: {value}\n"));
            }
            prompt.push('\n');
        }

        // Add requirements
        if !request.requirements.is_empty() {
            prompt.push_str("Requirements:\n");
            for requirement in &request.requirements {
                prompt.push_str(&format!("- {requirement}\n"));
            }
            prompt.push('\n');
        }

        // Add existing code if provided
        if let Some(existing) = &request.existing_code {
            prompt.push_str(&format!(
                "Existing code to modify:\n```{}\n{}\n```\n\n",
                request.language, existing
            ));
        }

        prompt.push_str(
            "IMPORTANT: After implementing the code, you MUST:\n\
             1. Verify that the code compiles without errors\n\
             2. Run all tests and ensure they pass\n\
             3. Fix any compilation errors or test failures\n\
             4. If creating a package, verify it builds successfully\n\
             5. Ensure type safety and no unused imports\n\n\
             Use the available tools to compile, test, and validate your implementation. \
             Do not consider the task complete until the code compiles cleanly and all tests pass.\n\n\
             Provide the complete implementation with explanations."
        );
        prompt
    }

    fn parse_code_generation_response(
        &self,
        response: ClaudeCodeCliResponse,
        language: String,
        session_id: Option<&str>,
        workspace_path: &PathBuf,
    ) -> Result<CodeGenerationResult> {
        let result_text = &response.result;

        // Extract code blocks from the response
        let code = self.extract_code_block(result_text, &language);

        // ARCHITECTURE DECISION: Claude Code CLI returns unstructured text responses
        // Why: Current CLI doesn't provide structured file operation metadata
        // Alternative: Parse response text for file operations (future enhancement)
        // Current: Return empty vectors until structured output is available
        let files_to_create = Vec::new();
        let files_to_modify = Vec::new();

        Ok(CodeGenerationResult {
            code,
            language,
            explanation: result_text.clone(),
            files_to_create,
            files_to_modify,
            session_id: session_id.map(|s| s.to_string()).or_else(|| {
                // Extract session ID from response if available
                Some(response.session_id.clone())
            }),
            workspace_path: workspace_path.to_string_lossy().to_string(),
        })
    }

    fn extract_code_block(&self, text: &str, language: &str) -> String {
        let patterns = [format!("```{language}\n"), "```\n".to_string()];

        for pattern in &patterns {
            if let Some(start) = text.find(pattern) {
                let code_start = start + pattern.len();
                if let Some(end) = text[code_start..].find("\n```") {
                    return text[code_start..code_start + end].to_string();
                }
            }
        }

        // If no code block found, return the whole response
        text.to_string()
    }

    fn extract_complexity(&self, text: &str) -> String {
        let text_lower = text.to_lowercase();
        if text_lower.contains("high") {
            "High".to_string()
        } else if text_lower.contains("medium") {
            "Medium".to_string()
        } else {
            "Low".to_string()
        }
    }

    fn extract_time_estimate(&self, _text: &str) -> u32 {
        30 // Default to 30 minutes
    }

    fn extract_required_skills(&self, text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();
        let mut skills = Vec::new();

        let languages = ["rust", "python", "javascript", "typescript", "go", "java"];
        let technologies = ["docker", "kubernetes", "aws", "git", "sql", "api", "rest"];

        for skill_set in [&languages[..], &technologies[..]].iter() {
            for skill in *skill_set {
                if text_lower.contains(skill) {
                    skills.push(skill.to_string());
                }
            }
        }

        if skills.is_empty() {
            skills.push("general programming".to_string());
        }

        skills
    }

    fn extract_challenges(&self, text: &str) -> Vec<String> {
        let mut challenges = Vec::new();
        let text_lower = text.to_lowercase();

        let challenge_indicators = [
            ("complex", "Implementation complexity"),
            ("difficult", "Technical difficulty"),
            ("challenge", "Development challenges"),
            ("risk", "Project risks"),
            ("error", "Error handling requirements"),
        ];

        for (keyword, challenge) in &challenge_indicators {
            if text_lower.contains(keyword) {
                challenges.push(challenge.to_string());
            }
        }

        if challenges.is_empty() {
            challenges.push("Standard implementation challenges".to_string());
        }

        challenges
    }

    fn extract_approach(&self, text: &str) -> String {
        let text_lower = text.to_lowercase();

        let approach_keywords = ["approach", "strategy", "method", "plan", "implementation"];

        for keyword in &approach_keywords {
            if let Some(start) = text_lower.find(keyword) {
                let section_start = text[..start].rfind('.').map(|i| i + 1).unwrap_or(0);
                let section_end = text[start..]
                    .find('.')
                    .map(|i| start + i + 1)
                    .unwrap_or(text.len());

                let approach_text = text[section_start..section_end].trim();
                if !approach_text.is_empty() {
                    return approach_text.to_string();
                }
            }
        }

        "Implement using best practices and modular design".to_string()
    }

    /// Get circuit breaker status and metrics
    pub async fn get_circuit_breaker_metrics(
        &self,
    ) -> crate::claude_code::circuit_breaker::CircuitBreakerMetrics {
        self.circuit_breaker.get_metrics().await
    }
}

#[derive(Debug, Clone)]
pub struct TaskAnalysis {
    pub complexity: String,
    pub estimated_minutes: u32,
    pub required_skills: Vec<String>,
    pub challenges: Vec<String>,
    pub approach: String,
    pub raw_analysis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceStats {
    pub total_workspaces: u32,
    pub total_size_mb: u64,
    pub oldest_workspace_age_hours: u64,
}
