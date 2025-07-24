# CLAUDE-integrations-discord.md

**Purpose**: Discord bot integration patterns and conversational agent mention system
**Dependencies**: [Coding Standards](CLAUDE-core-coding-standards.md)
**Updated**: 2024-07-24

## Quick Start

Discord integration uses conversational agent mentions instead of commands:

```typescript
// Instead of: !dev "create an API"
// Use: @SpiralDev can you create a FastAPI todo application?

// Instead of: !pm "analyze architecture"  
// Use: @SpiralPM what do you think about microservices vs monolith?
```

## Discord Architecture Overview

### Two-Service Architecture
1. **Discord Bot Service** (Node.js/TypeScript) - Handles Discord API and user interaction
2. **Rust Backend** (HTTP API) - Processes agent logic and Claude Code integration

```
Discord User -> @SpiralDev -> Discord Bot -> HTTP API -> Rust Agent -> Claude Code -> Results -> Discord
```

## Discord Bot Implementation

### Agent Mention Handling
```typescript
// discord-bot/src/agents/dev-bot.ts
export class DevBot extends DiscordAgent {
  constructor() {
    super({
      name: "SpiralDev",
      avatar: "🚀",
      personality: "enthusiastic_developer",
      backend_endpoint: "http://localhost:8080/api/agents/developer"
    });
  }

  async handleMention(message: Message): Promise<void> {
    const userId = message.author.id;
    const content = message.content.replace(/<@!?\d+>/g, '').trim();
    
    // Send to Rust backend with agent context
    const response = await this.sendToBackend({
      agent_type: "developer",
      user_id: userId,
      channel_id: message.channel.id,
      message: content,
      conversation_context: await this.getConversationHistory(message.channel.id)
    });
    
    // Respond as SpiralDev personality
    await message.reply({
      content: `${response.message}`,
      embeds: response.suggested_actions ? [{
        title: "🛠️ I can help with:",
        description: response.suggested_actions.map(action => `• ${action}`).join('\n'),
        color: 0x00ff00
      }] : []
    });
    
    // If user wants to proceed, offer to start work
    if (response.can_execute) {
      await message.react('✅'); // User can react to confirm execution
    }
  }
}
```

### Agent Personality System
```typescript
// discord-bot/src/agents/pm-bot.ts  
export class PMBot extends DiscordAgent {
  constructor() {
    super({
      name: "SpiralPM", 
      avatar: "📊",
      personality: "strategic_project_manager",
      backend_endpoint: "http://localhost:8080/api/agents/project-manager"
    });
  }

  async handleMention(message: Message): Promise<void> {
    // Similar pattern but focuses on analysis, not execution
    const response = await this.sendToBackend({
      agent_type: "project_manager",
      analysis_request: content,
      project_context: await this.getProjectContext(message.channel.id)
    });
    
    await message.reply({
      content: response.analysis,
      embeds: [{
        title: "📋 Strategic Recommendations:",
        description: response.recommendations.join('\n'),
        color: 0x0066cc
      }]
    });
  }
}
```

### Base Discord Agent Class
```typescript
// discord-bot/src/base/discord-agent.ts
export abstract class DiscordAgent {
  protected config: AgentConfig;
  protected httpClient: AxiosInstance;

  constructor(config: AgentConfig) {
    this.config = config;
    this.httpClient = axios.create({
      baseURL: config.backend_endpoint,
      timeout: 30000,
    });
  }

  protected async sendToBackend(payload: AgentRequest): Promise<AgentResponse> {
    try {
      const response = await this.httpClient.post('/mention', payload);
      return response.data;
    } catch (error) {
      console.error(`Error communicating with ${this.config.name} backend:`, error);
      throw new Error(`Backend communication failed for ${this.config.name}`);
    }
  }

  protected async getConversationHistory(channelId: string): Promise<ConversationContext> {
    // Fetch recent message history for context
    // Implementation depends on Discord.js version and caching strategy
  }

  abstract handleMention(message: Message): Promise<void>;
}
```

## Rust Backend Integration

### HTTP API Endpoints
```rust
// src/discord/mod.rs
use warp::Filter;

pub fn discord_routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let mention_route = warp::path!("api" / "agents" / String / "mention")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_agent_mention);

    let status_route = warp::path!("api" / "agents" / "status")
        .and(warp::get())
        .and_then(get_agents_status);

    let confirm_route = warp::path!("api" / "agents" / String / "confirm")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(handle_confirmation);

    mention_route.or(status_route).or(confirm_route)
}

async fn handle_agent_mention(
    agent_type: String,
    request: DiscordMentionRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    let agent_manager = get_agent_manager();
    
    let response = match agent_type.as_str() {
        "developer" => agent_manager.handle_developer_mention(request).await?,
        "project-manager" => agent_manager.handle_pm_mention(request).await?,
        "qa" => agent_manager.handle_qa_mention(request).await?,
        _ => return Err(warp::reject::not_found()),
    };

    Ok(warp::reply::json(&response))
}
```

