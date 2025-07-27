use super::*;
use crate::{
    config::{ApiConfig, ClaudeCodeConfig, Config, DiscordConfig},
    models::{AgentType, Priority, Task},
};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_orchestrator_with_auth_config() {
    let config = create_test_config_with_auth();
    let orchestrator = Arc::new(AgentOrchestrator::new(config).await.unwrap());
    
    // Test that we can create tasks
    let task = Task::new(
        AgentType::SoftwareDeveloper,
        "Test task for auth verification".to_string(),
        Priority::Medium,
    );
    
    let task_id = orchestrator.submit_task(task).await.unwrap();
    assert!(!task_id.is_empty());
    
    // Test task status retrieval
    let status = orchestrator.get_task_status(&task_id).await;
    assert!(status.is_some());
    
    let task_status = status.unwrap();
    assert_eq!(task_status.id, task_id);
}

#[tokio::test]
async fn test_system_uptime_tracking() {
    let config = create_test_config_with_auth();
    let orchestrator = Arc::new(AgentOrchestrator::new(config).await.unwrap());
    
    // Get initial uptime
    let uptime1 = orchestrator.get_system_uptime().await;
    assert!(uptime1 >= 0.0);
    
    // Wait a bit
    sleep(Duration::from_millis(10)).await;
    
    // Get uptime again - should be higher
    let uptime2 = orchestrator.get_system_uptime().await;
    assert!(uptime2 > uptime1);
}

#[tokio::test]
async fn test_task_queue_limits() {
    let config = create_test_config_with_auth();
    let orchestrator = Arc::new(AgentOrchestrator::new(config).await.unwrap());
    
    // Test queue length starts at 0
    let initial_length = orchestrator.get_queue_length().await;
    assert_eq!(initial_length, 0);
    
    // Add a task
    let task = Task::new(
        AgentType::SoftwareDeveloper,
        "Test task".to_string(),
        Priority::Low,
    );
    
    orchestrator.submit_task(task).await.unwrap();
    
    // Queue length should increase
    let new_length = orchestrator.get_queue_length().await;
    assert_eq!(new_length, 1);
}

#[tokio::test]
async fn test_auth_config_loading() {
    let config_with_auth = create_test_config_with_auth();
    // Authentication is always enabled
    assert!(config_with_auth.api.api_key.is_some());
    
    let config_without_auth = create_test_config_without_auth();
    // Authentication is always enabled
    assert!(config_without_auth.api.api_key.is_none());
}

#[tokio::test]
async fn test_agent_status_tracking() {
    let config = create_test_config_with_auth();
    let orchestrator = Arc::new(AgentOrchestrator::new(config).await.unwrap());
    
    // Test getting status for existing agent
    let status = orchestrator.get_agent_status(&AgentType::SoftwareDeveloper).await;
    assert!(status.is_some());
    
    let agent_status = status.unwrap();
    assert_eq!(agent_status.agent_type, AgentType::SoftwareDeveloper);
    assert!(!agent_status.is_busy);
    assert_eq!(agent_status.tasks_completed, 0);
    
    // Test getting all statuses
    let all_statuses = orchestrator.get_all_agent_statuses().await;
    assert!(!all_statuses.is_empty());
    assert!(all_statuses.contains_key(&AgentType::SoftwareDeveloper));
}

#[tokio::test]
async fn test_task_submission_with_invalid_agent() {
    let config = create_test_config_with_auth();
    let orchestrator = Arc::new(AgentOrchestrator::new(config).await.unwrap());
    
    // Try to submit task for unimplemented agent type
    let task = Task::new(
        AgentType::ProjectManager, // Not implemented yet
        "This should fail".to_string(),
        Priority::Medium,
    );
    
    let result = orchestrator.submit_task(task).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_validation() {
    // Test that config loads properly with all required fields
    let config = create_test_config_with_auth();
    
    // Validate Claude Code config
    assert!(!config.claude_code.api_key.is_empty());
    assert!(!config.claude_code.base_url.is_empty());
    assert!(!config.claude_code.model.is_empty());
    assert!(config.claude_code.max_tokens > 0);
    assert!(config.claude_code.language_detection_tokens > 0);
    assert!(config.claude_code.task_analysis_tokens > 0);
    
    // Validate API config
    assert!(!config.api.host.is_empty());
    assert!(config.api.port > 0);
    assert!(config.api.api_key.is_some());
    // Authentication is always enabled
    
    // Validate Discord config
    assert!(!config.discord.token.is_empty());
    assert!(!config.discord.command_prefix.is_empty());
    assert!(!config.discord.agent_mention_pattern.is_empty());
}

fn create_test_config_with_auth() -> Config {
    Config {
        claude_code: ClaudeCodeConfig {
            api_key: "test-claude-key".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            language_detection_tokens: 100,
            language_detection_temperature: 0.1,
            task_analysis_tokens: 1000,
            task_analysis_temperature: 0.3,
        },
        discord: DiscordConfig {
            token: "test-discord-token".to_string(),
            command_prefix: "!spiral".to_string(),
            agent_mention_pattern: r"@Spiral(\w+)".to_string(),
        },
        api: ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            api_key: Some("test-auth-key-12345".to_string()),
            allowed_origins: vec!["http://localhost:3000".to_string()],
        },
    }
}

fn create_test_config_without_auth() -> Config {
    Config {
        claude_code: ClaudeCodeConfig {
            api_key: "test-claude-key".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            language_detection_tokens: 100,
            language_detection_temperature: 0.1,
            task_analysis_tokens: 1000,
            task_analysis_temperature: 0.3,
        },
        discord: DiscordConfig {
            token: "test-discord-token".to_string(),
            command_prefix: "!spiral".to_string(),
            agent_mention_pattern: r"@Spiral(\w+)".to_string(),
        },
        api: ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            api_key: None,
            allowed_origins: vec!["http://localhost:3000".to_string()],
        },
    }
}