# CLAUDE-integrations-claude-code.md

**Purpose**: Claude Code integration patterns and primary intelligence engine implementation
**Dependencies**: [Coding Standards](CLAUDE-core-coding-standards.md)
**Updated**: 2024-07-24

## Quick Start

Claude Code serves as the primary intelligence engine for all agents in the Spiral Core system:

```rust
impl ClaudeCodeClient {
    pub async fn execute_task(&self, prompt: String, language: ProgrammingLanguage) -> Result<ClaudeResult, ClaudeError>;
    pub async fn stream_response(&self, prompt: String) -> Result<ClaudeStream, ClaudeError>;
    pub async fn get_usage_stats(&self) -> Result<UsageStats, ClaudeError>;
}
```

## Core Architecture

### Claude Code Client Implementation

```rust
// ✅ Good - Single responsibility for Claude Code communication
pub struct ClaudeCodeClient {
    api_key: String,
    base_url: String,
    model: String,
    http_client: reqwest::Client,
    rate_limiter: Arc<RateLimiter>,
    usage_tracker: Arc<Mutex<UsageTracker>>,
    retry_config: RetryConfig,
}

impl ClaudeCodeClient {
    pub fn new(config: ClaudeCodeConfig) -> Result<Self, ClaudeError> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent("Spiral-Core-Agent/1.0")
            .build()?;

        Ok(Self {
            api_key: config.api_key,
            base_url: config.base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string()),
            model: config.model.unwrap_or_else(|| "claude-3-sonnet-20240229".to_string()),
            http_client,
            rate_limiter: Arc::new(RateLimiter::new(config.rate_limit)),
            usage_tracker: Arc::new(Mutex::new(UsageTracker::new())),
            retry_config: config.retry_config.unwrap_or_default(),
        })
    }

    pub async fn execute_task(&self, prompt: String, language: ProgrammingLanguage) -> Result<ClaudeResult, ClaudeError> {
        // Rate limiting check
        self.rate_limiter.acquire().await?;

        // Build system context based on language and task type
        let system_prompt = self.build_system_context(language)?;
        let optimized_prompt = self.optimize_prompt_for_task(&prompt, language)?;

        // Prepare the request
        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: self.calculate_max_tokens(&optimized_prompt),
            temperature: self.get_optimal_temperature(language),
            system: Some(system_prompt),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: optimized_prompt,
            }],
        };

        // Execute with retry logic
        let response = self.execute_with_retry(request).await?;
        
        // Track usage
        self.track_usage(&response).await;

        // Parse and validate response
        let result = ClaudeResult::from_response(response, language)?;
        
        Ok(result)
    }

    async fn execute_with_retry(&self, request: ClaudeRequest) -> Result<ClaudeResponse, ClaudeError> {
        let mut attempts = 0;
        let max_attempts = self.retry_config.max_attempts;

        loop {
            attempts += 1;
            
            match self.send_request(&request).await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    if attempts >= max_attempts || !self.should_retry(&error) {
                        return Err(error);
                    }
                    
                    let delay = self.calculate_retry_delay(attempts);
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    async fn send_request(&self, request: &ClaudeRequest) -> Result<ClaudeResponse, ClaudeError> {
        let response = self.http_client
            .post(&format!("{}/v1/messages", self.base_url))
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            return self.handle_api_error(response).await;
        }

        let claude_response: ClaudeResponse = response.json().await?;
        Ok(claude_response)
    }
}
```

## Language-Specific Optimization

### Programming Language Context

