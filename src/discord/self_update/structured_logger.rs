//! Structured logging for self-update operations
//!
//! This module provides logging functionality that organizes logs by update ID,
//! creating a dedicated log directory for each update operation with all relevant
//! logs, making debugging and auditing much easier.

use crate::{error::SpiralError, Result};
use chrono::Utc;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Directory for all update logs
const UPDATE_LOGS_DIR: &str = "logs/updates";

/// Structured logger for self-update operations
pub struct StructuredLogger {
    /// Base directory for this update's logs
    log_dir: PathBuf,
    /// Update ID
    update_id: String,
    /// Codename of the update
    codename: String,
    /// Main log file handle
    main_log: Arc<RwLock<Option<fs::File>>>,
    /// Phase-specific log handles
    phase_logs: Arc<RwLock<std::collections::HashMap<String, fs::File>>>,
}

impl StructuredLogger {
    /// Create a new logger for an update
    pub fn new(update_id: String, codename: String) -> Result<Self> {
        // Create timestamp for directory name
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");

        // Create directory name: codename_timestamp_id
        let dir_name = format!(
            "{}_{}_{}",
            Self::sanitize_filename(&codename),
            timestamp,
            Self::sanitize_filename(&update_id)
        );

        let log_dir = PathBuf::from(UPDATE_LOGS_DIR).join(dir_name);

        // Create the directory
        fs::create_dir_all(&log_dir).map_err(|e| {
            SpiralError::SystemError(format!("Failed to create log directory: {}", e))
        })?;

        // Create main log file
        let main_log_path = log_dir.join("main.log");
        let main_log_file = fs::File::create(&main_log_path).map_err(|e| {
            SpiralError::SystemError(format!("Failed to create main log file: {}", e))
        })?;

        // Write header to main log
        let mut logger = Self {
            log_dir: log_dir.clone(),
            update_id: update_id.clone(),
            codename: codename.clone(),
            main_log: Arc::new(RwLock::new(Some(main_log_file))),
            phase_logs: Arc::new(RwLock::new(std::collections::HashMap::new())),
        };

        // Log initial header
        logger.log_to_main(&format!(
            "=== Self-Update Log ===\n\
             Update ID: {}\n\
             Codename: {}\n\
             Started: {}\n\
             Log Directory: {}\n\
             =======================\n",
            update_id,
            codename,
            Utc::now().to_rfc3339(),
            log_dir.display()
        ))?;

        info!(
            "[StructuredLogger] Created structured log directory: {}",
            log_dir.display()
        );

        Ok(logger)
    }

    /// Sanitize a string for use in filename
    fn sanitize_filename(s: &str) -> String {
        s.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .chars()
            .take(50) // Limit length
            .collect()
    }

    /// Log a message to the main log file
    pub fn log_to_main(&mut self, message: &str) -> Result<()> {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let formatted = format!("[{}] {}\n", timestamp, message);

        // Write to file
        if let Ok(mut guard) = self.main_log.try_write() {
            if let Some(ref mut file) = *guard {
                file.write_all(formatted.as_bytes()).map_err(|e| {
                    SpiralError::SystemError(format!("Failed to write to main log: {}", e))
                })?;
                file.flush().map_err(|e| {
                    SpiralError::SystemError(format!("Failed to flush main log: {}", e))
                })?;
            }
        }

        Ok(())
    }

    /// Log a message to a phase-specific log file
    pub async fn log_to_phase(&mut self, phase: &str, message: &str) -> Result<()> {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let formatted = format!("[{}] {}\n", timestamp, message);

        let mut phase_logs = self.phase_logs.write().await;

        // Create phase log file if it doesn't exist
        if !phase_logs.contains_key(phase) {
            let phase_filename = format!("phase_{}.log", Self::sanitize_filename(phase));
            let phase_path = self.log_dir.join(phase_filename);
            let phase_file = fs::File::create(phase_path).map_err(|e| {
                SpiralError::SystemError(format!("Failed to create phase log: {}", e))
            })?;
            phase_logs.insert(phase.to_string(), phase_file);

            // Log header to phase file
            if let Some(file) = phase_logs.get_mut(phase) {
                let header = format!(
                    "=== Phase: {} ===\n\
                     Update ID: {}\n\
                     Started: {}\n\
                     =================\n",
                    phase,
                    self.update_id,
                    Utc::now().to_rfc3339()
                );
                file.write_all(header.as_bytes()).map_err(|e| {
                    SpiralError::SystemError(format!("Failed to write phase header: {}", e))
                })?;
            }
        }

        // Write to phase log
        if let Some(file) = phase_logs.get_mut(phase) {
            file.write_all(formatted.as_bytes()).map_err(|e| {
                SpiralError::SystemError(format!("Failed to write to phase log: {}", e))
            })?;
            file.flush().map_err(|e| {
                SpiralError::SystemError(format!("Failed to flush phase log: {}", e))
            })?;
        }

        Ok(())
    }

