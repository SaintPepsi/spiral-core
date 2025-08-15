use serenity::model::Colour;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// 🏗️ ARCHITECTURE DECISION: Dynamic agent registry for role management
/// Why: Agents self-register their Discord personas, no hardcoding
/// Alternative: Static list in roles.rs (rejected: violates DRY, hard to maintain)
/// Trade-off: More complex but extensible and maintainable
#[derive(Debug, Clone)]
pub struct AgentPersona {
    pub name: String,
    pub emoji: &'static str,
    pub color: Colour,
    pub description: String,
    pub available: bool,
}

/// Central registry for all agent personas
pub struct AgentRegistry {
    personas: Arc<RwLock<HashMap<String, AgentPersona>>>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            personas: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new agent persona
    /// 🔍 AUDIT CHECKPOINT: Validates single-word Spiral names
    pub async fn register_agent(&self, persona: AgentPersona) -> Result<(), String> {
        // Validate that Spiral roles are single-word
        if persona.name.starts_with("Spiral") && persona.name.contains(' ') {
            return Err(format!(
                "Spiral agent names must be single words: {}",
                persona.name
            ));
        }

        let mut personas = self.personas.write().await;

        if personas.contains_key(&persona.name) {
            return Err(format!("Agent {} already registered", persona.name));
        }

        info!(
            "[AgentRegistry] Registered agent: {} (available: {})",
            persona.name, persona.available
        );

        personas.insert(persona.name.clone(), persona);
        Ok(())
    }

    /// Unregister an agent (when it becomes unavailable)
    pub async fn unregister_agent(&self, name: &str) -> Result<(), String> {
        let mut personas = self.personas.write().await;

        if personas.remove(name).is_some() {
            info!("[AgentRegistry] Unregistered agent: {}", name);
            Ok(())
        } else {
            Err(format!("Agent {name} not found"))
        }
    }

    /// Update agent availability
    pub async fn set_agent_availability(&self, name: &str, available: bool) -> Result<(), String> {
        let mut personas = self.personas.write().await;

        if let Some(persona) = personas.get_mut(name) {
            persona.available = available;
            info!(
                "[AgentRegistry] Updated agent {} availability to {}",
                name, available
            );
            Ok(())
        } else {
            Err(format!("Agent {name} not found"))
        }
    }

    /// Get all registered agents
    pub async fn get_all_agents(&self) -> Vec<AgentPersona> {
        let personas = self.personas.read().await;
        personas.values().cloned().collect()
    }

    /// Get only available agents
    pub async fn get_available_agents(&self) -> Vec<AgentPersona> {
        let personas = self.personas.read().await;
        personas.values().filter(|p| p.available).cloned().collect()
    }

    /// Get reserved role names (all registered agent names)
    pub async fn get_reserved_names(&self) -> Vec<String> {
        let personas = self.personas.read().await;
        let mut names: Vec<String> = personas.keys().map(|k| k.to_lowercase()).collect();

        // Add some always-reserved names
        names.push("spiral".to_lowercase());
        names.push("spiralconstellation".to_lowercase());

        names
    }

    /// Check if a role name is reserved
    pub async fn is_reserved(&self, name: &str) -> bool {
        let reserved = self.get_reserved_names().await;
        reserved.contains(&name.to_lowercase())
    }
}

/// Global agent registry instance
/// 📐 SOLID: Single source of truth for agent personas
/// Using OnceCell for lazy initialization without external dependencies
use std::sync::OnceLock;

static AGENT_REGISTRY_INSTANCE: OnceLock<AgentRegistry> = OnceLock::new();

pub fn get_agent_registry() -> &'static AgentRegistry {
    AGENT_REGISTRY_INSTANCE.get_or_init(AgentRegistry::new)
}
