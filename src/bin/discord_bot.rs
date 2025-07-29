use anyhow::Result;
use spiral_core::{
    agents::SoftwareDeveloperAgent,
    claude_code::ClaudeCodeClient,
    config::Config,
    discord::{SpiralConstellationBot, SpiralConstellationBotRunner},
};
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Spiral Constellation Discord Bot");

    let config = Config::load()?;

    // Create Claude Code client
    let claude_client = ClaudeCodeClient::new(config.claude_code.clone()).await?;

    // Create developer agent
    let developer_agent = SoftwareDeveloperAgent::new(claude_client.clone());

    // Create constellation bot
    let constellation_bot =
        SpiralConstellationBot::new(developer_agent, claude_client, config.discord.clone()).await?;

    // Create bot runner
    let bot_runner =
        SpiralConstellationBotRunner::new(constellation_bot, config.discord.token.clone());

    info!("🌌 Starting SpiralConstellation Discord Bot...");

    // Run the bot
    bot_runner.run().await?;

    Ok(())
}
