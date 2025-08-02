//! Agent-specific session management
//!
//! This module extends the base session management with agent-specific tracking.
//! It follows SOLID principles by extending rather than duplicating functionality.

use super::{Session as BaseSession, SessionConfig as BaseSessionConfig, SessionManager};
use crate::error::Result;
use crate::models::AgentType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Agent-specific session that extends the base session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    /// Base session information
    #[serde(flatten)]
    pub base: BaseSession,

    /// Type of agent handling this session
    pub agent_type: AgentType,

    /// Agent-specific metadata
    pub agent_metadata: HashMap<String, String>,
}

/// Agent session manager that tracks agent assignments
pub struct AgentSessionManager<S: super::SessionStore> {
    /// Underlying session manager
    base_manager: SessionManager<S>,

    /// Track which agents are handling which sessions
    agent_assignments: tokio::sync::RwLock<HashMap<Uuid, AgentType>>,
}

impl<S: super::SessionStore> AgentSessionManager<S> {
    /// Create new agent session manager
    pub fn new(store: S, config: BaseSessionConfig) -> Self {
        Self {
            base_manager: SessionManager::new(store, config),
            agent_assignments: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Create a session assigned to a specific agent
    pub async fn create_agent_session(
        &self,
        user_id: String,
        agent_type: AgentType,
    ) -> Result<AgentSession> {
        // Create base session
        let base_session = self.base_manager.create_session(user_id).await?;

        // Track agent assignment
        let mut assignments = self.agent_assignments.write().await;
        assignments.insert(base_session.id, agent_type.clone());

        // Return agent session
        Ok(AgentSession {
            base: base_session,
            agent_type,
            agent_metadata: HashMap::new(),
        })
    }

    /// Get agent assignment for a session
    pub async fn get_agent_for_session(&self, session_id: &Uuid) -> Option<AgentType> {
        let assignments = self.agent_assignments.read().await;
        assignments.get(session_id).cloned()
    }

    /// Get all sessions for a specific agent type
    pub async fn get_sessions_by_agent(&self, agent_type: &AgentType) -> Vec<Uuid> {
        let assignments = self.agent_assignments.read().await;
        assignments
            .iter()
            .filter(|(_, a)| *a == agent_type)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get statistics by agent type
    pub async fn get_agent_statistics(&self) -> HashMap<AgentType, usize> {
        let assignments = self.agent_assignments.read().await;
        let mut stats = HashMap::new();

        for agent_type in assignments.values() {
            *stats.entry(agent_type.clone()).or_insert(0) += 1;
        }

        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::InMemorySessionStore;
    use chrono::Duration;

    async fn create_test_manager() -> AgentSessionManager<InMemorySessionStore> {
        let store = InMemorySessionStore::new();
        let config = BaseSessionConfig {
            max_duration: Duration::hours(1),
            max_concurrent: 10,
            auto_extend: true,
            extend_duration: Duration::minutes(30),
        };
        AgentSessionManager::new(store, config)
    }

    #[tokio::test]
    async fn test_create_agent_session() {
        let manager = create_test_manager().await;
        let session = manager
            .create_agent_session("user1".to_string(), AgentType::SoftwareDeveloper)
            .await
            .unwrap();

        assert_eq!(session.base.user_id, "user1");
        assert_eq!(session.agent_type, AgentType::SoftwareDeveloper);
    }

    #[tokio::test]
    async fn test_agent_assignment_tracking() {
        let manager = create_test_manager().await;

        let session1 = manager
            .create_agent_session("user1".to_string(), AgentType::SoftwareDeveloper)
            .await
            .unwrap();

        let session2 = manager
            .create_agent_session("user2".to_string(), AgentType::ProjectManager)
            .await
            .unwrap();

        assert_eq!(
            manager.get_agent_for_session(&session1.base.id).await,
            Some(AgentType::SoftwareDeveloper)
        );

        assert_eq!(
            manager.get_agent_for_session(&session2.base.id).await,
            Some(AgentType::ProjectManager)
        );
    }

    #[tokio::test]
    async fn test_get_sessions_by_agent() {
        let manager = create_test_manager().await;

        // Create multiple sessions
        let dev_session1 = manager
            .create_agent_session("user1".to_string(), AgentType::SoftwareDeveloper)
            .await
            .unwrap();

        let _pm_session = manager
            .create_agent_session("user2".to_string(), AgentType::ProjectManager)
            .await
            .unwrap();

        let dev_session2 = manager
            .create_agent_session("user3".to_string(), AgentType::SoftwareDeveloper)
            .await
            .unwrap();

        let dev_sessions = manager
            .get_sessions_by_agent(&AgentType::SoftwareDeveloper)
            .await;
        assert_eq!(dev_sessions.len(), 2);
        assert!(dev_sessions.contains(&dev_session1.base.id));
        assert!(dev_sessions.contains(&dev_session2.base.id));
    }

    #[tokio::test]
    async fn test_agent_statistics() {
        let manager = create_test_manager().await;

        // Create sessions with different agents
        manager
            .create_agent_session("user1".to_string(), AgentType::SoftwareDeveloper)
            .await
            .unwrap();
        manager
            .create_agent_session("user2".to_string(), AgentType::SoftwareDeveloper)
            .await
            .unwrap();
        manager
            .create_agent_session("user3".to_string(), AgentType::ProjectManager)
            .await
            .unwrap();

        let stats = manager.get_agent_statistics().await;
        assert_eq!(stats.get(&AgentType::SoftwareDeveloper), Some(&2));
        assert_eq!(stats.get(&AgentType::ProjectManager), Some(&1));
    }
}
