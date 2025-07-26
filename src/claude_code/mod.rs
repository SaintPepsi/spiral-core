use crate::{config::ClaudeCodeConfig, Result, SpiralError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use url::Url;

#[derive(Debug, Clone)]
pub struct ClaudeCodeClient {
    client: Client,
    config: ClaudeCodeConfig,
}

#[derive(Debug, Serialize)]
pub struct ClaudeCodeRequest {
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub messages: Vec<Message>,
    pub tools: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}


#[derive(Debug, Deserialize)]
pub struct ClaudeCodeResponse {
    pub content: Vec<ContentBlock>,
    pub usage: Usage,
    pub stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
    pub tool_use: Option<ToolUse>,
}

#[derive(Debug, Deserialize)]
pub struct ToolUse {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct CodeGenerationRequest {
    pub language: String,
    pub description: String,
    pub context: HashMap<String, String>,
    pub existing_code: Option<String>,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CodeGenerationResult {
    pub code: String,
    pub language: String,
    pub explanation: String,
    pub files_to_create: Vec<FileCreation>,
    pub files_to_modify: Vec<FileModification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCreation {
    pub path: String,
    pub content: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileModification {
    pub path: String,
    pub changes: String,
    pub description: String,
}

// SECURITY: Allowed Claude API domains
const ALLOWED_CLAUDE_DOMAINS: &[&str] = &[
    "api.anthropic.com",
    "api.claude.ai",
];

impl ClaudeCodeClient {
    pub fn new(config: ClaudeCodeConfig) -> Result<Self> {
        // SECURITY: Validate base URL is a trusted Claude endpoint
        Self::validate_base_url(&config.base_url)?;
        
        let client = Client::new();
        Ok(Self { client, config })
    }

    // SECURITY: Validate that base_url points to legitimate Claude API
    fn validate_base_url(base_url: &str) -> Result<()> {
        let url = Url::parse(base_url)
            .map_err(|_| SpiralError::ConfigurationError("Invalid Claude API base URL".to_string()))?;

        // SECURITY: Ensure HTTPS only
        if url.scheme() != "https" {
            return Err(SpiralError::ConfigurationError(
                "Claude API base URL must use HTTPS".to_string()
            ));
        }

        // SECURITY: Validate domain is in allowed list
        if let Some(domain) = url.domain() {
            if !ALLOWED_CLAUDE_DOMAINS.contains(&domain) {
                return Err(SpiralError::ConfigurationError(
                    format!("Claude API domain '{}' is not in allowed list", domain)
                ));
            }
        } else {
            return Err(SpiralError::ConfigurationError(
                "Claude API URL must have a valid domain".to_string()
            ));
        }

        // SECURITY: Ensure no path traversal or suspicious paths
        // Check both the original URL and the normalized path
        if base_url.contains("..") {
            return Err(SpiralError::ConfigurationError(
                "Claude API URL contains path traversal patterns".to_string()
            ));
        }
        
        let path = url.path();
        if path.contains("..") {
            return Err(SpiralError::ConfigurationError(
                "Claude API URL contains invalid path characters".to_string()
            ));
        }

        // SECURITY: Validate path is acceptable for Claude API
        // Allow empty path, root path, or /v1 paths
        if !path.is_empty() && path != "/" && !path.starts_with("/v1") {
            return Err(SpiralError::ConfigurationError(
                format!("Claude API URL path '{}' is not valid", path)
            ));
        }

        Ok(())
    }

    /// 🤖 CLAUDE CODE GENERATION: Primary AI intelligence interface
    /// AUDIT CHECKPOINT: External API integration with security and cost implications
    /// Verify: Request sanitization, response validation, API key handling, rate limiting
    pub async fn generate_code(&self, request: CodeGenerationRequest) -> Result<CodeGenerationResult> {
        info!("Generating code for language: {}", request.language);
        
        // 🔍 REQUEST VALIDATION AUDIT CHECKPOINT: Sanitize before external API call
        // CRITICAL: Prevent injection attacks and data exfiltration via AI prompts
        if request.description.len() > 10000 {
            warn!("Code generation request exceeds safe length: {} chars", request.description.len());
            return Err(SpiralError::Validation("Request description too long".to_string()));
        }
        
        // 🛡️ CONTENT SAFETY: Check for potential prompt injection patterns
        let description_lower = request.description.to_lowercase();
        let injection_patterns = [
            "ignore previous instructions",
            "system:",
            "jailbreak",
            "previous instructions are wrong",
            "new instruction:",
            "override previous context",
            "system: override",
            "break out of your constraints",
        ];
        
        for pattern in &injection_patterns {
            if description_lower.contains(pattern) {
                warn!("Potential prompt injection detected in code generation request: {}", pattern);
                return Err(SpiralError::Validation("Invalid request content".to_string()));
            }
        }
        
        let system_prompt = self.build_system_prompt(&request);
        let user_prompt = self.build_user_prompt(&request);

        // 📊 USAGE TRACKING AUDIT CHECKPOINT: Monitor AI API consumption
        // CRITICAL: Track costs and usage patterns for capacity planning
        let request_start = std::time::Instant::now();
        let claude_request = ClaudeCodeRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens,
            temperature: self.config.temperature,
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: format!("{system_prompt}\n\n{user_prompt}"),
                }
            ],
            tools: None,
        };

        let response = self.send_request(claude_request).await?;
        let duration = request_start.elapsed();
        
        // 📈 PERFORMANCE METRICS: Log API call performance for monitoring
        // Why: Essential for capacity planning and cost optimization on 8GB VPS
        info!(
            "Claude Code API call completed - Duration: {:?}ms, Model: {}, Tokens: {}", 
            duration.as_millis(), 
            self.config.model, 
            self.config.max_tokens
        );
        
        self.parse_code_generation_response(response, request.language)
    }

    pub async fn detect_language(&self, code_snippet: &str, context: &str) -> Result<String> {
        debug!("Detecting language for code snippet");
        
        let prompt = format!(
            "Analyze the following code snippet and context to determine the programming language. \
             Respond with just the language name (e.g., 'rust', 'python', 'javascript').\n\n\
             Context: {context}\n\nCode:\n```\n{code_snippet}\n```"
        );

        let claude_request = ClaudeCodeRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.language_detection_tokens,
            temperature: self.config.language_detection_temperature,
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
            tools: None,
        };

        let response = self.send_request(claude_request).await?;
        
        if let Some(content) = response.content.first() {
            if let Some(text) = &content.text {
                let language = text.trim().to_lowercase();
                info!("Detected language: {}", language);
                return Ok(language);
            }
        }

        Err(SpiralError::LanguageDetection("No language detected".to_string()))
    }

    pub async fn analyze_task(&self, task_description: &str, context: HashMap<String, String>) -> Result<TaskAnalysis> {
        info!("Analyzing task: {}", task_description.chars().take(crate::constants::TASK_DESCRIPTION_PREVIEW_LENGTH).collect::<String>());
        
        let context_str = context.iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Analyze the following task and provide a structured analysis:\n\n\
             Task: {task_description}\n\n\
             Context:\n{context_str}\n\n\
             Provide analysis including:\n\
             1. Task complexity (Low/Medium/High)\n\
             2. Estimated time (in minutes)\n\
             3. Required skills/technologies\n\
             4. Potential challenges\n\
             5. Suggested approach"
        );

        let claude_request = ClaudeCodeRequest {
            model: self.config.model.clone(),
            max_tokens: self.config.task_analysis_tokens,
            temperature: self.config.task_analysis_temperature,
            messages: vec![
                Message {
                    role: "user".to_string(),
                    content: prompt,
                }
            ],
            tools: None,
        };

        let response = self.send_request(claude_request).await?;
        
        if let Some(content) = response.content.first() {
            if let Some(text) = &content.text {
                return Ok(TaskAnalysis {
                    complexity: self.extract_complexity(text),
                    estimated_minutes: self.extract_time_estimate(text),
                    required_skills: self.extract_required_skills(text),
                    challenges: self.extract_challenges(text),
                    approach: self.extract_approach(text),
                    raw_analysis: text.clone(),
                });
            }
        }

        Err(SpiralError::Agent {
            message: "Failed to analyze task".to_string(),
        })
    }

    async fn send_request(&self, request: ClaudeCodeRequest) -> Result<ClaudeCodeResponse> {
        let response = self.client
            .post(format!("{}/v1/messages", self.config.base_url))
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", crate::constants::ANTHROPIC_API_VERSION)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            warn!("Claude Code API error: {}", error_text);
            return Err(SpiralError::Agent {
                message: format!("Claude Code API error: {error_text}"),
            });
        }

        let response_body: ClaudeCodeResponse = response.json().await?;
        debug!("Received response with {} content blocks", response_body.content.len());
        
        Ok(response_body)
    }

    fn build_system_prompt(&self, request: &CodeGenerationRequest) -> String {
        format!(
            "You are a Software Developer Agent in the Spiral Core orchestration system. \
             Generate high-quality {} code following these principles:\n\n\
             1. Follow SOLID principles (Single Responsibility, Open-Closed, Liskov Substitution, Interface Segregation, Dependency Inversion)\n\
             2. Apply DRY principle (Don't Repeat Yourself)\n\
             3. Use SID naming (Short, Intuitive, Descriptive)\n\
             4. Ensure compile-time safety and error handling\n\
             5. Include comprehensive documentation\n\
             6. Follow language-specific best practices\n\
             7. Implement security best practices and validate all inputs\n\
             8. Maintain code quality standards and consistent formatting\n\n\
             Context: {}\n\n\
             Requirements: {}",
            request.language,
            request.context.iter()
                .map(|(k, v)| format!("{k}: {v}"))
                .collect::<Vec<_>>()
                .join(", "),
            request.requirements.join("\n- ")
        )
    }

    fn build_user_prompt(&self, request: &CodeGenerationRequest) -> String {
        let mut prompt = format!("Generate {} code for: {}", request.language, request.description);
        
        // Include context information
        if !request.context.is_empty() {
            prompt.push_str("\n\nContext:");
            for (key, value) in &request.context {
                prompt.push_str(&format!("\n- {}: {}", key, value));
            }
        }
        
        // Include requirements
        if !request.requirements.is_empty() {
            prompt.push_str("\n\nRequirements:");
            for requirement in &request.requirements {
                prompt.push_str(&format!("\n- {}", requirement));
            }
        }
        
        if let Some(existing) = &request.existing_code {
            prompt.push_str(&format!("\n\nExisting code to modify:\n```{}\n{}\n```", request.language, existing));
        }

        prompt.push_str("\n\nProvide the complete implementation with explanations.");
        prompt
    }


    fn parse_code_generation_response(&self, response: ClaudeCodeResponse, language: String) -> Result<CodeGenerationResult> {
        let mut code = String::new();
        let mut explanation = String::new();
        let mut files_to_create = Vec::new();
        let mut files_to_modify = Vec::new();

        for content in response.content {
            match content.content_type.as_str() {
                "text" => {
                    if let Some(text) = content.text {
                        if code.is_empty() && text.contains("```") {
                            code = self.extract_code_block(&text, &language);
                        }
                        explanation.push_str(&text);
                        explanation.push('\n');
                    }
                }
                "tool_use" => {
                    if let Some(tool_use) = content.tool_use {
                        match tool_use.name.as_str() {
                            "create_file" => {
                                if let Ok(file_creation) = serde_json::from_value::<FileCreation>(tool_use.input) {
                                    files_to_create.push(file_creation);
                                }
                            }
                            "modify_file" => {
                                if let Ok(file_modification) = serde_json::from_value::<FileModification>(tool_use.input) {
                                    files_to_modify.push(file_modification);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(CodeGenerationResult {
            code,
            language,
            explanation: explanation.trim().to_string(),
            files_to_create,
            files_to_modify,
        })
    }

    fn extract_code_block(&self, text: &str, language: &str) -> String {
        let patterns = [
            format!("```{language}\n"),
            "```\n".to_string(),
        ];

        for pattern in &patterns {
            if let Some(start) = text.find(pattern) {
                let code_start = start + pattern.len();
                if let Some(end) = text[code_start..].find("\n```") {
                    return text[code_start..code_start + end].to_string();
                }
            }
        }

        text.to_string()
    }

    fn extract_complexity(&self, text: &str) -> String {
        if text.to_lowercase().contains("high") {
            "High".to_string()
        } else if text.to_lowercase().contains("medium") {
            "Medium".to_string()
        } else {
            "Low".to_string()
        }
    }

    fn extract_time_estimate(&self, _text: &str) -> u32 {
        crate::constants::DEFAULT_TIME_ESTIMATE_MINUTES
    }

    fn extract_required_skills(&self, text: &str) -> Vec<String> {
        // Extract skills mentioned in the analysis text
        let text_lower = text.to_lowercase();
        let mut skills = Vec::new();
        
        // Common programming languages
        let languages = ["rust", "python", "javascript", "typescript", "go", "java", "c++", "c#"];
        // Technologies and frameworks
        let technologies = ["docker", "kubernetes", "aws", "git", "sql", "nosql", "api", "rest", "graphql"];
        // Development practices
        let practices = ["testing", "debugging", "refactoring", "architecture", "microservices"];
        
        for skill_set in [&languages[..], &technologies[..], &practices[..]].iter() {
            for skill in *skill_set {
                if text_lower.contains(skill) {
                    skills.push(skill.to_string());
                }
            }
        }

        if skills.is_empty() {
            skills.push(crate::constants::DEFAULT_PROGRAMMING_SKILL.to_string());
        }

        skills
    }

    fn extract_challenges(&self, text: &str) -> Vec<String> {
        let mut challenges = Vec::new();
        let text_lower = text.to_lowercase();
        
        // Look for challenge-related keywords
        let challenge_indicators = [
            ("complex", "Implementation complexity"),
            ("difficult", "Technical difficulty"),
            ("challenge", "Development challenges"),
            ("risk", "Project risks"),
            ("error", "Error handling requirements"),
            ("performance", "Performance optimization"),
            ("scale", "Scalability concerns"),
            ("security", "Security considerations"),
            ("integration", "Integration complexity"),
        ];
        
        for (keyword, challenge) in &challenge_indicators {
            if text_lower.contains(keyword) {
                challenges.push(challenge.to_string());
            }
        }
        
        if challenges.is_empty() {
            challenges.push(crate::constants::DEFAULT_IMPLEMENTATION_CHALLENGE.to_string());
        }
        
        challenges
    }

    fn extract_approach(&self, text: &str) -> String {
        let text_lower = text.to_lowercase();
        
        // Look for approach-related sections
        let approach_keywords = ["approach", "strategy", "method", "plan", "implementation"];
        
        for keyword in &approach_keywords {
            if let Some(start) = text_lower.find(keyword) {
                // Find the sentence containing the approach
                let section_start = text[..start].rfind('.').map(|i| i + 1).unwrap_or(0);
                let section_end = text[start..].find('.').map(|i| start + i + 1).unwrap_or(text.len());
                
                let approach_text = text[section_start..section_end].trim();
                if !approach_text.is_empty() {
                    return approach_text.to_string();
                }
            }
        }
        
        crate::constants::DEFAULT_IMPLEMENTATION_APPROACH.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct TaskAnalysis {
    pub complexity: String,
    pub estimated_minutes: u32,
    pub required_skills: Vec<String>,
    pub challenges: Vec<String>,
    pub approach: String,
    pub raw_analysis: String,
}

// 🧪 TEST MODULE: Comprehensive testing for external AI integration
#[cfg(test)]
mod tests;

// 🔧 PUBLIC TEST UTILITIES: For integration testing from other modules
// #[cfg(test)]
// pub use tests::security::*;  // Disabled until security module has exports