use crate::{
    models::{AgentType, Task, TaskResult, TaskStatus},
    Result, SpiralError,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, warn};

/// Atomic state management for task execution to prevent race conditions
/// and ensure consistent state across all storage systems
pub struct AtomicTaskStateManager {
    task_storage: Arc<Mutex<HashMap<String, Task>>>,
    task_results: Arc<Mutex<HashMap<String, TaskResult>>>,
    agent_statuses: Arc<RwLock<HashMap<AgentType, super::AgentStatus>>>,
}

impl AtomicTaskStateManager {
    pub fn new(
        task_storage: Arc<Mutex<HashMap<String, Task>>>,
        task_results: Arc<Mutex<HashMap<String, TaskResult>>>,
        agent_statuses: Arc<RwLock<HashMap<AgentType, super::AgentStatus>>>,
    ) -> Self {
        Self {
            task_storage,
            task_results,
            agent_statuses,
        }
    }

    /// Atomically transition task to InProgress state
    /// Returns Ok(()) if successful, Err if state transition is invalid
    pub async fn start_task_atomic(&self, task: &mut Task) -> Result<()> {
        let task_id = task.id.clone();
        let agent_type = task.agent_type.clone();

        // Acquire all locks in consistent order to prevent deadlocks
        let mut storage = self.task_storage.lock().await;
        let mut statuses = self.agent_statuses.write().await;

        // Verify task is still in valid state for execution
        if let Some(stored_task) = storage.get(&task_id) {
            match stored_task.status {
                TaskStatus::Pending => {
                    // Valid state transition
                }
                TaskStatus::InProgress => {
                    return Err(SpiralError::Agent {
                        message: format!("Task {task_id} is already in progress"),
                    });
                }
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
                    return Err(SpiralError::Agent {
                        message: format!("Task {task_id} has already been processed"),
                    });
                }
            }
        } else {
            return Err(SpiralError::Agent {
                message: format!("Task {task_id} not found in storage"),
            });
        }

        // Update task state
        task.status = TaskStatus::InProgress;
        task.updated_at = chrono::Utc::now();

        // Update storage
        storage.insert(task_id.clone(), task.clone());

        // Update agent status
        if let Some(status) = statuses.get_mut(&agent_type) {
            status.start_task(task_id.clone());
        } else {
            warn!("Agent status not found for type: {:?}", agent_type);
        }

        debug!("Task {} atomically transitioned to InProgress", task_id);
        Ok(())
    }

    /// Atomically complete task with result
    pub async fn complete_task_atomic(
        &self,
        task_id: &str,
        task_result: TaskResult,
        execution_time: f64,
    ) -> Result<()> {
        // Acquire all locks in consistent order
        let mut storage = self.task_storage.lock().await;
        let mut results = self.task_results.lock().await;
        let mut statuses = self.agent_statuses.write().await;

        // Get task from storage
        let task = storage.get_mut(task_id).ok_or_else(|| SpiralError::Agent {
            message: format!("Task {task_id} not found in storage"),
        })?;

        // Verify task is in correct state
        if task.status != TaskStatus::InProgress {
            return Err(SpiralError::Agent {
                message: format!(
                    "Task {} is not in progress (status: {:?})",
                    task_id, task.status
                ),
            });
        }

        let agent_type = task.agent_type.clone();

        // Update task status
        task.status = match &task_result.result {
            crate::models::TaskExecutionResult::Success { .. } => TaskStatus::Completed,
            crate::models::TaskExecutionResult::Failure { .. } => TaskStatus::Failed,
        };
        task.updated_at = chrono::Utc::now();

        // Store result
        results.insert(task_id.to_string(), task_result);

        // Update agent status
        if let Some(status) = statuses.get_mut(&agent_type) {
            status.complete_task(execution_time);
        }

        debug!("Task {} atomically completed", task_id);
        Ok(())
    }

    /// Atomically fail task with error
    pub async fn fail_task_atomic(
        &self,
        task_id: &str,
        error: &SpiralError,
        execution_time: f64,
    ) -> Result<()> {
        // Acquire all locks in consistent order
        let mut storage = self.task_storage.lock().await;
        let mut statuses = self.agent_statuses.write().await;

        // Get task from storage
        let task = storage.get_mut(task_id).ok_or_else(|| SpiralError::Agent {
            message: format!("Task {task_id} not found in storage"),
        })?;

        // Verify task is in correct state
        if task.status != TaskStatus::InProgress {
            return Err(SpiralError::Agent {
                message: format!(
                    "Task {} is not in progress (status: {:?})",
                    task_id, task.status
                ),
            });
        }

        let agent_type = task.agent_type.clone();

        // Update task status
        task.status = TaskStatus::Failed;
        task.updated_at = chrono::Utc::now();

        // Update agent status
        if let Some(status) = statuses.get_mut(&agent_type) {
            status.complete_task(execution_time);
        }

        debug!("Task {} atomically marked as failed: {}", task_id, error);
        Ok(())
    }

    /// Cleanup task state if execution fails before completion
    pub async fn cleanup_task_state(&self, task_id: &str) {
        let mut storage = self.task_storage.lock().await;
        let mut statuses = self.agent_statuses.write().await;

        if let Some(task) = storage.get_mut(task_id) {
            if task.status == TaskStatus::InProgress {
                // Reset to pending if task was in progress
                task.status = TaskStatus::Pending;
                task.updated_at = chrono::Utc::now();

                // Remove from agent's active tasks
                if let Some(status) = statuses.get_mut(&task.agent_type) {
                    // We need to remove this task from the agent's active set
                    // This requires exposing a method in AgentStatus
                    status.is_busy = false;
                    status.current_task_id = None;
                }

                warn!("Cleaned up incomplete task {}", task_id);
            }
        }
    }
}
