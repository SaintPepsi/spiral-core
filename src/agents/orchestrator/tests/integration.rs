use super::super::*;
use crate::{
    config::{ApiConfig, ClaudeCodeConfig, Config, DiscordConfig},
    models::{AgentType, Priority, Task},
};
use std::sync::Arc;

#[tokio::test]
async fn test_task_submission_and_queue_management() {
    let config = create_test_config();
    let orchestrator = Arc::new(AgentOrchestrator::new(config).await.unwrap());
    
    // Verify initial queue is empty
    let initial_length = orchestrator.get_queue_length().await;
    assert_eq!(initial_length, 0);
    
    // Submit tasks with different priorities
    let low_priority_task = Task::new(
        AgentType::SoftwareDeveloper,
        "Low priority task".to_string(),
        Priority::Low,
    );
    
    let high_priority_task = Task::new(
        AgentType::SoftwareDeveloper,
        "High priority task".to_string(),
        Priority::High,
    );
    
    // Submit tasks and verify they're queued
    let result1 = orchestrator.submit_task(low_priority_task).await;
    assert!(result1.is_ok());
    
    let result2 = orchestrator.submit_task(high_priority_task).await;
    assert!(result2.is_ok());
    
    // Verify queue length increased
    let final_length = orchestrator.get_queue_length().await;
    assert_eq!(final_length, 2);
}

#[tokio::test]
async fn test_agent_status_tracking() {
    let config = create_test_config();
    let orchestrator = Arc::new(AgentOrchestrator::new(config).await.unwrap());
    
    // Test getting status for existing agent
    let status = orchestrator.get_agent_status(&AgentType::SoftwareDeveloper).await;
    assert!(status.is_some());
    
    let status = status.unwrap();
    assert_eq!(status.agent_type, AgentType::SoftwareDeveloper);
    assert!(!status.is_busy);
    assert_eq!(status.tasks_completed, 0);
    assert_eq!(status.tasks_failed, 0);
    
    // Test getting status for non-existent agent
    let missing_status = orchestrator.get_agent_status(&AgentType::ProjectManager).await;
    assert!(missing_status.is_none());
}

#[tokio::test]
async fn test_invalid_agent_type_submission() {
    let config = create_test_config();
    let orchestrator = Arc::new(AgentOrchestrator::new(config).await.unwrap());
    
    // Try to submit task for unimplemented agent type
    let task = Task::new(
        AgentType::ProjectManager, // Not implemented
        "This should fail".to_string(),
        Priority::Medium,
    );
    
    let result = orchestrator.submit_task(task).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_multiple_agent_status_retrieval() {
    let config = create_test_config();
    let orchestrator = Arc::new(AgentOrchestrator::new(config).await.unwrap());
    
    let all_statuses = orchestrator.get_all_agent_statuses().await;
    
    // Should have at least the SoftwareDeveloper agent
    assert!(!all_statuses.is_empty());
    assert!(all_statuses.contains_key(&AgentType::SoftwareDeveloper));
    
    // All agents should start as not busy
    for (_, status) in all_statuses {
        assert!(!status.is_busy);
        assert_eq!(status.tasks_completed, 0);
        assert_eq!(status.tasks_failed, 0);
    }
}

fn create_test_config() -> Config {
    Config {
        claude_code: ClaudeCodeConfig {
            api_key: "test-key".to_string(),
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
            token: "test-token".to_string(),
            command_prefix: "!spiral".to_string(),
            agent_mention_pattern: r"@(\w+)agent".to_string(),
        },
        api: ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 3000,
            api_key: Some("test-api-key".to_string()),
            enable_auth: false,
            allowed_origins: vec!["http://localhost:3000".to_string()],
        },
    }
}