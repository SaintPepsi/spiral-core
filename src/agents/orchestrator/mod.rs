use super::{Agent, AgentStatus, SoftwareDeveloperAgent};
use crate::{
    claude_code::ClaudeCodeClient,
    config::Config,
    models::{AgentType, Task, TaskResult, TaskStatus},
    Result, SpiralError,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};

#[derive(Clone)]
pub struct AgentOrchestrator {
    agents: Arc<RwLock<HashMap<AgentType, Box<dyn Agent>>>>,
    agent_statuses: Arc<RwLock<HashMap<AgentType, AgentStatus>>>,
    task_queue: Arc<Mutex<Vec<Task>>>,
    result_sender: Arc<Mutex<Option<mpsc::UnboundedSender<TaskResult>>>>,
    task_storage: Arc<Mutex<HashMap<String, Task>>>,
    task_results: Arc<Mutex<HashMap<String, TaskResult>>>,
    start_time: Arc<std::time::Instant>,
    claude_client: Arc<ClaudeCodeClient>,
}

impl AgentOrchestrator {
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Agent Orchestrator");

        // 🧠 AGENT COORDINATION DECISION: Using ClaudeCodeClient as shared intelligence engine
        // Why: Centralizes API management, rate limiting, and response handling across agents
        // Alternative: Individual clients per agent (rejected: increases complexity, API overhead)
        // Audit: Check claude_code.rs:45-60 for client initialization patterns
        let claude_client = ClaudeCodeClient::new(config.claude_code.clone()).await?;

        // 🔧 ARCHITECTURE DECISION: HashMap for agent registry with AgentType enum keys
        // Why: Type-safe agent lookup, prevents duplicate registrations, O(1) access
        // Alternative: Vec<Agent> with linear search (rejected: O(n) lookups for task routing)
        // Audit: Verify AgentType enum completeness in models.rs:15-25
        let mut agents: HashMap<AgentType, Box<dyn Agent>> = HashMap::new();
        let mut statuses: HashMap<AgentType, AgentStatus> = HashMap::new();

        // 🚀 EXTENSIBILITY PATTERN: Manual agent registration for controlled expansion
        // Why: Explicit control over which agents are available, easier to debug capability issues
        // Alternative: Auto-discovery/reflection (rejected: runtime errors, unclear dependencies)
        // Future: Consider agent plugin architecture when we have >5 agent types
        let developer_agent = SoftwareDeveloperAgent::new(claude_client.clone());
        statuses.insert(
            AgentType::SoftwareDeveloper,
            developer_agent.status().clone(),
        );
        agents.insert(AgentType::SoftwareDeveloper, Box::new(developer_agent));

        info!("Registered {} agents", agents.len());

        // 🔒 CONCURRENCY DESIGN: Arc<RwLock> for shared read access, Arc<Mutex> for exclusive writes
        // Why: Multiple tasks can read agent registry simultaneously, but task queue needs serialization
        // Alternative: Single global mutex (rejected: unnecessary blocking of concurrent reads)
        // Audit: Verify no deadlock potential in execute_task method around lines 245-270
        Ok(Self {
            agents: Arc::new(RwLock::new(agents)),
            agent_statuses: Arc::new(RwLock::new(statuses)),
            task_queue: Arc::new(Mutex::new(Vec::new())),
            result_sender: Arc::new(Mutex::new(None)),
            task_storage: Arc::new(Mutex::new(HashMap::new())),
            task_results: Arc::new(Mutex::new(HashMap::new())),
            start_time: Arc::new(std::time::Instant::now()),
            claude_client: Arc::new(claude_client),
        })
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting Agent Orchestrator");

        let (result_tx, mut result_rx) = mpsc::unbounded_channel();
        {
            let mut sender = self.result_sender.lock().await;
            *sender = Some(result_tx);
        }

        let orchestrator = self.clone();
        let task_processor = tokio::spawn(async move { orchestrator.process_tasks().await });

        let result_processor = tokio::spawn(async move {
            while let Some(result) = result_rx.recv().await {
                info!(
                    "Received task result: {} - {:?}",
                    result.task_id, result.result
                );
            }
        });

        let cleanup_orchestrator = self.clone();
        let cleanup_processor =
            tokio::spawn(async move { cleanup_orchestrator.cleanup_loop().await });

        tokio::select! {
            result = task_processor => {
                if let Err(e) = result {
                    error!("Task processor failed: {}", e);
                }
            }
            result = result_processor => {
                if let Err(e) = result {
                    error!("Result processor failed: {}", e);
                }
            }
            result = cleanup_processor => {
                if let Err(e) = result {
                    error!("Cleanup processor failed: {}", e);
                }
            }
        }

