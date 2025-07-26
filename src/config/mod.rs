use crate::{Result, SpiralError};
use dotenv::dotenv;
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub api_key: Option<String>,
    pub enable_auth: bool,
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

        // SECURITY: Validate Discord token is provided
        let discord_token = env::var("DISCORD_TOKEN").map_err(|_| {
            SpiralError::ConfigurationError(
                "DISCORD_TOKEN environment variable is required".to_string(),
            )
        })?;

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

        let discord = DiscordConfig {
            token: discord_token,
            command_prefix: env::var("DISCORD_PREFIX").unwrap_or_else(|_| "!spiral".to_string()),
            agent_mention_pattern: env::var("AGENT_MENTION_PATTERN")
                .unwrap_or_else(|_| r"@(\w+)agent".to_string()),
        };

        // SECURITY: Authentication is always enabled in production
        let api_key = env::var("API_KEY").ok();

        tracing::info!("API authentication is enforced for security");

        // SECURITY: Validate required API authentication
        match &api_key {
            Some(key) if key.trim().is_empty() => {
                tracing::error!("SECURITY ERROR: API_KEY is blank");
                tracing::error!("Set API_KEY to a secure value: openssl rand -hex 32");
                return Err(crate::SpiralError::ConfigurationError(
                    "API key is required and cannot be blank".to_string(),
                ));
            }
            None => {
                tracing::error!("SECURITY ERROR: API_KEY environment variable not set");
                tracing::error!("Generate and set API_KEY: openssl rand -hex 32");
                return Err(crate::SpiralError::ConfigurationError(
                    "API key is required for security".to_string(),
                ));
            }
            Some(key) if key.len() < 32 => {
                tracing::error!("SECURITY ERROR: API key is too short (minimum 32 characters)");
                tracing::error!("Generate a secure key with: openssl rand -hex 32");
                return Err(crate::SpiralError::ConfigurationError(
                    "API key must be at least 32 characters for security".to_string(),
                ));
            }
            Some(_) => {
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
            enable_auth: true, // SECURITY: Always enforce authentication
            allowed_origins,
        };

        Ok(Config {
            claude_code,
            discord,
            api,
        })
    }
}
