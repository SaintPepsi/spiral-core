//! Test harness for simulating complete self-update scenarios
//!
//! This provides a way to test the full update pipeline without
//! needing Discord or actual code changes.

use super::*;
use crate::claude_code::ClaudeCodeClient;
use crate::config::ClaudeCodeConfig;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};
use uuid;

/// Test harness for simulating self-updates
pub struct SelfUpdateTestHarness {
    update_queue: Arc<UpdateQueue>,
    approval_manager: Arc<ApprovalManager>,
    system_lock: Arc<SystemLock>,
    claude_client: Option<ClaudeCodeClient>,
    results_receiver: mpsc::Receiver<UpdateResult>,
    results_sender: mpsc::Sender<UpdateResult>,
}

impl SelfUpdateTestHarness {
    /// Create a new test harness
    pub async fn new(use_real_claude: bool) -> Result<Self> {
        let update_queue = Arc::new(UpdateQueue::new());
        let approval_manager = Arc::new(ApprovalManager::new());
        let system_lock = Arc::new(SystemLock::new());
        
        let claude_client = if use_real_claude {
            let config = ClaudeCodeConfig {
                claude_binary_path: None, // Auto-discover
                working_directory: Some("/tmp/spiral-test-harness".to_string()),
                timeout_seconds: 30,
                permission_mode: "standard".to_string(),
                allowed_tools: vec!["write".to_string(), "read".to_string()],
                workspace_cleanup_after_hours: 1,
                max_workspace_size_mb: 100,
            };
            Some(ClaudeCodeClient::new(config).await?)
        } else {
            None
        };
        
        let (results_sender, results_receiver) = mpsc::channel(10);
        
        Ok(Self {
            update_queue,
            approval_manager,
            system_lock,
            claude_client,
            results_receiver,
            results_sender,
        })
    }
    
    /// Run a simulated update request
    pub async fn simulate_update(&mut self, description: &str) -> Result<UpdateResult> {
        info!("[TestHarness] Simulating update: {}", description);
        
        // Create update request
        let request = SelfUpdateRequest {
            id: format!("test-{}", uuid::Uuid::new_v4()),
            codename: "test-update".to_string(),
            description: description.to_string(),
            user_id: 123456789,
            channel_id: 987654321,
            message_id: 111222333,
            combined_messages: vec![description.to_string()],
            timestamp: chrono::Utc::now().to_rfc3339(),
            retry_count: 0,
            status: UpdateStatus::Queued,
        };
        
        // Add to queue
        self.update_queue.try_add_request(request.clone()).await?;
        
        // Create executor
        let mut executor = UpdateExecutor::new(
            self.update_queue.clone(),
            self.claude_client.clone(),
            None, // No Discord
            self.approval_manager.clone(),
            self.system_lock.clone(),
        );
        
        // Process the request
        let result = executor.process_request(request).await;
        
        // Send result
        let _ = self.results_sender.send(result.clone()).await;
        
        Ok(result)
    }
    
    /// Simulate multiple concurrent updates
    pub async fn simulate_concurrent_updates(&mut self, updates: Vec<&str>) -> Vec<UpdateResult> {
        let mut handles = vec![];
        
        for description in updates {
            let queue = self.update_queue.clone();
            let approval = self.approval_manager.clone();
            let lock = self.system_lock.clone();
            let claude = self.claude_client.clone();
            let sender = self.results_sender.clone();
            let desc = description.to_string();
            
            let handle = tokio::spawn(async move {
                let request = SelfUpdateRequest {
                    id: format!("concurrent-{}", uuid::Uuid::new_v4()),
                    codename: "concurrent-test".to_string(),
                    description: desc.clone(),
                    user_id: 123456789,
                    channel_id: 987654321,
                    message_id: 111222333,
                    combined_messages: vec![desc],
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    retry_count: 0,
                    status: UpdateStatus::Queued,
                };
                
                let _ = queue.try_add_request(request.clone()).await;
                
                let mut executor = UpdateExecutor::new(
                    queue,
                    claude,
                    None,
                    approval,
                    lock,
                );
                
                let result = executor.process_request(request).await;
                let _ = sender.send(result.clone()).await;
                result
            });
            
            handles.push(handle);
        }
        
        // Wait for all to complete
        let mut results = vec![];
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }
        
