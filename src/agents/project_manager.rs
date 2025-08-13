//! Project Manager Agent - Strategic analysis and task coordination
//!
//! This agent provides high-level strategic thinking, breaking down complex problems
//! into manageable phases while coordinating between different specialists.

use super::{Agent, AgentStatus};
use crate::{
    claude_code::{ClaudeCodeClient, TaskAnalysis},
    models::{AgentType, Task, TaskExecutionResult, TaskResult},
    Result,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Project Manager Agent for strategic analysis and coordination
pub struct ProjectManagerAgent {
    claude_client: Option<ClaudeCodeClient>,
    status: Arc<RwLock<AgentStatus>>,
}

impl ProjectManagerAgent {
    /// Create a new Project Manager Agent
    pub fn new(claude_client: Option<ClaudeCodeClient>) -> Self {
        Self {
            claude_client,
            status: Arc::new(RwLock::new(AgentStatus::new(AgentType::ProjectManager))),
        }
    }

    /// Analyze a project request and create an execution plan
    pub async fn create_project_plan(&self, task: &Task) -> Result<ProjectPlan> {
        info!(
            "[ProjectManager] Creating project plan for task: {}",
            task.id
        );

        // If Claude client is available, use it for comprehensive planning
        if let Some(ref claude_client) = self.claude_client {
            self.create_plan_with_claude(task, claude_client).await
        } else {
            // Fallback to heuristic-based planning
            self.create_plan_with_heuristics(task).await
        }
    }

    /// Create a comprehensive plan using Claude Code
    async fn create_plan_with_claude(
        &self,
        task: &Task,
        claude_client: &ClaudeCodeClient,
    ) -> Result<ProjectPlan> {
        let prompt = self.build_planning_prompt(task);

        let code_request = crate::claude_code::CodeGenerationRequest {
            language: "json".to_string(),
            description: prompt,
            context: std::collections::HashMap::from([
                ("task_type".to_string(), "project_planning".to_string()),
                ("task_id".to_string(), task.id.clone()),
            ]),
            existing_code: None,
            requirements: vec![
                "Create a comprehensive project plan".to_string(),
                "Break down into logical phases".to_string(),
                "Identify dependencies and risks".to_string(),
                "Define clear success criteria".to_string(),
            ],
            session_id: Some(format!("pm-{}", task.id)),
        };

        match claude_client.generate_code(code_request).await {
            Ok(result) => {
                // Parse the JSON response into a ProjectPlan
                match serde_json::from_str::<ProjectPlan>(&result.code) {
                    Ok(mut plan) => {
                        plan.task_id = task.id.clone();
                        info!(
                            "[ProjectManager] Created plan with {} phases",
                            plan.phases.len()
                        );
                        Ok(plan)
                    }
                    Err(e) => {
                        warn!("[ProjectManager] Failed to parse Claude's response: {}", e);
                        self.create_plan_with_heuristics(task).await
                    }
                }
            }
            Err(e) => {
                warn!("[ProjectManager] Claude planning failed: {}", e);
                self.create_plan_with_heuristics(task).await
            }
        }
    }

    /// Build a comprehensive planning prompt
    fn build_planning_prompt(&self, task: &Task) -> String {
        format!(
            r#"You are a Project Manager AI agent specializing in strategic analysis and coordination.
            
Analyze the following task and create a comprehensive project plan in JSON format.

## Task Details:
- ID: {}
- Content: {}
- Agent Type: {:?}
- Priority: {:?}

## Your Objectives:
1. Break down the task into logical phases
2. Identify dependencies between phases
3. Assess risks and mitigation strategies
4. Define resource requirements
5. Create clear success criteria

## Output Format:
Return a JSON object with this structure:
{{
    "summary": "Brief project summary",
    "complexity": "Low|Medium|High|Critical",
    "phases": [
        {{
            "id": "phase-1",
            "name": "Phase name",
            "description": "What this phase accomplishes",
            "dependencies": [],
            "required_agents": ["Developer", "QA"],
            "deliverables": ["Deliverable 1", "Deliverable 2"],
            "risks": ["Risk 1", "Risk 2"],
            "success_criteria": ["Criterion 1", "Criterion 2"]
        }}
    ],
    "overall_risks": ["Project-level risk 1", "Project-level risk 2"],
    "resource_requirements": {{
        "agents": ["Developer", "QA", "Security"],
        "special_resources": []
    }},
    "success_metrics": ["Metric 1", "Metric 2"],
    "estimated_complexity_score": 5  // Fibonacci scale: 1,2,3,5,8,13,21
}}

Provide a thorough analysis that ensures project success."#,
            task.id, task.content, task.agent_type, task.priority
        )
    }

    /// Create a plan using heuristics when Claude is unavailable
    async fn create_plan_with_heuristics(&self, task: &Task) -> Result<ProjectPlan> {
        info!("[ProjectManager] Using heuristic-based planning");

        let complexity = self.assess_complexity(&task.content);
        let phases = self.generate_phases(task);

        Ok(ProjectPlan {
            task_id: task.id.clone(),
            summary: format!("Implementation plan for task: {}", task.id),
            complexity,
            phases,
            overall_risks: vec![
                "Requirements may change during implementation".to_string(),
                "Integration complexity with existing systems".to_string(),
            ],
            resource_requirements: ResourceRequirements {
                agents: vec!["Developer".to_string()],
                special_resources: vec![],
            },
            success_metrics: vec![
                "All phases completed successfully".to_string(),
                "No regression in existing functionality".to_string(),
                "Tests pass with 100% coverage of new code".to_string(),
            ],
            estimated_complexity_score: 5,
        })
    }

    /// Assess task complexity based on content
    fn assess_complexity(&self, content: &str) -> ProjectComplexity {
        let desc_lower = content.to_lowercase();

        if desc_lower.contains("critical") || desc_lower.contains("urgent") {
            ProjectComplexity::Critical
        } else if desc_lower.contains("complex") || desc_lower.contains("refactor") {
            ProjectComplexity::High
        } else if desc_lower.contains("enhance") || desc_lower.contains("improve") {
            ProjectComplexity::Medium
        } else {
            ProjectComplexity::Low
        }
    }

    /// Generate phases based on task type
    fn generate_phases(&self, _task: &Task) -> Vec<ProjectPhase> {
        let mut phases = Vec::new();

        // Analysis phase - always first
        phases.push(ProjectPhase {
            id: "phase-1".to_string(),
            name: "Requirements Analysis".to_string(),
            description: "Analyze requirements and create technical specifications".to_string(),
            dependencies: vec![],
            required_agents: vec!["ProjectManager".to_string()],
            deliverables: vec![
                "Technical specification document".to_string(),
                "Risk assessment".to_string(),
            ],
            risks: vec!["Incomplete requirements".to_string()],
            success_criteria: vec!["Clear understanding of all requirements".to_string()],
        });

        // Implementation phase
        phases.push(ProjectPhase {
            id: "phase-2".to_string(),
            name: "Implementation".to_string(),
            description: "Develop the solution according to specifications".to_string(),
            dependencies: vec!["phase-1".to_string()],
            required_agents: vec!["Developer".to_string()],
            deliverables: vec![
                "Working implementation".to_string(),
                "Unit tests".to_string(),
            ],
            risks: vec!["Technical complexity".to_string()],
            success_criteria: vec![
                "Code compiles without errors".to_string(),
                "Unit tests pass".to_string(),
            ],
        });

        // Testing phase
        phases.push(ProjectPhase {
            id: "phase-3".to_string(),
            name: "Testing & Validation".to_string(),
            description: "Comprehensive testing and validation of the implementation".to_string(),
            dependencies: vec!["phase-2".to_string()],
            required_agents: vec!["QA".to_string()],
            deliverables: vec![
                "Test results".to_string(),
                "Performance metrics".to_string(),
            ],
            risks: vec!["Edge cases not covered".to_string()],
            success_criteria: vec![
                "All tests pass".to_string(),
                "No critical issues found".to_string(),
            ],
        });

        phases
    }
}

#[async_trait]
impl Agent for ProjectManagerAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::ProjectManager
    }

    fn name(&self) -> String {
        "Project Manager Agent".to_string()
    }

    fn description(&self) -> String {
        "Strategic analysis, project planning, and multi-agent coordination".to_string()
    }

    async fn can_handle(&self, task: &Task) -> bool {
        // Project Manager can handle strategic and coordination tasks
        task.agent_type == AgentType::ProjectManager
            || task.content.to_lowercase().contains("plan")
            || task.content.to_lowercase().contains("coordinate")
            || task.content.to_lowercase().contains("strategy")
            || task.content.to_lowercase().contains("analyze")
    }

    async fn execute(&self, task: Task) -> Result<TaskResult> {
        let start_time = std::time::Instant::now();

        // Update status
        {
            let mut status = self.status.write().await;
            status.start_task(task.id.clone());
        }

        info!("[ProjectManager] Executing task: {}", task.id);

        // Create project plan
        let plan = match self.create_project_plan(&task).await {
            Ok(p) => p,
            Err(e) => {
                let mut status = self.status.write().await;
                status.fail_task();
                return Ok(TaskResult {
                    task_id: task.id,
                    agent_type: self.agent_type(),
                    result: TaskExecutionResult::Failure {
                        error: format!("Failed to create project plan: {}", e),
                        partial_output: None,
                    },
                    metadata: std::collections::HashMap::from([(
                        "error".to_string(),
                        "Planning failed".to_string(),
                    )]),
                    completed_at: chrono::Utc::now(),
                });
            }
        };

        // Convert plan to JSON output
        let output = serde_json::to_string_pretty(&plan)
            .unwrap_or_else(|e| format!("Failed to serialize plan: {}", e));

        // Update status
        {
            let mut status = self.status.write().await;
            status.complete_task(start_time.elapsed().as_secs_f64());
        }

        Ok(TaskResult {
            task_id: task.id,
            agent_type: self.agent_type(),
            result: TaskExecutionResult::Success {
                output: output.clone(),
                files_created: vec![],
                files_modified: vec![],
            },
            metadata: std::collections::HashMap::from([
                ("phases".to_string(), plan.phases.len().to_string()),
                ("complexity".to_string(), format!("{:?}", plan.complexity)),
                (
                    "agents".to_string(),
                    plan.resource_requirements.agents.join(", "),
                ),
            ]),
            completed_at: chrono::Utc::now(),
        })
    }

    async fn analyze_task(&self, task: &Task) -> Result<TaskAnalysis> {
        debug!("[ProjectManager] Analyzing task: {}", task.id);

        // Analyze from a strategic perspective
        let complexity_level = self.assess_complexity(&task.content);

        let (complexity_str, estimated_minutes) = match complexity_level {
            ProjectComplexity::Low => ("Low".to_string(), 30),
            ProjectComplexity::Medium => ("Medium".to_string(), 120),
            ProjectComplexity::High => ("High".to_string(), 480),
            ProjectComplexity::Critical => ("Critical".to_string(), 960),
        };

        Ok(TaskAnalysis {
            complexity: complexity_str,
            estimated_minutes,
            required_skills: vec![
                "Strategic thinking".to_string(),
                "Project planning".to_string(),
                "Risk assessment".to_string(),
                "Multi-agent coordination".to_string(),
            ],
            challenges: vec![
                "Breaking down complex requirements".to_string(),
                "Coordinating multiple agents".to_string(),
                "Managing dependencies between phases".to_string(),
            ],
            approach: "Analyze requirements, create phased execution plan, coordinate agents"
                .to_string(),
            raw_analysis: format!("Project Manager analysis for task: {}", task.id),
        })
    }
}

