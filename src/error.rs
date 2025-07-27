use thiserror::Error;

pub type Result<T> = std::result::Result<T, SpiralError>;

#[derive(Error, Debug)]
pub enum SpiralError {
    #[error("Claude Code API error: {0}")]
    ClaudeCodeApi(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Discord error: {0}")]
    Discord(#[from] Box<serenity::Error>),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Agent error: {message}")]
    Agent { message: String },

    #[error("Task execution error: {task_id} - {message}")]
    TaskExecution { task_id: String, message: String },

    #[error("Language detection failed: {0}")]
    LanguageDetection(String),

    #[error("GitHub integration error: {0}")]
    GitHub(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Timeout error: {message}")]
    Timeout { message: String },

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}
