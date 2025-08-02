use crate::{Result, SpiralError};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub claude_code: ClaudeCodeConfig,
    pub discord: DiscordConfig,
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    pub claude_binary_path: Option<String>,
    pub working_directory: Option<String>,
    pub timeout_seconds: u64,
    pub permission_mode: String,
    pub allowed_tools: Vec<String>,
    pub workspace_cleanup_after_hours: u64,
    pub max_workspace_size_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    pub command_prefix: String,
    pub agent_mention_pattern: String,
    pub authorized_users: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub api_key: Option<String>,
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Load environment variables from .env file
        match dotenv() {
            Ok(path) => tracing::info!("Loaded .env file from: {:?}", path),
            Err(e) => tracing::warn!("Could not load .env file: {}", e),
        }

        let claude_code = ClaudeCodeConfig {
            claude_binary_path: env::var("CLAUDE_BINARY_PATH").ok(),
            working_directory: env::var("CLAUDE_WORKING_DIR").ok(),
            timeout_seconds: env::var("CLAUDE_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
            permission_mode: env::var("CLAUDE_PERMISSION_MODE").unwrap_or_else(|_| {
                // Use more permissive mode in development environments
                if cfg!(debug_assertions) {
                    "bypassPermissions".to_string()
                } else {
                    "acceptEdits".to_string()
                }
            }),
            allowed_tools: env::var("CLAUDE_ALLOWED_TOOLS")
                .unwrap_or_else(|_| {
                    "Edit,Write,Read,Bash,MultiEdit,Glob,Grep,TodoWrite,NotebookEdit,WebFetch"
                        .to_string()
                })
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            workspace_cleanup_after_hours: env::var("CLAUDE_WORKSPACE_CLEANUP_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            max_workspace_size_mb: env::var("CLAUDE_MAX_WORKSPACE_SIZE_MB")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
        };

        // OPTIONAL: Discord integration configuration
        let discord_token = env::var("DISCORD_TOKEN").unwrap_or_else(|_| "".to_string());

        // Only validate Discord token if provided
        if !discord_token.is_empty() {
            if discord_token.trim().is_empty() {
                return Err(SpiralError::ConfigurationError(
                    "DISCORD_TOKEN cannot be empty".to_string(),
                ));
            }

            // SECURITY: Validate Discord token format (basic check)
            if discord_token.len() < 50
                || !discord_token
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-')
            {
                return Err(SpiralError::ConfigurationError(
                    "DISCORD_TOKEN appears to be invalid".to_string(),
                ));
            }
        }

        // Load authorized users from environment
        let authorized_users = env::var("DISCORD_AUTHORIZED_USERS")
            .unwrap_or_else(|_| "".to_string())
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    trimmed.parse::<u64>().ok()
                }
            })
            .collect();

        let discord = DiscordConfig {
            token: discord_token,
            command_prefix: env::var("DISCORD_PREFIX").unwrap_or_else(|_| "!spiral".to_string()),
            agent_mention_pattern: env::var("AGENT_MENTION_PATTERN")
                .unwrap_or_else(|_| r"@Spiral(\w+)".to_string()),
            authorized_users,
        };

        // 🔐 SECURE API KEY LOADING: Environment variable or generated secure key
        // DECISION: Prioritize env var, fall back to secure file-based key
        let api_key = match env::var("API_KEY").ok() {
            Some(key) if !key.trim().is_empty() => {
                tracing::info!("Using API key from environment variable");
                Some(key)
            }
            _ => {
                tracing::info!(
                    "No API_KEY environment variable set, checking for generated key file"
                );
                // Try to load from secure file, don't generate here (will be done in startup validation)
                match crate::security::load_api_key_from_file() {
                    Ok(Some(key)) => {
                        tracing::info!("Using existing API key from secure file");
                        Some(key)
                    }
                    Ok(None) => {
                        tracing::info!("No API key file found, will generate during startup");
                        None
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to load API key from file: {}, will generate new one",
                            e
                        );
                        None
                    }
                }
            }
        };

        tracing::info!("API authentication is enforced for security");

        // SECURITY: Validate API key if provided via environment
        if let Some(key) = &api_key {
            if key.len() < 32 {
                tracing::error!("SECURITY ERROR: API key is too short (minimum 32 characters)");
                tracing::error!("Generate a secure key with: openssl rand -hex 32");
                return Err(crate::SpiralError::ConfigurationError(
                    "API key must be at least 32 characters for security".to_string(),
                ));
            } else {
                tracing::info!("API authentication configured with secure key");
            }
        }

        // SECURITY: Configure allowed CORS origins
        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://127.0.0.1:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let api = ApiConfig {
            host: env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()), // SECURITY: Default to localhost only
            port: env::var("API_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            api_key,
            allowed_origins,
        };

        Ok(Config {
            claude_code,
            discord,
            api,
        })
    }

    /// Create a test configuration with sensible defaults
    #[cfg(test)]
    pub fn test_config() -> Self {
        Self {
            claude_code: ClaudeCodeConfig {
                claude_binary_path: Some("mock-claude".to_string()),
                working_directory: Some("/tmp/test".to_string()),
                timeout_seconds: 30,
                permission_mode: "ask".to_string(),
                allowed_tools: vec!["edit".to_string(), "read".to_string()],
                workspace_cleanup_after_hours: 1,
                max_workspace_size_mb: 100,
            },
            discord: DiscordConfig {
                token: "mock-discord-token-for-testing-only".to_string(),
                command_prefix: "!test".to_string(),
                agent_mention_pattern: r"@Test(\w+)".to_string(),
                authorized_users: vec![123456789],
            },
            api: ApiConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                api_key: Some("test-api-key-32-characters-long-for-security".to_string()),
                allowed_origins: vec!["http://localhost:3000".to_string()],
            },
        }
    }
}
