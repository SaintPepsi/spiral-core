use super::{Agent, AgentStatus};
use crate::{
    claude_code::{ClaudeCodeClient, CodeGenerationRequest, TaskAnalysis},
    models::{AgentType, Task, TaskResult},
    Result, SpiralError,
};
// 🔧 UTILITY IMPORTS: Using extracted modules via 3-strikes abstraction rule
use super::language_detection::{detect_language_from_context, extract_requirements_from_content};
use super::task_utils::{build_enriched_context, create_failure_result, create_success_result};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub struct SoftwareDeveloperAgent {
    claude_client: ClaudeCodeClient,
    status: AgentStatus,
}

impl SoftwareDeveloperAgent {
    pub fn new(claude_client: ClaudeCodeClient) -> Self {
        Self {
            claude_client,
            status: AgentStatus::new(AgentType::SoftwareDeveloper),
        }
    }

    pub fn status(&self) -> &AgentStatus {
        &self.status
    }

    /// 🎯 LANGUAGE DETECTION: Enhanced with local analysis + Claude Code intelligence
    /// DECISION: Hybrid approach - fast local detection with Claude Code fallback
    /// Why: Reduces API calls for obvious cases while maintaining accuracy for complex scenarios
    async fn detect_language_from_task(&self, task: &Task) -> Result<String> {
        // 🚀 FAST PATH: Try local detection first using utility module
        let local_detected = detect_language_from_context(
            task.context.get("file_path").map(|s| s.as_str()),
            task.context.get("project_type").map(|s| s.as_str()),
            &task.content,
        );

        // ✅ CONFIDENCE CHECK: If local detection is confident (not default), use it
        // Logic: Local detection returns "rust" as fallback, so we can detect low confidence
        let has_file_context = task.context.contains_key("file_path");
        let has_project_context = task.context.contains_key("project_type");
        let content_has_indicators = !task.content.to_lowercase().eq(&task.content); // Has language keywords

        if local_detected != "rust"
            || has_file_context
            || has_project_context
            || content_has_indicators
        {
            info!("Using local language detection: {}", local_detected);
            return Ok(local_detected);
        }

        // 🤖 CLAUDE CODE FALLBACK: Use AI for ambiguous cases
        let context = task
            .context
            .get("project_type")
            .or_else(|| task.context.get("file_extension"))
            .or_else(|| task.context.get("repository_context"))
            .unwrap_or(&task.content);

        let mut code_snippet = task.content.clone();

        if let Some(existing_code) = task.context.get("existing_code") {
            code_snippet = existing_code.clone();
        }

        if code_snippet.len() > crate::constants::CODE_SNIPPET_TRUNCATION_LENGTH {
            code_snippet = code_snippet
                .chars()
                .take(crate::constants::CODE_SNIPPET_TRUNCATION_LENGTH)
                .collect();
        }

        let detected = self
            .claude_client
            .detect_language(&code_snippet, context)
            .await?;

        if detected.is_empty() {
            info!("Using fallback language: {}", local_detected);
            Ok(local_detected)
        } else {
            info!("Claude Code detected language: {}", detected);
            Ok(detected)
        }
    }

    /// 🔨 REQUEST BUILDER: Constructs optimized Claude Code generation requests
    /// REFACTORING: Now uses extracted utilities for context building and requirement extraction
    async fn build_code_generation_request(&self, task: &Task) -> Result<CodeGenerationRequest> {
        // 🎯 LANGUAGE DETECTION: Hybrid local + AI approach
        let language = self.detect_language_from_task(task).await?;

        // 📝 REQUIREMENT EXTRACTION: Using centralized utility function
        let requirements = extract_requirements_from_content(&task.content, &task.context);

        let existing_code = task.context.get("existing_code").cloned();

        // 📄 CONTEXT BUILDING: Using standardized context builder
        let context = build_enriched_context(
            task,
            AgentType::SoftwareDeveloper,
            Some({
                let mut extra = HashMap::new();
                extra.insert("detected_language".to_string(), language.clone());
                extra.insert(
                    "requirements_count".to_string(),
                    requirements.len().to_string(),
                );
                extra
            }),
        );

        Ok(CodeGenerationRequest {
            language,
            description: task.content.clone(),
            context,
            existing_code,
            requirements,
        })
    }