```rust
impl ClaudeCodeClient {
    fn build_system_context(&self, language: ProgrammingLanguage) -> Result<String, ClaudeError> {
        let base_context = "You are an expert software developer assistant integrated into the Spiral Core AI agent system. You generate high-quality, production-ready code with comprehensive testing and documentation.";

        let language_context = match language {
            ProgrammingLanguage::Rust => {
                "**Rust Development Context**:\n\
                • Follow Rust 2021 edition idioms and best practices\n\
                • Use async/await with tokio for concurrent operations\n\
                • Implement proper error handling with Result<T, E> and thiserror\n\
                • Apply ownership and borrowing principles correctly\n\
                • Include comprehensive #[cfg(test)] modules with tokio::test\n\
                • Use Cargo.toml with appropriate dependencies and features\n\
                • Follow memory safety and zero-cost abstraction principles\n\
                • Generate rustdoc-compatible documentation comments\n\
                • Implement traits for extensibility and polymorphism\n\
                • Use structured logging with tracing crate\n\n\
                **System Integration Requirements**:\n\
                • Memory efficient (8GB VPS constraint)\n\
                • Fast startup time (<1 second)\n\
                • Compatible with multi-agent architecture\n\
                • Thread-safe for concurrent agent operations"
            },
            ProgrammingLanguage::Python => {
                "**Python Development Context**:\n\
                • Use Python 3.9+ features and type hints\n\
                • Follow PEP 8 style guidelines and PEP 257 docstring conventions\n\
                • Implement async/await patterns for I/O operations\n\
                • Use dataclasses or Pydantic models for structured data\n\
                • Include pytest test suite with fixtures and parametrization\n\
                • Handle exceptions with custom exception classes\n\
                • Use virtual environments and requirements.txt\n\
                • Generate comprehensive docstrings with examples\n\
                • Apply SOLID principles and design patterns\n\
                • Use logging module for structured logging\n\n\
                **Framework Preferences**:\n\
                • FastAPI for web APIs (with OpenAPI documentation)\n\
                • SQLAlchemy for database operations\n\
                • Pydantic for data validation and serialization"
            },
            ProgrammingLanguage::TypeScript => {
                "**TypeScript Development Context**:\n\
                • Use strict TypeScript configuration with noImplicitAny\n\
                • Implement interfaces and generic types for type safety\n\
                • Follow ESLint and Prettier formatting standards\n\
                • Use async/await with proper Promise handling\n\
                • Include Jest test suite with TypeScript support\n\
                • Implement proper error handling with custom Error classes\n\
                • Use npm/yarn with package.json and lock files\n\
                • Generate TSDoc comments for API documentation\n\
                • Apply functional programming patterns where appropriate\n\
                • Use structured logging with winston or similar\n\n\
                **Framework Integration**:\n\
                • Node.js for backend services\n\
                • Express.js or NestJS for API development\n\
                • Discord.js for Discord bot integration"
            },
            ProgrammingLanguage::JavaScript => {
                "**JavaScript Development Context**:\n\
                • Use ES2022+ features and modern syntax\n\
                • Implement async/await patterns consistently\n\
                • Follow ESLint standard configuration\n\
                • Include comprehensive Jest test coverage\n\
                • Handle errors with try/catch and custom Error classes\n\
                • Use npm/yarn with semantic versioning\n\
                • Generate JSDoc comments for documentation\n\
                • Apply functional programming concepts\n\
                • Use console logging with structured format\n\n\
                **Runtime Optimization**:\n\
                • Node.js performance best practices\n\
                • Memory management for long-running processes\n\
                • Event loop optimization"
            },
            ProgrammingLanguage::Go => {
                "**Go Development Context**:\n\
                • Follow Go idioms and effective Go principles\n\
                • Use goroutines and channels for concurrency\n\
                • Implement proper error handling with explicit returns\n\
                • Include table-driven tests with testing package\n\
                • Use go.mod for dependency management\n\
                • Generate godoc-compatible comments\n\
                • Apply interface-based design patterns\n\
                • Use structured logging with logrus or zap\n\
                • Follow Go project layout standards\n\
                • Optimize for garbage collector efficiency\n\n\
                **Performance Focus**:\n\
                • Memory allocation optimization\n\
                • CPU efficiency for concurrent operations\n\
                • Network I/O optimization"
            },
            ProgrammingLanguage::ArchitecturalAnalysis => {
                "**Strategic Analysis Context**:\n\
                • Focus on high-level architectural decisions\n\
                • Consider scalability, maintainability, and performance\n\
                • Analyze trade-offs between different approaches\n\
                • Provide specific, actionable recommendations\n\
                • Consider resource constraints (8GB VPS environment)\n\
                • Account for multi-agent system requirements\n\
                • Evaluate security implications\n\
                • Assess technical debt and long-term maintainability\n\n\
                **Analysis Framework**:\n\
                • Business value assessment\n\
                • Technical feasibility evaluation\n\
                • Risk analysis with mitigation strategies\n\
                • Implementation timeline estimation\n\
                • Resource requirement analysis"
            },
            ProgrammingLanguage::CodeReview => {
                "**Code Review Context**:\n\
                • Perform comprehensive code quality analysis\n\
                • Check for security vulnerabilities and best practices\n\
                • Verify adherence to SOLID principles\n\
                • Assess performance implications\n\
                • Review test coverage and quality\n\
                • Evaluate documentation completeness\n\
                • Check for potential bugs and edge cases\n\
                • Ensure consistency with existing codebase\n\n\
                **Review Criteria**:\n\
                • Code correctness and logic\n\
                • Architecture and design patterns\n\
                • Security best practices\n\
                • Performance optimization\n\
                • Error handling robustness\n\
                • Testing adequacy\n\
                • Documentation clarity"
            }
        };

        Ok(format!("{}\n\n{}", base_context, language_context))
    }

    fn get_optimal_temperature(&self, language: ProgrammingLanguage) -> f32 {
        match language {
            ProgrammingLanguage::Rust 
            | ProgrammingLanguage::Go 
            | ProgrammingLanguage::TypeScript => 0.1, // Lower temperature for systems languages
            
            ProgrammingLanguage::Python 
            | ProgrammingLanguage::JavaScript => 0.2, // Slightly higher for dynamic languages
            
            ProgrammingLanguage::ArchitecturalAnalysis => 0.3, // Higher for creative analysis
            
            ProgrammingLanguage::CodeReview => 0.1, // Low for consistent, thorough reviews
        }
    }

    fn calculate_max_tokens(&self, prompt: &str) -> u32 {
        // Estimate based on prompt length and expected response complexity
        let base_tokens = 2000;
        let prompt_factor = (prompt.len() / 100) as u32; // Rough estimation
        
        // Cap at reasonable limits based on task complexity
        (base_tokens + prompt_factor).min(4000)
    }
}
```

