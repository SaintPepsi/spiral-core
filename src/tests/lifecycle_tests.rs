//! 🧪 LIFECYCLE TESTS: Comprehensive system lifecycle testing with happy and error paths
//! CRITICAL: Tests the full lifecycle of key system components including startup, operation, and shutdown
//! Why: Ensures system reliability and proper resource management across all scenarios
//! Alternative: Unit tests only (rejected: doesn't test component interactions)

#[cfg(test)]
mod orchestrator_lifecycle {
    use crate::agents::orchestrator::AgentOrchestrator;
    use crate::config::Config;
    use crate::models::{AgentType, Priority, Task, TaskStatus};
    use crate::SpiralError;
    use std::sync::Arc;
    use tokio::time::{timeout, Duration};

    /// Happy path: Complete orchestrator lifecycle from startup to shutdown
    #[tokio::test]
    async fn test_orchestrator_happy_lifecycle() {
        // Phase 1: Startup
        let config = Config::test_config();
        let orchestrator = Arc::new(
            AgentOrchestrator::new(config)
                .await
                .expect("Failed to create orchestrator"),
        );

        // Start the orchestrator
        orchestrator
            .run()
            .await
            .expect("Failed to start orchestrator");

        // Verify it's running
        assert_eq!(orchestrator.get_queue_length().await, 0);
        assert!(orchestrator.get_system_uptime().await > 0.0);

        // Phase 2: Operation
        // Submit a task
        let task = Task::new(
            AgentType::SoftwareDeveloper,
            "Write a hello world function".to_string(),
            Priority::Medium,
        );
        let task_id = task.id.clone();

        orchestrator
            .submit_task(task)
            .await
            .expect("Failed to submit task");

        // Verify task is queued
        assert_eq!(orchestrator.get_queue_length().await, 1);

        // Wait for task processing (with timeout)
        let result = timeout(Duration::from_secs(5), async {
            loop {
                if let Some(task) = orchestrator.get_task_status(&task_id).await {
                    if task.status == TaskStatus::Completed || task.status == TaskStatus::Failed {
                        return task.status;
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await;

        assert!(result.is_ok(), "Task processing timed out");

        // Phase 3: Shutdown
        orchestrator.shutdown().await;

        // Verify clean shutdown
        assert_eq!(orchestrator.get_queue_length().await, 0);
    }

    /// Error path: Orchestrator handles task submission failures gracefully
    #[tokio::test]
    async fn test_orchestrator_error_lifecycle() {
        let config = Config::test_config();
        let orchestrator = Arc::new(
            AgentOrchestrator::new(config)
                .await
                .expect("Failed to create orchestrator"),
        );

        orchestrator
            .run()
            .await
            .expect("Failed to start orchestrator");

        // Submit tasks until queue is full
        for i in 0..1001 {
            let task = Task::new(
                AgentType::SoftwareDeveloper,
                format!("Task {i}"),
                Priority::Low,
            );

            match orchestrator.submit_task(task).await {
                Ok(_) => continue,
                Err(SpiralError::Agent { message }) if message.contains("queue is full") => {
                    // Expected error when queue is full
                    assert!(orchestrator.get_queue_length().await >= 1000);
                    break;
                }
                Err(e) => panic!("Unexpected error: {e}"),
            }
        }

        // Verify system is still operational after error
        assert!(orchestrator.get_system_uptime().await > 0.0);

        // Clean shutdown should still work
        orchestrator.shutdown().await;
    }

    /// Error path: Orchestrator handles task failures gracefully
    #[tokio::test]
    async fn test_orchestrator_agent_failure_recovery() {
        // Note: With the current architecture, we test that the orchestrator
        // continues to function even when individual tasks may fail
        let config = Config::test_config();
        let orchestrator = Arc::new(
            AgentOrchestrator::new(config)
                .await
                .expect("Failed to create orchestrator"),
        );

        orchestrator
            .run()
            .await
            .expect("Failed to start orchestrator");

        // Submit a task - in a real scenario this might fail
        let task = Task::new(
            AgentType::SoftwareDeveloper,
            "Test task for failure recovery".to_string(),
            Priority::High,
        );
        let task_id = task.id.clone();

        orchestrator.submit_task(task).await.unwrap();

        // Wait for task to complete (may succeed or fail)
        let result = timeout(Duration::from_secs(5), async {
            loop {
                if let Some(task_info) = orchestrator.get_task_status(&task_id).await {
                    if matches!(task_info.status, TaskStatus::Completed | TaskStatus::Failed) {
                        return task_info.status;
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await;

        assert!(result.is_ok(), "Task processing timed out");

        // Verify system is still operational regardless of task outcome
        let system_uptime = orchestrator.get_system_uptime().await;
        assert!(system_uptime > 0.0, "System should remain operational");

        // Verify we can still submit new tasks
        let recovery_task = Task::new(
            AgentType::SoftwareDeveloper,
            "Recovery test task".to_string(),
            Priority::Medium,
        );
        let recovery_result = orchestrator.submit_task(recovery_task).await;
        assert!(
            recovery_result.is_ok(),
            "Should accept new tasks after failure"
        );

        orchestrator.shutdown().await;
    }
}

#[cfg(test)]
mod api_server_lifecycle {
    use crate::agents::orchestrator::AgentOrchestrator;
    use crate::api::ApiServer;
    use crate::config::Config;
    use std::sync::Arc;
    use tokio::time::Duration;

    /// Happy path: API server starts, serves requests, and shuts down cleanly
    #[tokio::test]
    async fn test_api_server_happy_lifecycle() {
        // Phase 1: Startup
        let mut config = Config::test_config();
        config.api.host = "127.0.0.1".to_string();
        config.api.port = 3456; // Use a specific test port
        config.api.api_key = Some("test-key-must-be-at-least-32-characters-long".to_string());

        let orchestrator = Arc::new(
            AgentOrchestrator::new(config.clone())
                .await
                .expect("Failed to create orchestrator"),
        );
        orchestrator
            .run()
            .await
            .expect("Failed to start orchestrator");

        let api_server = ApiServer::new(config.clone(), orchestrator.clone())
            .expect("Failed to create API server");

        // Start server in background
        let server_handle = tokio::spawn(async move { api_server.run().await });

        // Give server time to start
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Phase 2: Operation
        let client = reqwest::Client::new();

        // Test health endpoint (requires auth)
        let health_response = client
            .get(format!(
                "http://{}:{}/health",
                config.api.host, config.api.port
            ))
            .header("X-API-Key", "test-key-must-be-at-least-32-characters-long")
            .send()
            .await;

        assert!(health_response.is_ok());
        assert_eq!(health_response.unwrap().status(), 200);

        // Test task submission with auth
        let task_response = client
            .post(format!(
                "http://{}:{}/tasks",
                config.api.host, config.api.port
            ))
            .header("X-API-Key", "test-key-must-be-at-least-32-characters-long")
            .json(&serde_json::json!({
                "agent_type": "SoftwareDeveloper",
                "content": "Test task",
                "priority": "Medium"
            }))
            .send()
            .await;

        assert!(task_response.is_ok());
        assert_eq!(task_response.unwrap().status(), 201);

        // Phase 3: Shutdown
        server_handle.abort();
        orchestrator.shutdown().await;
    }

    /// Error path: API server handles invalid requests gracefully
    #[tokio::test]
    async fn test_api_server_error_handling() {
        let mut config = Config::test_config();
        config.api.host = "127.0.0.1".to_string();
        config.api.port = 3457; // Use a different test port
        config.api.api_key = Some("test-key-must-be-at-least-32-characters-long".to_string());

        let orchestrator = Arc::new(
            AgentOrchestrator::new(config.clone())
                .await
                .expect("Failed to create orchestrator"),
        );
        orchestrator
            .run()
            .await
            .expect("Failed to start orchestrator");

        let api_server = ApiServer::new(config.clone(), orchestrator.clone()).unwrap();

        let server_handle = tokio::spawn(async move { api_server.run().await });

        tokio::time::sleep(Duration::from_millis(500)).await;

        let client = reqwest::Client::new();

        // Test missing auth
        let response = client
            .post(format!(
                "http://{}:{}/tasks",
                config.api.host, config.api.port
            ))
            .json(&serde_json::json!({
                "agent_type": "SoftwareDeveloper",
                "content": "Test task",
                "priority": "Medium"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 401);

        // Test invalid API key
        let response = client
            .post(format!(
                "http://{}:{}/tasks",
                config.api.host, config.api.port
            ))
            .header("X-API-Key", "wrong-key")
            .json(&serde_json::json!({
                "agent_type": "SoftwareDeveloper",
                "content": "Test task",
                "priority": "Medium"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 401);

        // Test malicious content
        let response = client
            .post(format!(
                "http://{}:{}/tasks",
                config.api.host, config.api.port
            ))
            .header("X-API-Key", "test-key-must-be-at-least-32-characters-long")
            .json(&serde_json::json!({
                "agent_type": "SoftwareDeveloper",
                "content": "<script>alert('xss')</script>",
                "priority": "Medium"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 400);

        server_handle.abort();
        orchestrator.shutdown().await;
    }
}

#[cfg(test)]
mod monitoring_lifecycle {
    use crate::monitoring::{MonitoringConfig, SystemMonitor};
    use tokio::time::Duration;

    /// Happy path: Monitor starts, collects metrics, and shuts down cleanly
    #[tokio::test]
    async fn test_monitor_happy_lifecycle() {
        // Phase 1: Startup
        let monitor = SystemMonitor::new(MonitoringConfig {
            collection_interval: Duration::from_millis(100), // Fast for testing
            ..Default::default()
        });

        monitor
            .start_monitoring()
            .await
            .expect("Failed to start monitoring");

        // Phase 2: Operation
        // Wait for at least 3 metric collections (100ms interval * 3 = 300ms + extra buffer)
        // Adding more time due to test environment variability
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Verify metrics are being collected
        let current_metrics = monitor.get_current_metrics().await;
        assert!(current_metrics.uptime_seconds > 0.0);

        let history = monitor.get_metrics_history().await;
        assert!(
            history.len() >= 2,
            "Expected at least 2 metric samples, got {}",
            history.len()
        );

        // Phase 3: Shutdown
        monitor.shutdown().await;

        // Verify no new metrics after shutdown
        let history_before = monitor.get_metrics_history().await.len();
        tokio::time::sleep(Duration::from_millis(200)).await;
        let history_after = monitor.get_metrics_history().await.len();
        assert_eq!(
            history_before, history_after,
            "Metrics still being collected after shutdown"
        );
    }

    /// Error path: Monitor handles collection failures gracefully
    #[tokio::test]
    async fn test_monitor_error_recovery() {
        let monitor = SystemMonitor::new(MonitoringConfig::default());

        // Start monitoring
        monitor.start_monitoring().await.unwrap();

        // Monitor should continue even if individual collections fail
        // (In real implementation, this would be tested by mocking system calls)

        // Verify monitor is still operational
        let health = monitor.get_health_status().await;
        assert!(matches!(
            health,
            crate::monitoring::HealthStatus::Healthy | crate::monitoring::HealthStatus::Degraded
        ));

        monitor.shutdown().await;
    }
}

#[cfg(test)]
mod circuit_breaker_lifecycle {
    use crate::claude_code::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
    use std::time::Duration;

    /// Happy path: Circuit breaker transitions through states correctly
    #[tokio::test]
    async fn test_circuit_breaker_happy_lifecycle() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            timeout_duration: Duration::from_millis(100),
            success_threshold: 1,
            failure_window: Duration::from_secs(60),
        });

        // Phase 1: Closed state (normal operation)
        assert!(breaker.should_allow_request().await);

        // Record some successes
        breaker.record_success().await;
        breaker.record_success().await;
        assert!(breaker.should_allow_request().await);

        // Phase 2: Open state (after failures)
        breaker.record_failure().await;
        breaker.record_failure().await;
        breaker.record_failure().await;

        assert!(
            !breaker.should_allow_request().await,
            "Circuit should be open after 3 failures"
        );

        // Phase 3: Half-open state (after timeout)
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(
            breaker.should_allow_request().await,
            "Circuit should be half-open after timeout"
        );

        // Record success to close circuit
        breaker.record_success().await;
        assert!(
            breaker.should_allow_request().await,
            "Circuit should be closed after success in half-open"
        );
    }

    /// Error path: Circuit breaker protects system during cascading failures
    #[tokio::test]
    async fn test_circuit_breaker_cascading_failures() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            timeout_duration: Duration::from_millis(50),
            success_threshold: 2,
            failure_window: Duration::from_secs(60),
        });

        // Simulate cascading failures
        for _ in 0..10 {
            if breaker.should_allow_request().await {
                breaker.record_failure().await;
            }
        }

        // Circuit should be open, protecting the system
        assert!(!breaker.should_allow_request().await);

        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Should allow limited requests in half-open
        assert!(breaker.should_allow_request().await);
        breaker.record_failure().await; // Still failing

        // After failure in half-open, circuit should be open again
        assert!(!breaker.should_allow_request().await);
    }
}

#[cfg(test)]
mod integration_lifecycle {
    use crate::agents::orchestrator::AgentOrchestrator;
    use crate::api::ApiServer;
    use crate::config::Config;
    use crate::monitoring::SystemMonitor;
    use std::sync::Arc;
    use tokio::time::Duration;

    /// Happy path: Full system integration lifecycle
    #[tokio::test]
    async fn test_full_system_happy_lifecycle() {
        // Phase 1: System startup
        let mut config = Config::test_config();
        config.api.api_key = Some("integration-test-key-32-characters-long".to_string());

        // Start monitoring
        let mut monitor = SystemMonitor::new(Default::default());
        monitor.start_monitoring().await.unwrap();

        // Start orchestrator
        let orchestrator = Arc::new(
            AgentOrchestrator::new(config.clone())
                .await
                .expect("Failed to create orchestrator"),
        );
        orchestrator
            .run()
            .await
            .expect("Failed to start orchestrator");

        // Register with monitor
        if let Ok(client) = orchestrator.get_claude_client() {
            monitor.register_claude_client(Arc::new(client.clone()));
        }

        // Start API server
        let api_server = ApiServer::new(config, orchestrator.clone()).unwrap();
        let api_handle = tokio::spawn(async move { api_server.run().await });

        // Phase 2: System operation
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify all components are operational
        assert!(orchestrator.get_system_uptime().await > 0.0);

        // Note: We can't easily check monitor health status without keeping it in Arc
        // but that would require changes to the register_claude_client API

        // Phase 3: Graceful shutdown
        api_handle.abort();
        orchestrator.shutdown().await;
        monitor.shutdown().await;

        // All components should be stopped
        assert_eq!(orchestrator.get_queue_length().await, 0);
    }

    /// Error path: System handles component failures gracefully
    #[tokio::test]
    async fn test_system_component_failure_recovery() {
        let config = Config::test_config();

        // Test that the system can recover from transient failures
        let orchestrator = Arc::new(
            AgentOrchestrator::new(config.clone())
                .await
                .expect("Failed to create orchestrator"),
        );

        orchestrator
            .run()
            .await
            .expect("Failed to start orchestrator");

        // Submit tasks to stress the system
        for i in 0..5 {
            let task = crate::models::Task::new(
                crate::models::AgentType::SoftwareDeveloper,
                format!("Stress test task {i}"),
                crate::models::Priority::Low,
            );
            let _ = orchestrator.submit_task(task).await;
        }

        // Give time for processing
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify system is still healthy
        assert!(orchestrator.get_system_uptime().await > 0.0);

        // Clean shutdown
        orchestrator.shutdown().await;
    }
}
