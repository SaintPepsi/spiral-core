use anyhow::Result;
use spiral_core::{agents::AgentOrchestrator, api::ApiServer, config::Config, discord::DiscordBot};
use std::sync::Arc;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Spiral Discord Bot with API Server");

    let config = Config::load()?;

    let orchestrator = Arc::new(AgentOrchestrator::new(config.clone()).await?);
    let discord_bot = DiscordBot::new(config.discord.clone(), orchestrator.clone())?;
    let api_server = ApiServer::new(config.clone(), orchestrator.clone())?;

    tokio::select! {
        result = orchestrator.run() => {
            if let Err(e) = result {
                tracing::error!("Agent orchestrator failed: {}", e);
            }
        }
        result = discord_bot.run() => {
            if let Err(e) = result {
                tracing::error!("Discord bot failed: {}", e);
            }
        }
        result = api_server.run() => {
            if let Err(e) = result {
                tracing::error!("API server failed: {}", e);
            }
        }
    }

    Ok(())
}
