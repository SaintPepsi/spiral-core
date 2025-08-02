/// 🧪 MOCK CLAUDE CLIENT: Test double for Claude Code client
/// CRITICAL: Provides controlled behavior for testing without external dependencies
/// Why: Enables reliable unit and integration testing
/// Alternative: Real API calls (rejected: slow, expensive, non-deterministic)
use crate::claude_code::{CodeGenerationRequest, CodeGenerationResult};
use crate::models::AgentType;
use crate::SpiralError;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MockClaudeClient {
    failure_mode: Arc<Mutex<bool>>,
    call_count: Arc<Mutex<usize>>,
    responses: Arc<Mutex<Vec<String>>>,
}

impl Default for MockClaudeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MockClaudeClient {
    pub fn new() -> Self {
        Self {
            failure_mode: Arc::new(Mutex::new(false)),
            call_count: Arc::new(Mutex::new(0)),
            responses: Arc::new(Mutex::new(vec![
                "Mock response 1".to_string(),
                "Mock response 2".to_string(),
                "Mock response 3".to_string(),
            ])),
        }
    }

    pub fn set_failure_mode(&mut self, fail: bool) {
        *self.failure_mode.lock().unwrap() = fail;
    }

    pub fn get_call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    pub fn add_response(&mut self, response: String) {
        self.responses.lock().unwrap().push(response);
    }

    /// Generate code implementation for testing
    pub async fn generate_code(
        &self,
        request: CodeGenerationRequest,
    ) -> Result<CodeGenerationResult, SpiralError> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;

        if *self.failure_mode.lock().unwrap() {
            return Err(SpiralError::Agent {
                message: "Mock failure".to_string(),
            });
        }

        let responses = self.responses.lock().unwrap();
        let response_text = responses
            .get(*count - 1)
            .cloned()
            .unwrap_or_else(|| format!("// Mock generated code for task: {}", request.description));

        Ok(CodeGenerationResult {
            code: response_text,
            language: "rust".to_string(),
            explanation: "Mock execution completed".to_string(),
            files_to_create: vec![],
            files_to_modify: vec![],
            session_id: Some("mock-session".to_string()),
            workspace_path: "/tmp/mock-workspace".to_string(),
        })
    }

    /// Execute task as agent for testing
    pub async fn execute_as_agent(
        &self,
        agent_type: AgentType,
        task_content: String,
        _context: std::collections::HashMap<String, String>,
    ) -> Result<CodeGenerationResult, SpiralError> {
        let mut context = std::collections::HashMap::new();
        context.insert("agent_type".to_string(), format!("{agent_type:?}"));

        let request = CodeGenerationRequest {
            description: task_content,
            language: "rust".to_string(),
            context,
            existing_code: None,
            requirements: vec![],
            session_id: None,
        };

        self.generate_code(request).await
    }

    pub async fn get_circuit_breaker_metrics(
        &self,
    ) -> crate::claude_code::circuit_breaker::CircuitBreakerMetrics {
        crate::claude_code::circuit_breaker::CircuitBreakerMetrics {
            state: crate::claude_code::circuit_breaker::CircuitState::Closed,
            failure_count: 0,
            success_count: *self.call_count.lock().unwrap() as u32,
            total_requests: *self.call_count.lock().unwrap() as u64,
            total_failures: 0,
            last_state_change_seconds: 0,
        }
    }
}
