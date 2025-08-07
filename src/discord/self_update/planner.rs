//! Planning & Analysis Phase for self-update requests
//!
//! This module implements the planning phase that analyzes update requests,
//! decomposes them into tasks, and generates implementation plans for user approval.

use super::types::SelfUpdateRequest;
use crate::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Represents a single task in the implementation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedTask {
    /// Unique identifier for the task
    pub id: String,
    /// Human-readable task description
    pub description: String,
    /// Task category (e.g., "code_change", "test_addition", "documentation")
    pub category: TaskCategory,
    /// Estimated complexity using Fibonacci scale (1, 2, 3, 5, 8, 13)
    pub complexity: u8,
    /// Dependencies on other task IDs
    pub dependencies: Vec<String>,
    /// Specific files or components affected
    pub affected_components: Vec<String>,
    /// Validation steps for this task
    pub validation_steps: Vec<String>,
}

/// Categories of tasks that can be planned
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskCategory {
    CodeChange,
    TestAddition,
    Documentation,
    Configuration,
    Refactoring,
    BugFix,
    FeatureAddition,
    Security,
    Performance,
}

/// Risk level assessment for the update using Fibonacci scale
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Unknown,        // ? - Needs investigation
    Low,            // 1 - Minimal risk with negligible impact
    Potential,      // 2 - Some risk but manageable
    Medium,         // 3 - Moderate risk requiring attention
    Certain,        // 5 - High probability of issues
    High,           // 8 - Serious risk with major implications
    Nuclear,        // 13 - Critical risk that could be catastrophic
    DoNotImplement, // ∞ - Unacceptable risk level
}

/// The complete implementation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPlan {
    /// Unique plan identifier
    pub plan_id: String,
    /// Original request that generated this plan
    pub request_id: String,
    /// Human-readable summary of what will be done
    pub summary: String,
    /// Overall risk assessment
    pub risk_level: RiskLevel,
    /// Whether this plan requires human approval regardless of risk
    pub requires_human_approval: bool,
    /// Reason for requiring human approval (if applicable)
    pub approval_reason: Option<String>,
    /// Ordered list of tasks to execute
    pub tasks: Vec<PlannedTask>,
    /// Key risks identified
    pub identified_risks: Vec<String>,
    /// Rollback strategy if something goes wrong
    pub rollback_strategy: String,
    /// Success criteria - how we know the update succeeded
    pub success_criteria: Vec<String>,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// Approval status
    pub approval_status: ApprovalStatus,
}

/// Resource requirements for the update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Which agents are needed
    pub required_agents: Vec<String>,
    /// Any special tools or access needed
    pub special_requirements: Vec<String>,
}

/// Approval status of the plan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected(String), // Reason for rejection
    Modified,         // User requested changes
}

/// The update planner that analyzes requests and creates plans
pub struct UpdatePlanner {
    claude_client: Option<crate::claude_code::ClaudeCodeClient>,
}

impl UpdatePlanner {
    /// Create a new update planner with optional Claude client
    pub fn new(claude_client: Option<crate::claude_code::ClaudeCodeClient>) -> Self {
        Self { claude_client }
    }

    /// Analyze an update request and create an implementation plan
    pub async fn create_plan(&self, request: &SelfUpdateRequest) -> Result<ImplementationPlan> {
        info!(
            "[UpdatePlanner] Creating plan for request: {} ({})",
            request.id, request.codename
        );

        // If Claude client is available, use it for comprehensive planning
        if let Some(claude_client) = &self.claude_client {
            self.create_plan_with_claude(request, claude_client).await
        } else {
            // Fallback to keyword-based analysis
            self.create_plan_with_keywords(request).await
        }
    }
    
