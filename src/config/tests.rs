use super::*;
use crate::SpiralError;
use std::env;
use serial_test::serial;

// These tests modify global environment variables and use #[serial] to ensure
// they run sequentially, preventing interference between parallel tests.

/// Clean up test environment variables
fn cleanup_test_env() {
    env::remove_var("API_KEY");
    env::remove_var("CLAUDE_API_KEY");
    env::remove_var("DISCORD_TOKEN");
    env::remove_var("API_HOST");
    env::remove_var("API_PORT");
    env::remove_var("CLAUDE_BASE_URL");
    env::remove_var("CLAUDE_MODEL");
    env::remove_var("ALLOWED_ORIGINS");
}

/// Helper to create a test config with environment variables
fn create_test_config_with_env() -> Result<Config, SpiralError> {
    Config::load()
}

#[test]
#[serial]
fn test_config_validation_missing_api_key() {
    // Clean up environment first
    cleanup_test_env();
    
    // Set up environment without API key
    env::remove_var("API_KEY");
    env::set_var("CLAUDE_API_KEY", "sk-test-key-1234567890123456789012345678901234567890");
    env::set_var("DISCORD_TOKEN", "test-discord-token-12345678901234567890123456789012345678901234567890");
    
    // Create config - should fail because API key is required
    let result = create_test_config_with_env();
    
    assert!(result.is_err());
    match result.unwrap_err() {
        SpiralError::ConfigurationError(msg) => {
            assert_eq!(msg, "API key is required for security");
        }
        _ => panic!("Expected ConfigurationError"),
    }
    
    // Clean up
    cleanup_test_env();
}

#[test]
#[serial]
fn test_config_validation_blank_api_key() {
    // Clean up environment first
    cleanup_test_env();
    
    // Set up environment with blank API key
    env::set_var("API_KEY", "");
    env::set_var("CLAUDE_API_KEY", "sk-test-key-1234567890123456789012345678901234567890");
    env::set_var("DISCORD_TOKEN", "test-discord-token-12345678901234567890123456789012345678901234567890");
    
    // Create config - should fail
    let result = create_test_config_with_env();
    
    assert!(result.is_err());
    match result.unwrap_err() {
        SpiralError::ConfigurationError(msg) => {
            assert_eq!(msg, "API key is required and cannot be blank");
        }
        _ => panic!("Expected ConfigurationError"),
    }
    
    // Clean up
    cleanup_test_env();
}

#[test]
#[serial]
fn test_config_validation_short_api_key() {
    // Clean up environment first
    cleanup_test_env();
    
    // Set up environment with short API key
    env::set_var("API_KEY", "too-short");
    env::set_var("CLAUDE_API_KEY", "sk-test-key-1234567890123456789012345678901234567890");
    env::set_var("DISCORD_TOKEN", "test-discord-token-12345678901234567890123456789012345678901234567890");
    
    // Create config - should fail
    let result = create_test_config_with_env();
    
    assert!(result.is_err());
    match result.unwrap_err() {
        SpiralError::ConfigurationError(msg) => {
            assert_eq!(msg, "API key must be at least 32 characters for security");
        }
        _ => panic!("Expected ConfigurationError"),
    }
    
    // Clean up
    cleanup_test_env();
}

#[test]
#[serial]
fn test_config_validation_with_secure_api_key() {
    // Clean up environment first
    cleanup_test_env();
    
    // Set up environment with secure API key
    env::set_var("API_KEY", "secure-api-key-1234567890123456789012345678901234567890");
    env::set_var("CLAUDE_API_KEY", "sk-test-key-1234567890123456789012345678901234567890");
    env::set_var("DISCORD_TOKEN", "test-discord-token-12345678901234567890123456789012345678901234567890");
    
    // Create config - should succeed
    let result = create_test_config_with_env();
    
    assert!(result.is_ok());
    let config = result.unwrap();
    assert!(config.api.enable_auth); // Should always be true
    assert_eq!(config.api.api_key, Some("secure-api-key-1234567890123456789012345678901234567890".to_string()));
    
    // Clean up
    cleanup_test_env();
}

#[test]
#[serial]
fn test_config_validation_missing_claude_key() {
    // Clean up environment first
    cleanup_test_env();
    
    // Set up environment without Claude API key
    env::set_var("API_KEY", "secure-api-key-1234567890123456789012345678901234567890");
    env::remove_var("CLAUDE_API_KEY");
    env::set_var("DISCORD_TOKEN", "test-discord-token-12345678901234567890123456789012345678901234567890");
    
    // Create config - should fail
    let result = create_test_config_with_env();
    
    assert!(result.is_err());
    match result.unwrap_err() {
        SpiralError::ConfigurationError(msg) => {
            assert_eq!(msg, "CLAUDE_API_KEY environment variable is required");
        }
        _ => panic!("Expected ConfigurationError"),
    }
    
    // Clean up
    cleanup_test_env();
}

#[test]
#[serial]
fn test_config_validation_invalid_claude_key() {
    // Clean up environment first
    cleanup_test_env();
    
    // Set up environment with invalid Claude API key (doesn't start with sk-)
    env::set_var("API_KEY", "secure-api-key-1234567890123456789012345678901234567890");
    env::set_var("CLAUDE_API_KEY", "invalid-claude-key-format");
    env::set_var("DISCORD_TOKEN", "test-discord-token-12345678901234567890123456789012345678901234567890");
    
    // Create config - should fail
    let result = create_test_config_with_env();
    
    assert!(result.is_err());
    match result.unwrap_err() {
        SpiralError::ConfigurationError(msg) => {
            assert!(msg.contains("CLAUDE_API_KEY appears to be invalid"));
        }
        _ => panic!("Expected ConfigurationError"),
    }
    
    // Clean up
    cleanup_test_env();
}