/// Project plan structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPlan {
    pub task_id: String,
    pub summary: String,
    pub complexity: ProjectComplexity,
    pub phases: Vec<ProjectPhase>,
    pub overall_risks: Vec<String>,
    pub resource_requirements: ResourceRequirements,
    pub success_metrics: Vec<String>,
    pub estimated_complexity_score: u32,
}

/// Project phase structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPhase {
    pub id: String,
    pub name: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub required_agents: Vec<String>,
    pub deliverables: Vec<String>,
    pub risks: Vec<String>,
    pub success_criteria: Vec<String>,
}

/// Project complexity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectComplexity {
    Low,
    Medium,
    High,
    Critical,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub agents: Vec<String>,
    pub special_resources: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_project_manager_creation() {
        let agent = ProjectManagerAgent::new(None);
        assert_eq!(agent.agent_type(), AgentType::ProjectManager);
        assert_eq!(agent.name(), "Project Manager Agent");
    }

    #[tokio::test]
    async fn test_can_handle_planning_tasks() {
        let agent = ProjectManagerAgent::new(None);

        let task = Task {
            id: "test-1".to_string(),
            agent_type: AgentType::ProjectManager,
            content: "Create project plan for the new feature implementation".to_string(),
            context: std::collections::HashMap::new(),
            priority: crate::models::Priority::High,
            status: crate::models::TaskStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert!(agent.can_handle(&task).await);
    }

    #[tokio::test]
    async fn test_complexity_assessment() {
        let agent = ProjectManagerAgent::new(None);

        assert!(matches!(
            agent.assess_complexity("simple task"),
            ProjectComplexity::Low
        ));

        assert!(matches!(
            agent.assess_complexity("complex refactoring needed"),
            ProjectComplexity::High
        ));

        assert!(matches!(
            agent.assess_complexity("critical security fix"),
            ProjectComplexity::Critical
        ));
    }

    #[tokio::test]
    async fn test_phase_generation() {
        let agent = ProjectManagerAgent::new(None);

        let task = Task {
            id: "test-1".to_string(),
            agent_type: AgentType::SoftwareDeveloper,
            content: "Implement a new REST API endpoint".to_string(),
            context: std::collections::HashMap::new(),
            priority: crate::models::Priority::Medium,
            status: crate::models::TaskStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let phases = agent.generate_phases(&task);

        // Should have at least 3 phases: Analysis, Implementation, Testing
        assert!(phases.len() >= 3);

        // First phase should be analysis
        assert_eq!(phases[0].name, "Requirements Analysis");

        // Phases should have dependencies
        assert!(phases[1].dependencies.contains(&"phase-1".to_string()));
    }
}
