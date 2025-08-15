use crate::{
    models::{Task, TaskStatus},
    Result, SpiralError,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// 🏗️ ARCHITECTURE DECISION: Separated TaskQueue Service
/// Why: Single Responsibility - manages only task queuing logic
/// Alternative: Keep in orchestrator (rejected: god object anti-pattern)
/// Benefits: Can swap queue implementation (memory, Redis, RabbitMQ) without affecting orchestrator
#[derive(Clone)]
pub struct TaskQueue {
    queue: Arc<Mutex<Vec<Task>>>,
    max_queue_size: usize,
}

impl TaskQueue {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            max_queue_size,
        }
    }

    /// Add a task to the queue
    pub async fn enqueue(&self, mut task: Task) -> Result<()> {
        let mut queue = self.queue.lock().await;

        // Check queue size limit
        if queue.len() >= self.max_queue_size {
            return Err(SpiralError::Agent {
                message: format!("Task queue full (max: {})", self.max_queue_size),
            });
        }

        task.status = TaskStatus::Pending;
        queue.push(task.clone());
        info!(
            "Task {} added to queue (position: {})",
            task.id,
            queue.len()
        );

        Ok(())
    }

    /// Get the next task from the queue
    pub async fn dequeue(&self) -> Option<Task> {
        let mut queue = self.queue.lock().await;
        let task = queue.pop();

        if let Some(ref t) = task {
            debug!("Task {} dequeued (remaining: {})", t.id, queue.len());
        }

        task
    }

    /// Peek at the next task without removing it
    pub async fn peek(&self) -> Option<Task> {
        let queue = self.queue.lock().await;
        queue.last().cloned()
    }

    /// Get current queue size
    pub async fn size(&self) -> usize {
        let queue = self.queue.lock().await;
        queue.len()
    }

    /// Clear all tasks from the queue
    pub async fn clear(&self) {
        let mut queue = self.queue.lock().await;
        let count = queue.len();
        queue.clear();
        info!("Cleared {} tasks from queue", count);
    }

    /// Get all pending tasks (for monitoring)
    pub async fn get_all(&self) -> Vec<Task> {
        let queue = self.queue.lock().await;
        queue.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AgentType, Priority};

    #[tokio::test]
    async fn test_enqueue_dequeue() {
        let queue = TaskQueue::new(10);

        let task = Task::new(
            AgentType::SoftwareDeveloper,
            "test task".to_string(),
            Priority::Medium,
        );

        queue.enqueue(task.clone()).await.unwrap();
        assert_eq!(queue.size().await, 1);

        let dequeued = queue.dequeue().await.unwrap();
        assert_eq!(dequeued.id, task.id);
        assert_eq!(queue.size().await, 0);
    }

    #[tokio::test]
    async fn test_queue_limit() {
        let queue = TaskQueue::new(2);

        for i in 0..2 {
            let task = Task::new(
                AgentType::SoftwareDeveloper,
                format!("task {}", i),
                Priority::Medium,
            );
            queue.enqueue(task).await.unwrap();
        }

        // Third task should fail
        let task = Task::new(
            AgentType::SoftwareDeveloper,
            "overflow task".to_string(),
            Priority::Medium,
        );

        let result = queue.enqueue(task).await;
        assert!(result.is_err());
    }
}
