# CLAUDE-agents-developer.md

**Purpose**: Developer agent implementation guidance and Claude Code integration patterns
**Dependencies**: [Coding Standards](CLAUDE-core-coding-standards.md), [Discord Integration](CLAUDE-integrations-discord.md)
**Updated**: 2024-07-24

## Quick Start

The Developer Agent specializes in autonomous code generation using Claude Code as its intelligence engine:

```rust
impl DeveloperAgent {
    pub async fn execute_autonomous_task(&mut self, task: &str, language_context: &LanguageContext) -> Result<TaskResult, AgentError>;
    pub async fn execute_parallel_tasks(&mut self, tasks: Vec<String>) -> Vec<TaskResult>;
}
```

## Core Architecture

### Single Responsibility Focus

The Developer Agent has one clear responsibility: coordinate Claude Code for development tasks.

```rust
// ✅ Good - Single responsibility
pub struct DeveloperAgent {
    claude_client: Box<dyn ClaudeClient>,
    language_detector: LanguageInferenceEngine,
    task_queue: TaskQueue,
    prompts_remaining: u32,
}

impl DeveloperAgent {
    pub async fn generate_code(&mut self, requirements: &str, language_context: &LanguageContext) -> Result<CodeResult, AgentError> {
        // Only handles code generation orchestration
        let optimized_prompt = self.build_development_prompt(requirements, language_context)?;

        let claude_result = self.claude_client
            .execute_task(optimized_prompt, language_context.programming_language())
            .await?;

        self.prompts_remaining -= 1;
        Ok(CodeResult::from_claude_response(claude_result))
    }
}
```

### Language-Agnostic Design

The Developer Agent supports any programming language through intelligent inference:

```rust
pub struct LanguageInferenceEngine {
    framework_patterns: HashMap<String, ProgrammingLanguage>,
    explicit_keywords: HashMap<String, ProgrammingLanguage>,
    confidence_threshold: f32,
}

impl LanguageInferenceEngine {
    pub async fn infer_language(&self, user_prompt: &str, context: Option<&ConversationContext>) -> LanguageContext {
        let mut detected_language = None;
        let mut confidence = 0.0;
        let mut source = InferenceSource::Unknown;

        // 1. Check for explicit language mentions
        if let Some(lang) = self.extract_explicit_language(user_prompt) {
            detected_language = Some(lang);
            confidence = 0.95;
            source = InferenceSource::UserExplicit;
        }

        // 2. Check conversation history for context
        else if let Some(ctx) = context {
            if let Some(lang) = self.infer_from_conversation_history(ctx).await {
                detected_language = Some(lang);
                confidence = 0.70;
                source = InferenceSource::PreviousConversation;
            }
        }

        // 3. Detect from framework/library keywords
        else if let Some(lang) = self.infer_from_frameworks(user_prompt) {
            detected_language = Some(lang);
            confidence = 0.65;
            source = InferenceSource::UserPromptKeywords;
        }

        LanguageContext {
            detected_language,
            confidence,
            inference_source: source,
            original_prompt: user_prompt.to_string(),
        }
    }

    fn extract_explicit_language(&self, prompt: &str) -> Option<ProgrammingLanguage> {
        let lowercased = prompt.to_lowercase();

        // Direct language mentions
        if lowercased.contains("python") || lowercased.contains("django") || lowercased.contains("fastapi") {
            return Some(ProgrammingLanguage::Python);
        }
        if lowercased.contains("javascript") || lowercased.contains("node.js") || lowercased.contains("react") {
            return Some(ProgrammingLanguage::JavaScript);
        }
        if lowercased.contains("typescript") || lowercased.contains("nestjs") {
            return Some(ProgrammingLanguage::TypeScript);
        }
        if lowercased.contains("rust") || lowercased.contains("actix") || lowercased.contains("warp") {
            return Some(ProgrammingLanguage::Rust);
        }
        if lowercased.contains("go") || lowercased.contains("golang") || lowercased.contains("gin") {
            return Some(ProgrammingLanguage::Go);
        }
        if lowercased.contains("java") || lowercased.contains("spring boot") {
            return Some(ProgrammingLanguage::Java);
        }

        None
    }

    fn infer_from_frameworks(&self, prompt: &str) -> Option<ProgrammingLanguage> {
        let lowercased = prompt.to_lowercase();

        for (framework, language) in &self.framework_patterns {
            if lowercased.contains(framework) {
                return Some(language.clone());
            }
        }

        None
    }
}
```

