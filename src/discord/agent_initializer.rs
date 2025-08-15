use crate::discord::agent_registry::{get_agent_registry, AgentPersona};
use serenity::model::Colour;
use tracing::info;

/// 🏗️ ARCHITECTURE DECISION: Central agent initialization
/// Why: Single place to register all available agents
/// Alternative: Scattered registration (rejected: hard to track)
/// Trade-off: Need to update when new agents are added
pub async fn initialize_available_agents() -> Result<(), String> {
    info!("[AgentInitializer] Registering available agents");

    let registry = get_agent_registry();

    // Register Orchestrator (currently available)
    registry
        .register_agent(AgentPersona {
            name: "Orchestrator".to_string(),
            emoji: "🎭",
            color: Colour::from_rgb(147, 112, 219), // Medium Purple
            description: "Agent orchestration and coordination".to_string(),
            available: true,
        })
        .await?;

    info!("[AgentInitializer] Registered Orchestrator agent");

    // Register SpiralDev (the main development bot - currently active)
    registry
        .register_agent(AgentPersona {
            name: "SpiralDev".to_string(),
            emoji: "💻",
            color: Colour::from_rgb(0, 162, 232), // Blue
            description: "Code generation and development".to_string(),
            available: true, // This is the main bot that's actually running
        })
        .await?;

    info!("[AgentInitializer] Registered SpiralDev agent");

    // Note: Other agents (SpiralPM, SpiralQA, etc.) will be registered
    // when they are implemented. For now, we don't register them at all
    // to avoid confusion.

    // Verify registration succeeded
    let registered_count = registry.get_all_agents().await.len();
    info!(
        "[AgentInitializer] Agent registration complete. Total registered: {}",
        registered_count
    );

    if registered_count == 0 {
        return Err("Failed to register agents - registry is empty".to_string());
    }

    Ok(())
}
