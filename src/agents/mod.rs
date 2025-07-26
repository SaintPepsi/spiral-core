pub mod developer;
pub mod orchestrator;
// 🔧 UTILITY MODULES: Extracted via 3-strikes abstraction rule
pub mod language_detection;
pub mod task_utils;

pub use developer::SoftwareDeveloperAgent;
pub use orchestrator::AgentOrchestrator;

use crate::{
    claude_code::TaskAnalysis,
    models::{AgentType, Task, TaskResult},
    Result,
};
use async_trait::async_trait;

#[async_trait]
pub trait Agent: Send + Sync {
    fn agent_type(&self) -> AgentType;
    fn name(&self) -> String;
    fn description(&self) -> String;

    async fn can_handle(&self, task: &Task) -> bool;
    async fn execute(&self, task: Task) -> Result<TaskResult>;
    async fn analyze_task(&self, task: &Task) -> Result<TaskAnalysis>;
}

#[derive(Debug, Clone)]
pub struct AgentStatus {
    pub agent_type: AgentType,
    pub is_busy: bool,
    pub current_task_id: Option<String>,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub average_execution_time: f64,
}

impl AgentStatus {
    pub fn new(agent_type: AgentType) -> Self {
        Self {
            agent_type,
            is_busy: false,
            current_task_id: None,
            tasks_completed: 0,
            tasks_failed: 0,
            average_execution_time: 0.0,
        }
    }

    pub fn start_task(&mut self, task_id: String) {
        self.is_busy = true;
        self.current_task_id = Some(task_id);
    }

    pub fn complete_task(&mut self, execution_time: f64) {
        self.is_busy = false;
        self.current_task_id = None;
        self.tasks_completed += 1;

        self.average_execution_time =
            (self.average_execution_time * (self.tasks_completed - 1) as f64 + execution_time)
                / self.tasks_completed as f64;
    }

    pub fn fail_task(&mut self) {
        self.is_busy = false;
        self.current_task_id = None;
        self.tasks_failed += 1;
    }
}
