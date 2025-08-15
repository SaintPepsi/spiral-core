use crate::{
    models::{Task, TaskResult},
    Result,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// 🏗️ ARCHITECTURE DECISION: Separated ResultStore Service
/// Why: Single Responsibility - manages task results and storage
/// Alternative: Keep in orchestrator (rejected: violates SRP)
/// Benefits: Can swap storage backend (memory, database, S3) without affecting orchestrator
#[derive(Clone)]
pub struct ResultStore {
    results: Arc<Mutex<HashMap<String, TaskResult>>>,
    tasks: Arc<Mutex<HashMap<String, Task>>>,
    max_results: usize,
}

impl ResultStore {
    pub fn new(max_results: usize) -> Self {
        Self {
            results: Arc::new(Mutex::new(HashMap::new())),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            max_results,
        }
    }

    /// Store a task
    pub async fn store_task(&self, task: Task) -> Result<()> {
        let mut tasks = self.tasks.lock().await;
        tasks.insert(task.id.clone(), task.clone());
        debug!("Stored task: {}", task.id);
        Ok(())
    }

    /// Store a task result
    pub async fn store_result(&self, task_id: String, result: TaskResult) -> Result<()> {
        let mut results = self.results.lock().await;

        // Check storage limit
        if results.len() >= self.max_results {
            // Remove oldest result (simple FIFO for now)
            if let Some(oldest_key) = results.keys().next().cloned() {
                results.remove(&oldest_key);
                info!("Evicted oldest result to maintain limit: {}", oldest_key);
            }
        }

        results.insert(task_id.clone(), result);
        info!("Stored result for task: {}", task_id);
        Ok(())
    }

    /// Get a task by ID
    pub async fn get_task(&self, task_id: &str) -> Option<Task> {
        let tasks = self.tasks.lock().await;
        tasks.get(task_id).cloned()
    }

    /// Get a task result by ID
    pub async fn get_result(&self, task_id: &str) -> Option<TaskResult> {
        let results = self.results.lock().await;
        results.get(task_id).cloned()
    }

    /// Remove a task and its result
    pub async fn remove(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.lock().await;
        let mut results = self.results.lock().await;

        tasks.remove(task_id);
        results.remove(task_id);

        debug!("Removed task and result: {}", task_id);
        Ok(())
    }

    /// Get all tasks
    pub async fn get_all_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().await;
        tasks.values().cloned().collect()
    }

    /// Get all results
    pub async fn get_all_results(&self) -> HashMap<String, TaskResult> {
        let results = self.results.lock().await;
        results.clone()
    }

    /// Clear all storage
    pub async fn clear(&self) {
        let mut tasks = self.tasks.lock().await;
        let mut results = self.results.lock().await;

        let task_count = tasks.len();
        let result_count = results.len();

        tasks.clear();
        results.clear();

        info!("Cleared {} tasks and {} results", task_count, result_count);
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> (usize, usize) {
        let tasks = self.tasks.lock().await;
        let results = self.results.lock().await;
        (tasks.len(), results.len())
    }
}

/// 🏗️ ARCHITECTURE DECISION: Repository pattern for future database integration
/// Why: Abstract storage details from business logic
/// Alternative: Direct database access (rejected: tight coupling to storage implementation)
#[async_trait::async_trait]
pub trait TaskRepository: Send + Sync {
    async fn save_task(&self, task: Task) -> Result<()>;
    async fn find_task(&self, id: &str) -> Result<Option<Task>>;
    async fn save_result(&self, id: String, result: TaskResult) -> Result<()>;
    async fn find_result(&self, id: &str) -> Result<Option<TaskResult>>;
    async fn delete(&self, id: &str) -> Result<()>;
}

#[async_trait::async_trait]
impl TaskRepository for ResultStore {
    async fn save_task(&self, task: Task) -> Result<()> {
        self.store_task(task).await
    }

    async fn find_task(&self, id: &str) -> Result<Option<Task>> {
        Ok(self.get_task(id).await)
    }

    async fn save_result(&self, id: String, result: TaskResult) -> Result<()> {
        self.store_result(id, result).await
    }

    async fn find_result(&self, id: &str) -> Result<Option<TaskResult>> {
        Ok(self.get_result(id).await)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        self.remove(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AgentType, Priority, TaskExecutionResult};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let store = ResultStore::new(10);

        let task = Task::new(
            AgentType::SoftwareDeveloper,
            "test task".to_string(),
            Priority::Medium,
        );

        store.store_task(task.clone()).await.unwrap();

        let retrieved = store.get_task(&task.id).await.unwrap();
        assert_eq!(retrieved.id, task.id);

        // Store result
        let result = TaskResult {
            task_id: task.id.clone(),
            agent_type: AgentType::SoftwareDeveloper,
            result: TaskExecutionResult::Success {
                output: "test output".to_string(),
                files_created: vec![],
                files_modified: vec![],
            },
            metadata: HashMap::new(),
            completed_at: chrono::Utc::now(),
        };

        store
            .store_result(task.id.clone(), result.clone())
            .await
            .unwrap();

        let retrieved_result = store.get_result(&task.id).await.unwrap();
        assert_eq!(retrieved_result.task_id, task.id);
    }
}
