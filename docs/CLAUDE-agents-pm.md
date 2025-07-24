# CLAUDE-agents-pm.md

**Purpose**: Project Manager agent implementation for strategic analysis and coordination patterns
**Dependencies**: [Coding Standards](CLAUDE-core-coding-standards.md), [Developer Agent](CLAUDE-agents-developer.md)
**Updated**: 2024-07-24

## Quick Start

The Project Manager Agent specializes in strategic analysis, task coordination, and architectural decision-making:

```rust
impl ProjectManagerAgent {
    pub async fn analyze_request(&mut self, request: &str) -> Result<StrategicAnalysis, AgentError>;
    pub async fn coordinate_multi_agent_task(&mut self, task: &ComplexTask) -> Result<CoordinationPlan, AgentError>;
    pub async fn evaluate_architecture_decision(&mut self, context: &ArchitectureContext) -> Result<Recommendation, AgentError>;
}
```

## Core Architecture

### Strategic Analysis Focus

The PM Agent's primary responsibility is high-level strategic thinking and coordination:

```rust
// ✅ Good - Single responsibility for strategic analysis
pub struct ProjectManagerAgent {
    claude_client: Box<dyn ClaudeClient>,
    analysis_templates: AnalysisTemplateEngine,
    decision_framework: ArchitectureDecisionFramework,
    coordination_history: Vec<CoordinationDecision>,
    prompts_remaining: u32,
}

impl ProjectManagerAgent {
    pub async fn analyze_request(&mut self, request: &str) -> Result<StrategicAnalysis, AgentError> {
        // Focuses on strategic analysis, not implementation
        let context = self.build_strategic_context(request)?;

        let analysis_prompt = self.analysis_templates.render("strategic_analysis", &context)?;

        let claude_result = self.claude_client
            .execute_task(analysis_prompt, ProgrammingLanguage::ArchitecturalAnalysis)
            .await?;

        self.prompts_remaining -= 1;
        Ok(StrategicAnalysis::from_claude_response(claude_result))
    }
}
```

## Strategic Analysis Patterns

### Architecture Decision Framework

```rust
pub struct ArchitectureDecisionFramework {
    decision_criteria: Vec<DecisionCriterion>,
    risk_assessment_templates: HashMap<String, String>,
    trade_off_matrices: Vec<TradeOffMatrix>,
}

impl ArchitectureDecisionFramework {
    pub fn evaluate_decision(&self, context: &ArchitectureContext) -> DecisionAnalysis {
        let mut analysis = DecisionAnalysis::new();

        // Evaluate technical criteria
        for criterion in &self.decision_criteria {
            let score = criterion.evaluate(&context);
            analysis.add_criterion_score(criterion.name.clone(), score);
        }

        // Assess risks
        let risks = self.assess_risks(&context);
        analysis.set_risk_assessment(risks);

        // Analyze trade-offs
        let trade_offs = self.analyze_trade_offs(&context);
        analysis.set_trade_offs(trade_offs);

        analysis
    }

    fn assess_risks(&self, context: &ArchitectureContext) -> RiskAssessment {
        RiskAssessment {
            technical_risks: self.evaluate_technical_risks(context),
            operational_risks: self.evaluate_operational_risks(context),
            business_risks: self.evaluate_business_risks(context),
            mitigation_strategies: self.suggest_mitigations(context),
        }
    }
}
```

### Multi-Agent Coordination

