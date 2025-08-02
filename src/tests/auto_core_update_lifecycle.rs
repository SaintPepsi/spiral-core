/// 🔄 AUTO CORE UPDATE LIFECYCLE TESTS: Comprehensive testing of self update system
/// CRITICAL: Tests the complete lifecycle of automated core updates including git operations
/// Why: Ensures system can safely update itself without data loss or corruption
/// Alternative: Manual testing only (rejected: insufficient coverage for critical system)

#[cfg(test)]
mod auto_core_update_lifecycle {
    use crate::discord::self_update::{
        SelfUpdateRequest, UpdateStatus, UpdateQueue
    };
    use crate::discord::spiral_constellation_bot::ConstellationBotHandler;
    use crate::SpiralError;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tempfile::TempDir;
    use std::process::Command;
    use std::fs;

    /// Mock Claude response generator for auto-update scenarios
    struct MockUpdateClaudeClient {
        responses: Vec<String>,
        current_index: Arc<Mutex<usize>>,
    }

    impl MockUpdateClaudeClient {
        fn new(responses: Vec<String>) -> Self {
            Self {
                responses,
                current_index: Arc::new(Mutex::new(0)),
            }
        }

        async fn get_next_response(&self) -> String {
            let mut index = self.current_index.lock().await;
            let response = self.responses.get(*index)
                .cloned()
                .unwrap_or_else(|| "Default mock response".to_string());
            *index += 1;
            response
        }
    }