    /// ✅ SUCCESS RESULT CREATOR: Formats Claude Code generation results
    /// REFACTORING: Now uses standardized result builder from task_utils
    fn create_success_result(
        &self,
        task: &Task,
        code_result: crate::claude_code::CodeGenerationResult,
    ) -> TaskResult {
        let files_created = code_result
            .files_to_create
            .iter()
            .map(|f| f.path.clone())
            .collect();

        let files_modified = code_result
            .files_to_modify
            .iter()
            .map(|f| f.path.clone())
            .collect();

        let output = format!(
            "Generated {} code:\n\n{}\n\nExplanation:\n{}",
            code_result.language, code_result.code, code_result.explanation
        );

        // 📈 AGENT-SPECIFIC METADATA: Development-focused metrics
        let mut metadata = HashMap::new();
        metadata.insert("language".to_string(), code_result.language);
        metadata.insert(
            "code_length".to_string(),
            code_result.code.len().to_string(),
        );
        metadata.insert(
            "explanation_length".to_string(),
            code_result.explanation.len().to_string(),
        );
        metadata.insert(
            "has_tests".to_string(),
            code_result.code.to_lowercase().contains("test").to_string(),
        );

        // 🔧 STANDARDIZED RESULT: Using utility function for consistency
        create_success_result(
            task,
            AgentType::SoftwareDeveloper,
            output,
            files_created,
            files_modified,
            Some(metadata),
        )
    }

    /// ❌ FAILURE RESULT CREATOR: Standardized error result formatting
    /// REFACTORING: Now uses standardized error result builder from task_utils  
    fn create_failure_result(&self, task: &Task, error: &SpiralError) -> TaskResult {
        // 📈 AGENT-SPECIFIC ERROR METADATA: Development context for debugging
        let mut metadata = HashMap::new();
        metadata.insert(
            "agent_capability".to_string(),
            "code_generation".to_string(),
        );
        metadata.insert("claude_code_integration".to_string(), "true".to_string());

        // 🔧 STANDARDIZED RESULT: Using utility function for consistency
        create_failure_result(
            task,
            AgentType::SoftwareDeveloper,
            error,
            None, // No partial output for developer agent failures
            Some(metadata),
        )
    }
}

#[async_trait]
impl Agent for SoftwareDeveloperAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::SoftwareDeveloper
    }

    fn name(&self) -> String {
        "Spiral Developer Agent".to_string()
    }

    fn description(&self) -> String {
        "Autonomous code generation with language detection and Claude Code integration".to_string()
    }

    async fn can_handle(&self, task: &Task) -> bool {
        matches!(task.agent_type, AgentType::SoftwareDeveloper)
    }

    async fn execute(&self, task: Task) -> Result<TaskResult> {
        info!("SoftwareDeveloperAgent executing task: {}", task.id);

        if self.status.is_busy {
            warn!("Agent is busy, cannot execute task: {}", task.id);
            return Ok(self.create_failure_result(
                &task,
                &SpiralError::Agent {
                    message: "Agent is currently busy".to_string(),
                },
            ));
        }

        let start_time = std::time::Instant::now();

        let code_request = match self.build_code_generation_request(&task).await {
            Ok(request) => request,
            Err(e) => {
                warn!("Failed to build code generation request: {}", e);
                return Ok(self.create_failure_result(&task, &e));
            }
        };

        debug!(
            "Generated code request for language: {}",
            code_request.language
        );

        match self.claude_client.generate_code(code_request).await {
            Ok(code_result) => {
                let execution_time = start_time.elapsed().as_secs_f64();
                info!(
                    "Successfully generated code for task: {} in {:.2}s",
                    task.id, execution_time
                );

                Ok(self.create_success_result(&task, code_result))
            }
            Err(e) => {
                warn!("Code generation failed for task {}: {}", task.id, e);
                Ok(self.create_failure_result(&task, &e))
            }
        }
    }

    async fn analyze_task(&self, task: &Task) -> Result<TaskAnalysis> {
        debug!("Analyzing task: {}", task.id);

        // 📄 STANDARDIZED CONTEXT: Using centralized context builder
        let context = build_enriched_context(
            task,
            AgentType::SoftwareDeveloper,
            Some({
                let mut extra = HashMap::new();
                extra.insert(
                    "analysis_type".to_string(),
                    "development_planning".to_string(),
                );
                extra.insert(
                    "capability_focus".to_string(),
                    "code_generation".to_string(),
                );
                extra
            }),
        );

        self.claude_client
            .analyze_task(&task.content, context)
            .await
    }
}
