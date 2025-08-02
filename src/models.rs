use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

/// Represents a task to be processed by an agent
///
/// Tasks are the fundamental unit of work in the Spiral Core system.
/// Each task is assigned to a specific agent type and includes priority
/// and status tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub agent_type: AgentType,
    pub content: String,
    pub context: HashMap<String, String>,
    pub priority: Priority,
    pub status: TaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Types of specialized agents available in the system
///
/// Each agent type has specific capabilities and responsibilities
/// within the orchestration system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    SoftwareDeveloper,
    ProjectManager,
    QualityAssurance,
    DecisionMaker,
    CreativeInnovator,
    ProcessCoach,
    SpiralKing,
}

/// Task priority levels
///
/// Higher priority tasks are processed before lower priority ones
/// when multiple tasks are queued.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Current status of a task in the processing pipeline
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Result of a completed task execution
///
/// Contains the outcome of task processing along with any metadata
/// generated during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub agent_type: AgentType,
    pub result: TaskExecutionResult,
    pub metadata: HashMap<String, String>,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskExecutionResult {
    Success {
        output: String,
        files_created: Vec<String>,
        files_modified: Vec<String>,
    },
    Failure {
        error: String,
        partial_output: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub name: String,
    pub description: String,
    pub supported_languages: Vec<String>,
    pub required_tools: Vec<String>,
}

/// Represents a message from Discord that needs processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordMessage {
    pub content: String,
    pub author_id: u64,
    pub channel_id: u64,
    pub mentioned_agents: Vec<AgentType>,
    pub message_id: u64,
}

impl Task {
    /// Creates a new task with the specified agent type, content, and priority
    ///
    /// The task is assigned a unique ID and initialized with pending status.
    pub fn new(agent_type: AgentType, content: String, priority: Priority) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            agent_type,
            content,
            context: HashMap::new(),
            priority,
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }
}

impl FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SoftwareDeveloper" => Ok(AgentType::SoftwareDeveloper),
            "ProjectManager" => Ok(AgentType::ProjectManager),
            "QualityAssurance" => Ok(AgentType::QualityAssurance),
            "DecisionMaker" => Ok(AgentType::DecisionMaker),
            "CreativeInnovator" => Ok(AgentType::CreativeInnovator),
            "ProcessCoach" => Ok(AgentType::ProcessCoach),
            "SpiralKing" => Ok(AgentType::SpiralKing),
            _ => Err(format!("Unknown agent type: {s}")),
        }
    }
}

impl AgentType {
    pub fn from_mention(mention: &str) -> Option<Self> {
        match mention.to_lowercase().as_str() {
            "dev" | "developer" | "code" => Some(AgentType::SoftwareDeveloper),
            "pm" | "manager" | "project" => Some(AgentType::ProjectManager),
            "qa" | "quality" | "test" => Some(AgentType::QualityAssurance),
            "decision" | "decide" => Some(AgentType::DecisionMaker),
            "creative" | "innovate" => Some(AgentType::CreativeInnovator),
            "coach" | "process" => Some(AgentType::ProcessCoach),
            "king" | "spiralking" | "lordgenome" => Some(AgentType::SpiralKing),
            _ => None,
        }
    }

    pub fn capabilities(&self) -> AgentCapability {
        match self {
            AgentType::SoftwareDeveloper => AgentCapability {
                name: "Software Developer".to_string(),
                description:
                    "Autonomous code generation with language detection and Claude Code integration"
                        .to_string(),
                supported_languages: vec![
                    "rust".to_string(),
                    "python".to_string(),
                    "javascript".to_string(),
                    "typescript".to_string(),
                    "go".to_string(),
                    "java".to_string(),
                    "cpp".to_string(),
                    "c".to_string(),
                ],
                required_tools: vec!["claude_code_client".to_string()],
            },
            AgentType::ProjectManager => AgentCapability {
                name: "Project Manager".to_string(),
                description: "Strategic analysis and task coordination".to_string(),
                supported_languages: vec![],
                required_tools: vec![
                    "claude_code_client".to_string(),
                    "github_client".to_string(),
                ],
            },
            AgentType::SpiralKing => AgentCapability {
                name: "The Immortal Spiral King".to_string(),
                description: "Comprehensive code review with deep architectural analysis and long-term system stability assessment from a millennial perspective".to_string(),
                supported_languages: vec![
                    "rust".to_string(),
                    "python".to_string(),
                    "javascript".to_string(),
                    "typescript".to_string(),
                    "go".to_string(),
                    "java".to_string(),
                    "c".to_string(),
                    "cpp".to_string(),
                ],
                required_tools: vec![
                    "claude_code_client".to_string(),
                    "static_analysis".to_string(),
                    "architectural_review".to_string(),
                ],
            },
            _ => AgentCapability {
                name: format!("{self:?}"),
                description: "Agent capability not yet implemented".to_string(),
                supported_languages: vec![],
                required_tools: vec![],
            },
        }
    }
}