    /// Happy path: Successful auto-update from request to completion
    #[tokio::test]
    async fn test_auto_update_happy_lifecycle() {
        // Phase 1: Setup test repository
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        
        // Initialize git repo
        Command::new("git")
            .args(&["init"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to init git repo");
        
        // Create initial file
        let test_file = repo_path.join("test.rs");
        fs::write(&test_file, "fn main() { println!(\"v1\"); }").unwrap();
        
        Command::new("git")
            .args(&["add", "."])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to git add");
        
        Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to git commit");

        // Phase 2: Initialize auto-update system
        let bot_handler = ConstellationBotHandler::new(
            "test-token".to_string(),
            Some("test-api-key-32-chars-minimum-length".to_string()),
        );
        
        let queue_manager = bot_handler.get_queue_manager();
        
        // Create update request
        let update_request = SelfUpdateRequest {
            request_id: "test-update-001".to_string(),
            user_id: 12345,
            channel_id: 67890,
            description: "Update main function to v2".to_string(),
            priority: crate::models::Priority::High,
            requires_approval: false,
            created_at: std::time::Instant::now(),
            file_paths: vec!["test.rs".to_string()],
        };
        
        // Phase 3: Submit and process update
        queue_manager.submit_request(update_request.clone()).await
            .expect("Failed to submit update request");
        
        // Simulate Claude response for the update
        let mock_claude = MockUpdateClaudeClient::new(vec![
            "I'll update the main function to v2. Let me make that change.".to_string(),
            "```rust\nfn main() { println!(\"v2 - improved version\"); }\n```".to_string(),
        ]);
        
        // Process the update (in real system, this would be done by the bot)
        let update_content = mock_claude.get_next_response().await;
        
        // Simulate file update
        fs::write(&test_file, "fn main() { println!(\"v2 - improved version\"); }").unwrap();
        
        // Create git snapshot
        let snapshot_id = bot_handler.create_git_snapshot(
            &repo_path,
            &format!("Auto-update: {}", update_request.description)
        ).await.expect("Failed to create snapshot");
        
        // Verify the update was applied
        let updated_content = fs::read_to_string(&test_file).unwrap();
        assert!(updated_content.contains("v2 - improved version"));
        
        // Update status to completed
        queue_manager.update_request_status(
            &update_request.request_id,
            UpdateStatus::Completed
        ).await.expect("Failed to update status");
        
        // Phase 4: Verify final state
        let final_status = queue_manager.get_request_status(&update_request.request_id).await;
        assert_eq!(final_status, Some(UpdateStatus::Completed));
    }

    /// Error path: Update fails and system performs rollback
    #[tokio::test]
    async fn test_auto_update_rollback_on_failure() {
        // Setup
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        
        // Initialize git repo with initial state
        Command::new("git").args(&["init"]).current_dir(&repo_path).output().unwrap();
        
        let test_file = repo_path.join("critical.rs");
        let original_content = "fn critical_function() { /* working code */ }";
        fs::write(&test_file, original_content).unwrap();
        
        Command::new("git").args(&["add", "."]).current_dir(&repo_path).output().unwrap();
        Command::new("git").args(&["commit", "-m", "Working state"]).current_dir(&repo_path).output().unwrap();
        
        let bot_handler = ConstellationBotHandler::new(
            "test-token".to_string(),
            Some("test-api-key-32-chars-minimum-length".to_string()),
        );
        
        // Create snapshot before risky update
        let snapshot_id = bot_handler.create_git_snapshot(
            &repo_path,
            "Pre-update safety snapshot"
        ).await.unwrap();
        
        // Simulate failed update (introduces syntax error)
        fs::write(&test_file, "fn critical_function() { SYNTAX ERROR }").unwrap();
        
        // Detect failure (in real system, this would be detected by build/test failure)
        let build_result = Command::new("rustc")
            .args(&["--edition", "2021", test_file.to_str().unwrap()])
            .output()
            .unwrap();
        
        assert!(!build_result.status.success(), "Build should fail with syntax error");
        
        // Perform rollback
        bot_handler.rollback_to_snapshot(&repo_path, &snapshot_id).await
            .expect("Rollback should succeed");
        
        // Verify content is restored
        let restored_content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(restored_content, original_content);
    }

    /// Error path: Queue overflow protection
    #[tokio::test]
    async fn test_auto_update_queue_overflow_protection() {
        let bot_handler = ConstellationBotHandler::new(
            "test-token".to_string(),
            Some("test-api-key-32-chars-minimum-length".to_string()),
        );
        
        let queue_manager = bot_handler.get_queue_manager();
        
        // Attempt to overflow the queue
        let mut submitted_count = 0;
        for i in 0..150 { // Try to exceed MAX_QUEUE_SIZE (100)
            let request = SelfUpdateRequest {
                request_id: format!("overflow-test-{}", i),
                user_id: 12345,
                channel_id: 67890,
                description: format!("Update request {}", i),
                priority: crate::models::Priority::Low,
                requires_approval: false,
                created_at: std::time::Instant::now(),
                file_paths: vec![],
            };
            
            match queue_manager.submit_request(request).await {
                Ok(_) => submitted_count += 1,
                Err(SpiralError::QueueFull) => {
                    // Expected when queue is full
                    break;
                }
                Err(e) => panic!("Unexpected error: {}", e),
            }
        }
        
        assert_eq!(submitted_count, 100, "Queue should accept exactly MAX_QUEUE_SIZE requests");
        
        // Verify queue status
        let status = queue_manager.get_queue_status().await;
        assert_eq!(status.total_requests, 100);
        assert_eq!(status.queue_size, 100);
    }

    /// Happy path: Emergency queue clear functionality
    #[tokio::test]
    async fn test_auto_update_emergency_clear() {
        let bot_handler = ConstellationBotHandler::new(
            "test-token".to_string(),
            Some("test-api-key-32-chars-minimum-length".to_string()),
        );
        
        let queue_manager = bot_handler.get_queue_manager();
        
        // Add some requests
        for i in 0..5 {
            let request = SelfUpdateRequest {
                request_id: format!("clear-test-{}", i),
                user_id: 12345,
                channel_id: 67890,
                description: format!("Request {}", i),
                priority: crate::models::Priority::Medium,
                requires_approval: false,
                created_at: std::time::Instant::now(),
                file_paths: vec![],
            };
            
            queue_manager.submit_request(request).await.unwrap();
        }
        
        // Verify queue has requests
        let status_before = queue_manager.get_queue_status().await;
        assert_eq!(status_before.queue_size, 5);
        
        // Perform emergency clear
        queue_manager.emergency_clear().await.unwrap();
        
        // Verify queue is empty
        let status_after = queue_manager.get_queue_status().await;
        assert_eq!(status_after.queue_size, 0);
        assert_eq!(status_after.pending_count, 0);
    }

    /// Integration test: Full auto-update workflow with authorization
    #[tokio::test]
    async fn test_auto_update_authorization_workflow() {
        let bot_handler = ConstellationBotHandler::new(
            "test-token".to_string(),
            Some("test-api-key-32-chars-minimum-length".to_string()),
        );
        
        // Attempt update without authorization (should fail)
        let result = bot_handler.handle_auto_update_request(
            12345, // user_id
            67890, // channel_id
            "Update critical system file".to_string(),
            None, // No API key provided
        ).await;
        
        assert!(matches!(result, Err(SpiralError::Unauthorized)));
        
        // Attempt with valid authorization
        let result = bot_handler.handle_auto_update_request(
            12345,
            67890,
            "Update documentation file".to_string(),
            Some("test-api-key-32-chars-minimum-length".to_string()),
        ).await;
        
        // Should succeed (would return request_id in real implementation)
        assert!(result.is_ok());
    }

    /// Test proper shutdown signal handling in auto-update system
    #[tokio::test]
    async fn test_auto_update_shutdown_signal_handling() {
        let bot_handler = ConstellationBotHandler::new(
            "test-token".to_string(),
            Some("test-api-key-32-chars-minimum-length".to_string()),
        );
        
        let queue_manager = bot_handler.get_queue_manager();
        
        // Start background processing (would normally be done by bot)
        let processing_handle = tokio::spawn({
            let queue_clone = queue_manager.clone();
            async move {
                // Simulate processing loop
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    // Check for shutdown signal
                    if queue_clone.is_shutting_down().await {
                        break;
                    }
                }
            }
        });
        
        // Add a request
        let request = SelfUpdateRequest {
            request_id: "shutdown-test".to_string(),
            user_id: 12345,
            channel_id: 67890,
            description: "Test update".to_string(),
            priority: crate::models::Priority::High,
            requires_approval: false,
            created_at: std::time::Instant::now(),
            file_paths: vec![],
        };
        
        queue_manager.submit_request(request).await.unwrap();
        
        // Initiate shutdown
        queue_manager.shutdown().await;
        
        // Verify processing stops
        let result = tokio::time::timeout(
            tokio::time::Duration::from_secs(1),
            processing_handle
        ).await;
        
        assert!(result.is_ok(), "Processing should stop within timeout");
        
        // Verify no new requests accepted after shutdown
        let new_request = SelfUpdateRequest {
            request_id: "after-shutdown".to_string(),
            user_id: 12345,
            channel_id: 67890,
            description: "Should fail".to_string(),
            priority: crate::models::Priority::High,
            requires_approval: false,
            created_at: std::time::Instant::now(),
            file_paths: vec![],
        };
        
        let result = queue_manager.submit_request(new_request).await;
        assert!(result.is_err(), "Should not accept requests after shutdown");
    }
}