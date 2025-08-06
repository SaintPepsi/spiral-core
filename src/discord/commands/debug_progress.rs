use super::CommandHandler;
use crate::discord::{
    spiral_constellation_bot::SpiralConstellationBot,
    self_update::{ProgressReporter, UpdatePhase},
};
use serenity::{model::channel::Message, prelude::Context};
use std::time::Duration;
use tracing::info;

/// Debug command for testing progress bar
pub struct DebugProgressCommand;

impl DebugProgressCommand {
    pub fn new() -> Self {
        Self
    }
    
    /// Demo the progress bar for testing
    async fn demo_progress_bar(&self, msg: &Message, ctx: &Context) -> String {
        info!("[DebugProgressCommand] Starting progress bar demo");
        
        // Create a progress reporter for demo
        let progress_reporter = ProgressReporter::new(
            "demo-progress".to_string(),
            Some(ctx.http.clone()),
            msg.channel_id,
            5, // 5 demo tasks
        );
        
        // Send initial progress message that we'll keep editing
        let initial_message = self.create_progress_message(&progress_reporter).await;
        let progress_msg = match msg.channel_id.say(&ctx.http, &initial_message).await {
            Ok(m) => m,
            Err(e) => return format!("Failed to send initial message: {e}"),
        };
        
        // Set the message ID so the reporter will edit this message
        progress_reporter.set_message_id(progress_msg.id).await;
        
        // Start reporting with background updates (as backup)
        progress_reporter.start_reporting(Duration::from_millis(500));
        
        // Reset the timer now that setup is complete and we're starting the actual demo
        progress_reporter.reset_timer().await;
        
        // Phase 1: Initializing
        progress_reporter.set_phase(UpdatePhase::Initializing).await;
        progress_reporter.set_status("Starting demo update...".to_string()).await;
        
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Phase 2: Preflight
        progress_reporter.set_phase(UpdatePhase::PreflightChecks).await;
        progress_reporter.set_status("Running preflight checks...".to_string()).await;
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Phase 3: Planning  
        progress_reporter.set_phase(UpdatePhase::Planning).await;
        progress_reporter.set_status("Creating implementation plan...".to_string()).await;
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Phase 4: Snapshot
        progress_reporter.set_phase(UpdatePhase::CreatingSnapshot).await;
        progress_reporter.set_status("Creating git snapshot...".to_string()).await;
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Phase 5: Implementing
        progress_reporter.set_phase(UpdatePhase::Implementing).await;
        progress_reporter.set_status("Implementing changes...".to_string()).await;
        
        // Simulate task completion (3 tasks during implementation)
        for i in 1..=3 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            progress_reporter.increment_tasks_completed().await;
            progress_reporter.set_status(format!("Implementing task {i} of 5...")).await;
        }
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Phase 6: Validating
        progress_reporter.set_phase(UpdatePhase::Validating).await;
        progress_reporter.set_status("Running validation pipeline...".to_string()).await;
        
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Complete remaining tasks during validation
        progress_reporter.increment_tasks_completed().await;
        progress_reporter.set_status("Validating implementation...".to_string()).await;
        
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        progress_reporter.increment_tasks_completed().await;
        progress_reporter.set_status("Final checks...".to_string()).await;
        
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Phase 7: Complete
        progress_reporter.set_phase(UpdatePhase::Complete).await;
        progress_reporter.set_status("Demo completed successfully!".to_string()).await;
        
        // Get the actual elapsed time before stopping
        let elapsed = {
            let progress = progress_reporter.progress.read().await;
            progress.started_at.elapsed()
        };
        
        // Stop reporting
        progress_reporter.stop().await;
        
        format!(
            "✅ **Progress Bar Demo Complete!**\nThe progress bar ran for {} showing various update phases in a single message.",
            ProgressReporter::format_duration(elapsed)
        )
    }
    
    /// Create the initial progress message
    async fn create_progress_message(&self, reporter: &ProgressReporter) -> String {
        reporter.format_progress_message().await
    }
}

impl CommandHandler for DebugProgressCommand {
    async fn handle(
        &self,
        content: &str,
        msg: &Message,
        ctx: &Context,
        _bot: &SpiralConstellationBot,
    ) -> Option<String> {
        // Check if this is the progress bar command
        if content.contains("progress") || content.contains("progressbar") {
            info!(
                "[DebugProgressCommand] Progress bar demo requested by {} ({})",
                msg.author.name,
                msg.author.id.get()
            );
            let result = self.demo_progress_bar(msg, ctx).await;
            Some(result)
        } else {
            None
        }
    }

    fn command_prefix(&self) -> &str {
        "!spiral debug progress"
    }

    fn description(&self) -> &str {
        "Demo the progress bar functionality for testing"
    }
}

impl Default for DebugProgressCommand {
    fn default() -> Self {
        Self::new()
    }
}