## Prompt Engineering Patterns

### Task-Specific Prompt Templates

```rust
pub struct PromptTemplateEngine {
    templates: HashMap<String, PromptTemplate>,
    context_builders: HashMap<String, Box<dyn ContextBuilder>>,
}

impl PromptTemplateEngine {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        // Development task templates
        templates.insert("code_generation".to_string(), PromptTemplate {
            system_context: "You are generating production-ready code for the Spiral Core system.".to_string(),
            user_template: "Generate a complete {language} implementation for: {task}\n\nRequirements:\n{requirements}\n\nPlease provide:\n1. Complete, runnable code\n2. Comprehensive test suite\n3. Clear documentation\n4. Error handling\n5. Performance considerations".to_string(),
            variables: vec!["language".to_string(), "task".to_string(), "requirements".to_string()],
        });

        templates.insert("code_review".to_string(), PromptTemplate {
            system_context: "You are performing a thorough code review for a production system.".to_string(),
            user_template: "Please review this code:\n\n```{language}\n{code}\n```\n\nFocus on:\n1. Code quality and correctness\n2. Security vulnerabilities\n3. Performance issues\n4. Best practices adherence\n5. Test coverage\n\nProvide specific, actionable feedback.".to_string(),
            variables: vec!["language".to_string(), "code".to_string()],
        });

        templates.insert("architecture_analysis".to_string(), PromptTemplate {
            system_context: "You are providing strategic architectural guidance for a multi-agent system.".to_string(),
            user_template: "Analyze this architectural decision:\n\n{decision_context}\n\nSystem constraints:\n- 8GB VPS deployment\n- Multi-agent coordination\n- <1s response time requirement\n- Rust backend with Claude Code integration\n\nProvide:\n1. Strategic assessment\n2. Technical trade-offs\n3. Implementation recommendations\n4. Risk analysis\n5. Alternative approaches".to_string(),
            variables: vec!["decision_context".to_string()],
        });

        Self {
            templates,
            context_builders: HashMap::new(),
        }
    }

    pub fn render(&self, template_name: &str, context: &PromptContext) -> Result<String, PromptError> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| PromptError::TemplateNotFound(template_name.to_string()))?;

        let mut rendered = template.user_template.clone();
        
        for variable in &template.variables {
            let value = context.get(variable)
                .ok_or_else(|| PromptError::MissingVariable(variable.clone()))?;
            
            rendered = rendered.replace(&format!("{{{}}}", variable), value);
        }

        Ok(rendered)
    }
}
```

## Usage Tracking and Analytics

### Resource Management

