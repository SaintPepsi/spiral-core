# CLAUDE-core-coding-standards.md

**Purpose**: Establishes SOLID, DRY, and SID coding principles for all Spiral Core development
**Dependencies**: None (this is a foundational reference)
**Updated**: 2024-07-24

## Quick Start

All Spiral Core code must follow these three core principles:

- **SOLID**: Single Responsibility, Open-Closed, Liskov Substitution, Interface Segregation, Dependency Inversion
- **DRY**: Don't Repeat Yourself - single source of truth for all knowledge
- **SID**: Short, Intuitive, Descriptive naming for maximum clarity

## SOLID Principles

### Single Responsibility Principle (SRP)

Each component has one clear responsibility and one reason to change.

```rust
// ✅ Good - Single responsibility
pub struct DeveloperAgent {
    llm_client: Box<dyn LLMClient>,
    prompts_remaining: u32,
}

impl DeveloperAgent {
    pub async fn generate_code(&mut self, requirements: &str) -> Result<CodeResult, AgentError> {
        // Only handles code generation
    }
}

// ✅ Good - Separate responsibility
pub struct DiscordInterface {
    bot_client: serenity::Client,
}

impl DiscordInterface {
    pub async fn send_message(&self, content: &str) -> Result<(), DiscordError> {
        // Only handles Discord communication
    }
}

// ❌ Bad - Mixed responsibilities
pub struct BadAgentManager {
    pub async fn generate_code(&mut self, requirements: &str) -> Result<CodeResult, AgentError> {
        // Handles code generation
    }

    pub async fn send_discord_message(&self, content: &str) -> Result<(), DiscordError> {
        // Also handles Discord communication - violates SRP
    }

    pub async fn manage_database(&self, query: &str) -> Result<DbResult, DbError> {
        // Also handles database operations - violates SRP
    }
}
```

### Open-Closed Principle (OCP)

Components extensible through traits, not modification.

```rust
// ✅ Good - Open for extension, closed for modification
pub trait LLMClient: Send + Sync {
    async fn generate_response(&self, prompt: &str) -> Result<String, LLMError>;
}

pub struct ClaudeClient { /* implementation */ }
pub struct OllamaClient { /* implementation */ }
pub struct OpenAIClient { /* implementation */ }

// All implement LLMClient trait - no modification of existing code needed

// ❌ Bad - Requires modification to add new providers
pub struct BadLLMManager {
    pub fn generate_response(&self, provider: &str, prompt: &str) -> Result<String, Error> {
        match provider {
            "claude" => self.call_claude_api(prompt),
            "openai" => self.call_openai_api(prompt),
            // Adding new provider requires modifying this function!
            "ollama" => self.call_ollama_api(prompt), // Violates OCP
            _ => Err(Error::UnsupportedProvider),
        }
    }

    // Every new provider needs a new method here
    fn call_claude_api(&self, prompt: &str) -> Result<String, Error> { /* ... */ }
    fn call_openai_api(&self, prompt: &str) -> Result<String, Error> { /* ... */ }
    fn call_ollama_api(&self, prompt: &str) -> Result<String, Error> { /* ... */ }
}
```

### Liskov Substitution Principle (LSP)

Any component implementing a trait must be interchangeable with others.

```rust
// ✅ Good - All agents substitutable
pub trait Agent: Send + Sync {
    async fn process_request(&mut self, request: &AgentRequest) -> Result<AgentResponse, AgentError>;
    fn agent_type(&self) -> AgentType;
    fn prompts_remaining(&self) -> u32;
}

// Any Agent can be used in agent manager without knowing specific type
pub struct AgentManager {
    agents: Vec<Box<dyn Agent>>,
}

impl AgentManager {
    pub async fn process_with_any_agent(&mut self, request: &AgentRequest) -> Result<AgentResponse, Error> {
        // Can use any agent interchangeably - LSP satisfied
        let agent = &mut self.agents[0];
        agent.process_request(request).await
    }
}

// ❌ Bad - LSP violation through different contracts
pub trait BadAgent: Send + Sync {
    async fn process_request(&mut self, request: &AgentRequest) -> Result<AgentResponse, AgentError>;
}

pub struct BadDeveloperAgent;
impl BadAgent for BadDeveloperAgent {
    async fn process_request(&mut self, request: &AgentRequest) -> Result<AgentResponse, AgentError> {
        // Developer agent returns immediate response
        Ok(AgentResponse::immediate("Task started"))
    }
}

pub struct BadProjectManagerAgent;
impl BadAgent for BadProjectManagerAgent {
    async fn process_request(&mut self, request: &AgentRequest) -> Result<AgentResponse, AgentError> {
        // PM agent requires different usage pattern - violates LSP!
        if request.requires_analysis {
            Ok(AgentResponse::analysis("Analysis complete"))
        } else {
            panic!("PM agent requires analysis flag!") // Breaks substitutability
        }
    }
}
```