    /// Create a comprehensive plan using Claude Code
    async fn create_plan_with_claude(
        &self,
        request: &SelfUpdateRequest,
        claude_client: &crate::claude_code::ClaudeCodeClient,
    ) -> Result<ImplementationPlan> {
        info!("[UpdatePlanner] Using Claude Code for comprehensive planning");
        
        // Build a comprehensive prompt that encapsulates all planning aspects
        let planning_prompt = self.build_comprehensive_planning_prompt(request);
        
        // Create a code generation request for the planning task
        let code_request = crate::claude_code::CodeGenerationRequest {
            language: "json".to_string(), // We want JSON output for structured data
            description: planning_prompt,
            context: std::collections::HashMap::from([
                ("task_type".to_string(), "self_update_planning".to_string()),
                ("request_id".to_string(), request.id.clone()),
                ("codename".to_string(), request.codename.clone()),
            ]),
            existing_code: None,
            requirements: vec![
                "Output must be valid JSON matching the ImplementationPlan structure".to_string(),
                "Include specific, actionable tasks with clear validation steps".to_string(),
                "Assess risks realistically based on the changes".to_string(),
                "Provide accurate time estimates".to_string(),
            ],
            session_id: Some(format!("planning-{}", request.id)),
        };
        
        // Execute the planning request
        match claude_client.generate_code(code_request).await {
            Ok(result) => {
                // Parse the JSON response into an ImplementationPlan
                match serde_json::from_str::<ImplementationPlan>(&result.code) {
                    Ok(mut plan) => {
                        // Ensure some fields are set correctly
                        plan.plan_id = format!("plan-{}-{}", request.codename, chrono::Utc::now().timestamp());
                        plan.request_id = request.id.clone();
                        plan.approval_status = ApprovalStatus::Pending;
                        
                        // Check if human approval is required based on risk and content
                        self.check_human_approval_requirements(&mut plan);
                        
                        info!(
                            "[UpdatePlanner] Claude created plan {} with {} tasks, risk level: {:?}",
                            plan.plan_id,
                            plan.tasks.len(),
                            plan.risk_level
                        );
                        
                        Ok(plan)
                    }
                    Err(e) => {
                        warn!("[UpdatePlanner] Failed to parse Claude's JSON response: {}", e);
                        // Fallback to keyword analysis
                        self.create_plan_with_keywords(request).await
                    }
                }
            }
            Err(e) => {
                warn!("[UpdatePlanner] Claude Code planning failed: {}", e);
                // Fallback to keyword analysis
                self.create_plan_with_keywords(request).await
            }
        }
    }
    
    /// Build a comprehensive prompt for Claude Code planning
    fn build_comprehensive_planning_prompt(&self, request: &SelfUpdateRequest) -> String {
        let combined_messages = request.combined_messages.join("\n");
        
        format!(
            r#"You are a Project Manager AI agent specialized in planning self-update operations for the Spiral Core system.
            
Analyze the following self-update request and create a comprehensive implementation plan in JSON format.

## Update Request Details:
- Request ID: {}
- Codename: {}
- Description: {}
- User Messages:
{}

## Your Task:
Create a detailed implementation plan that includes:

1. **Task Decomposition**: Break down the request into specific, actionable tasks with:
   - Unique task IDs (task-1, task-2, etc.)
   - Clear descriptions of what needs to be done
   - Task category (CodeChange, TestAddition, Documentation, Configuration, Refactoring, BugFix, FeatureAddition, Security, Performance)
   - Complexity rating using Fibonacci scale (1, 2, 3, 5, 8, 13 - where 13 is extremely complex)
   - Dependencies on other tasks
   - Affected components (file paths or module names)
   - Validation steps to verify task completion

2. **Risk Assessment**: 
   - Identify the overall risk level using Fibonacci scale:
     * Unknown (?) - Needs investigation
     * Low (1) - Minimal risk with negligible impact
     * Potential (2) - Some risk but manageable
     * Medium (3) - Moderate risk requiring attention
     * Certain (5) - High probability of issues
     * High (8) - Serious risk with major implications
     * Nuclear (13) - Critical risk that could be catastrophic
     * DoNotImplement (∞) - Unacceptable risk level
   - List specific risks associated with the changes
   - Consider security implications, breaking changes, and system stability

3. **Success Criteria**:
   - Define clear, measurable criteria for success
   - Include compilation checks, test requirements, and quality standards

4. **Resource Requirements**:
   - Identify which agents or tools are needed
   - Note any special access or permissions required

## Output Format:
Return a JSON object matching this structure:
{{
    "summary": "Brief summary of what will be done",
    "risk_level": "Unknown|Low|Potential|Medium|Certain|High|Nuclear|DoNotImplement",
    "tasks": [
        {{
            "id": "task-1",
            "description": "Clear task description",
            "category": "TaskCategory",
            "complexity": 3,  // Use Fibonacci scale: 1, 2, 3, 5, 8, 13
            "dependencies": [],
            "affected_components": ["file1.rs", "module2"],
            "validation_steps": ["Step 1", "Step 2"]
        }}
    ],
    "identified_risks": ["Risk 1", "Risk 2"],
    "rollback_strategy": "How to undo changes if needed",
    "success_criteria": ["Criterion 1", "Criterion 2"],
    "resource_requirements": {{
        "required_agents": ["Claude Code"],
        "special_requirements": []
    }}
}}

## Important Considerations:
- Be thorough but practical in your planning
- Consider the existing codebase structure and conventions
- Include tests for any new functionality
- Account for validation and security checks
- Think about edge cases and error handling
- Ensure the plan is specific to the Spiral Core Rust project

Analyze the request carefully and provide a comprehensive plan that a developer can follow step-by-step."#,
            request.id,
            request.codename,
            request.description,
            combined_messages
        )
    }
    