```rust
pub struct UsageTracker {
    daily_requests: HashMap<String, u32>, // date -> count
    monthly_requests: HashMap<String, u32>, // month -> count
    token_usage: TokenUsageStats,
    error_rates: ErrorRateTracker,
    performance_metrics: PerformanceMetrics,
}

impl UsageTracker {
    pub fn track_request(&mut self, request: &ClaudeRequest, response: &ClaudeResponse, duration: Duration) {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let month = chrono::Utc::now().format("%Y-%m").to_string();

        // Track request counts
        *self.daily_requests.entry(today).or_insert(0) += 1;
        *self.monthly_requests.entry(month).or_insert(0) += 1;

        // Track token usage
        self.token_usage.input_tokens += response.usage.input_tokens;
        self.token_usage.output_tokens += response.usage.output_tokens;
        self.token_usage.total_cost += self.calculate_cost(&response.usage);

        // Track performance
        self.performance_metrics.add_request_duration(duration);
        self.performance_metrics.update_average_response_time();
    }

    pub fn track_error(&mut self, error: &ClaudeError) {
        self.error_rates.record_error(error);
        
        // Alert if error rate exceeds threshold
        if self.error_rates.current_error_rate() > 0.05 { // 5% error rate
            self.send_error_rate_alert().unwrap_or_else(|e| {
                eprintln!("Failed to send error rate alert: {}", e);
            });
        }
    }

    pub fn get_usage_summary(&self) -> UsageSummary {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let month = chrono::Utc::now().format("%Y-%m").to_string();

        UsageSummary {
            requests_today: self.daily_requests.get(&today).copied().unwrap_or(0),
            requests_this_month: self.monthly_requests.get(&month).copied().unwrap_or(0),
            total_tokens_used: self.token_usage.input_tokens + self.token_usage.output_tokens,
            estimated_monthly_cost: self.token_usage.total_cost,
            average_response_time: self.performance_metrics.average_response_time,
            current_error_rate: self.error_rates.current_error_rate(),
            quota_utilization: self.calculate_quota_utilization(),
        }
    }

    fn calculate_cost(&self, usage: &TokenUsage) -> f64 {
        // Claude 3 Sonnet pricing (as of 2024)
        const INPUT_TOKEN_COST: f64 = 0.003 / 1000.0; // $0.003 per 1K input tokens
        const OUTPUT_TOKEN_COST: f64 = 0.015 / 1000.0; // $0.015 per 1K output tokens

        let input_cost = usage.input_tokens as f64 * INPUT_TOKEN_COST;
        let output_cost = usage.output_tokens as f64 * OUTPUT_TOKEN_COST;
        
        input_cost + output_cost
    }
}
```

## Streaming Responses

### Real-time Code Generation

```rust
impl ClaudeCodeClient {
    pub async fn stream_response(&self, prompt: String) -> Result<ClaudeStream, ClaudeError> {
        self.rate_limiter.acquire().await?;

        let request = ClaudeStreamRequest {
            model: self.model.clone(),
            max_tokens: 4000,
            temperature: 0.1,
            stream: true,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let response = self.http_client
            .post(&format!("{}/v1/messages", self.base_url))
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return self.handle_api_error(response).await;
        }

        Ok(ClaudeStream::new(response))
    }
}

pub struct ClaudeStream {
    inner: Box<dyn futures::Stream<Item = Result<ClaudeStreamChunk, ClaudeError>> + Unpin + Send>,
}

impl ClaudeStream {
    pub fn new(response: reqwest::Response) -> Self {
        let stream = response.bytes_stream()
            .map_err(ClaudeError::from)
            .try_filter_map(|chunk| async move {
                let text = String::from_utf8_lossy(&chunk);
                
                // Parse Server-Sent Events format
                if text.starts_with("data: ") {
                    let json_part = &text[6..]; // Skip "data: "
                    if json_part.trim() == "[DONE]" {
                        return Ok(None);
                    }
                    
                    match serde_json::from_str::<ClaudeStreamChunk>(json_part) {
                        Ok(chunk) => Ok(Some(chunk)),
                        Err(e) => Err(ClaudeError::ParseError(e.to_string())),
                    }
                } else {
                    Ok(None)
                }
            });

        Self {
            inner: Box::new(stream),
        }
    }
}

impl futures::Stream for ClaudeStream {
    type Item = Result<ClaudeStreamChunk, ClaudeError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}
```

## Error Handling and Recovery

### Comprehensive Error Management