### Discord Message Models
```rust
// src/discord/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct DiscordMentionRequest {
    pub agent_type: String,
    pub user_id: u64,
    pub channel_id: u64,
    pub message: String,
    pub conversation_context: Option<ConversationContext>,
}

#[derive(Debug, Serialize)]
pub struct DiscordAgentResponse {
    pub message: String,
    pub suggested_actions: Vec<String>,
    pub can_execute: bool,
    pub requires_followup: bool,
    pub pending_context: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct ConversationContext {
    pub recent_messages: Vec<DiscordMessage>,
    pub topic: Option<String>,
    pub participants: Vec<u64>,
}
```

## Language Detection Integration

### Discord Context for Language Inference
```rust
impl LanguageInferenceEngine {
    pub async fn infer_from_discord_context(&self, request: &DiscordMentionRequest) -> LanguageContext {
        let mut detected_language = None;
        let mut confidence = 0.0;
        let mut source = InferenceSource::Unknown;

        // 1. Check explicit language mentions in message
        if let Some(lang) = self.extract_explicit_language(&request.message) {
            detected_language = Some(lang);
            confidence = 0.95;
            source = InferenceSource::UserExplicit;
        }

        // 2. Check conversation history for language context
        else if let Some(context) = &request.conversation_context {
            if let Some(lang) = self.infer_from_conversation_history(context).await {
                detected_language = Some(lang);
                confidence = 0.70;
                source = InferenceSource::PreviousConversation;
            }
        }

        // 3. Check for framework/library keywords
        else if let Some(lang) = self.infer_from_frameworks(&request.message) {
            detected_language = Some(lang);
            confidence = 0.65;
            source = InferenceSource::UserPromptKeywords;
        }

        LanguageContext {
            detected_language,
            confidence,
            inference_source: source,
            discord_context: Some(request.clone()),
        }
    }
}
```

## Agent Personality Responses

### SpiralDev Personality
```rust
impl DeveloperAgent {
    fn create_discord_response(&self, task_result: &TaskResult, language_context: &LanguageContext) -> DiscordAgentResponse {
        let enthusiasm_emojis = ["🚀", "✨", "🔥", "⚡", "🎯"];
        let emoji = enthusiasm_emojis[fastrand::usize(..enthusiasm_emojis.len())];
        
        let message = match language_context.confidence {
            conf if conf >= 0.8 => {
                format!(
                    "Hey there! {} I'm excited to work on this!\n\n\
                    **Task**: {}\n\
                    **Language**: {} ({}% confidence)\n\n\
                    I'll create a complete implementation with:\n\
                    • Best practices and clean architecture 🏗️\n\
                    • Comprehensive testing suite 🧪\n\
                    • Production-ready error handling ⚠️\n\
                    • Clear documentation with examples 📚\n\n\
                    Ready to start? React with ✅!",
                    emoji,
                    task_result.task,
                    language_context.detected_language.as_ref().unwrap().display_name(),
                    (language_context.confidence * 100.0) as u8
                )
            },
            _ => {
                self.create_language_clarification_response(&task_result.task, language_context)
            }
        };

        DiscordAgentResponse {
            message,
            suggested_actions: vec![
                "Create complete project structure".to_string(),
                "Add comprehensive test suite".to_string(),
                "Generate documentation".to_string(),
            ],
            can_execute: language_context.confidence >= 0.8,
            requires_followup: language_context.confidence < 0.8,
            pending_context: None,
        }
    }
}
```

### SpiralPM Personality
```rust
impl ProjectManagerAgent {
    fn create_discord_response(&self, analysis: &StrategicAnalysis) -> DiscordAgentResponse {
        let message = format!(
            "Great strategic question! 📊 Let me break this down:\n\n\
            **Analysis**: {}\n\n\
            **Key Considerations**:\n{}\n\n\
            **My Recommendation**: {}\n\n\
            **Next Steps**:\n{}\n\n\
            Want me to dive deeper into any of these areas?",
            analysis.summary,
            analysis.considerations.iter()
                .map(|c| format!("• {}", c))
                .collect::<Vec<_>>()
                .join("\n"),
            analysis.recommendation,
            analysis.next_steps.iter()
                .enumerate()
                .map(|(i, step)| format!("{}. {}", i + 1, step))
                .collect::<Vec<_>>()
                .join("\n")
        );

        DiscordAgentResponse {
            message,
            suggested_actions: vec![
                "Create detailed implementation plan".to_string(),
                "Analyze risks and mitigation strategies".to_string(),
                "Estimate timeline and resources".to_string(),
            ],
            can_execute: false, // PM analyzes, doesn't directly execute
            requires_followup: false,
            pending_context: None,
        }
    }
}
```

## Conversation Flow Examples

### High Confidence Language Detection
```markdown
You: @SpiralDev create a Python FastAPI application for user management

SpiralDev: Hey! 🚀 Perfect - I can see you want a **Python FastAPI** application! 

I'll create a complete user management API with:
• FastAPI framework with automatic OpenAPI docs 📚
• User registration, authentication, and CRUD operations 👥
• SQLAlchemy with Alembic migrations 🗄️
• JWT token authentication 🔐
• Pydantic models for validation ✅
• Comprehensive pytest test suite 🧪
• Production-ready error handling and logging 📊

React with ✅ to start building!
```