### Clarification Patterns

When language cannot be inferred with sufficient confidence, the agent requests clarification:

```rust
impl DeveloperAgent {
    fn create_language_clarification_response(&self, task: &str, language_context: &LanguageContext) -> String {
        format!(
            "Hey! 🚀 I'd love to help you with that!\n\n\
            I need to know what programming language you'd like me to use.\n\n\
            **Your request**: {}\n\n\
            Which language would you prefer? Here are some popular options:\n\
            • **Python** 🐍 (great for APIs, FastAPI/Django)\n\
            • **JavaScript/Node.js** ⚡ (Express, Koa)\n\
            • **TypeScript** 📘 (NestJS, type-safe Node.js)\n\
            • **Rust** 🦀 (Actix-web, Warp - high performance)\n\
            • **Go** 🔄 (Gin, Echo - great for microservices)\n\
            • **Java** ☕ (Spring Boot - enterprise ready)\n\
            • **Or any other language you prefer!**\n\n\
            Just mention me again with your language choice! 🎯",
            task
        )
    }
}
```

## Claude Code Integration

### Core Client Implementation

```rust
pub trait ClaudeClient: Send + Sync {
    async fn execute_task(&self, prompt: String, language: ProgrammingLanguage) -> Result<ClaudeResult, ClaudeError>;
    async fn get_available_prompts(&self) -> Result<u32, ClaudeError>;
    async fn validate_connection(&self) -> Result<(), ClaudeError>;
}

pub struct ClaudeCodeClient {
    api_key: String,
    base_url: String,
    http_client: reqwest::Client,
    rate_limiter: Arc<RateLimiter>,
}

impl ClaudeClient for ClaudeCodeClient {
    async fn execute_task(&self, prompt: String, language: ProgrammingLanguage) -> Result<ClaudeResult, ClaudeError> {
        // Rate limiting check
        self.rate_limiter.check_rate().await?;

        let request = ClaudeRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: self.build_development_context(prompt, language),
            }],
            max_tokens: 4000,
            temperature: 0.1, // Low temperature for consistent code generation
        };

        let response = self.http_client
            .post(&format!("{}/v1/messages", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ClaudeError::ApiError(response.status().as_u16()));
        }

        let claude_response: ClaudeResponse = response.json().await?;
        Ok(ClaudeResult::from_response(claude_response))
    }

    async fn get_available_prompts(&self) -> Result<u32, ClaudeError> {
        // Implementation depends on Claude Code's quota API
        // For now, return a reasonable default
        Ok(1000)
    }
}
```

### Prompt Optimization

```rust
impl ClaudeCodeClient {
    fn build_development_context(&self, user_request: String, language: ProgrammingLanguage) -> String {
        let language_specific_context = match language {
            ProgrammingLanguage::Python => {
                "You are an expert Python developer. Focus on:\n\
                • PEP 8 compliance and Pythonic patterns\n\
                • Type hints and comprehensive docstrings\n\
                • pytest for testing with fixtures and parametrization\n\
                • Virtual environment and requirements.txt\n\
                • Error handling with custom exceptions\n\
                • FastAPI/Django patterns when building APIs"
            },
            ProgrammingLanguage::JavaScript => {
                "You are an expert JavaScript developer. Focus on:\n\
                • ES6+ features and modern syntax\n\
                • Jest for testing with mocks and spies\n\
                • npm/yarn package management\n\
                • Error handling with try/catch and async patterns\n\
                • Express.js/React patterns when building applications"
            },
            ProgrammingLanguage::TypeScript => {
                "You are an expert TypeScript developer. Focus on:\n\
                • Strict type checking and interfaces\n\
                • Generic types and utility types\n\
                • Jest with @types packages for testing\n\
                • tsconfig.json optimization\n\
                • Error handling with custom Error classes\n\
                • NestJS/React with TypeScript patterns"
            },
            ProgrammingLanguage::Rust => {
                "You are an expert Rust developer. Focus on:\n\
                • Memory safety and ownership patterns\n\
                • Error handling with Result<T, E> and thiserror\n\
                • Cargo.toml dependencies and features\n\
                • #[cfg(test)] modules with tokio::test for async\n\
                • Traits and generics for code reuse\n\
                • Actix-web/Warp patterns for web services"
            },
            ProgrammingLanguage::Go => {
                "You are an expert Go developer. Focus on:\n\
                • Go conventions and idiomatic patterns\n\
                • Error handling with explicit error returns\n\
                • go.mod and go.sum dependency management\n\
                • Testing with table-driven tests and testify\n\
                • Interfaces and composition over inheritance\n\
                • Gin/Echo patterns for web services"
            },
            ProgrammingLanguage::Java => {
                "You are an expert Java developer. Focus on:\n\
                • Java 17+ features and modern syntax\n\
                • Maven/Gradle build management\n\
                • JUnit 5 for testing with annotations\n\
                • Exception handling with custom exceptions\n\
                • SOLID principles and design patterns\n\
                • Spring Boot patterns for enterprise applications"
            },
        };

        format!(
            "{}\n\n\
            **Development Requirements:**\n\
            • Create complete, production-ready code\n\
            • Include comprehensive tests\n\
            • Add clear documentation and comments\n\
            • Follow language-specific best practices\n\
            • Handle errors gracefully\n\
            • Include package/dependency files\n\n\
            **User Request:**\n\
            {}\n\n\
            Please provide a complete implementation with file structure, code, tests, and setup instructions.",
            language_specific_context,
            user_request
        )
    }
}
```

