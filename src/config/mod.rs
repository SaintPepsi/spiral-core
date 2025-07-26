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
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub language_detection_tokens: u32,
    pub language_detection_temperature: f32,
    pub task_analysis_tokens: u32,
    pub task_analysis_temperature: f32,
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

        // SECURITY: Validate Claude API key is provided
        let claude_api_key = env::var("CLAUDE_API_KEY").map_err(|_| {
            SpiralError::ConfigurationError(
                "CLAUDE_API_KEY environment variable is required".to_string(),
            )
        })?;

        if claude_api_key.trim().is_empty() {
            return Err(SpiralError::ConfigurationError(
                "CLAUDE_API_KEY cannot be empty".to_string(),
            ));
        }

        // SECURITY: Basic Claude API key format validation
        if !claude_api_key.starts_with("sk-") || claude_api_key.len() < 40 {
            return Err(SpiralError::ConfigurationError(
                "CLAUDE_API_KEY appears to be invalid (should start with 'sk-' and be at least 40 characters)".to_string()
            ));
        }

        let claude_code = ClaudeCodeConfig {
            api_key: claude_api_key,
            base_url: env::var("CLAUDE_BASE_URL")
                .unwrap_or_else(|_| "https://api.anthropic.com".to_string()),
            model: env::var("CLAUDE_MODEL")
                .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string()),
            max_tokens: env::var("CLAUDE_MAX_TOKENS")
                .unwrap_or_else(|_| "4096".to_string())
                .parse()
                .unwrap_or(4096),
            temperature: env::var("CLAUDE_TEMPERATURE")
                .unwrap_or_else(|_| "0.7".to_string())
                .parse()
                .unwrap_or(0.7),
            language_detection_tokens: env::var("CLAUDE_LANGUAGE_DETECTION_TOKENS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            language_detection_temperature: env::var("CLAUDE_LANGUAGE_DETECTION_TEMPERATURE")
                .unwrap_or_else(|_| "0.1".to_string())
                .parse()
                .unwrap_or(0.1),
            task_analysis_tokens: env::var("CLAUDE_TASK_ANALYSIS_TOKENS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            task_analysis_temperature: env::var("CLAUDE_TASK_ANALYSIS_TEMPERATURE")
                .unwrap_or_else(|_| "0.3".to_string())
                .parse()
                .unwrap_or(0.3),
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