    /// Fallback method using keyword analysis
    async fn create_plan_with_keywords(&self, request: &SelfUpdateRequest) -> Result<ImplementationPlan> {
        info!("[UpdatePlanner] Using keyword-based planning (fallback)");
        
        // Analyze the request to understand what's being asked
        let analysis = Self::analyze_request(request)?;
        
        // Decompose into specific tasks
        let tasks = Self::decompose_into_tasks(&analysis, request)?;
        
        // Assess overall risk
        let risk_level = Self::assess_risk(&tasks, &analysis);
        
        // Identify specific risks
        let identified_risks = Self::identify_risks(&tasks, &analysis, request);
        
        // Define success criteria
        let success_criteria = Self::define_success_criteria(&tasks, request);
        
        // Create the plan
        let mut plan = ImplementationPlan {
            plan_id: format!("plan-{}-{}", request.codename, chrono::Utc::now().timestamp()),
            request_id: request.id.clone(),
            summary: Self::generate_summary(&tasks, request),
            risk_level,
            tasks,
            identified_risks,
            rollback_strategy: "Git snapshot created before changes. Can rollback to previous commit if needed.".to_string(),
            success_criteria,
            resource_requirements: ResourceRequirements {
                required_agents: vec!["Claude Code".to_string()],
                special_requirements: vec![],
            },
            approval_status: ApprovalStatus::Pending,
            requires_human_approval: false, // Will be checked next
            approval_reason: None,
        };
        
        // Check if human approval is required
        self.check_human_approval_requirements(&mut plan);
        
        info!(
            "[UpdatePlanner] Created keyword-based plan {} with {} tasks, risk level: {:?}",
            plan.plan_id,
            plan.tasks.len(),
            plan.risk_level
        );
        
        Ok(plan)
    }
    
    /// Analyze the request to understand scope and intent
    fn analyze_request(request: &SelfUpdateRequest) -> Result<RequestAnalysis> {
        debug!("[UpdatePlanner] Analyzing request: {}", request.description);
        
        let combined_text = format!(
            "{}\n{}",
            request.description,
            request.combined_messages.join("\n")
        );
        
        // Simple keyword-based analysis for now
        // In a real implementation, this would use NLP or Claude to understand intent
        let analysis = RequestAnalysis {
            primary_intent: Self::detect_primary_intent(&combined_text),
            scope: Self::detect_scope(&combined_text),
            affected_areas: Self::detect_affected_areas(&combined_text),
            complexity_factors: Self::detect_complexity_factors(&combined_text),
        };
        
        Ok(analysis)
    }
    