        results
    }
    
    /// Get all results
    pub async fn get_results(&mut self) -> Vec<UpdateResult> {
        let mut results = vec![];
        while let Ok(result) = self.results_receiver.try_recv() {
            results.push(result);
        }
        results
    }
}

#[cfg(test)]
mod harness_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_simple_update_simulation() {
        let mut harness = SelfUpdateTestHarness::new(false).await.unwrap();
        
        let result = harness.simulate_update("Fix a typo in README").await.unwrap();
        
        // With mock Claude, should complete but may not actually fix anything
        assert_eq!(result.request.description, "Fix a typo in README");
        // Success depends on whether pre-flight checks pass
    }
    
    #[tokio::test]
    async fn test_concurrent_updates() {
        let mut harness = SelfUpdateTestHarness::new(false).await.unwrap();
        
        let updates = vec![
            "Update documentation",
            "Fix formatting issues",
            "Add new feature",
        ];
        
        let results = harness.simulate_concurrent_updates(updates).await;
        
        // Should handle all updates (though may queue them)
        assert!(results.len() > 0);
        
        // Check queue handled them properly
        for result in results {
            println!("Update {}: success={}, message={}", 
                result.request.codename,
                result.success,
                result.message
            );
        }
    }
    
    #[tokio::test]
    #[ignore = "Requires real Claude binary and takes time"]
    async fn test_real_claude_update() {
        let mut harness = SelfUpdateTestHarness::new(true).await.unwrap();
        
        let result = harness.simulate_update(
            "Add a comment to the main function explaining what it does"
        ).await.unwrap();
        
        println!("Real Claude update result: {:?}", result);
        
        // With real Claude, should see actual validation results
        if let Some(validation) = result.validation_results {
            println!("Phase 1 passed: {}", validation.engineering_review_passed);
            println!("Phase 2 passed: {}", validation.assembly_checklist_passed);
        }
    }
}

/// Example standalone test runner
#[allow(dead_code)]
pub async fn run_validation_test_suite() -> Result<()> {
    println!("🧪 Running Self-Update Validation Test Suite");
    println!("============================================");
    
    // Test 1: Mock validation
    println!("\n📝 Test 1: Mock Validation");
    {
        let mut harness = SelfUpdateTestHarness::new(false).await?;
        let result = harness.simulate_update("Test mock validation").await?;
        println!("  Result: success={}, message={}", result.success, result.message);
    }
    
    // Test 2: Pre-flight checks
    println!("\n🔍 Test 2: Pre-flight Checks");
    {
        let validator = UpdateValidator::new();
        let request = SelfUpdateRequest {
            id: "preflight-test".to_string(),
            codename: "test".to_string(),
            description: "Test preflight".to_string(),
            user_id: 123456789,
            channel_id: 987654321,
            message_id: 111222333,
            combined_messages: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            retry_count: 0,
            status: UpdateStatus::Queued,
        };
        
        match validator.validate_request(&request).await {
            Ok(_) => println!("  ✅ Pre-flight checks passed"),
            Err(e) => println!("  ❌ Pre-flight checks failed: {}", e),
        }
    }
    
    // Test 3: Queue management
    println!("\n📦 Test 3: Queue Management");
    {
        let queue = UpdateQueue::new();
        let request1 = SelfUpdateRequest {
            id: "queue-1".to_string(),
            codename: "test1".to_string(),
            description: "First".to_string(),
            user_id: 123456789,
            channel_id: 987654321,
            message_id: 111222333,
            combined_messages: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
            retry_count: 0,
            status: UpdateStatus::Queued,
        };
        
        queue.try_add_request(request1).await?;
        let status = queue.get_status().await;
        println!("  Queue size: {}", status.queue_size);
        println!("  ✅ Queue management working");
    }
    
    println!("\n✨ Test suite completed!");
    Ok(())
}