use crate::{
    agents::AgentStatus,
    models::{AgentType, TaskStatus},
    Result, SpiralError,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// 🏗️ ARCHITECTURE DECISION: Separated StatusManager Service
/// Why: Single Responsibility - manages agent and task status tracking
/// Alternative: Keep in orchestrator (rejected: violates SRP)
/// Benefits: Centralized status management, can add metrics/monitoring without touching orchestrator
#[derive(Clone)]
pub struct StatusManager {
    agent_statuses: Arc<RwLock<HashMap<AgentType, AgentStatus>>>,
    task_statuses: Arc<RwLock<HashMap<String, TaskStatus>>>,
}

impl StatusManager {
    pub fn new() -> Self {
        Self {
            agent_statuses: Arc::new(RwLock::new(HashMap::new())),
            task_statuses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize agent status
    pub async fn initialize_agent_status(&self, agent_type: AgentType) {
        let mut statuses = self.agent_statuses.write().await;
        statuses.insert(agent_type.clone(), AgentStatus::new(agent_type.clone()));
        debug!("Initialized status for agent: {:?}", agent_type);
    }

    /// Update agent status
    pub async fn update_agent_status(&self, agent_type: &AgentType, status: AgentStatus) {
        let mut statuses = self.agent_statuses.write().await;
        statuses.insert(agent_type.clone(), status);
        debug!("Updated status for agent: {:?}", agent_type);
    }

    /// Get agent status
    pub async fn get_agent_status(&self, agent_type: &AgentType) -> Option<AgentStatus> {
        let statuses = self.agent_statuses.read().await;
        statuses.get(agent_type).cloned()
    }

    /// Get all agent statuses
    pub async fn get_all_agent_statuses(&self) -> HashMap<AgentType, AgentStatus> {
        let statuses = self.agent_statuses.read().await;
        statuses.clone()
    }

    /// Mark agent as busy
    pub async fn mark_agent_busy(&self, agent_type: &AgentType, task_id: String) -> Result<()> {
        let mut statuses = self.agent_statuses.write().await;
        if let Some(status) = statuses.get_mut(agent_type) {
            status.start_task(task_id);
            Ok(())
        } else {
            Err(SpiralError::Agent {
                message: format!("Agent {:?} not found", agent_type),
            })
        }
    }

    /// Mark agent as idle
    pub async fn mark_agent_idle(&self, agent_type: &AgentType) -> Result<()> {
        let mut statuses = self.agent_statuses.write().await;
        if let Some(status) = statuses.get_mut(agent_type) {
            status.is_busy = false;
            Ok(())
        } else {
            Err(SpiralError::Agent {
                message: format!("Agent {:?} not found", agent_type),
            })
        }
    }

    /// Increment agent task completion count
    pub async fn increment_agent_completed(
        &self,
        agent_type: &AgentType,
        execution_time: f64,
    ) -> Result<()> {
        let mut statuses = self.agent_statuses.write().await;
        if let Some(status) = statuses.get_mut(agent_type) {
            status.complete_task(execution_time);
            Ok(())
        } else {
            Err(SpiralError::Agent {
                message: format!("Agent {:?} not found", agent_type),
            })
        }
    }

    /// Increment agent task failure count
    pub async fn increment_agent_failed(&self, agent_type: &AgentType) -> Result<()> {
        let mut statuses = self.agent_statuses.write().await;
        if let Some(status) = statuses.get_mut(agent_type) {
            status.fail_task();
            Ok(())
        } else {
            Err(SpiralError::Agent {
                message: format!("Agent {:?} not found", agent_type),
            })
        }
    }

    /// Update task status
    pub async fn update_task_status(&self, task_id: &str, status: TaskStatus) {
        let mut statuses = self.task_statuses.write().await;
        statuses.insert(task_id.to_string(), status.clone());
        debug!("Updated task {} status to {:?}", task_id, status);
    }

    /// Get task status
    pub async fn get_task_status(&self, task_id: &str) -> Option<TaskStatus> {
        let statuses = self.task_statuses.read().await;
        statuses.get(task_id).cloned()
    }

    /// Remove task status (for cleanup)
    pub async fn remove_task_status(&self, task_id: &str) {
        let mut statuses = self.task_statuses.write().await;
        statuses.remove(task_id);
        debug!("Removed status for task: {}", task_id);
    }

    /// Get status statistics
    pub async fn get_stats(&self) -> StatusStatistics {
        let agent_statuses = self.agent_statuses.read().await;
        let task_statuses = self.task_statuses.read().await;

        let total_agents = agent_statuses.len();
        let busy_agents = agent_statuses.values().filter(|s| s.is_busy).count();
        let total_completed: u64 = agent_statuses.values().map(|s| s.tasks_completed).sum();
        let total_failed: u64 = agent_statuses.values().map(|s| s.tasks_failed).sum();

        let pending_tasks = task_statuses
            .values()
            .filter(|s| matches!(s, TaskStatus::Pending))
            .count();
        let in_progress_tasks = task_statuses
            .values()
            .filter(|s| matches!(s, TaskStatus::InProgress))
            .count();

        StatusStatistics {
            total_agents,
            busy_agents,
            total_completed,
            total_failed,
            pending_tasks,
            in_progress_tasks,
        }
    }

    /// Clear all statuses (for testing or reset)
    pub async fn clear(&self) {
        let mut agent_statuses = self.agent_statuses.write().await;
        let mut task_statuses = self.task_statuses.write().await;

        agent_statuses.clear();
        task_statuses.clear();

        info!("Cleared all status tracking");
    }
}

/// 🏗️ ARCHITECTURE DECISION: Statistics DTO for monitoring
/// Why: Separate concerns - status tracking vs reporting
pub struct StatusStatistics {
    pub total_agents: usize,
    pub busy_agents: usize,
    pub total_completed: u64,
    pub total_failed: u64,
    pub pending_tasks: usize,
    pub in_progress_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_status_lifecycle() {
        let manager = StatusManager::new();

        // Initialize agent
        manager
            .initialize_agent_status(AgentType::SoftwareDeveloper)
            .await;

        // Mark as busy
        manager
            .mark_agent_busy(&AgentType::SoftwareDeveloper, "test-task-1".to_string())
            .await
            .unwrap();
        let status = manager
            .get_agent_status(&AgentType::SoftwareDeveloper)
            .await
            .unwrap();
        assert!(status.is_busy);

        // Complete task
        manager
            .increment_agent_completed(&AgentType::SoftwareDeveloper, 1.5)
            .await
            .unwrap();
        let status = manager
            .get_agent_status(&AgentType::SoftwareDeveloper)
            .await
            .unwrap();
        assert!(!status.is_busy);
        assert_eq!(status.tasks_completed, 1);
    }

    #[tokio::test]
    async fn test_task_status_tracking() {
        let manager = StatusManager::new();

        manager
            .update_task_status("task-1", TaskStatus::Pending)
            .await;
        manager
            .update_task_status("task-2", TaskStatus::InProgress)
            .await;

        assert_eq!(
            manager.get_task_status("task-1").await,
            Some(TaskStatus::Pending)
        );
        assert_eq!(
            manager.get_task_status("task-2").await,
            Some(TaskStatus::InProgress)
        );

        manager.remove_task_status("task-1").await;
        assert_eq!(manager.get_task_status("task-1").await, None);
    }
}