## Task Execution Patterns

### Autonomous Task Processing

```rust
impl DeveloperAgent {
    pub async fn execute_autonomous_task(&mut self, task: &str, language_context: &LanguageContext) -> Result<TaskResult, AgentError> {
        // Validate we have sufficient prompts
        if self.prompts_remaining == 0 {
            return Err(AgentError::InsufficientPrompts {
                remaining: 0,
                required: 1,
            });
        }

        // Check language confidence
        if language_context.confidence < 0.8 {
            return Ok(TaskResult::RequiresLanguageClarity {
                task: task.to_string(),
                clarification_message: self.create_language_clarification_response(task, language_context),
            });
        }

        let language = language_context.detected_language.as_ref().unwrap();

        // Execute with Claude Code
        let start_time = std::time::Instant::now();

        let code_result = self.generate_code(task, language_context).await?;

        // Track performance metrics
        let execution_time = start_time.elapsed();

        Ok(TaskResult::Completed {
            task: task.to_string(),
            language: language.clone(),
            code_output: code_result,
            execution_time,
            prompts_used: 1,
        })
    }

    pub async fn execute_parallel_tasks(&mut self, tasks: Vec<String>) -> Vec<TaskResult> {
        let mut results = Vec::new();

        // Process tasks concurrently up to our prompt limit
        let available_prompts = self.prompts_remaining.min(tasks.len() as u32);
        let (immediate_tasks, queued_tasks) = tasks.split_at(available_prompts as usize);

        // Execute immediate tasks in parallel
        let mut futures = Vec::new();
        for task in immediate_tasks {
            let language_context = self.language_detector.infer_language(task, None).await;
            futures.push(self.execute_autonomous_task(task, &language_context));
        }

        let immediate_results = futures::future::join_all(futures).await;
        results.extend(immediate_results.into_iter().map(|r| r.unwrap_or_else(|e| TaskResult::Failed {
            task: "Unknown".to_string(),
            error: e,
        })));

        // Queue remaining tasks
        for task in queued_tasks {
            results.push(TaskResult::Queued {
                task: task.to_string(),
                estimated_wait: std::time::Duration::from_mins(5), // Estimate based on rate limits
            });
        }

        results
    }
}
```

### Error Recovery Strategies

```rust
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Insufficient prompts: {remaining} remaining, {required} required")]
    InsufficientPrompts { remaining: u32, required: u32 },

    #[error("Claude Code API error: {status_code}")]
    ClaudeApiError { status_code: u16 },

    #[error("Language inference failed: {prompt}")]
    LanguageInferenceFailed { prompt: String },

    #[error("Task execution timeout after {duration:?}")]
    TaskTimeout { duration: std::time::Duration },

    #[error("Invalid task format: {details}")]
    InvalidTaskFormat { details: String },
}

impl DeveloperAgent {
    async fn handle_agent_error(&mut self, error: AgentError, task: &str) -> TaskResult {
        match error {
            AgentError::InsufficientPrompts { remaining, required } => {
                TaskResult::ResourceExhausted {
                    task: task.to_string(),
                    message: format!(
                        "I've run out of Claude Code prompts! 😅\n\
                        I need {} more prompts but only have {} remaining.\n\
                        Please wait for my quota to refresh, or consider upgrading the plan.",
                        required, remaining
                    ),
                    retry_after: std::time::Duration::from_hours(1),
                }
            },
            AgentError::ClaudeApiError { status_code } => {
                TaskResult::Failed {
                    task: task.to_string(),
                    error: format!(
                        "Claude Code API returned error {}. \n\
                        This might be a temporary issue - please try again in a few minutes!",
                        status_code
                    ),
                }
            },
            AgentError::LanguageInferenceFailed { .. } => {
                let language_context = LanguageContext::unknown(task);
                TaskResult::RequiresLanguageClarity {
                    task: task.to_string(),
                    clarification_message: self.create_language_clarification_response(task, &language_context),
                }
            },
            _ => {
                TaskResult::Failed {
                    task: task.to_string(),
                    error: format!("Unexpected error: {}", error),
                }
            }
        }
    }
}
```

