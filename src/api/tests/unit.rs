use super::*;
use crate::{
    config::{ClaudeCodeConfig, DiscordConfig, Config},
    agents::AgentOrchestrator,
};
use std::sync::Arc;
use tokio::net::TcpListener;

fn create_test_config() -> Config {
    Config {
        claude_code: ClaudeCodeConfig {
            api_key: "sk-test-1234567890123456789012345678901234567890".to_string(),
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
            token: "test-discord-token-12345678901234567890123456789012345678901234567890".to_string(),
            command_prefix: "!spiral".to_string(),
            agent_mention_pattern: r"@Spiral(\w+)".to_string(),
        },
        api: ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 0, // Use random port for testing
            api_key: Some("test-secret-key-1234567890123456789012345678901234567890".to_string()),
            allowed_origins: vec!["http://localhost:3000".to_string()],
        },
    }
}

#[tokio::test]
async fn test_api_authentication() {
    // Create test server
    let config = create_test_config();
    let orchestrator = Arc::new(AgentOrchestrator::new(config.clone()).await.unwrap());
    let api_server = ApiServer::new(config.clone(), orchestrator).unwrap();
    
    // Start server on random port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    let app = api_server.build_router();
    
    // Start server in background
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    // Wait for server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let base_url = format!("http://{}", addr);
    let client = reqwest::Client::new();
    
    // Test 1: Health check should require auth (security hardened)
    let response = client
        .get(&format!("{}/health", base_url))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401, "Health check should require auth");
    
    // Test 2: Protected endpoint should fail without auth
    let response = client
        .get(&format!("{}/system/status", base_url))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    
    // Test 3: Protected endpoint should work with correct API key
    let response = client
        .get(&format!("{}/system/status", base_url))
        .header("x-api-key", "test-secret-key-1234567890123456789012345678901234567890")
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Test 4: Protected endpoint should fail with wrong API key
    let response = client
        .get(&format!("{}/system/status", base_url))
        .header("x-api-key", "wrong-key-1234567890123456789012345678901234567890")
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 401);
    
    // Test 5: Bearer token should work
    let response = client
        .get(&format!("{}/system/status", base_url))
        .header("Authorization", "Bearer test-secret-key-1234567890123456789012345678901234567890")
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn test_create_task_validation() {
    let config = create_test_config();
    let orchestrator = Arc::new(AgentOrchestrator::new(config.clone()).await.unwrap());
    let api_server = ApiServer::new(config.clone(), orchestrator).unwrap();
    
    // Start server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    let app = api_server.build_router();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let base_url = format!("http://{}", addr);
    let client = reqwest::Client::new();
    
    // Test: Invalid task content should be rejected
    let response = client
        .post(&format!("{}/tasks", base_url))
        .header("x-api-key", "test-secret-key-1234567890123456789012345678901234567890")
        .json(&serde_json::json!({
            "agent_type": "SoftwareDeveloper",
            "content": "<script>alert('xss')</script>",
            "priority": "Medium"
        }))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 400, "Malicious content should be rejected");
    
    // Test: Valid task should be accepted
    let response = client
        .post(&format!("{}/tasks", base_url))
        .header("x-api-key", "test-secret-key-1234567890123456789012345678901234567890")
        .json(&serde_json::json!({
            "agent_type": "SoftwareDeveloper",
            "content": "Create a hello world function in Rust",
            "priority": "Medium"
        }))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200, "Valid content should be accepted");
}

#[tokio::test]
async fn test_cors_headers() {
    let config = create_test_config();
    let orchestrator = Arc::new(AgentOrchestrator::new(config.clone()).await.unwrap());
    let api_server = ApiServer::new(config.clone(), orchestrator).unwrap();
    
    // Start server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    
    let app = api_server.build_router();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let base_url = format!("http/{}", addr);
    let client = reqwest::Client::new();
    
    // Test: CORS should only allow configured origins
    let response = client
        .get(&format!("{}/health", base_url))
        .header("Origin", "http://localhost:3000")
        .header("x-api-key", "test-secret-key-1234567890123456789012345678901234567890")
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    assert!(response.headers().contains_key("access-control-allow-origin"));
    
    // Test: Unconfigured origin should not get CORS headers
    let response = client
        .get(&format!("{}/health", base_url))
        .header("Origin", "http://evil.com")
        .header("x-api-key", "test-secret-key-1234567890123456789012345678901234567890")
        .send()
        .await
        .unwrap();
    
    // Note: The request should still succeed (200) but without CORS headers
    assert_eq!(response.status(), 200);
}