### Interface Segregation Principle (ISP)

Components only depend on interfaces they actually use.

```rust
// ✅ Good - Segregated interfaces
pub trait MessageSender {
    async fn send_message(&self, msg: &AgentMessage) -> Result<(), MessageError>;
}

pub trait MessageReceiver {
    async fn receive_messages(&mut self) -> Result<Vec<AgentMessage>, MessageError>;
}

pub trait ConversationManager {
    async fn start_conversation(&self, topic: &str) -> Result<ConversationId, ConversationError>;
}

// Agents only implement what they need
impl MessageSender for DeveloperAgent { /* only sending */ }
impl MessageReceiver for QAAgent { /* only receiving */ }

// ❌ Bad - Monolithic interface forces unnecessary dependencies
pub trait BadAgentInterface {
    async fn send_message(&self, msg: &AgentMessage) -> Result<(), MessageError>;
    async fn receive_messages(&mut self) -> Result<Vec<AgentMessage>, MessageError>;
    async fn start_conversation(&self, topic: &str) -> Result<ConversationId, ConversationError>;
    async fn manage_database(&self, query: &str) -> Result<DbResult, DbError>;
    async fn handle_file_upload(&self, file: File) -> Result<(), FileError>;
    async fn process_payments(&self, payment: Payment) -> Result<(), PaymentError>;
}

// Forces all agents to implement irrelevant methods
impl BadAgentInterface for DeveloperAgent {
    // Developer agent shouldn't handle payments!
    async fn process_payments(&self, payment: Payment) -> Result<(), PaymentError> {
        Err(PaymentError::NotSupported) // Violates ISP
    }
}
```

### Dependency Inversion Principle (DIP)

Depend on abstractions (traits), not concrete implementations.

```rust
// ✅ Good - Depends on abstraction
pub struct ProjectManager {
    llm_client: Box<dyn LLMClient>,
    message_sender: Box<dyn MessageSender>,
    tool_registry: Box<dyn ToolRegistry>,
}

// ❌ Bad - Depends on concrete types
pub struct BadProjectManager {
    claude_client: ClaudeClient,
    discord_sender: DiscordSender,
    file_tool_registry: FileToolRegistry,
}
```

## DRY Principle (Don't Repeat Yourself)

Every piece of knowledge must have a single, unambiguous, authoritative representation.

### Configuration Management

```rust
// ✅ Good - Single source of truth
#[derive(Deserialize)]
pub struct Config {
    pub agents: AgentConfig,
    pub llm: LLMConfig,
    pub discord: DiscordConfig,
}

// All components use the same config instance
```

### Error Handling

```rust
// ✅ Good - Centralized error types
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    #[error("LLM error: {0}")]
    LLM(#[from] LLMError),
    #[error("Communication error: {0}")]
    Communication(#[from] MessageError),
}

// ❌ Bad - Repeated error handling in each module
// Each module defines its own similar error types and handling
```

### Prompt Templates

```rust
// ✅ Good - Reusable prompt templates
pub struct PromptTemplateEngine {
    templates: HashMap<String, String>,
}

impl PromptTemplateEngine {
    pub fn render(&self, template_name: &str, context: &Context) -> Result<String, TemplateError> {
        // Single implementation used by all agents
    }
}
```

## SID Naming Convention (Short, Intuitive, Descriptive)

### Short

Names should be concise but not cryptic.

```rust
// ✅ Good
let agent_count = 6;
let msg_queue = MessageQueue::new();
let llm_client = create_client();

// ❌ Bad - Too long
let number_of_currently_active_agents = 6;
let message_queue_for_inter_agent_communication = MessageQueue::new();

// ❌ Bad - Too cryptic
let n = 6;
let mq = MessageQueue::new();
let c = create_client();
```

