//! 📊 STATUS TRACKER: Tracks and updates implementation status
//!
//! This module manages the SELF_UPDATE_IMPLEMENTATION_STATUS.md file,
//! automatically updating checklist items as the system achieves milestones.

use crate::error::{Result, SpiralError};
use std::fs;
use tracing::{info, warn};

pub struct StatusTracker;

impl StatusTracker {
    /// Check and update the implementation status based on completed actions
    pub async fn update_status(update_type: UpdateType) -> Result<()> {
        let status_file = "docs/SELF_UPDATE_IMPLEMENTATION_STATUS.md";

        // Read current status
        let content = fs::read_to_string(status_file)
            .map_err(|e| SpiralError::SystemError(format!("Failed to read status file: {e}")))?;

        // Update based on type
        let updated_content = match update_type {
            UpdateType::SimpleUpdate => {
                Self::increment_counter(&content, "successful simple updates")
            }
            UpdateType::TestModification => {
                Self::increment_counter(&content, "successful test additions/modifications")
            }
            UpdateType::FeatureAddition => {
                Self::increment_counter(&content, "successful feature addition")
            }
            UpdateType::DataLossIncident => Self::mark_failed(&content, "Zero data loss incidents"),
        }?;

        // Write back if changed
        if content != updated_content {
            fs::write(status_file, updated_content).map_err(|e| {
                SpiralError::SystemError(format!("Failed to write status file: {e}"))
            })?;
            info!(
                "[StatusTracker] Updated implementation status for {:?}",
                update_type
            );
        }

        Ok(())
    }

    /// Increment counter in checklist item (e.g., "3/10 successful simple updates")
    fn increment_counter(content: &str, item_text: &str) -> Result<String> {
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        for line in &mut lines {
            if line.contains(item_text) && line.starts_with("- [ ]") {
                // Extract current count if present
                if let Some(count_match) = Self::extract_count(line, item_text) {
                    let (current, total) = count_match;
                    let new_count = current + 1;

                    if new_count >= total {
                        // Mark as complete
                        *line = line.replace("- [ ]", "- [x]");
                        info!("[StatusTracker] Completed: {}", item_text);
                    } else {
                        // Update counter
                        let old_text = format!("{current}/{total}");
                        let new_text = format!("{new_count}/{total}");
                        *line = line.replace(&old_text, &new_text);
                    }
                } else {
                    // No counter yet, add one
                    *line = line.replace(item_text, &format!("1/10 {item_text}"));
                }
                break;
            }
        }

        Ok(lines.join("\n"))
    }

    /// Mark an item as failed
    fn mark_failed(content: &str, item_text: &str) -> Result<String> {
        let updated = content.replace(
            &format!("- [x] {item_text}"),
            &format!("- [ ] {item_text} ❌ FAILED"),
        );

        if updated != content {
            warn!("[StatusTracker] Failed criterion: {}", item_text);
        }

        Ok(updated)
    }

    /// Extract current/total count from a line
    fn extract_count(line: &str, item_text: &str) -> Option<(u32, u32)> {
        // Look for pattern like "3/10 successful simple updates"
        let pattern = r"(\d+)/(\d+)\s+";
        let regex = regex::Regex::new(&format!("{}{}", pattern, regex::escape(item_text))).ok()?;

        let captures = regex.captures(line)?;
        let current = captures.get(1)?.as_str().parse().ok()?;
        let total = captures.get(2)?.as_str().parse().ok()?;

        Some((current, total))
    }

    /// Get current implementation progress
    pub async fn get_progress() -> Result<ImplementationProgress> {
        let status_file = "docs/SELF_UPDATE_IMPLEMENTATION_STATUS.md";
        let content = fs::read_to_string(status_file)
            .map_err(|e| SpiralError::SystemError(format!("Failed to read status file: {e}")))?;

        let mut progress = ImplementationProgress::default();

        // Parse checklist items
        for line in content.lines() {
            if line.contains("successful simple updates") {
                progress.simple_updates = Self::extract_count(line, "successful simple updates")
                    .map(|(c, _)| c)
                    .unwrap_or(0);
            } else if line.contains("successful test additions") {
                progress.test_modifications =
                    Self::extract_count(line, "successful test additions/modifications")
                        .map(|(c, _)| c)
                        .unwrap_or(0);
            } else if line.contains("successful feature addition") {
                progress.feature_additions =
                    Self::extract_count(line, "successful feature addition")
                        .map(|(c, _)| c)
                        .unwrap_or(0);
            } else if line.contains("Zero data loss") && line.starts_with("- [x]") {
                progress.zero_data_loss = true;
            }
        }

        Ok(progress)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UpdateType {
    SimpleUpdate,
    TestModification,
    FeatureAddition,
    DataLossIncident,
}

#[derive(Debug, Default)]
pub struct ImplementationProgress {
    pub simple_updates: u32,
    pub test_modifications: u32,
    pub feature_additions: u32,
    pub zero_data_loss: bool,
}

impl ImplementationProgress {
    /// Check if Phase 1 criteria are met
    pub fn is_phase_1_complete(&self) -> bool {
        self.simple_updates >= 10
            && self.test_modifications >= 3
            && self.feature_additions >= 1
            && self.zero_data_loss
    }
}
