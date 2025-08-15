use super::{
    agent_initializer::initialize_available_agents,
    spiral_constellation_bot::{SpiralConstellationBot, SpiralConstellationBotRunner},
};
use crate::{
    agents::{AgentOrchestrator, SoftwareDeveloperAgent},
    claude_code::ClaudeCodeClient,
    config::Config,
    Result, SpiralError,
};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

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
        debug!("[Discord Startup] Checking Discord configuration...");

        if self.config.discord.token.is_empty() {
            warn!("[Discord Startup] Discord token not provided - Discord integration disabled");
            return Ok(());
        }

        debug!(
            "[Discord Startup] Discord token found, length: {}",
            self.config.discord.token.len()
        );
        info!("[Discord Startup] 🌌 Starting SpiralConstellation bot with agent personas");

        match self.start_constellation_bot().await {
            Ok(()) => {
                info!("[Discord Startup] ✅ Discord bot started successfully");
                Ok(())
            }
            Err(e) => {
                error!("[Discord Startup] ❌ Failed to start Discord bot: {}", e);
                Err(e)
            }
        }
    }

    /// 🌌 CONSTELLATION BOT: Launch unified bot with persona switching
    async fn start_constellation_bot(&self) -> Result<()> {
        info!("[Discord Startup] Initializing SpiralConstellation bot...");

        // Initialize agent registry
        debug!("[Discord Startup] Initializing agent registry...");
        if let Err(e) = initialize_available_agents().await {
            error!(
                "[Discord Startup] Failed to initialize agent registry: {}",
                e
            );
            // Continue anyway - roles just won't be available
        }

        debug!("[Discord Startup] Creating developer agent...");

        // Create developer agent (currently the only implemented agent)
        let developer_agent = SoftwareDeveloperAgent::new(self.claude_client.clone());
        debug!("[Discord Startup] Developer agent created successfully");

        // Create constellation bot with persona system
        debug!("[Discord Startup] Creating constellation bot with persona system...");
        let constellation_bot = match SpiralConstellationBot::new(
            developer_agent,
            self.claude_client.clone(),
            self.config.discord.clone(),
        )
        .await
        {
            Ok(bot) => {
                debug!("[Discord Startup] Constellation bot created successfully");
                bot
            }
            Err(e) => {
                error!(
                    "[Discord Startup] Failed to create constellation bot: {}",
                    e
                );
                return Err(e);
            }
        };

        // 🏗️ ARCHITECTURE DECISION: Dynamic agent listing from registry
        // Why: Single source of truth for available agents
        // Alternative: Hardcoded list (rejected: violates DRY, maintenance burden)
        info!("[Discord Startup] SpiralConstellation bot initialized");
        info!("[Discord Startup] Agents will register themselves dynamically");

        // Create and run bot
        debug!("[Discord Startup] Creating bot runner...");
        let runner =
            SpiralConstellationBotRunner::new(constellation_bot, self.config.discord.token.clone());
        debug!("[Discord Startup] Bot runner created successfully");

        info!("[Discord Startup] 🚀 Starting SpiralConstellation Discord bot...");
        info!("[Discord Startup] Attempting to connect to Discord API...");

        match runner.run().await {
            Ok(()) => {
                info!("[Discord Startup] Discord bot shutdown gracefully");
                Ok(())
            }
            Err(e) => {
                error!("[Discord Startup] Discord bot encountered error: {}", e);
                Err(e)
            }
        }
    }

    /// 📊 VALIDATE DISCORD CONFIG: Check Discord configuration for issues
    pub fn validate_discord_config(&self) -> Result<()> {
        let discord_config = &self.config.discord;

        if discord_config.token.is_empty() {
            return Ok(()); // Discord disabled, nothing to validate
        }

        // Basic token validation
        debug!("[Discord Startup] Validating Discord token format...");

        if discord_config.token == "your-discord-token" {
            error!("[Discord Startup] Token validation failed: placeholder token detected");
            return Err(SpiralError::ConfigurationError(
                "Discord token appears to be a placeholder".to_string(),
            ));
        }

        if discord_config.token.len() < 50 {
            error!(
                "[Discord Startup] Token validation failed: token too short ({})",
                discord_config.token.len()
            );
            return Err(SpiralError::ConfigurationError(
                "Discord token appears to be too short".to_string(),
            ));
        }

        info!(
            "[Discord Startup] Discord token validation passed (length: {})",
            discord_config.token.len()
        );
        Ok(())
    }
}

/// 🎯 CONVENIENCE FUNCTION: Quick start for Discord integration (standalone mode)
pub async fn start_discord_bots(config: Config, claude_client: ClaudeCodeClient) -> Result<()> {
    info!("[Discord Startup] Starting Discord bots in standalone mode");
    let startup = SpiralDiscordStartup::new(config, claude_client);

    // Validate configuration first
    debug!("[Discord Startup] Validating Discord configuration...");
    startup.validate_discord_config()?;
    debug!("[Discord Startup] Configuration validation successful");

    // Start Discord integration
    startup.start_discord_integration().await
}

/// 🎛️ ORCHESTRATOR INTEGRATION: Start Discord with full orchestration capabilities
pub async fn start_discord_with_orchestrator(
    config: Config,
    orchestrator: Arc<AgentOrchestrator>,
) -> Result<()> {
    info!("[Discord Startup] Starting Discord with orchestrator integration");
    debug!("[Discord Startup] Checking Discord token...");

    if config.discord.token.is_empty() {
        warn!("[Discord Startup] Discord token not provided - Discord integration disabled");
        return Ok(());
    }

    debug!(
        "[Discord Startup] Discord token found, length: {}",
        config.discord.token.len()
    );
    info!("[Discord Startup] 🌌 Starting SpiralConstellation bot with orchestrator integration");

    // Create constellation bot with orchestrator
    debug!("[Discord Startup] Creating constellation bot with orchestrator...");
    let constellation_bot =
        match SpiralConstellationBot::new_with_orchestrator(orchestrator, config.discord.clone())
            .await
        {
            Ok(bot) => {
                debug!("[Discord Startup] Constellation bot created successfully");
                bot
            }
            Err(e) => {
                error!(
                    "[Discord Startup] Failed to create constellation bot: {}",
                    e
                );
                return Err(e);
            }
        };

    // 🏗️ ARCHITECTURE DECISION: Dynamic agent listing from registry
    // Why: Single source of truth for available agents
    // Alternative: Hardcoded list (rejected: violates DRY, maintenance burden)
    info!("[Discord Startup] SpiralConstellation bot initialized with orchestrator");
    info!("[Discord Startup] Active agents will register themselves dynamically");

    // Create and run bot
    debug!("[Discord Startup] Creating bot runner...");
    let bot_runner = SpiralConstellationBotRunner::new(constellation_bot, config.discord.token);
    debug!("[Discord Startup] Bot runner created, starting bot...");

    info!("[Discord Startup] Attempting to connect to Discord API...");
    match bot_runner.run().await {
        Ok(()) => {
            info!("[Discord Startup] Discord bot shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            error!("[Discord Startup] Discord bot failed: {}", e);
            Err(e)
        }
    }
}