```rust
impl ProjectManagerAgent {
    pub async fn coordinate_multi_agent_task(&mut self, task: &ComplexTask) -> Result<CoordinationPlan, AgentError> {
        // Break down complex tasks into agent-specific subtasks
        let task_breakdown = self.analyze_task_complexity(task).await?;

        // Determine optimal agent assignment
        let agent_assignments = self.assign_agents_to_subtasks(&task_breakdown)?;

        // Create coordination timeline
        let timeline = self.create_execution_timeline(&agent_assignments)?;

        // Identify dependencies and potential conflicts
        let dependencies = self.analyze_task_dependencies(&agent_assignments)?;

        Ok(CoordinationPlan {
            task_id: task.id.clone(),
            subtasks: task_breakdown.subtasks,
            agent_assignments,
            execution_timeline: timeline,
            dependencies,
            checkpoints: self.define_progress_checkpoints(&timeline),
            rollback_strategies: self.define_rollback_points(&task_breakdown),
        })
    }

    async fn analyze_task_complexity(&mut self, task: &ComplexTask) -> Result<TaskBreakdown, AgentError> {
        let complexity_prompt = format!(
            "Analyze this complex task and break it down into manageable subtasks:\n\n\
            **Task**: {}\n\
            **Context**: {}\n\
            **Requirements**: {}\n\n\
            Please provide:\n\
            1. Task complexity assessment (1-10)\n\
            2. Subtask breakdown with clear deliverables\n\
            3. Estimated effort for each subtask\n\
            4. Dependencies between subtasks\n\
            5. Risk factors and mitigation approaches",
            task.description,
            task.context.unwrap_or_default(),
            task.requirements.join(\", \")
        );

        let claude_result = self.claude_client
            .execute_task(complexity_prompt, ProgrammingLanguage::ArchitecturalAnalysis)
            .await?;

        TaskBreakdown::from_claude_analysis(claude_result)
    }
}
```

## Claude Code Integration Patterns

### Strategic Analysis Prompt Templates

```rust
impl ProjectManagerAgent {
    fn build_strategic_context(&self, request: &str) -> Result<AnalysisContext, AgentError> {
        AnalysisContext {
            request: request.to_string(),
            system_context: self.get_system_context(),
            historical_decisions: self.get_relevant_history(request),
            stakeholder_priorities: self.get_stakeholder_context(),
            technical_constraints: self.get_technical_constraints(),
            business_objectives: self.get_business_objectives(),
        }
    }

    fn create_architecture_analysis_prompt(&self, context: &AnalysisContext) -> String {
        format!(
            "You are an expert software architect and project manager. Analyze this request:\n\n\
            **Request**: {}\n\n\
            **System Context**:\n\
            - Current Architecture: {}\n\
            - Technology Stack: Rust backend, Claude Code integration, Discord interface\n\
            - Deployment: 8GB VPS, resource-constrained environment\n\
            - Performance Requirements: <1s response time, 6+ concurrent agents\n\n\
            **Historical Context**:\n{}\n\n\
            **Analysis Framework**:\n\
            1. **Strategic Assessment**:\n\
               - Business value and impact\n\
               - Technical feasibility\n\
               - Resource requirements\n\
               - Risk factors\n\n\
            2. **Implementation Approach**:\n\
               - Recommended architecture patterns\n\
               - Technology choices and rationale\n\
               - Integration points with existing system\n\
               - Performance considerations\n\n\
            3. **Timeline and Resources**:\n\
               - Implementation phases\n\
               - Effort estimation\n\
               - Required expertise\n\
               - Dependencies and blockers\n\n\
            4. **Risk Analysis**:\n\
               - Technical risks and mitigations\n\
               - Operational risks\n\
               - Alternative approaches\n\n\
            Provide a comprehensive strategic analysis with specific, actionable recommendations.",
            context.request,
            context.system_context,
            context.historical_decisions.iter()
                .map(|d| format!(\"• {}\", d))
                .collect::<Vec<_>>()
                .join(\"\\n\")
        )
    }
}
```

## Discord Integration Patterns

### PM Agent Personality and Responses

