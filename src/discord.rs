// 🌌 SPIRAL DISCORD INTEGRATION: SpiralConstellation bot with dynamic agent personas
// ARCHITECTURE DECISION: Single bot with persona switching for different agent types
// Why: Simpler deployment, dynamic personality, maintains distinct agent identities

pub mod spiral_constellation_bot;
pub mod startup;

// Re-export main types for convenience
pub use spiral_constellation_bot::{
    SpiralConstellationBot, 
    SpiralConstellationBotRunner, 
    AgentPersona, 
    MessageContext
};
pub use startup::{start_discord_bots, SpiralDiscordStartup};