    /// Log an error with context
    pub async fn log_error(
        &mut self,
        phase: &str,
        error: &str,
        context: Option<&str>,
    ) -> Result<()> {
        let error_msg = if let Some(ctx) = context {
            format!("ERROR in {}: {} (Context: {})", phase, error, ctx)
        } else {
            format!("ERROR in {}: {}", phase, error)
        };

        // Log to main log
        self.log_to_main(&error_msg)?;

        // Log to phase log
        self.log_to_phase(phase, &error_msg).await?;

        // Also create an errors.log file
        let errors_path = self.log_dir.join("errors.log");
        let mut errors_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(errors_path)
            .map_err(|e| SpiralError::SystemError(format!("Failed to open errors log: {}", e)))?;

        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
        writeln!(errors_file, "[{}] {}", timestamp, error_msg).map_err(|e| {
            SpiralError::SystemError(format!("Failed to write to errors log: {}", e))
        })?;

        Ok(())
    }

    /// Log a warning
    pub async fn log_warning(&mut self, phase: &str, warning: &str) -> Result<()> {
        let warning_msg = format!("WARNING in {}: {}", phase, warning);

        // Log to main log
        self.log_to_main(&warning_msg)?;

        // Log to phase log
        self.log_to_phase(phase, &warning_msg).await?;

        Ok(())
    }

    /// Log validation results
    pub async fn log_validation_results(&mut self, phase: &str, results: &str) -> Result<()> {
        // Create validation results file
        let validation_path = self.log_dir.join("validation_results.log");
        let mut validation_file = fs::File::create(validation_path).map_err(|e| {
            SpiralError::SystemError(format!("Failed to create validation log: {}", e))
        })?;

        let header = format!(
            "=== Validation Results ===\n\
             Phase: {}\n\
             Timestamp: {}\n\
             ==========================\n\n",
            phase,
            Utc::now().to_rfc3339()
        );

        validation_file.write_all(header.as_bytes()).map_err(|e| {
            SpiralError::SystemError(format!("Failed to write validation header: {}", e))
        })?;
        validation_file.write_all(results.as_bytes()).map_err(|e| {
            SpiralError::SystemError(format!("Failed to write validation results: {}", e))
        })?;
        validation_file.flush().map_err(|e| {
            SpiralError::SystemError(format!("Failed to flush validation log: {}", e))
        })?;

        // Also log summary to main
        self.log_to_main(&format!("Validation completed for phase: {}", phase))?;

        Ok(())
    }

    /// Create a summary file at the end of the update
    pub async fn create_summary(
        &mut self,
        success: bool,
        message: &str,
        duration: std::time::Duration,
    ) -> Result<()> {
        let summary_path = self.log_dir.join("summary.json");

        let summary = serde_json::json!({
            "update_id": self.update_id,
            "codename": self.codename,
            "success": success,
            "message": message,
            "duration_seconds": duration.as_secs(),
            "completed_at": Utc::now().to_rfc3339(),
            "log_directory": self.log_dir.display().to_string(),
        });

        let summary_str = serde_json::to_string_pretty(&summary)
            .map_err(|e| SpiralError::SystemError(format!("Failed to serialize summary: {}", e)))?;
        fs::write(summary_path, summary_str)
            .map_err(|e| SpiralError::SystemError(format!("Failed to write summary: {}", e)))?;

        // Log completion to main log
        self.log_to_main(&format!(
            "\n=== Update Complete ===\n\
             Success: {}\n\
             Message: {}\n\
             Duration: {}s\n\
             ======================",
            success,
            message,
            duration.as_secs()
        ))?;

        Ok(())
    }

    /// Get the log directory path
    pub fn get_log_dir(&self) -> &Path {
        &self.log_dir
    }

    /// Archive logs for a completed update (compress to save space)
    pub async fn archive_logs(&self) -> Result<PathBuf> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::fs::File;
        use tar::Builder;

        let archive_name = format!(
            "{}.tar.gz",
            self.log_dir.file_name().unwrap().to_string_lossy()
        );
        let archive_path = PathBuf::from(UPDATE_LOGS_DIR)
            .join("archives")
            .join(archive_name);

        // Create archives directory if it doesn't exist
        fs::create_dir_all(archive_path.parent().unwrap()).map_err(|e| {
            SpiralError::SystemError(format!("Failed to create archives directory: {}", e))
        })?;

        // Create tar.gz archive
        let tar_gz = File::create(&archive_path).map_err(|e| {
            SpiralError::SystemError(format!("Failed to create archive file: {}", e))
        })?;
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = Builder::new(enc);

        // Add all files from log directory to archive
        tar.append_dir_all(self.log_dir.file_name().unwrap(), &self.log_dir)
            .map_err(|e| {
                SpiralError::SystemError(format!("Failed to add files to archive: {}", e))
            })?;

        tar.finish()
            .map_err(|e| SpiralError::SystemError(format!("Failed to finish archive: {}", e)))?;

        info!(
            "[StructuredLogger] Archived logs to: {}",
            archive_path.display()
        );

        // Optionally remove original log directory to save space
        // fs::remove_dir_all(&self.log_dir)?;

        Ok(archive_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(
            StructuredLogger::sanitize_filename("test-update_123"),
            "test-update_123"
        );
        assert_eq!(
            StructuredLogger::sanitize_filename("test/update:123"),
            "test_update_123"
        );
        assert_eq!(
            StructuredLogger::sanitize_filename("a".repeat(100).as_str()).len(),
            50
        );
    }

    #[tokio::test]
    async fn test_logger_creation() {
        let logger = StructuredLogger::new("test-123".to_string(), "test-update".to_string());

        assert!(logger.is_ok());
        let logger = logger.unwrap();
        assert!(logger.get_log_dir().exists());
    }
}