## Discord Integration Patterns

### Conversational Responses

```rust
impl DeveloperAgent {
    pub fn create_discord_response(&self, task_result: &TaskResult, language_context: &LanguageContext) -> DiscordAgentResponse {
        match task_result {
            TaskResult::Completed { task, language, code_output, execution_time, .. } => {
                let enthusiasm_emojis = ["🚀", "✨", "🔥", "⚡", "🎯"];
                let emoji = enthusiasm_emojis[fastrand::usize(..enthusiasm_emojis.len())];

                DiscordAgentResponse {
                    message: format!(
                        "Awesome! {} I've completed your **{}** project!\n\n\
                        **Task**: {}\n\
                        **Language**: {} \n\
                        **Files Created**: {}\n\
                        **Tests**: {} passing\n\
                        **Execution Time**: {:.2}s\n\n\
                        The code is ready to run! Check out the implementation - \
                        it follows all the best practices and includes comprehensive documentation. 📚\n\n\
                        Want me to create another project or enhance this one? 🎨",
                        emoji,
                        language.display_name(),
                        task,
                        language.display_name(),
                        code_output.files_created.len(),
                        code_output.tests_passing,
                        execution_time.as_secs_f64()
                    ),
                    suggested_actions: vec![
                        "Add more features to this project".to_string(),
                        "Create a related microservice".to_string(),
                        "Build a frontend for this API".to_string(),
                        "Add deployment configuration".to_string(),
                    ],
                    can_execute: true,
                    requires_followup: false,
                    pending_context: None,
                }
            },
            TaskResult::RequiresLanguageClarity { clarification_message, .. } => {
                DiscordAgentResponse {
                    message: clarification_message.clone(),
                    suggested_actions: vec![
                        "Specify Python for web APIs".to_string(),
                        "Choose JavaScript for frontend".to_string(),
                        "Select Rust for performance".to_string(),
                        "Pick Go for microservices".to_string(),
                    ],
                    can_execute: false,
                    requires_followup: true,
                    pending_context: Some(serde_json::json!({
                        "awaiting_language_selection": true,
                        "original_task": task_result.task()
                    })),
                }
            },
            TaskResult::ResourceExhausted { message, retry_after, .. } => {
                DiscordAgentResponse {
                    message: message.clone(),
                    suggested_actions: vec![
                        "Queue this task for later".to_string(),
                        "Try a simpler task first".to_string(),
                        "Check quota status".to_string(),
                    ],
                    can_execute: false,
                    requires_followup: false,
                    pending_context: Some(serde_json::json!({
                        "retry_after_seconds": retry_after.as_secs()
                    })),
                }
            },
            TaskResult::Failed { task, error } => {
                DiscordAgentResponse {
                    message: format!(
                        "Oops! 😅 I encountered an issue with your request:\n\n\
                        **Task**: {}\n\
                        **Error**: {}\n\n\
                        Don't worry - this happens sometimes! Could you try rephrasing your request \
                        or provide more specific details about what you'd like me to build?",
                        task, error
                    ),
                    suggested_actions: vec![
                        "Rephrase the request with more details".to_string(),
                        "Break the task into smaller parts".to_string(),
                        "Retry with explicit language choice".to_string(),
                    ],
                    can_execute: false,
                    requires_followup: true,
                    pending_context: None,
                }
            },
        }
    }
}
```

## Performance Optimization

### Rate Limiting and Resource Management