    /// Decompose the request into specific tasks
    fn decompose_into_tasks(
        analysis: &RequestAnalysis,
        request: &SelfUpdateRequest,
    ) -> Result<Vec<PlannedTask>> {
        let mut tasks = Vec::new();
        let mut task_counter = 1;
        
        // Based on the analysis, create appropriate tasks
        match analysis.primary_intent {
            Intent::BugFix => {
                tasks.push(PlannedTask {
                    id: format!("task-{}", task_counter),
                    description: format!("Identify and fix the bug: {}", request.description),
                    category: TaskCategory::BugFix,
                    complexity: 3,
                    dependencies: vec![],
                    affected_components: analysis.affected_areas.clone(),
                    validation_steps: vec![
                        "Run existing tests to verify fix".to_string(),
                        "Add regression test for the bug".to_string(),
                    ],
                });
                task_counter += 1;
                
                tasks.push(PlannedTask {
                    id: format!("task-{}", task_counter),
                    description: "Add tests to prevent regression".to_string(),
                    category: TaskCategory::TestAddition,
                    complexity: 2,
                    dependencies: vec!["task-1".to_string()],
                    affected_components: vec!["tests".to_string()],
                    validation_steps: vec!["Ensure new tests pass".to_string()],
                });
            }
            Intent::FeatureAddition => {
                tasks.push(PlannedTask {
                    id: format!("task-{}", task_counter),
                    description: format!("Implement new feature: {}", request.description),
                    category: TaskCategory::FeatureAddition,
                    complexity: 4,
                    dependencies: vec![],
                    affected_components: analysis.affected_areas.clone(),
                    validation_steps: vec![
                        "Feature works as described".to_string(),
                        "Integration with existing code verified".to_string(),
                    ],
                });
                task_counter += 1;
                
                tasks.push(PlannedTask {
                    id: format!("task-{}", task_counter),
                    description: "Add tests for new feature".to_string(),
                    category: TaskCategory::TestAddition,
                    complexity: 3,
                    dependencies: vec!["task-1".to_string()],
                    affected_components: vec!["tests".to_string()],
                    validation_steps: vec!["All tests pass".to_string()],
                });
                task_counter += 1;
                
                tasks.push(PlannedTask {
                    id: format!("task-{}", task_counter),
                    description: "Update documentation".to_string(),
                    category: TaskCategory::Documentation,
                    complexity: 1,
                    dependencies: vec!["task-1".to_string()],
                    affected_components: vec!["README.md".to_string(), "docs".to_string()],
                    validation_steps: vec!["Documentation is accurate".to_string()],
                });
            }
            Intent::Improvement => {
                tasks.push(PlannedTask {
                    id: format!("task-{}", task_counter),
                    description: format!("Improve: {}", request.description),
                    category: TaskCategory::Refactoring,
                    complexity: 3,
                    dependencies: vec![],
                    affected_components: analysis.affected_areas.clone(),
                    validation_steps: vec![
                        "Functionality unchanged".to_string(),
                        "Performance or quality improved".to_string(),
                    ],
                });
            }
            _ => {
                // Generic task for unclear intents
                tasks.push(PlannedTask {
                    id: format!("task-{}", task_counter),
                    description: request.description.clone(),
                    category: TaskCategory::CodeChange,
                    complexity: 3,
                    dependencies: vec![],
                    affected_components: vec!["unknown".to_string()],
                    validation_steps: vec![
                        "Changes implemented as requested".to_string(),
                        "No regressions introduced".to_string(),
                    ],
                });
            }
        }
        
        Ok(tasks)
    }
    
