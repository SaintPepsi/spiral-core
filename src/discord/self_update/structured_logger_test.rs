#[cfg(test)]
mod tests {
    use super::super::UpdateLogger;
    use std::fs;
    use std::path::Path;
    
    #[tokio::test]
    async fn test_structured_logging() {
        // Create a logger
        let mut logger = UpdateLogger::new(
            "test-update-123".to_string(),
            "test-feature".to_string()
        ).expect("Failed to create logger");
        
        // Test main log
        logger.log_to_main("Test main log entry").expect("Failed to log to main");
        
        // Test phase logging
        logger.log_to_phase("Planning", "Starting planning phase").await
            .expect("Failed to log to phase");
        logger.log_to_phase("Planning", "Created implementation plan").await
            .expect("Failed to log to phase");
        
        // Test error logging
        logger.log_error("Validation", "Test error message", Some("During compilation")).await
            .expect("Failed to log error");
        
        // Test warning logging
        logger.log_warning("Implementation", "Test warning message").await
            .expect("Failed to log warning");
        
        // Test validation results
        logger.log_validation_results("Final", "All tests passed\nNo errors found").await
            .expect("Failed to log validation results");
        
        // Create summary
        logger.create_summary(true, "Test completed successfully", std::time::Duration::from_secs(42)).await
            .expect("Failed to create summary");
        
        // Verify files were created
        let log_dir = logger.get_log_dir();
        assert!(log_dir.exists(), "Log directory should exist");
        assert!(log_dir.join("main.log").exists(), "Main log should exist");
        assert!(log_dir.join("phase_Planning.log").exists(), "Phase log should exist");
        assert!(log_dir.join("errors.log").exists(), "Errors log should exist");
        assert!(log_dir.join("validation_results.log").exists(), "Validation results should exist");
        assert!(log_dir.join("summary.json").exists(), "Summary should exist");
        
        // Read and verify summary content
        let summary_content = fs::read_to_string(log_dir.join("summary.json"))
            .expect("Failed to read summary");
        assert!(summary_content.contains("\"success\": true"));
        assert!(summary_content.contains("\"codename\": \"test-feature\""));
        assert!(summary_content.contains("\"duration_seconds\": 42"));
        
        // Test archiving
        let archive_path = logger.archive_logs().await
            .expect("Failed to archive logs");
        assert!(archive_path.exists(), "Archive should exist");
        
        println!("Test logs created at: {}", log_dir.display());
        println!("Archive created at: {}", archive_path.display());
    }
}