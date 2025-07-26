use anyhow::Result;
use spiral_core::{
    agents::AgentOrchestrator,
    api::ApiServer,
    config::Config,
};
use std::sync::Arc;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting Spiral Core Agent Orchestration System");

    let config = Config::load()?;
    
    let orchestrator = Arc::new(AgentOrchestrator::new(config.clone()).await?);
    let api_server = ApiServer::new(config.clone(), orchestrator.clone())?;

    tokio::select! {
        result = orchestrator.run() => {
            if let Err(e) = result {
                tracing::error!("Agent orchestrator failed: {}", e);
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