    /// Assess the overall risk level using Fibonacci scale
    fn assess_risk(tasks: &[PlannedTask], analysis: &RequestAnalysis) -> RiskLevel {
        let max_complexity = tasks.iter().map(|t| t.complexity).max().unwrap_or(1);
        let total_components = analysis.affected_areas.len();
        
        // Check for unknown factors first
        if analysis.affected_areas.contains(&"unknown".to_string()) {
            return RiskLevel::Unknown;
        }
        
        // Critical scope always has high risk
        if analysis.scope == Scope::Critical {
            if max_complexity >= 8 {
                RiskLevel::Nuclear  // Critical + very complex = nuclear risk
            } else if max_complexity >= 5 {
                RiskLevel::High
            } else {
                RiskLevel::Certain  // Critical changes always have certain risk
            }
        } else if max_complexity >= 13 {
            RiskLevel::Nuclear  // Extremely complex tasks are nuclear risk
        } else if max_complexity >= 8 || total_components > 8 {
            RiskLevel::High  // Very complex or many components
        } else if max_complexity >= 5 || total_components > 5 {
            RiskLevel::Certain  // Complex tasks have certain risk of issues
        } else if max_complexity >= 3 || total_components > 3 {
            RiskLevel::Medium
        } else if max_complexity == 2 || total_components == 2 {
            RiskLevel::Potential
        } else {
            RiskLevel::Low
        }
    }
    
    /// Identify specific risks
    fn identify_risks(
        tasks: &[PlannedTask],
        analysis: &RequestAnalysis,
        _request: &SelfUpdateRequest,
    ) -> Vec<String> {
        let mut risks = Vec::new();
        
        // Check for high complexity
        if tasks.iter().any(|t| t.complexity >= 4) {
            risks.push("High complexity changes may introduce bugs".to_string());
        }
        
        // Check for critical areas
        if analysis.scope == Scope::Critical {
            risks.push("Changes affect critical system components".to_string());
        }
        
        // Check for test coverage
        if !tasks.iter().any(|t| t.category == TaskCategory::TestAddition) {
            risks.push("No tests planned - may miss regressions".to_string());
        }
        
        // Check for broad impact
        if analysis.affected_areas.len() > 3 {
            risks.push("Changes affect multiple components".to_string());
        }
        
        if risks.is_empty() {
            risks.push("Low risk - straightforward changes".to_string());
        }
        
        risks
    }
    
    /// Define success criteria
    fn define_success_criteria(tasks: &[PlannedTask], _request: &SelfUpdateRequest) -> Vec<String> {
        let mut criteria = vec![
            "All compilation checks pass".to_string(),
            "All existing tests continue to pass".to_string(),
        ];
        
        // Add task-specific criteria
        for task in tasks {
            criteria.extend(task.validation_steps.clone());
        }
        
        criteria.push("No performance regressions".to_string());
        criteria.push("Code follows project standards".to_string());
        
        criteria
    }
    
    /// Generate a summary of the plan
    fn generate_summary(tasks: &[PlannedTask], request: &SelfUpdateRequest) -> String {
        let task_summary = tasks
            .iter()
            .map(|t| format!("- {}", t.description))
            .collect::<Vec<_>>()
            .join("\n");
        
        format!(
            "Plan to address: {}\n\nTasks:\n{}",
            request.description, task_summary
        )
    }
    
    /// Detect the primary intent of the request
    fn detect_primary_intent(text: &str) -> Intent {
        let text_lower = text.to_lowercase();
        
        if text_lower.contains("fix") || text_lower.contains("bug") || text_lower.contains("error") {
            Intent::BugFix
        } else if text_lower.contains("add") || text_lower.contains("implement") || text_lower.contains("create") {
            Intent::FeatureAddition
        } else if text_lower.contains("improve") || text_lower.contains("enhance") || text_lower.contains("optimize") {
            Intent::Improvement
        } else if text_lower.contains("update") || text_lower.contains("change") {
            Intent::Update
        } else {
            Intent::Unknown
        }
    }
    
    /// Detect the scope of changes
    fn detect_scope(text: &str) -> Scope {
        let text_lower = text.to_lowercase();
        
        if text_lower.contains("security") || text_lower.contains("auth") || text_lower.contains("critical") {
            Scope::Critical
        } else if text_lower.contains("api") || text_lower.contains("interface") || text_lower.contains("public") {
            Scope::Public
        } else if text_lower.contains("internal") || text_lower.contains("private") {
            Scope::Internal
        } else {
            Scope::Unknown
        }
    }
    