```rust
#[derive(Debug, thiserror::Error)]
pub enum ClaudeError {
    #[error("API request failed: {status_code} - {message}")]
    ApiError { status_code: u16, message: String },
    
    #[error("Rate limit exceeded. Retry after: {retry_after:?}")]
    RateLimited { retry_after: Duration },
    
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
    
    #[error("Request timeout after {0:?}")]
    Timeout(Duration),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("JSON parsing error: {0}")]
    ParseError(String),
    
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
    
    #[error("Quota exceeded: {current_usage} / {quota_limit}")]
    QuotaExceeded { current_usage: u32, quota_limit: u32 },
    
    #[error("Model not available: {model}")]
    ModelNotAvailable { model: String },
}

impl ClaudeCodeClient {
    async fn handle_api_error(&self, response: reqwest::Response) -> Result<ClaudeResponse, ClaudeError> {
        let status_code = response.status().as_u16();
        
        match status_code {
            429 => {
                // Rate limited - extract retry-after header
                let retry_after = response.headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(Duration::from_secs)
                    .unwrap_or(Duration::from_secs(60));
                
                Err(ClaudeError::RateLimited { retry_after })
            },
            401 | 403 => {
                let error_body = response.text().await.unwrap_or_default();
                Err(ClaudeError::AuthenticationError(error_body))
            },
            402 => {
                // Payment required - quota exceeded
                Err(ClaudeError::QuotaExceeded {
                    current_usage: 0, // Would extract from response
                    quota_limit: 0,   // Would extract from response
                })
            },
            404 => {
                Err(ClaudeError::ModelNotAvailable {
                    model: self.model.clone(),
                })
            },
            500..=599 => {
                // Server error - should retry
                let error_body = response.text().await.unwrap_or_default();
                Err(ClaudeError::ApiError {
                    status_code,
                    message: format!("Server error: {}", error_body),
                })
            },
            _ => {
                let error_body = response.text().await.unwrap_or_default();
                Err(ClaudeError::ApiError {
                    status_code,
                    message: error_body,
                })
            }
        }
    }

    fn should_retry(&self, error: &ClaudeError) -> bool {
        match error {
            ClaudeError::ApiError { status_code, .. } => {
                // Retry on server errors and certain client errors
                *status_code >= 500 || *status_code == 429
            },
            ClaudeError::NetworkError(_) => true,
            ClaudeError::Timeout(_) => true,
            _ => false,
        }
    }

    fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        // Exponential backoff with jitter
        let base_delay = Duration::from_millis(100);
        let exponential_delay = base_delay * 2_u32.pow(attempt.saturating_sub(1));
        let max_delay = Duration::from_secs(30);
        
        let delay = exponential_delay.min(max_delay);
        
        // Add jitter (±25%)
        let jitter_range = delay.as_millis() / 4;
        let jitter = Duration::from_millis(
            fastrand::u64(0..=jitter_range as u64 * 2)
        ) - Duration::from_millis(jitter_range as u64);
        
        delay + jitter
    }
}
```

## Performance Optimization

### Request Batching and Caching

```rust
pub struct ClaudeRequestBatcher {
    pending_requests: Vec<BatchableRequest>,
    batch_size: usize,
    batch_timeout: Duration,
    cache: Arc<Mutex<LruCache<String, CachedResponse>>>,
}

impl ClaudeRequestBatcher {
    pub async fn add_request(&mut self, request: BatchableRequest) -> Result<ClaudeResult, ClaudeError> {
        // Check cache first
        let cache_key = self.generate_cache_key(&request);
        if let Some(cached) = self.get_cached_response(&cache_key).await {
            if !cached.is_expired() {
                return Ok(cached.result);
            }
        }

        // Add to batch
        self.pending_requests.push(request.clone());

        // Execute batch if threshold reached
        if self.pending_requests.len() >= self.batch_size {
            let results = self.execute_batch().await?;
            return Ok(results.into_iter().find(|r| r.request_id == request.id)
                .map(|r| r.result)
                .unwrap_or_else(|| panic!("Request not found in batch results")));
        }

        // Otherwise, wait for timeout or batch completion
        self.wait_for_batch_completion(request.id).await
    }

    async fn execute_batch(&mut self) -> Result<Vec<BatchResult>, ClaudeError> {
        let requests = std::mem::take(&mut self.pending_requests);
        
        // Combine similar requests or execute in parallel
        let futures = requests.into_iter().map(|req| async move {
            let result = self.execute_single_request(req.clone()).await;
            BatchResult {
                request_id: req.id,
                result: result.unwrap_or_else(|e| {
                    // Return error result
                    ClaudeResult::error(e.to_string())
                }),
            }
        });

        let results = futures::future::join_all(futures).await;
        
        // Cache successful results
        for result in &results {
            if result.result.is_success() {
                let cache_key = self.generate_cache_key_from_id(&result.request_id);
                self.cache_response(cache_key, &result.result).await;
            }
        }

        Ok(results)
    }
}
```