```rust
impl ProjectManagerAgent {
    pub fn create_discord_response(&self, analysis: &StrategicAnalysis) -> DiscordAgentResponse {
        let message = match analysis.complexity_level {
            ComplexityLevel::Low => {
                format!(
                    "Great question! 📊 This looks straightforward:\n\n\
                    **Quick Analysis**: {}\n\n\
                    **My Recommendation**: {}\n\n\
                    **Next Steps**:\n{}\n\n\
                    This should be pretty quick to implement! Want me to coordinate with @SpiralDev?",
                    analysis.summary,
                    analysis.primary_recommendation,
                    analysis.immediate_actions.iter()
                        .enumerate()
                        .map(|(i, action)| format!(\"{}. {}\", i + 1, action))
                        .collect::<Vec<_>>()
                        .join(\"\\n\")
                )
            },
            ComplexityLevel::Medium | ComplexityLevel::High => {
                format!(
                    "This is an interesting strategic challenge! 🎯 Let me break it down:\n\n\
                    **Strategic Analysis**: {}\n\n\
                    **Key Considerations**:\n{}\n\n\
                    **My Recommendation**: {}\n\n\
                    **Implementation Phases**:\n{}\n\n\
                    **Risk Factors**: {}\n\n\
                    This will require careful coordination. Should I create a detailed implementation plan?",
                    analysis.summary,
                    analysis.key_considerations.iter()
                        .map(|c| format!(\"• {}\", c))
                        .collect::<Vec<_>>()
                        .join(\"\\n\"),
                    analysis.primary_recommendation,
                    analysis.implementation_phases.iter()
                        .enumerate()
                        .map(|(i, phase)| format!(\"**Phase {}**: {}\", i + 1, phase.description))
                        .collect::<Vec<_>>()
                        .join(\"\\n\"),
                    analysis.risk_summary
                )
            }
        };

        DiscordAgentResponse {
            message,
            suggested_actions: vec![
                \"Create detailed implementation plan\".to_string(),
                \"Coordinate with development team\".to_string(),
                \"Analyze risks and alternatives\".to_string(),
                \"Estimate timeline and resources\".to_string(),
            ],
            can_execute: false, // PM analyzes and coordinates, doesn't directly implement
            requires_followup: analysis.requires_detailed_planning,
            pending_context: Some(serde_json::json!({
                \"analysis_id\": analysis.id,
                \"complexity_level\": analysis.complexity_level,
                \"requires_coordination\": analysis.requires_multi_agent_coordination
            })),
        }
    }
}
```

## Task Coordination Patterns

### Agent Assignment Logic

```rust
impl ProjectManagerAgent {
    fn assign_agents_to_subtasks(&self, breakdown: &TaskBreakdown) -> Result<Vec<AgentAssignment>, AgentError> {
        let mut assignments = Vec::new();

        for subtask in &breakdown.subtasks {
            let optimal_agent = self.determine_optimal_agent(subtask)?;
            let assignment = AgentAssignment {
                subtask_id: subtask.id.clone(),
                agent_type: optimal_agent,
                priority: subtask.priority,
                estimated_effort: subtask.estimated_hours,
                dependencies: subtask.dependencies.clone(),
                deliverables: subtask.expected_deliverables.clone(),
            };
            assignments.push(assignment);
        }

        // Optimize for parallel execution
        self.optimize_parallel_execution(&mut assignments)?;

        Ok(assignments)
    }

    fn determine_optimal_agent(&self, subtask: &Subtask) -> Result<AgentType, AgentError> {
        match subtask.category {
            TaskCategory::CodeGeneration => Ok(AgentType::Developer),
            TaskCategory::CodeReview | TaskCategory::Testing => Ok(AgentType::QA),
            TaskCategory::Architecture | TaskCategory::Planning => Ok(AgentType::ProjectManager),
            TaskCategory::Integration | TaskCategory::Deployment => Ok(AgentType::Developer),
            TaskCategory::Documentation => Ok(AgentType::Developer), // Can handle docs
            TaskCategory::Research => Ok(AgentType::ProjectManager),
            TaskCategory::Coordination => Ok(AgentType::ProjectManager),
            _ => Err(AgentError::UnknownTaskCategory(subtask.category.clone()))
        }
    }
}
```

## Performance Monitoring

### Strategic Decision Tracking