    /// Detect affected areas
    fn detect_affected_areas(text: &str) -> Vec<String> {
        let mut areas = Vec::new();
        let text_lower = text.to_lowercase();
        
        // Check for common component mentions
        if text_lower.contains("discord") {
            areas.push("discord".to_string());
        }
        if text_lower.contains("api") {
            areas.push("api".to_string());
        }
        if text_lower.contains("agent") {
            areas.push("agents".to_string());
        }
        if text_lower.contains("test") {
            areas.push("tests".to_string());
        }
        if text_lower.contains("validation") || text_lower.contains("validate") {
            areas.push("validation".to_string());
        }
        if text_lower.contains("config") {
            areas.push("configuration".to_string());
        }
        
        if areas.is_empty() {
            areas.push("unknown".to_string());
        }
        
        areas
    }
    
    /// Detect complexity factors
    fn detect_complexity_factors(text: &str) -> Vec<String> {
        let mut factors = Vec::new();
        let text_lower = text.to_lowercase();
        
        if text_lower.contains("refactor") {
            factors.push("refactoring required".to_string());
        }
        if text_lower.contains("multiple") || text_lower.contains("several") {
            factors.push("multiple components affected".to_string());
        }
        if text_lower.contains("async") || text_lower.contains("concurrent") {
            factors.push("async/concurrent code".to_string());
        }
        if text_lower.contains("breaking") {
            factors.push("breaking changes".to_string());
        }
        
        factors
    }
    
    /// Check if the plan requires human approval based on risk and content
    fn check_human_approval_requirements(&self, plan: &mut ImplementationPlan) {
        // Always require human approval for high-risk changes
        if matches!(plan.risk_level, RiskLevel::High | RiskLevel::Nuclear | RiskLevel::DoNotImplement) {
            plan.requires_human_approval = true;
            plan.approval_reason = Some(format!(
                "High risk level ({:?}) requires human review",
                plan.risk_level
            ));
            return;
        }
        
        // Check for critical task categories
        let has_critical_tasks = plan.tasks.iter().any(|task| {
            matches!(task.category, TaskCategory::Security | TaskCategory::Configuration)
        });
        
        if has_critical_tasks {
            plan.requires_human_approval = true;
            plan.approval_reason = Some("Security or configuration changes require human review".to_string());
            return;
        }
        
        // Check for changes to critical files
        let critical_paths = vec![
            ".env", "Cargo.toml", "package.json", ".github", 
            "src/main.rs", "src/lib.rs", "src/discord/self_update/"
        ];
        
        let touches_critical = plan.tasks.iter().any(|task| {
            task.affected_components.iter().any(|component| {
                critical_paths.iter().any(|critical| component.contains(critical))
            })
        });
        
        if touches_critical {
            plan.requires_human_approval = true;
            plan.approval_reason = Some("Changes to critical system files require human review".to_string());
            return;
        }
        
        // Check total complexity
        let total_complexity: u32 = plan.tasks.iter().map(|t| t.complexity as u32).sum();
        if total_complexity > 21 { // Fibonacci: 13 + 8
            plan.requires_human_approval = true;
            plan.approval_reason = Some(format!(
                "High total complexity ({}) requires human review",
                total_complexity
            ));
            return;
        }
        
        // Check for many files being changed
        if plan.tasks.len() > 10 {
            plan.requires_human_approval = true;
            plan.approval_reason = Some(format!(
                "Large number of tasks ({}) requires human review",
                plan.tasks.len()
            ));
            return;
        }
        
        // Default: no special human approval required beyond normal flow
        plan.requires_human_approval = false;
        plan.approval_reason = None;
    }
}

/// Internal struct for request analysis
#[derive(Debug)]
struct RequestAnalysis {
    primary_intent: Intent,
    scope: Scope,
    affected_areas: Vec<String>,
    complexity_factors: Vec<String>,
}