## Testing Strategy

### Claude Code Integration Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn claude_client_handles_rust_code_generation() {
        let client = create_test_claude_client().await;
        
        let result = client.execute_task(
            "Create a Rust function that calculates fibonacci numbers".to_string(),
            ProgrammingLanguage::Rust
        ).await.unwrap();

        match result {
            ClaudeResult::CodeGeneration { code, tests, documentation, .. } => {
                assert!(code.contains("fn fibonacci"));
                assert!(code.contains("-> u64") || code.contains("-> i64"));
                assert!(!tests.is_empty());
                assert!(documentation.is_some());
            },
            _ => panic!("Expected code generation result"),
        }
    }

    #[tokio::test]
    async fn claude_client_handles_rate_limiting() {
        let client = create_rate_limited_claude_client().await;
        
        // Make requests that should trigger rate limiting
        let mut results = Vec::new();
        for i in 0..10 {
            let result = client.execute_task(
                format!("Simple task {}", i),
                ProgrammingLanguage::Rust
            ).await;
            results.push(result);
        }

        // Should have some rate limit errors
        let rate_limit_errors = results.iter()
            .filter(|r| matches!(r, Err(ClaudeError::RateLimited { .. })))
            .count();
        
        assert!(rate_limit_errors > 0);
    }

    #[tokio::test]
    async fn claude_client_retries_on_server_errors() {
        let client = create_test_claude_client_with_mock_server().await;
        
        // Mock server configured to return 500 then 200
        let result = client.execute_task(
            "Test retry logic".to_string(),
            ProgrammingLanguage::Python
        ).await.unwrap();

        // Should succeed after retry
        assert!(result.is_success());
    }

    #[tokio::test] 
    async fn usage_tracker_records_metrics_correctly() {
        let mut tracker = UsageTracker::new();
        
        let request = ClaudeRequest::test_request();
        let response = ClaudeResponse::test_response();
        let duration = Duration::from_millis(500);

        tracker.track_request(&request, &response, duration);

        let summary = tracker.get_usage_summary();
        assert_eq!(summary.requests_today, 1);
        assert!(summary.total_tokens_used > 0);
        assert!(summary.estimated_monthly_cost > 0.0);
    }
}
```

## Security Considerations

### API Key Management

```rust
pub struct ClaudeSecurityManager {
    key_rotation_schedule: KeyRotationSchedule,
    encrypted_keys: HashMap<String, EncryptedApiKey>,
    audit_logger: AuditLogger,
}

impl ClaudeSecurityManager {
    pub fn rotate_api_key(&mut self, client_id: &str) -> Result<String, SecurityError> {
        // Generate new API key (would involve Anthropic API)
        let new_key = self.generate_new_api_key(client_id)?;
        
        // Encrypt and store
        let encrypted = self.encrypt_api_key(&new_key)?;
        self.encrypted_keys.insert(client_id.to_string(), encrypted);
        
        // Log rotation event
        self.audit_logger.log_key_rotation(client_id)?;
        
        // Schedule old key for deactivation
        self.schedule_key_deactivation(client_id)?;
        
        Ok(new_key)
    }

    pub fn validate_request_security(&self, prompt: &str) -> Result<(), SecurityError> {
        // Check for sensitive data in prompts
        if self.contains_sensitive_data(prompt) {
            return Err(SecurityError::SensitiveDataDetected);
        }

        // Check for prompt injection attempts
        if self.is_potential_injection(prompt) {
            return Err(SecurityError::PotentialInjection);
        }

        Ok(())
    }
}
```

## Integration Points

This Claude Code integration module serves as the foundation for:
- [Developer Agent](CLAUDE-agents-developer.md) for code generation and analysis
- [Project Manager Agent](CLAUDE-agents-pm.md) for strategic analysis
- [Discord Integration](CLAUDE-integrations-discord.md) for conversational AI responses
- [GitHub Integration](CLAUDE-integrations-github.md) for automated code reviews

## Related Documentation

- See [Coding Standards](CLAUDE-core-coding-standards.md) for implementation patterns
- See [Implementation Phase 1](CLAUDE-implementation-phase1.md) for setup and deployment
- See [Implementation Phase 1](CLAUDE-implementation-phase1.md) for Claude Code client setup