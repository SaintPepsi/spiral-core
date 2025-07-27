use super::spiral_constellation_bot::{SpiralConstellationBot, SpiralConstellationBotRunner};
use crate::{
    agents::SoftwareDeveloperAgent,
    claude_code::ClaudeCodeClient,
    config::Config,
    Result, SpiralError,
};
use tracing::{info, warn};

/// 🌌 SPIRAL DISCORD STARTUP: Initialize and launch SpiralConstellation bot
/// ARCHITECTURE DECISION: Single bot with dynamic persona switching
/// Why: Simpler deployment, persona-based responses, maintains agent identity feel
pub struct SpiralDiscordStartup {
    config: Config,
    claude_client: ClaudeCodeClient,
}

impl SpiralDiscordStartup {
    pub fn new(config: Config, claude_client: ClaudeCodeClient) -> Self {
        Self {
            config,
            claude_client,
        }
    }

    /// 🎮 START DISCORD INTEGRATION: Launch SpiralConstellation bot with personas
    pub async fn start_discord_integration(&self) -> Result<()> {
        if self.config.discord.token.is_empty() {
            info!("Discord token not provided - Discord integration disabled");
            return Ok(());
        }

        info!("🌌 Starting SpiralConstellation bot with agent personas");
        self.start_constellation_bot().await
    }

    /// 🌌 CONSTELLATION BOT: Launch unified bot with persona switching
    async fn start_constellation_bot(&self) -> Result<()> {
        info!("Initializing SpiralConstellation bot...");

        // Create developer agent (currently the only implemented agent)
        let developer_agent = SoftwareDeveloperAgent::new(self.claude_client.clone());
        
        // Create constellation bot with persona system
        let constellation_bot = SpiralConstellationBot::new(
            developer_agent,
            self.claude_client.clone(),
        )?;

        info!("SpiralConstellation bot initialized with personas:");
        info!("  🚀 SpiralDev - Software Developer");
        info!("  📋 SpiralPM - Project Manager (coming soon)");  
        info!("  🔍 SpiralQA - Quality Assurance (coming soon)");
        info!("  🎯 SpiralDecide - Decision Maker (coming soon)");
        info!("  ✨ SpiralCreate - Creative Innovator (coming soon)");
        info!("  🧘 SpiralCoach - Process Coach (coming soon)");

        // Create and run bot
        let runner = SpiralConstellationBotRunner::new(
            constellation_bot,
            self.config.discord.token.clone(),
        );

        info!("🚀 Starting SpiralConstellation Discord bot...");
        runner.run().await?;

        Ok(())
    }


    /// 📊 VALIDATE DISCORD CONFIG: Check Discord configuration for issues
    pub fn validate_discord_config(&self) -> Result<()> {
        let discord_config = &self.config.discord;

        if discord_config.token.is_empty() {
            return Ok(()); // Discord disabled, nothing to validate
        }

        // Basic token validation
        if discord_config.token == "your-discord-token" {
            return Err(SpiralError::ConfigurationError(
                "Discord token appears to be a placeholder".to_string(),
            ));
        }

        if discord_config.token.len() < 50 {
            return Err(SpiralError::ConfigurationError(
                "Discord token appears to be too short".to_string(),
            ));
        }

        info!("Discord token validation passed for SpiralConstellation bot");
        Ok(())
    }
}

/// 🎯 CONVENIENCE FUNCTION: Quick start for Discord integration
pub async fn start_discord_bots(config: Config, claude_client: ClaudeCodeClient) -> Result<()> {
    let startup = SpiralDiscordStartup::new(config, claude_client);

    // Validate configuration first
    startup.validate_discord_config()?;

    // Start Discord integration
    startup.start_discord_integration().await
}