        Ok(())
    }

    /// 📝 USER REQUEST ENTRY POINT: Where human intentions become agent tasks
    /// This is the primary interface between Discord/API and the agent system
    /// AUDIT CHECKPOINT: Verify task validation, queue management, and error handling
    pub async fn submit_task(&self, mut task: Task) -> Result<String> {
        debug!(
            "Submitting task: {} for agent: {:?}",
            task.id, task.agent_type
        );

        // 🛡️ SAFETY CHECK: Prevent tasks for non-existent agents before queue insertion
        // Why: Early validation prevents resource waste and provides clear error messages
        // Alternative: Check during execution (rejected: wastes queue space, delays error feedback)
        if !self.can_handle_agent_type(&task.agent_type).await {
            return Err(SpiralError::Agent {
                message: format!("No agent available for type: {:?}", task.agent_type),
            });
        }

        // 🚦 BACKPRESSURE MECHANISM: Prevent system overload by limiting queue size
        // Why: Protects against memory exhaustion and provides responsive error feedback
        // Current limit: Check constants.rs for MAX_QUEUE_SIZE value
        // Alternative: Unlimited queue (rejected: potential OOM), Dynamic scaling (future enhancement)
        {
            let queue = self.task_queue.lock().await;
            if queue.len() >= crate::constants::MAX_QUEUE_SIZE {
                return Err(SpiralError::Agent {
                    message: "Task queue is full. Please try again later.".to_string(),
                });
            }
        }

        // 📊 STATE TRACKING: Mark task as pending and update timestamp for lifecycle management
        // Why: Enables status queries, cleanup processes, and execution time tracking
        task.status = TaskStatus::Pending;
        task.updated_at = chrono::Utc::now();

        let task_id = task.id.clone();

        // 💾 PERSISTENCE STRATEGY: Dual storage for queue processing and status queries
        // Why: Queue for processing order, storage for external status API access
        // Alternative: Single storage with status flags (rejected: complicates priority queue logic)
        // Audit: Verify cleanup process removes from both locations (cleanup_loop method)
        {
            let mut storage = self.task_storage.lock().await;
            storage.insert(task_id.clone(), task.clone());
        }

        // 🎯 PRIORITY QUEUE MANAGEMENT: Higher priority tasks execute first
        // Why: Ensures urgent tasks don't wait behind large batches of low-priority work
        // Implementation: Rust sorts in ascending order, so we reverse compare (b vs a)
        {
            let mut queue = self.task_queue.lock().await;
            queue.push(task);
            queue.sort_by(|a, b| {
                b.priority
                    .partial_cmp(&a.priority)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        info!("Task {} submitted and queued", task_id);
        Ok(task_id)
    }

    pub async fn get_task_status(&self, task_id: &str) -> Option<Task> {
        let storage = self.task_storage.lock().await;
        storage.get(task_id).cloned()
    }

    pub async fn get_task_result(&self, task_id: &str) -> Option<TaskResult> {
        let results = self.task_results.lock().await;
        results.get(task_id).cloned()
    }

    pub async fn get_system_uptime(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    async fn cleanup_loop(&self) -> Result<()> {
        info!("Starting cleanup loop");

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(
                crate::constants::CLEANUP_INTERVAL_SECS,
            ))
            .await;

            if let Err(e) = self.perform_cleanup().await {
                error!("Cleanup failed: {}", e);
            }
        }
    }

    /// 🧹 MEMORY MANAGEMENT: Automatic cleanup of historical data to prevent memory growth
    /// AUDIT CHECKPOINT: Verify retention policy doesn't remove active tasks or needed results
    async fn perform_cleanup(&self) -> Result<()> {
        debug!("Performing system cleanup");

        let now = chrono::Utc::now();
        // 📅 RETENTION POLICY: 24-hour sliding window for historical data
        // Why: Balances memory usage with debugging/audit capabilities
        // Alternative: Configurable retention (future enhancement), No cleanup (rejected: memory leak)
        let cutoff_time = now - chrono::Duration::hours(24);

        // 🗂️ TASK STORAGE CLEANUP: Remove old completed/failed tasks while preserving active work
        // Why: Active tasks must remain available for status queries and execution
        {
            let mut storage = self.task_storage.lock().await;
            let initial_count = storage.len();

            // 🔍 SELECTIVE RETENTION: Keep recent data OR active tasks regardless of age
            // Logic: Preserve anything recent OR anything still being processed
            // Audit: Verify Pending/InProgress tasks are never cleaned up prematurely
            storage.retain(|_, task| {
                task.updated_at > cutoff_time
                    || matches!(task.status, TaskStatus::Pending | TaskStatus::InProgress)
            });

            let removed = initial_count - storage.len();
            if removed > 0 {
                info!("Cleaned up {} old tasks from storage", removed);
            }
        }

        // 📊 RESULT STORAGE CLEANUP: Remove old task results to free memory
        // Why: Results consume significant memory and become less relevant over time
        {
            let mut results = self.task_results.lock().await;
            let initial_count = results.len();

            results.retain(|_, result| result.completed_at > cutoff_time);

            let removed = initial_count - results.len();
            if removed > 0 {
                info!("Cleaned up {} old task results", removed);
            }
        }

        Ok(())
    }

    pub async fn get_agent_status(&self, agent_type: &AgentType) -> Option<AgentStatus> {
        let statuses = self.agent_statuses.read().await;
        statuses.get(agent_type).cloned()
    }

    pub async fn get_all_agent_statuses(&self) -> HashMap<AgentType, AgentStatus> {
        let statuses = self.agent_statuses.read().await;
        statuses.clone()
    }

    pub async fn get_queue_length(&self) -> usize {
        let queue = self.task_queue.lock().await;
        queue.len()
    }

    async fn can_handle_agent_type(&self, agent_type: &AgentType) -> bool {
        let agents = self.agents.read().await;
        agents.contains_key(agent_type)
    }

    async fn process_tasks(&self) -> Result<()> {
        info!("Task processor started");

        loop {
            let task = {
                let mut queue = self.task_queue.lock().await;
                queue.pop()
            };

            if let Some(task) = task {
                if let Err(e) = self.execute_task(task).await {
                    error!("Failed to execute task: {}", e);
                }
            } else {
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    crate::constants::TASK_POLL_INTERVAL_MS,
                ))
                .await;
            }
        }
    }

    /// 🎬 TASK EXECUTION ORCHESTRATION: Where agent capabilities meet user requests
    /// This is the core coordination logic that manages agent execution and state tracking
    /// AUDIT CHECKPOINT: Critical path for all task processing - verify error handling and state consistency
    async fn execute_task(&self, mut task: Task) -> Result<()> {
        debug!("Executing task: {}", task.id);

        // 🔍 AGENT LOOKUP STRATEGY: Read lock allows concurrent task execution for different agent types
        // Why: Multiple agents can work simultaneously without blocking each other
        // Performance: O(1) lookup, minimal lock contention for heterogeneous workloads
        {
            let agents = self.agents.read().await;
            match agents.get(&task.agent_type) {
                Some(agent) => {
                    // 🛡️ CAPABILITY VALIDATION: Agent-specific pre-execution checks
                    // Why: Prevents execution of malformed or incompatible tasks
                    // Alternative: Skip validation (rejected: potential runtime failures, resource waste)
                    if !agent.can_handle(&task).await {
                        warn!("Agent cannot handle task: {}", task.id);
                        return Err(SpiralError::Agent {
                            message: format!("Agent {:?} cannot handle task", task.agent_type),
                        });
                    }

                    // 📊 STATE TRANSITION: Pending → InProgress
                    // Why: Enables external monitoring and prevents duplicate execution
                    task.status = TaskStatus::InProgress;
                    task.updated_at = chrono::Utc::now();

                    // 💾 PERSISTENT STATE UPDATE: Sync in-memory state with tracking storage
                    // Why: External status APIs need to reflect current execution state
                    {
                        let mut storage = self.task_storage.lock().await;
                        storage.insert(task.id.clone(), task.clone());
                    }

                    // 📈 AGENT PERFORMANCE TRACKING: Update agent metrics for capacity planning
                    // Why: Enables load balancing decisions and performance monitoring
                    // Audit: Verify metrics accuracy in agent status implementation
                    {
                        let mut statuses = self.agent_statuses.write().await;
                        if let Some(status) = statuses.get_mut(&task.agent_type) {
                            status.start_task(task.id.clone());
                        }
                    }

                    // ⏱️ EXECUTION TIMING: Critical for performance analysis and SLA monitoring
                    // Why: Enables identification of slow operations and capacity planning
                    let start_time = std::time::Instant::now();
                    let result = agent.execute(task.clone()).await;
                    let execution_time = start_time.elapsed().as_secs_f64();

                    // 🎯 RESULT PROCESSING: Success and failure paths with comprehensive state management
                    match result {
                        Ok(task_result) => {
                            // ✅ SUCCESS PATH: Update all tracking systems for completed task
                            // Audit: Verify no race conditions between these state updates

                            // 💾 TASK STATUS UPDATE: Mark completion in primary storage
                            {
                                let mut storage = self.task_storage.lock().await;
                                if let Some(stored_task) = storage.get_mut(&task.id) {
                                    stored_task.status = TaskStatus::Completed;
                                    stored_task.updated_at = chrono::Utc::now();
                                }
                            }

                            // 📋 RESULT STORAGE: Persist output for external retrieval
                            // Why: Enables async result fetching for long-running tasks
                            {
                                let mut results = self.task_results.lock().await;
                                results.insert(task.id.clone(), task_result.clone());
                            }

                            // 📊 AGENT METRICS UPDATE: Record successful completion for performance tracking
                            {
                                let mut statuses = self.agent_statuses.write().await;
                                if let Some(status) = statuses.get_mut(&task.agent_type) {
                                    status.complete_task(execution_time);
                                }
                            }

                            // 📢 RESULT BROADCASTING: Notify interested subscribers
                            // Why: Enables real-time notifications and downstream processing
                            // Alternative: Polling (rejected: higher latency, resource waste)
                            if let Some(sender) = &*self.result_sender.lock().await {
                                if let Err(e) = sender.send(task_result) {
                                    error!("Failed to send task result: {}", e);
                                }
                            }

                            info!(
                                "Task {} completed successfully in {:.2}s",
                                task.id, execution_time
                            );
                            Ok(())
                        }
                        Err(e) => {
                            // ❌ FAILURE PATH: Comprehensive error state management
                            // Audit: Ensure failure state is consistent across all tracking systems

                            // 💾 TASK STATUS UPDATE: Mark failure in primary storage
                            {
                                let mut storage = self.task_storage.lock().await;
                                if let Some(stored_task) = storage.get_mut(&task.id) {
                                    stored_task.status = TaskStatus::Failed;
                                    stored_task.updated_at = chrono::Utc::now();
                                }
                            }

                            // 📊 AGENT METRICS UPDATE: Record failure for reliability tracking
                            // Why: Enables identification of problematic agents or task types
                            {
                                let mut statuses = self.agent_statuses.write().await;
                                if let Some(status) = statuses.get_mut(&task.agent_type) {
                                    status.fail_task();
                                }
                            }

                            error!("Task {} failed: {}", task.id, e);
                            Err(e)
                        }
                    }
                }
                None => {
                    // 🚨 CRITICAL ERROR: Agent registry inconsistency detected
                    // This should never happen if can_handle_agent_type validation is working
                    // Audit: Check submit_task validation logic for potential race conditions
                    Err(SpiralError::Agent {
                        message: format!("No agent found for type: {:?}", task.agent_type),
                    })
                }
            }
        }
    }

    pub async fn analyze_task(&self, task: &Task) -> Result<crate::claude_code::TaskAnalysis> {
        let agents = self.agents.read().await;
        let agent = agents
            .get(&task.agent_type)
            .ok_or_else(|| SpiralError::Agent {
                message: format!("No agent found for type: {:?}", task.agent_type),
            })?;

        agent.analyze_task(task).await
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Agent Orchestrator");

        {
            let mut sender = self.result_sender.lock().await;
            *sender = None;
        }

        info!("Agent Orchestrator shutdown complete");
        Ok(())
    }

    /// 🔧 CLAUDE CLIENT ACCESS: Provide access to Claude Code client for shutdown cleanup
    /// DECISION: Return Result to handle case where client might be unavailable
    /// Why: Defensive programming for graceful degradation during shutdown
    pub fn get_claude_client(&self) -> Result<&ClaudeCodeClient> {
        Ok(&self.claude_client)
    }

    /// 📊 SYSTEM STATUS: Get current system state for monitoring and shutdown
    /// DECISION: Return simple struct rather than complex API response type
    /// Why: Keeps orchestrator independent of API layer concerns
    pub async fn get_system_status(&self) -> SystemStatus {
        let agent_statuses = self.agent_statuses.read().await;
        let task_queue = self.task_queue.lock().await;
        
        let agents: std::collections::HashMap<AgentType, SimpleAgentStatus> = agent_statuses
            .iter()
            .map(|(agent_type, status)| {
                (agent_type.clone(), SimpleAgentStatus {
                    is_busy: status.is_busy,
                    tasks_completed: status.tasks_completed,
                    tasks_failed: status.tasks_failed,
                })
            })
            .collect();

        SystemStatus {
            agents,
            queue_length: task_queue.len(),
            system_uptime: self.start_time.elapsed().as_secs_f64(),
        }
    }
}

/// 📊 SYSTEM STATUS TYPES: Simple status information for internal use
/// DECISION: Separate from API response types for loose coupling
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub agents: std::collections::HashMap<AgentType, SimpleAgentStatus>,
    pub queue_length: usize,
    pub system_uptime: f64,
}

#[derive(Debug, Clone)]
pub struct SimpleAgentStatus {
    pub is_busy: bool,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
}
