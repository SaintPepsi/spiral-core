/// 🛠️ TASK UTILITIES: Extracted via 3-strikes abstraction rule  
/// WHY SEPARATE FILE: Task processing patterns appear 3+ times across agents
/// AUDIT: Verify consistent context/metadata patterns across all agents

use std::collections::HashMap;
use crate::models::{Task, TaskResult, TaskExecutionResult, AgentType, Priority};
use crate::SpiralError;

/// 📋 CONTEXT BUILDER: Standardized task context enrichment
/// DECISION REASONING: Ensures consistent context format across all agent types
/// Why centralized: Prevents context inconsistencies, enables easy context enhancement
pub fn build_enriched_context(
    task: &Task, 
    agent_type: AgentType,
    additional_context: Option<HashMap<String, String>>
) -> HashMap<String, String> {
    let mut context = HashMap::new();
    
    // 📥 COPY EXISTING CONTEXT: Preserve user-provided context
    for (key, value) in &task.context {
        context.insert(key.clone(), value.clone());
    }
    
    // 🏷️ AGENT IDENTIFICATION: Standard agent metadata
    context.insert("agent_type".to_string(), format!("{:?}", agent_type));
    context.insert("task_id".to_string(), task.id.clone());
    context.insert("priority".to_string(), format!("{:?}", task.priority));
    
    // ⏰ TEMPORAL CONTEXT: Task timing information
    context.insert("created_at".to_string(), task.created_at.to_rfc3339());
    context.insert("updated_at".to_string(), task.updated_at.to_rfc3339());
    
    // 📊 TASK METADATA: Core task information
    context.insert("task_status".to_string(), format!("{:?}", task.status));
    
    // 🔧 ADDITIONAL CONTEXT: Merge any agent-specific context
    if let Some(extra) = additional_context {
        for (key, value) in extra {
            context.insert(key, value);
        }
    }
    
    context
}

/// ✅ SUCCESS RESULT BUILDER: Standardized successful task result creation
/// INLINE REASONING: Reduces boilerplate and ensures consistent result format
/// Audit: Verify all success paths use this builder for consistency
pub fn create_success_result(
    task: &Task,
    agent_type: AgentType,
    output: String,
    files_created: Vec<String>,
    files_modified: Vec<String>,
    custom_metadata: Option<HashMap<String, String>>,
) -> TaskResult {
    let mut metadata = HashMap::new();
    
    // 📈 STANDARD METRICS: Common performance and output metrics
    metadata.insert("output_length".to_string(), output.len().to_string());
    metadata.insert("files_created_count".to_string(), files_created.len().to_string());
    metadata.insert("files_modified_count".to_string(), files_modified.len().to_string());
    
    // 🔧 CUSTOM METADATA: Merge agent-specific metadata
    if let Some(custom) = custom_metadata {
        for (key, value) in custom {
            metadata.insert(key, value);
        }
    }
    
    TaskResult {
        task_id: task.id.clone(),
        agent_type,
        result: TaskExecutionResult::Success {
            output,
            files_created,
            files_modified,
        },
        metadata,
        completed_at: chrono::Utc::now(),
    }
}

/// ❌ FAILURE RESULT BUILDER: Standardized failed task result creation
/// INLINE REASONING: Consistent error reporting across all agent types
/// Why separate: Error handling patterns should be uniform for debugging
pub fn create_failure_result(
    task: &Task,
    agent_type: AgentType,
    error: &SpiralError,
    partial_output: Option<String>,
    custom_metadata: Option<HashMap<String, String>>,
) -> TaskResult {
    let mut metadata = HashMap::new();
    
    // 🚨 ERROR METADATA: Standard error classification and debugging info
    metadata.insert("error_type".to_string(), format!("{error:?}"));
    metadata.insert("has_partial_output".to_string(), partial_output.is_some().to_string());
    
    // 🔧 CUSTOM METADATA: Agent-specific error context
    if let Some(custom) = custom_metadata {
        for (key, value) in custom {
            metadata.insert(key, value);
        }
    }
    
    TaskResult {
        task_id: task.id.clone(),
        agent_type,
        result: TaskExecutionResult::Failure {
            error: error.to_string(),
            partial_output,
        },
        metadata,
        completed_at: chrono::Utc::now(),
    }
}