```rust
pub struct DeveloperAgentManager {
    agents: Vec<DeveloperAgent>,
    request_queue: Arc<Mutex<VecDeque<DevelopmentRequest>>>,
    rate_limiter: Arc<RateLimiter>,
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,
}

impl DeveloperAgentManager {
    pub async fn distribute_task(&mut self, request: DevelopmentRequest) -> Result<TaskResult, AgentError> {
        // Find agent with available prompts
        let available_agent = self.agents.iter_mut()
            .find(|agent| agent.prompts_remaining > 0)
            .ok_or(AgentError::NoAvailableAgents)?;

        // Execute with performance tracking
        let start_time = std::time::Instant::now();
        let result = available_agent.execute_autonomous_task(&request.task, &request.language_context).await?;
        let execution_time = start_time.elapsed();

        // Update metrics
        {
            let mut metrics = self.performance_metrics.lock().await;
            metrics.record_task_completion(execution_time, &result);
        }

        Ok(result)
    }

    pub async fn get_agent_status(&self) -> AgentStatusReport {
        let total_prompts: u32 = self.agents.iter().map(|a| a.prompts_remaining).sum();
        let active_agents = self.agents.iter().filter(|a| a.prompts_remaining > 0).count();

        AgentStatusReport {
            total_agents: self.agents.len(),
            active_agents,
            total_prompts_remaining: total_prompts,
            average_response_time: self.performance_metrics.lock().await.average_response_time(),
            tasks_completed_today: self.performance_metrics.lock().await.tasks_completed_today(),
        }
    }
}
```

## Testing Strategy

### Unit Tests for Language Detection

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn language_inference_detects_python_from_fastapi() {
        let engine = LanguageInferenceEngine::new();
        let result = engine.infer_language("create a FastAPI application with SQLAlchemy", None).await;

        assert_eq!(result.detected_language, Some(ProgrammingLanguage::Python));
        assert!(result.confidence > 0.9);
        assert_eq!(result.inference_source, InferenceSource::UserPromptKeywords);
    }

    #[tokio::test]
    async fn language_inference_requires_clarification_for_generic_request() {
        let engine = LanguageInferenceEngine::new();
        let result = engine.infer_language("build a REST API", None).await;

        assert_eq!(result.detected_language, None);
        assert!(result.confidence < 0.5);
    }

    #[tokio::test]
    async fn developer_agent_handles_insufficient_prompts_gracefully() {
        let mut agent = DeveloperAgent::new_with_prompts(0);
        let language_context = LanguageContext::python_with_confidence(0.95);

        let result = agent.execute_autonomous_task("create a Flask app", &language_context).await;

        match result {
            Err(AgentError::InsufficientPrompts { remaining: 0, required: 1 }) => {
                // Expected behavior
            },
            _ => panic!("Expected InsufficientPrompts error"),
        }
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn developer_agent_creates_complete_python_project() {
        let mut agent = create_test_developer_agent().await;
        let language_context = LanguageContext::python_with_confidence(0.95);

        let result = agent.execute_autonomous_task(
            "create a FastAPI todo application with SQLite and pytest tests",
            &language_context
        ).await.unwrap();

        match result {
            TaskResult::Completed { code_output, .. } => {
                assert!(code_output.files_created.len() >= 5); // main.py, models.py, tests, requirements.txt, etc.
                assert!(code_output.tests_passing > 0);
                assert!(code_output.documentation.is_some());
            },
            _ => panic!("Expected completed task result"),
        }
    }
}
```

## Common Pitfalls

### Language Detection Accuracy

- **Problem**: Misidentifying languages from ambiguous prompts
- **Solution**: Use confidence thresholds and clarification patterns

### Prompt Quota Management

- **Problem**: Running out of Claude Code prompts during high usage
- **Solution**: Implement rate limiting and queue management

### Error Recovery

- **Problem**: Failed tasks leaving users without clear next steps
- **Solution**: Provide specific error messages with suggested actions

## Integration Points

This developer agent module integrates with:

- [Discord Integration](CLAUDE-integrations-discord.md) for user interaction
- [Project Manager Agent](CLAUDE-agents-pm.md) for task coordination
- [GitHub Integration](CLAUDE-integrations-github.md) for repository management
- [Claude Code Integration](CLAUDE-integrations-claude-code.md) for AI intelligence engine

## Related Documentation

- See [Coding Standards](CLAUDE-core-coding-standards.md) for SOLID/DRY/SID implementation patterns
- See [Implementation Phase 1](CLAUDE-implementation-phase1.md) for deployment and setup
- See [Implementation Phase 1](CLAUDE-implementation-phase1.md) for deployment guidance
