use super::CommandHandler;
use crate::discord::{
    spiral_constellation_bot::SpiralConstellationBot,
    self_update::{ProgressReporter, UpdatePhase},
};
use serenity::{model::channel::Message, prelude::Context};
use std::{sync::Arc, time::Duration};
use tracing::{error, info};

/// Debug command for testing progress bar
pub struct DebugProgressCommand;

impl DebugProgressCommand {
    pub fn new() -> Self {
        Self
    }
    
    /// Demo the progress bar for testing
    async fn demo_progress_bar(&self, msg: &Message, ctx: &Context) -> String {
        info!("[DebugProgressCommand] Starting progress bar demo");
        
        // Send initial message
        if let Err(e) = msg.channel_id.say(&ctx.http, "🚀 Starting progress bar demo (15 seconds)...").await {
            return format!("Failed to send message: {}", e);
        }
        
        // Create a progress reporter for demo
        let progress_reporter = ProgressReporter::new(
            "demo-progress".to_string(),
            Some(ctx.http.clone()),
            msg.channel_id,
            5, // 5 demo tasks
        );
        
        // Start reporting with fast updates (every 3 seconds for demo)
        // This ensures we see updates since the min time between reports is 5 seconds
        progress_reporter.start_reporting(Duration::from_secs(3));
        
        // Wait a moment for the reporter to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Phase 1: Initializing
        progress_reporter.set_phase(UpdatePhase::Initializing).await;
        progress_reporter.set_status("Starting demo update...".to_string()).await;
        
        // Force an immediate update by manually sending
        Self::send_progress_update(&progress_reporter, &ctx.http, msg.channel_id).await;
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Phase 2: Preflight
        progress_reporter.set_phase(UpdatePhase::PreflightChecks).await;
        progress_reporter.set_status("Running preflight checks...".to_string()).await;
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Phase 3: Planning  
        progress_reporter.set_phase(UpdatePhase::Planning).await;
        progress_reporter.set_status("Creating implementation plan...".to_string()).await;
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Phase 4: Snapshot (this should trigger an auto-update since >5 seconds passed)
        progress_reporter.set_phase(UpdatePhase::CreatingSnapshot).await;
        progress_reporter.set_status("Creating git snapshot...".to_string()).await;
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Phase 5: Implementing
        progress_reporter.set_phase(UpdatePhase::Implementing).await;
        progress_reporter.set_status("Implementing changes...".to_string()).await;
        
        // Simulate task completion
        for i in 1..=3 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            progress_reporter.increment_tasks_completed().await;
            progress_reporter.set_status(format!("Implementing task {} of 5...", i)).await;
        }
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Phase 6: Validating (should trigger another auto-update)
        progress_reporter.set_phase(UpdatePhase::Validating).await;
        progress_reporter.set_status("Running validation pipeline...".to_string()).await;
        
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Phase 7: Complete
        progress_reporter.set_phase(UpdatePhase::Complete).await;
        progress_reporter.set_status("Demo completed successfully!".to_string()).await;
        
        // Force final update
        Self::send_progress_update(&progress_reporter, &ctx.http, msg.channel_id).await;
        
        // Stop reporting
        progress_reporter.stop().await;
        
        "✅ **Progress Bar Demo Complete!**\nThe progress bar ran for ~15 seconds showing various update phases.\n\nYou should have seen 3-4 progress updates during the demo.".to_string()
    }
    
    /// Manually send a progress update (for demo purposes)
    async fn send_progress_update(
        reporter: &ProgressReporter, 
        http: &Arc<serenity::http::Http>,
        channel_id: serenity::model::id::ChannelId
    ) {
        // Access the progress data
        let progress = reporter.progress.read().await;
        let elapsed = progress.started_at.elapsed();
        
        let message = format!(
            "{} **{}** ({}%)\n{}\n⏱️ {} | 📊 {}/{} tasks | 💬 {}",
            progress.current_phase.emoji(),
            progress.current_phase.name(),
            progress.percent_complete,
            ProgressReporter::create_progress_bar(progress.percent_complete),
            ProgressReporter::format_duration(elapsed),
            progress.tasks_completed,
            progress.total_tasks,
            progress.status_message
        );
        
        if let Err(e) = channel_id.say(http, &message).await {
            error!("[DebugProgressCommand] Failed to send progress update: {}", e);
        }
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