#[test]
#[serial]
fn test_config_validation_missing_discord_token() {
    // Clean up environment first
    cleanup_test_env();
    
    // Set up environment without Discord token
    env::set_var("API_KEY", "secure-api-key-1234567890123456789012345678901234567890");
    env::set_var("CLAUDE_API_KEY", "sk-test-key-1234567890123456789012345678901234567890");
    env::remove_var("DISCORD_TOKEN");
    
    // Create config - should fail
    let result = create_test_config_with_env();
    
    assert!(result.is_err());
    match result.unwrap_err() {
        SpiralError::ConfigurationError(msg) => {
            assert_eq!(msg, "DISCORD_TOKEN environment variable is required");
        }
        _ => panic!("Expected ConfigurationError"),
    }
    
    // Clean up
    cleanup_test_env();
}

#[test]
#[serial]
fn test_config_validation_invalid_discord_token() {
    // Clean up environment first
    cleanup_test_env();
    
    // Set up environment with too short Discord token
    env::set_var("API_KEY", "secure-api-key-1234567890123456789012345678901234567890");
    env::set_var("CLAUDE_API_KEY", "sk-test-key-1234567890123456789012345678901234567890");
    env::set_var("DISCORD_TOKEN", "too-short");
    
    // Create config - should fail
    let result = create_test_config_with_env();
    
    assert!(result.is_err());
    match result.unwrap_err() {
        SpiralError::ConfigurationError(msg) => {
            assert_eq!(msg, "DISCORD_TOKEN appears to be invalid");
        }
        _ => panic!("Expected ConfigurationError"),
    }
    
    // Clean up
    cleanup_test_env();
}

#[test]
#[serial]
fn test_config_default_values() {
    // Clean up environment first
    cleanup_test_env();
    
    // Set up minimal valid environment
    env::set_var("API_KEY", "secure-api-key-1234567890123456789012345678901234567890");
    env::set_var("CLAUDE_API_KEY", "sk-test-key-1234567890123456789012345678901234567890");
    env::set_var("DISCORD_TOKEN", "test-discord-token-12345678901234567890123456789012345678901234567890");
    
    // Create config
    let result = create_test_config_with_env();
    assert!(result.is_ok());
    
    let config = result.unwrap();
    
    // Check defaults
    assert_eq!(config.api.host, "127.0.0.1"); // Should default to localhost
    assert_eq!(config.api.port, 3000);
    assert!(config.api.enable_auth); // Always true
    assert_eq!(config.claude_code.base_url, "https://api.anthropic.com");
    assert_eq!(config.discord.command_prefix, "!spiral");
    
    // Clean up
    cleanup_test_env();
}

#[test]
#[serial]
fn test_config_allowed_origins() {
    // Clean up environment first
    cleanup_test_env();
    
    // Set up environment with custom allowed origins
    env::set_var("API_KEY", "secure-api-key-1234567890123456789012345678901234567890");
    env::set_var("CLAUDE_API_KEY", "sk-test-key-1234567890123456789012345678901234567890");
    env::set_var("DISCORD_TOKEN", "test-discord-token-12345678901234567890123456789012345678901234567890");
    env::set_var("ALLOWED_ORIGINS", "https://example.com,https://app.example.com");
    
    // Create config
    let result = create_test_config_with_env();
    assert!(result.is_ok());
    
    let config = result.unwrap();
    assert_eq!(config.api.allowed_origins.len(), 2);
    assert!(config.api.allowed_origins.contains(&"https://example.com".to_string()));
    assert!(config.api.allowed_origins.contains(&"https://app.example.com".to_string()));
    
    // Clean up
    cleanup_test_env();
}

#[test]
#[serial]
fn test_config_validation_full_config() {
    cleanup_test_env();
    
    // Set comprehensive test configuration
    env::set_var("API_HOST", "127.0.0.1");
    env::set_var("API_PORT", "8080");
    env::set_var("API_KEY", "test-api-key-1234567890123456789012345678901234567890");
    env::set_var("ALLOWED_ORIGINS", "http://localhost:3000,http://127.0.0.1:3000");
    
    env::set_var("CLAUDE_API_KEY", "sk-test-key-1234567890123456789012345678901234567890");
    env::set_var("CLAUDE_BASE_URL", "https://api.anthropic.com");
    env::set_var("CLAUDE_MODEL", "claude-3-opus-20240229");
    env::set_var("CLAUDE_MAX_TOKENS", "2048");
    env::set_var("CLAUDE_TEMPERATURE", "0.5");
    
    env::set_var("DISCORD_TOKEN", "test-discord-token-12345678901234567890123456789012345678901234567890");
    env::set_var("DISCORD_PREFIX", "!test");
    env::set_var("AGENT_MENTION_PATTERN", r"@(\w+)bot");
    
    // Test should succeed with all valid values
    let result = create_test_config_with_env();
    assert!(result.is_ok());
    
    let config = result.unwrap();
    assert_eq!(config.api.host, "127.0.0.1");
    assert_eq!(config.api.port, 8080);
    assert!(config.api.enable_auth);
    assert_eq!(config.api.api_key, Some("test-api-key-1234567890123456789012345678901234567890".to_string()));
    assert_eq!(config.api.allowed_origins.len(), 2);
    
    assert_eq!(config.claude_code.model, "claude-3-opus-20240229");
    assert_eq!(config.claude_code.max_tokens, 2048);
    assert_eq!(config.claude_code.temperature, 0.5);
    
    assert_eq!(config.discord.command_prefix, "!test");
    assert_eq!(config.discord.agent_mention_pattern, r"@(\w+)bot");
    
    cleanup_test_env();
}