### Language Clarification Required
```markdown
You: @SpiralDev build me a REST API for a todo application

SpiralDev: Hey! 🚀 I'd love to help you build that REST API!

I need to know what programming language you'd like me to use.

**Your request**: build me a REST API for a todo application

Which language would you prefer? Here are some popular options:
• **Python** 🐍 (great for APIs, FastAPI/Django)
• **JavaScript/Node.js** ⚡ (Express, Koa)
• **TypeScript** 📘 (NestJS, type-safe Node.js)
• **Rust** 🦀 (Actix-web, Warp - high performance)
• **Go** 🔄 (Gin, Echo - great for microservices)
• **Java** ☕ (Spring Boot - enterprise ready)
• **Or any other language you prefer!**

Just mention me again with your language choice! 🎯
```

## Error Handling Patterns

### Network Failures
```typescript
// discord-bot/src/utils/error-handler.ts
export class DiscordErrorHandler {
  static async handleBackendError(error: AxiosError, message: Message, agentName: string): Promise<void> {
    if (error.code === 'ECONNREFUSED') {
      await message.reply(
        `😔 Sorry, ${agentName} is temporarily unavailable (backend down). Please try again in a few minutes!`
      );
    } else if (error.code === 'TIMEOUT') {
      await message.reply(
        `⏱️ ${agentName} is taking longer than expected. The task might be complex - please be patient!`
      );
    } else {
      await message.reply(
        `⚠️ ${agentName} encountered an error. The development team has been notified!`
      );
      console.error(`Backend error for ${agentName}:`, error);
    }
  }
}
```

### Rate Limiting
```typescript
// discord-bot/src/utils/rate-limiter.ts
export class AgentRateLimiter {
  private userRequests = new Map<string, number[]>();
  private readonly maxRequestsPerMinute = 5;

  canMakeRequest(userId: string): boolean {
    const now = Date.now();
    const userRequestTimes = this.userRequests.get(userId) || [];
    
    // Remove requests older than 1 minute
    const recentRequests = userRequestTimes.filter(time => now - time < 60000);
    
    if (recentRequests.length >= this.maxRequestsPerMinute) {
      return false;
    }
    
    recentRequests.push(now);
    this.userRequests.set(userId, recentRequests);
    return true;
  }
}
```

## Testing Strategy

### Discord Bot Testing
```typescript
// discord-bot/tests/agents/dev-bot.test.ts
import { DevBot } from '../src/agents/dev-bot';
import { MockMessage, MockBackend } from './test-utils';

describe('DevBot', () => {
  let bot: DevBot;
  let mockBackend: MockBackend;

  beforeEach(() => {
    mockBackend = new MockBackend();
    bot = new DevBot();
  });

  test('handles Python FastAPI mention correctly', async () => {
    const message = new MockMessage('@SpiralDev create a Python FastAPI app');
    
    await bot.handleMention(message);
    
    expect(mockBackend.lastRequest).toMatchObject({
      agent_type: 'developer',
      message: 'create a Python FastAPI app'
    });
    
    expect(message.lastReply).toContain('Python FastAPI');
    expect(message.reactions).toContain('✅');
  });
});
```

## Deployment Configuration

### Environment Variables
```bash
# discord-bot/.env
DISCORD_TOKEN=your_bot_token_here
RUST_BACKEND_URL=http://localhost:8080
NODE_ENV=development
LOG_LEVEL=info

# Rate limiting
MAX_REQUESTS_PER_MINUTE=5
REQUEST_TIMEOUT_MS=30000

# Agent configuration
SPIRALDEV_ENABLED=true
SPIRALPM_ENABLED=true
SPIRALQA_ENABLED=false
```

### Docker Configuration
```dockerfile
# discord-bot/Dockerfile
FROM node:18-alpine

WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production

COPY src/ ./src/
COPY tsconfig.json ./

RUN npm run build

USER node
CMD ["npm", "start"]
```

## Common Pitfalls

### Bot Token Security
- **Problem**: Hardcoding Discord bot tokens in code
- **Solution**: Use environment variables and secure secret management

### Rate Limit Violations
- **Problem**: Hitting Discord API rate limits during high activity
- **Solution**: Implement request queuing and exponential backoff

### Memory Leaks in Conversation History
- **Problem**: Unbounded growth of conversation context storage
- **Solution**: Implement LRU cache with size limits for conversation history

## Integration Points

This Discord integration works with:
- [Developer Agent](CLAUDE-agents-developer.md) for code generation requests
- [Project Manager Agent](CLAUDE-agents-pm.md) for strategic analysis
- [Claude Code Integration](CLAUDE-integrations-claude-code.md) for AI responses

## Related Documentation

- See [Coding Standards](CLAUDE-core-coding-standards.md) for TypeScript and Rust best practices
- See [Implementation Phase 1](CLAUDE-implementation-phase1.md) for deployment steps
- See [Implementation Phase 1](CLAUDE-implementation-phase1.md) for Discord bot setup