#[derive(Debug, PartialEq)]
enum Intent {
    BugFix,
    FeatureAddition,
    Improvement,
    Update,
    Unknown,
}

#[derive(Debug, PartialEq)]
enum Scope {
    Critical,
    Public,
    Internal,
    Unknown,
}

/// Format risk level with appropriate emoji and description
fn format_risk_level(risk: &RiskLevel) -> String {
    match risk {
        RiskLevel::Unknown => "❓ Unknown - Needs investigation".to_string(),
        RiskLevel::Low => "✅ Low (1) - Minimal risk".to_string(),
        RiskLevel::Potential => "🟡 Potential (2) - Some manageable risk".to_string(),
        RiskLevel::Medium => "⚠️ Medium (3) - Requires attention".to_string(),
        RiskLevel::Certain => "🟠 Certain (5) - High probability of issues".to_string(),
        RiskLevel::High => "🔴 High (8) - Serious risk".to_string(),
        RiskLevel::Nuclear => "☢️ Nuclear (13) - Critical risk".to_string(),
        RiskLevel::DoNotImplement => "🚫 Do Not Implement (∞) - Unacceptable risk".to_string(),
    }
}

/// Format a plan for Discord display
pub fn format_plan_for_discord(plan: &ImplementationPlan) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("## 📋 Implementation Plan: {}\n\n", plan.plan_id));
    
    // Show human approval requirement prominently
    if plan.requires_human_approval {
        output.push_str("⚠️ **HUMAN APPROVAL REQUIRED** ⚠️\n");
        if let Some(ref reason) = plan.approval_reason {
            output.push_str(&format!("**Reason**: {}\n", reason));
        }
        output.push_str("\n");
    }
    
    output.push_str(&format!("**Summary**: {}\n\n", plan.summary));
    output.push_str(&format!("**Risk Level**: {}\n\n", format_risk_level(&plan.risk_level)));
    
    output.push_str("### 📝 Tasks\n");
    for (i, task) in plan.tasks.iter().enumerate() {
        output.push_str(&format!(
            "{}. **{}** (Complexity: {})\n   {}\n",
            i + 1,
            task.description,
            task.complexity,
            task.validation_steps.join(", ")
        ));
    }
    
    output.push_str("\n### ⚠️ Identified Risks\n");
    for risk in &plan.identified_risks {
        output.push_str(&format!("- {}\n", risk));
    }
    
    output.push_str("\n### ✅ Success Criteria\n");
    for criterion in &plan.success_criteria {
        output.push_str(&format!("- {}\n", criterion));
    }
    
    output.push_str(&format!("\n**Rollback Strategy**: {}\n", plan.rollback_strategy));
    
    output.push_str("\n---\n");
    output.push_str("Reply with **approve** to proceed or **modify** to request changes.");
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plan_creation() {
        let request = SelfUpdateRequest {
            id: "test-123".to_string(),
            codename: "fix-bug".to_string(),
            description: "Fix the memory leak in message handler".to_string(),
            user_id: 123456,
            channel_id: 789012,
            message_id: 345678,
            combined_messages: vec!["There's a memory leak when processing messages".to_string()],
            timestamp: chrono::Utc::now().to_rfc3339(),
            retry_count: 0,
            status: crate::discord::self_update::UpdateStatus::Queued,
        };
        
        let planner = UpdatePlanner::new(None);
        let plan = planner.create_plan(&request).await.unwrap();
        
        assert!(!plan.tasks.is_empty());
        assert_eq!(plan.request_id, "test-123");
        assert!(plan.tasks.iter().any(|t| t.category == TaskCategory::BugFix));
    }
    
    #[test]
    fn test_intent_detection() {
        assert_eq!(
            UpdatePlanner::detect_primary_intent("fix the bug in the code"),
            Intent::BugFix
        );
        assert_eq!(
            UpdatePlanner::detect_primary_intent("add a new feature"),
            Intent::FeatureAddition
        );
        assert_eq!(
            UpdatePlanner::detect_primary_intent("improve performance"),
            Intent::Improvement
        );
    }
}