```rust
pub struct PMAgentMetrics {
    decisions_made: u32,
    coordination_success_rate: f32,
    average_analysis_time: Duration,
    agent_utilization_efficiency: f32,
    stakeholder_satisfaction_score: f32,
}

impl ProjectManagerAgent {
    pub async fn track_decision_outcome(&mut self, decision_id: &str, outcome: DecisionOutcome) {
        let decision_record = DecisionRecord {
            id: decision_id.to_string(),
            timestamp: chrono::Utc::now(),
            context: outcome.context.clone(),
            recommendation_followed: outcome.recommendation_followed,
            actual_results: outcome.results.clone(),
            lessons_learned: outcome.lessons_learned.clone(),
        };

        // Store for future reference and learning
        self.coordination_history.push(decision_record);

        // Update decision-making templates based on outcomes
        if let Some(improvement) = outcome.suggested_improvement {
            self.update_analysis_templates(improvement).await;
        }
    }

    pub fn generate_performance_report(&self) -> PMPerformanceReport {
        PMPerformanceReport {
            decisions_tracked: self.coordination_history.len(),
            success_rate: self.calculate_success_rate(),
            most_effective_patterns: self.identify_effective_patterns(),
            areas_for_improvement: self.identify_improvement_areas(),
            agent_coordination_efficiency: self.measure_coordination_efficiency(),
        }
    }
}
```

## Testing Strategy

### Strategic Analysis Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn pm_agent_provides_architecture_analysis() {
        let mut agent = create_test_pm_agent().await;

        let result = agent.analyze_request(
            "Should we use microservices or monolith for our new API?"
        ).await.unwrap();

        match result {
            StrategicAnalysis { primary_recommendation, key_considerations, .. } => {
                assert!(!primary_recommendation.is_empty());
                assert!(key_considerations.len() > 2);
                assert!(result.includes_risk_analysis());
            }
        }
    }

    #[tokio::test]
    async fn pm_agent_coordinates_multi_agent_tasks() {
        let mut agent = create_test_pm_agent().await;
        let complex_task = ComplexTask::new(
            "Build a complete user authentication system with OAuth, JWT, and role-based access control"
        );

        let coordination_plan = agent.coordinate_multi_agent_task(&complex_task).await.unwrap();

        assert!(coordination_plan.subtasks.len() >= 3);
        assert!(coordination_plan.agent_assignments.len() > 0);
        assert!(coordination_plan.execution_timeline.phases.len() > 1);
        assert!(!coordination_plan.dependencies.is_empty());
    }

    #[tokio::test]
    async fn pm_agent_handles_resource_constraints() {
        let mut agent = create_test_pm_agent_with_limits(prompts_remaining: 1).await;

        let result = agent.analyze_request("Simple question about API design").await.unwrap();

        // Should provide analysis but flag resource constraints
        assert!(result.resource_constrained);
        assert_eq!(agent.prompts_remaining, 0);
    }
}
```

## Common Pitfalls

### Over-Analysis Paralysis

- **Problem**: Spending too much time on analysis without providing actionable recommendations
- **Solution**: Set analysis time limits and focus on key decision factors

### Micro-Management of Agents

- **Problem**: Over-coordinating simple tasks that agents can handle independently
- **Solution**: Use coordination thresholds based on task complexity

### Context Loss in Multi-Turn Analysis

- **Problem**: Losing track of previous analysis context in complex discussions
- **Solution**: Maintain analysis state and context across interactions

## Integration Points

This PM agent module integrates with:

- [Developer Agent](CLAUDE-agents-developer.md) for implementation coordination
- [Discord Integration](CLAUDE-integrations-discord.md) for strategic discussions
- [GitHub Integration](CLAUDE-integrations-github.md) for project management workflows
- [Claude Code Integration](CLAUDE-integrations-claude-code.md) for strategic analysis

## Related Documentation

- See [Coding Standards](CLAUDE-core-coding-standards.md) for SOLID principles in strategic analysis
- See [Implementation Phase 1](CLAUDE-implementation-phase1.md) for PM agent deployment
- See [Implementation Phase 1](CLAUDE-implementation-phase1.md) for PM agent deployment
