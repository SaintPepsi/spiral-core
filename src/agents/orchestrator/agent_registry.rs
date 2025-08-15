use crate::{
    agents::{Agent, AgentStatus},
    models::AgentType,
    Result, SpiralError,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// 🏗️ ARCHITECTURE DECISION: Separated AgentRegistry Service
/// Why: Single Responsibility - manages agent registration and lookup
/// Alternative: Keep in orchestrator (rejected: violates SRP)
/// Benefits: Can add dynamic agent loading, plugin architecture without touching orchestrator
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<AgentType, Arc<dyn Agent>>>>,
    statuses: Arc<RwLock<HashMap<AgentType, AgentStatus>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            statuses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an agent with the registry
    pub async fn register(&self, agent: Arc<dyn Agent>) -> Result<()> {
        let agent_type = agent.agent_type();
        let agent_name = agent.name();

        // Check if already registered
        {
            let agents = self.agents.read().await;
            if agents.contains_key(&agent_type) {
                warn!("Agent {:?} already registered", agent_type);
                return Err(SpiralError::Agent {
                    message: format!("Agent {:?} already registered", agent_type),
                });
            }
        }

        // Register agent and its status
        let mut agents = self.agents.write().await;
        let mut statuses = self.statuses.write().await;

        statuses.insert(agent_type.clone(), AgentStatus::new(agent_type.clone()));
        agents.insert(agent_type.clone(), agent);

        info!("Registered agent: {} ({:?})", agent_name, agent_type);
        Ok(())
    }

    /// Unregister an agent
    pub async fn unregister(&self, agent_type: &AgentType) -> Result<()> {
        let mut agents = self.agents.write().await;
        let mut statuses = self.statuses.write().await;

        agents.remove(agent_type);
        statuses.remove(agent_type);

        info!("Unregistered agent: {:?}", agent_type);
        Ok(())
    }

    /// Get an agent by type
    pub async fn get(&self, agent_type: &AgentType) -> Option<Arc<dyn Agent>> {
        let agents = self.agents.read().await;
        agents.get(agent_type).cloned()
    }

    /// Get all registered agents
    pub async fn get_all(&self) -> Vec<Arc<dyn Agent>> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }

    /// Get agent status
    pub async fn get_status(&self, agent_type: &AgentType) -> Option<AgentStatus> {
        let statuses = self.statuses.read().await;
        statuses.get(agent_type).cloned()
    }

    /// Update agent status
    pub async fn update_status(&self, agent_type: &AgentType, status: AgentStatus) {
        let mut statuses = self.statuses.write().await;
        statuses.insert(agent_type.clone(), status);
        debug!("Updated status for agent: {:?}", agent_type);
    }

    /// Get all agent statuses
    pub async fn get_all_statuses(&self) -> HashMap<AgentType, AgentStatus> {
        let statuses = self.statuses.read().await;
        statuses.clone()
    }

    /// Check if an agent type is registered
    pub async fn is_registered(&self, agent_type: &AgentType) -> bool {
        let agents = self.agents.read().await;
        agents.contains_key(agent_type)
    }

    /// Get count of registered agents
    pub async fn count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.len()
    }
}

/// 🏗️ ARCHITECTURE DECISION: AgentFactory for dynamic agent creation
/// Why: Decouples agent instantiation from orchestrator
/// Alternative: Hardcode in orchestrator (rejected: tight coupling)
pub struct AgentFactory;

impl AgentFactory {
    /// Create an agent based on type and configuration
    /// This is where we'd add plugin loading logic in the future
    pub async fn create_agent(
        agent_type: AgentType,
        claude_client: crate::claude_code::ClaudeCodeClient,
    ) -> Result<Arc<dyn Agent>> {
        match agent_type {
            AgentType::SoftwareDeveloper => {
                let agent = crate::agents::SoftwareDeveloperAgent::new(claude_client);
                Ok(Arc::new(agent) as Arc<dyn Agent>)
            }
            AgentType::ProjectManager => {
                let agent = crate::agents::ProjectManagerAgent::new(Some(claude_client));
                Ok(Arc::new(agent) as Arc<dyn Agent>)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::SoftwareDeveloperAgent;
    use crate::claude_code::ClaudeCodeClient;
    use crate::config::ClaudeCodeConfig;

    #[tokio::test]
    async fn test_agent_registration() {
        let registry = AgentRegistry::new();

        // Create a mock agent with test config
        let config = ClaudeCodeConfig {
            claude_binary_path: None,
            working_directory: None,
            timeout_seconds: 30,
            permission_mode: "permissive".to_string(),
            allowed_tools: vec![],
            workspace_cleanup_after_hours: 24,
            max_workspace_size_mb: 100,
        };
        let claude_client = ClaudeCodeClient::new(config).await.unwrap();
        let agent = Arc::new(SoftwareDeveloperAgent::new(claude_client));

        // Register agent
        registry
            .register(agent.clone() as Arc<dyn Agent>)
            .await
            .unwrap();
        assert_eq!(registry.count().await, 1);

        // Get agent
        let retrieved = registry.get(&AgentType::SoftwareDeveloper).await;
        assert!(retrieved.is_some());

        // Try to register again (should fail)
        let result = registry.register(agent as Arc<dyn Agent>).await;
        assert!(result.is_err());
    }
}