### Intuitive

Names should read naturally and be immediately understandable.

```rust
// ✅ Good - Natural language flow
if should_escalate_to_human {
    agent.request_human_approval().await?;
}

let can_process_request = agent.has_sufficient_prompts();

// ❌ Bad - Awkward or unclear
if escalation_human_needed {
    agent.human_request_approve().await?;
}

let prompt_check_ok = agent.prompt_count_sufficient();
```

### Descriptive

Names should clearly indicate purpose and content.

```rust
// ✅ Good - Clear purpose
pub struct ToolBuildingRequest {
    pub tool_name: String,
    pub requesting_agent: AgentId,
    pub requirements: Vec<String>,
    pub priority_score: u8,
}

pub async fn validate_tool_requirements(req: &ToolBuildingRequest) -> Result<ValidationResult, ValidationError> {
    // Function name clearly indicates what it does
}

// ❌ Bad - Vague purpose
pub struct Request {
    pub name: String,
    pub agent: AgentId,
    pub data: Vec<String>,
    pub score: u8,
}

pub async fn check(req: &Request) -> Result<bool, Error> {
    // Unclear what this function checks
}
```

## Rust-Specific Best Practices

### Type Safety

```rust
// ✅ Good - Strong typing prevents errors
#[derive(Debug, Clone, PartialEq)]
pub struct AgentId(String);

#[derive(Debug, Clone, PartialEq)]
pub struct ConversationId(String);

// Impossible to accidentally pass wrong ID type
pub fn get_agent_by_id(id: AgentId) -> Option<Box<dyn Agent>> { /* */ }
```

### Error Handling

```rust
// ✅ Good - Comprehensive error types with context
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Insufficient prompts: {remaining} remaining, {required} required")]
    InsufficientPrompts { remaining: u32, required: u32 },

    #[error("Agent {agent_id} not found")]
    AgentNotFound { agent_id: AgentId },

    #[error("Invalid agent state transition from {from:?} to {to:?}")]
    InvalidStateTransition { from: AgentState, to: AgentState },
}
```

### Memory Management

```rust
// ✅ Good - Clear ownership patterns
pub struct AgentManager {
    agents: HashMap<AgentId, Box<dyn Agent>>,
    message_queue: Arc<MessageQueue>,
    config: Arc<Config>,
}

// Use Arc for shared immutable data, Box for owned dynamic types
```

## Code Organization Rules

### Module Structure

```rust
// src/lib.rs
pub mod agents;      // All agent implementations
pub mod communication; // Message queues, Discord interface
pub mod llm;         // LLM client abstractions
pub mod tools;       // Tool building and registry
pub mod config;      // Configuration management
pub mod errors;      // Centralized error types

// Each module has clear, non-overlapping responsibilities
```

### Testing Standards

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn developer_agent_generates_valid_code() {
        // Test names describe exact behavior being tested
        // Use descriptive test data that represents real scenarios
    }

    #[tokio::test]
    async fn agent_escalates_when_prompts_exhausted() {
        // Each test validates one specific behavior
    }
}
```

## Common Pitfalls

### Violating Single Responsibility

- **Problem**: Agent classes that handle multiple concerns (communication + logic + persistence)
- **Solution**: Separate into focused components with clear interfaces

### Over-abstraction

- **Problem**: Creating traits/interfaces for every single struct
- **Solution**: Only abstract when you have multiple implementations or expect future extension

### Inconsistent Naming

- **Problem**: Mixed naming conventions across the codebase
- **Solution**: Use this SID guide consistently and review naming in code reviews

## Integration Points

This coding standards module is referenced by:

- All agent-specific documentation files
- Integration documentation (Discord, GitHub, Claude Code)
- Implementation phase documentation

## Testing Strategy

These standards are enforced through:

- Rust compiler (type safety, memory management)
- Code reviews (naming conventions, SOLID adherence)
- Automated linting (clippy, rustfmt)
- Architecture decision records (ADRs) for major design choices

## Related Documentation

- See [Developer Agent](CLAUDE-agents-developer.md) for agent-specific applications
- See [Discord Integration](CLAUDE-integrations-discord.md) for Discord-specific patterns
- See [Implementation Guide](CLAUDE-implementation-phase1.md) for practical application