/// 🎯 PRIORITY ANALYSIS: Determine task priority based on content and context
/// WHY INLINE: Simple heuristic that may evolve into more sophisticated analysis
/// Future: Consider ML-based priority classification
pub fn analyze_task_priority(content: &str, context: &HashMap<String, String>) -> Priority {
    let content_lower = content.to_lowercase();
    
    // 🚨 HIGH PRIORITY INDICATORS: Security, critical bugs, system failures
    if content_lower.contains("critical") 
        || content_lower.contains("urgent") 
        || content_lower.contains("security")
        || content_lower.contains("vulnerability")
        || content_lower.contains("production")
        || content_lower.contains("outage") {
        return Priority::High;
    }
    
    // ⚠️ MEDIUM PRIORITY INDICATORS: Features, enhancements, non-critical bugs
    if content_lower.contains("feature") 
        || content_lower.contains("enhancement")
        || content_lower.contains("improvement")
        || content_lower.contains("optimize") {
        return Priority::Medium;
    }
    
    // 📝 CONTEXT-BASED PRIORITY: Check for explicit priority indicators
    if let Some(priority_hint) = context.get("priority_hint") {
        match priority_hint.to_lowercase().as_str() {
            "high" | "urgent" | "critical" => return Priority::High,
            "medium" | "normal" => return Priority::Medium,
            "low" | "nice-to-have" => return Priority::Low,
            _ => {}
        }
    }
    
    // 🔧 DEFAULT PRIORITY: Medium for balanced processing
    Priority::Medium
}

/// 📊 TASK COMPLEXITY ESTIMATION: Estimate development effort required
/// INLINE REASONING: Helps with resource allocation and time estimation
/// Returns: (estimated_minutes, complexity_factors)
pub fn estimate_task_complexity(content: &str, context: &HashMap<String, String>) -> (u32, Vec<String>) {
    let mut base_minutes = 30; // Base assumption for simple tasks
    let mut complexity_factors = Vec::new();
    let content_lower = content.to_lowercase();
    
    // 🧩 COMPLEXITY MULTIPLIERS: Different factors that increase development time
    if content_lower.contains("api") || content_lower.contains("endpoint") {
        base_minutes += 45;
        complexity_factors.push("API development".to_string());
    }
    
    if content_lower.contains("database") || content_lower.contains("sql") {
        base_minutes += 60;
        complexity_factors.push("Database integration".to_string());
    }
    
    if content_lower.contains("test") || content_lower.contains("testing") {
        base_minutes += 30;
        complexity_factors.push("Test implementation".to_string());
    }
    
    if content_lower.contains("security") || content_lower.contains("auth") {
        base_minutes += 90;
        complexity_factors.push("Security implementation".to_string());
    }
    
    if content_lower.contains("async") || content_lower.contains("concurrent") {
        base_minutes += 45;
        complexity_factors.push("Async/concurrent programming".to_string());
    }
    
    // 📝 CONTEXT-BASED ADJUSTMENTS: Use existing code context to adjust estimates
    if context.get("existing_code").is_some() {
        base_minutes += 20; // Additional time for understanding existing code
        complexity_factors.push("Code modification/integration".to_string());
    }
    
    if complexity_factors.is_empty() {
        complexity_factors.push("Simple implementation".to_string());
    }
    
    (base_minutes, complexity_factors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{TaskStatus, Priority};
    use chrono::Utc;

    fn create_test_task() -> Task {
        Task {
            id: "test-123".to_string(),
            agent_type: AgentType::SoftwareDeveloper,
            content: "Create a test function".to_string(),
            priority: Priority::Medium,
            status: TaskStatus::Pending,
            context: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_build_enriched_context() {
        let task = create_test_task();
        let context = build_enriched_context(&task, AgentType::SoftwareDeveloper, None);
        
        assert_eq!(context.get("agent_type").unwrap(), "SoftwareDeveloper");
        assert_eq!(context.get("task_id").unwrap(), "test-123");
        assert!(context.contains_key("created_at"));
    }

    #[test]
    fn test_priority_analysis() {
        assert_eq!(analyze_task_priority("critical security fix", &HashMap::new()), Priority::High);
        assert_eq!(analyze_task_priority("add new feature", &HashMap::new()), Priority::Medium);
        assert_eq!(analyze_task_priority("simple documentation update", &HashMap::new()), Priority::Medium);
    }

    #[test]
    fn test_complexity_estimation() {
        let (minutes, factors) = estimate_task_complexity("create api endpoint with database", &HashMap::new());
        assert!(minutes > 100); // Should account for API + database complexity
        assert!(factors.len() >= 2); // Should identify multiple complexity factors
    }
}