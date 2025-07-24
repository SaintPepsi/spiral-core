# CLAUDE-implementation-phase1.md

**Purpose**: Phase 1 implementation guide for foundation setup and core systems
**Dependencies**: [Coding Standards](CLAUDE-core-coding-standards.md), [Claude Code Integration](CLAUDE-integrations-claude-code.md)
**Updated**: 2024-07-24

## Quick Start

Phase 1 establishes the foundation for the Spiral Core system with Discord bot integration and basic developer agent functionality:

```bash
# Phase 1 Success Criteria:
# User types: @SpiralDev "create a FastAPI todo app"
# System responds: 🚀 Building Python FastAPI application!
# Result: Complete project with tests deployed to GitHub
```

## Implementation Overview

### Phase 1 Goals

1. **Discord Bot Integration** - Primary user interface with conversational agent mentions
2. **Developer Agent** - Autonomous code generation with language detection
3. **Claude Code Client** - Primary intelligence engine integration
4. **Minimal HTTP API** - Agent communication endpoints

### Success Metrics

- Discord bot responds to @SpiralDev mentions within 2 seconds
- Language detection accuracy >85% for common frameworks
- Code generation produces runnable, tested applications
- System resource usage <2GB total memory

## Project Initialization

### Repository Setup

```bash
# 1. Initialize Rust project
cargo init spiral-core
cd spiral-core

# 2. Create module structure
mkdir -p src/{agents,claude,discord,coordination,errors,models}
mkdir -p discord-bot/src/{agents,handlers,utils}
mkdir -p tests/{unit,integration}
mkdir -p docs/examples
mkdir -p scripts

# 3. Setup development environment
cat > .env.example << 'EOF'
# Claude Code Configuration
CLAUDE_API_KEY=your_claude_api_key_here
CLAUDE_BASE_URL=https://api.anthropic.com
CLAUDE_MODEL=claude-3-sonnet-20240229

# Discord Bot Configuration
DISCORD_TOKEN=your_discord_bot_token_here
DISCORD_GUILD_ID=your_test_server_id

# Rust Backend Configuration
RUST_BACKEND_URL=http://localhost:8080
RUST_LOG=info

# Database Configuration (for future phases)
DATABASE_URL=postgresql://localhost/spiral_core_dev

# Security
ENCRYPTION_KEY=generate_32_byte_key_here
WEBHOOK_SECRET=your_github_webhook_secret
EOF

# 4. Copy to actual .env and fill in values
cp .env.example .env
```

### Cargo.toml Configuration

```toml
[package]
name = "spiral-core"
version = "0.1.0"
edition = "2021"
authors = ["Anti Spiral Interactive"]
description = "AI agent orchestration system with Claude Code integration"
license = "MIT"
repository = "https://github.com/your-org/spiral-core"

[dependencies]
# Core async runtime
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"

# HTTP server and client
warp = "0.3"
reqwest = { version = "0.11", features = ["json", "rustls-tls", "stream"] }
hyper = { version = "0.14", features = ["server", "tcp", "http1", "http2"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging and tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Configuration
config = "0.13"
dotenv = "0.15"

# Security and crypto
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Rate limiting and caching
governor = "0.6"
moka = { version = "0.12", features = ["future"] }

# Database (for future phases)
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "chrono", "uuid"], optional = true }

[dev-dependencies]
mockall = "0.12"
wiremock = "0.5"
tempfile = "3.8"

[features]
default = []
database = ["sqlx"]

[[bin]]
name = "spiral-core"
path = "src/main.rs"
```

### Discord Bot Package.json

```bash
# Initialize Discord bot
cd discord-bot
npm init -y

# Install dependencies
npm install discord.js@14 axios dotenv winston

# Install dev dependencies
npm install -D typescript @types/node ts-node nodemon jest @types/jest eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin prettier

# Create TypeScript config
cat > tsconfig.json << 'EOF'
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "commonjs",
    "lib": ["ES2022"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "**/*.test.ts"]
}
EOF

# Update package.json scripts
npm pkg set scripts.dev="nodemon --exec ts-node src/index.ts"
npm pkg set scripts.build="tsc"
npm pkg set scripts.start="node dist/index.js"
npm pkg set scripts.test="jest"
npm pkg set scripts.lint="eslint src/**/*.ts"
```

## Core System Implementation

### 1. Claude Code Client (Rust)

```rust
// src/claude/client.rs
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn, error};

#[derive(Debug, Clone)]
pub struct ClaudeCodeClient {
    api_key: String,
    base_url: String,
    model: String,
    http_client: HttpClient,
    rate_limiter: Arc<governor::RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>,
}

impl ClaudeCodeClient {
    pub fn new(config: ClaudeConfig) -> Result<Self, ClaudeError> {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Spiral-Core/1.0")
            .build()?;

        // Rate limit: 50 requests per minute (conservative)
        let rate_limiter = Arc::new(
            governor::RateLimiter::direct(
                governor::Quota::per_minute(std::num::NonZeroU32::new(50).unwrap())
            )
        );

        Ok(Self {
            api_key: config.api_key,
            base_url: config.base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string()),
            model: config.model.unwrap_or_else(|| "claude-3-sonnet-20240229".to_string()),
            http_client,
            rate_limiter,
        })
    }

    pub async fn execute_task(&self, prompt: String, language: ProgrammingLanguage) -> Result<ClaudeResult, ClaudeError> {
        // Rate limiting
        self.rate_limiter.until_ready().await;

        let system_prompt = self.build_system_context(language);
        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 4000,
            temperature: 0.1,
            system: Some(system_prompt),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        info!("Sending request to Claude API");

        let response = timeout(
            Duration::from_secs(30),
            self.send_request(&request)
        ).await??;

        Ok(ClaudeResult::from_response(response, language)?)
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
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Claude API error: {} - {}", status, error_text);
            return Err(ClaudeError::ApiError {
                status_code: status.as_u16(),
                message: error_text
            });
        }

        let claude_response: ClaudeResponse = response.json().await?;
        info!("Received response from Claude API");
        Ok(claude_response)
    }

    fn build_system_context(&self, language: ProgrammingLanguage) -> String {
        match language {
            ProgrammingLanguage::Python => {
                "You are an expert Python developer. Create production-ready code with:\n\
                • Type hints and comprehensive docstrings\n\
                • FastAPI for web APIs with OpenAPI docs\n\
                • pytest test suite with fixtures\n\
                • Error handling with custom exceptions\n\
                • Requirements.txt with pinned versions\n\
                • Follow PEP 8 and include comprehensive tests"
            }
            ProgrammingLanguage::Rust => {
                "You are an expert Rust developer. Create production-ready code with:\n\
                • Proper error handling with Result<T, E> and thiserror\n\
                • Async/await with tokio for concurrent operations\n\
                • Comprehensive #[cfg(test)] modules\n\
                • Cargo.toml with appropriate dependencies\n\
                • Memory safety and zero-cost abstractions\n\
                • Include benchmarks where appropriate"
            }
            // Add other languages as needed
            _ => "You are an expert software developer. Create high-quality, well-tested code."
        }.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub system: Option<String>,
    pub messages: Vec<ClaudeMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeResponse {
    pub id: String,
    pub content: Vec<ContentBlock>,
    pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}
```

### 2. Developer Agent (Rust)

```rust
// src/agents/developer.rs
use crate::claude::{ClaudeCodeClient, ProgrammingLanguage};
use crate::models::{TaskResult, LanguageContext, AgentError};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

pub struct DeveloperAgent {
    claude_client: ClaudeCodeClient,
    language_detector: LanguageInferenceEngine,
    prompts_remaining: u32,
}

impl DeveloperAgent {
    pub fn new(claude_client: ClaudeCodeClient) -> Self {
        Self {
            claude_client,
            language_detector: LanguageInferenceEngine::new(),
            prompts_remaining: 1000, // Start with reasonable quota
        }
    }

    pub async fn execute_autonomous_task(&mut self, task: &str) -> Result<TaskResult, AgentError> {
        info!("Developer agent executing task: {}", task);

        if self.prompts_remaining == 0 {
            warn!("Developer agent out of prompts");
            return Err(AgentError::InsufficientPrompts { remaining: 0, required: 1 });
        }

        // Detect programming language
        let language_context = self.language_detector.infer_language(task, None).await;

        if language_context.confidence < 0.8 {
            return Ok(TaskResult::RequiresLanguageClarity {
                task: task.to_string(),
                clarification_message: self.create_language_clarification_response(task, &language_context),
            });
        }

        let language = language_context.detected_language.as_ref().unwrap();

        // Execute with Claude Code
        let start_time = std::time::Instant::now();
        let code_result = self.claude_client
            .execute_task(task.to_string(), language.clone())
            .await?;

        let execution_time = start_time.elapsed();
        self.prompts_remaining -= 1;

        info!("Task completed in {:?}", execution_time);

        Ok(TaskResult::Completed {
            task: task.to_string(),
            language: language.clone(),
            code_output: code_result,
            execution_time,
            prompts_used: 1,
        })
    }

    fn create_language_clarification_response(&self, task: &str, _context: &LanguageContext) -> String {
        format!(
            "Hey! 🚀 I'd love to help you with that!\n\n\
            I need to know what programming language you'd like me to use.\n\n\
            **Your request**: {}\n\n\
            Which language would you prefer?\n\
            • **Python** 🐍 (FastAPI, Django)\n\
            • **Rust** 🦀 (High performance)\n\
            • **TypeScript** 📘 (Node.js, web apps)\n\
            • **JavaScript** ⚡ (Quick prototypes)\n\
            • **Go** 🔄 (Microservices)\n\
            • **Or specify another language!**\n\n\
            Just mention me again with your language choice! 🎯",
            task
        )
    }
}

pub struct LanguageInferenceEngine {
    framework_patterns: std::collections::HashMap<String, ProgrammingLanguage>,
}

impl LanguageInferenceEngine {
    pub fn new() -> Self {
        let mut patterns = std::collections::HashMap::new();

        // Python patterns
        patterns.insert("fastapi".to_string(), ProgrammingLanguage::Python);
        patterns.insert("django".to_string(), ProgrammingLanguage::Python);
        patterns.insert("flask".to_string(), ProgrammingLanguage::Python);
        patterns.insert("pytest".to_string(), ProgrammingLanguage::Python);
        patterns.insert("python".to_string(), ProgrammingLanguage::Python);

        // Rust patterns
        patterns.insert("rust".to_string(), ProgrammingLanguage::Rust);
        patterns.insert("cargo".to_string(), ProgrammingLanguage::Rust);
        patterns.insert("actix".to_string(), ProgrammingLanguage::Rust);
        patterns.insert("warp".to_string(), ProgrammingLanguage::Rust);
        patterns.insert("tokio".to_string(), ProgrammingLanguage::Rust);

        // TypeScript patterns
        patterns.insert("typescript".to_string(), ProgrammingLanguage::TypeScript);
        patterns.insert("nestjs".to_string(), ProgrammingLanguage::TypeScript);
        patterns.insert("react".to_string(), ProgrammingLanguage::TypeScript);

        Self { framework_patterns: patterns }
    }

    pub async fn infer_language(&self, prompt: &str, _context: Option<&str>) -> LanguageContext {
        let lowercased = prompt.to_lowercase();

        // Check for explicit mentions
        for (pattern, language) in &self.framework_patterns {
            if lowercased.contains(pattern) {
                return LanguageContext {
                    detected_language: Some(language.clone()),
                    confidence: 0.9,
                    inference_source: InferenceSource::UserPromptKeywords,
                    original_prompt: prompt.to_string(),
                };
            }
        }

        // No clear language detected
        LanguageContext {
            detected_language: None,
            confidence: 0.0,
            inference_source: InferenceSource::Unknown,
            original_prompt: prompt.to_string(),
        }
    }
}
```

### 3. Discord Bot Implementation (TypeScript)

```typescript
// discord-bot/src/index.ts
import { Client, GatewayIntentBits, Message } from "discord.js";
import axios from "axios";
import { config } from "dotenv";
import { DevBot } from "./agents/dev-bot";
import { Logger } from "./utils/logger";

config();

const logger = new Logger("DiscordBot");

class SpiralDiscordBot {
  private client: Client;
  private devBot: DevBot;
  private backendUrl: string;

  constructor() {
    this.client = new Client({
      intents: [
        GatewayIntentBits.Guilds,
        GatewayIntentBits.GuildMessages,
        GatewayIntentBits.MessageContent,
      ],
    });

    this.backendUrl = process.env.RUST_BACKEND_URL || "http://localhost:8080";
    this.devBot = new DevBot(this.backendUrl);

    this.setupEventHandlers();
  }

  private setupEventHandlers(): void {
    this.client.on("ready", () => {
      logger.info(`Bot logged in as ${this.client.user?.tag}`);
    });

    this.client.on("messageCreate", async (message) => {
      if (message.author.bot) return;

      try {
        await this.handleMessage(message);
      } catch (error) {
        logger.error("Error handling message:", error);
        await message.reply(
          "😅 Sorry, I encountered an error. Please try again!"
        );
      }
    });

    this.client.on("error", (error) => {
      logger.error("Discord client error:", error);
    });
  }

  private async handleMessage(message: Message): Promise<void> {
    // Check if bot is mentioned
    const botMention = `<@${this.client.user?.id}>`;
    const content = message.content;

    if (content.includes(botMention) || content.includes("@SpiralDev")) {
      await this.devBot.handleMention(message);
    }
  }

  public async start(): Promise<void> {
    const token = process.env.DISCORD_TOKEN;
    if (!token) {
      throw new Error("DISCORD_TOKEN environment variable is required");
    }

    await this.client.login(token);
  }
}

// discord-bot/src/agents/dev-bot.ts
export class DevBot {
  private backendUrl: string;

  constructor(backendUrl: string) {
    this.backendUrl = backendUrl;
  }

  async handleMention(message: Message): Promise<void> {
    const userId = message.author.id;
    const content = message.content
      .replace(/<@!?\d+>/g, "") // Remove mentions
      .replace(/@SpiralDev/g, "") // Remove @SpiralDev
      .trim();

    if (!content) {
      await message.reply(
        "Hey! 👋 I'm SpiralDev, your AI development assistant!\n\n" +
          "I can help you build applications in various programming languages. " +
          "Just tell me what you'd like me to create!\n\n" +
          "**Examples:**\n" +
          "• `@SpiralDev create a Python FastAPI todo app`\n" +
          "• `@SpiralDev build a Rust CLI tool for file processing`\n" +
          "• `@SpiralDev make a TypeScript Express API`"
      );
      return;
    }

    // Show typing indicator
    await message.channel.sendTyping();

    try {
      const response = await this.sendToBackend({
        agent_type: "developer",
        user_id: userId,
        channel_id: message.channel.id,
        message: content,
      });

      await this.sendResponse(message, response);
    } catch (error) {
      logger.error("Backend communication error:", error);
      await message.reply(
        "😔 Sorry, I'm having trouble connecting to my backend system. " +
          "Please try again in a moment!"
      );
    }
  }

  private async sendToBackend(payload: any): Promise<any> {
    const response = await axios.post(
      `${this.backendUrl}/api/agents/developer/mention`,
      payload,
      {
        timeout: 30000,
        headers: { "Content-Type": "application/json" },
      }
    );

    return response.data;
  }

  private async sendResponse(message: Message, response: any): Promise<void> {
    const { message: responseText, can_execute, requires_followup } = response;

    await message.reply(responseText);

    if (can_execute) {
      await message.react("✅");
    }
  }
}

// Start the bot
const bot = new SpiralDiscordBot();
bot.start().catch(console.error);
```

### 4. HTTP API Server (Rust)

```rust
// src/main.rs
use warp::Filter;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

mod agents;
mod claude;
mod models;
mod errors;

use agents::DeveloperAgent;
use claude::ClaudeCodeClient;

#[derive(Clone)]
pub struct AppState {
    developer_agent: Arc<Mutex<DeveloperAgent>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    dotenv::dotenv().ok();
    let claude_config = claude::ClaudeConfig::from_env()?;

    // Initialize services
    let claude_client = ClaudeCodeClient::new(claude_config)?;
    let developer_agent = Arc::new(Mutex::new(DeveloperAgent::new(claude_client)));

    let app_state = AppState { developer_agent };

    // API routes
    let api_routes = api_routes(app_state);

    // Health check route
    let health = warp::path("health")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({"status": "healthy"})));

    let routes = api_routes.or(health);

    info!("Starting Spiral Core API server on port 8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;

    Ok(())
}

fn api_routes(
    app_state: AppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let agent_routes = warp::path("api")
        .and(warp::path("agents"))
        .and(agent_mention_route(app_state.clone()));

    agent_routes
}

fn agent_mention_route(
    app_state: AppState,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("developer")
        .and(warp::path("mention"))
        .and(warp::post())
        .and(warp::body::json())
        .and(with_app_state(app_state))
        .and_then(handle_developer_mention)
}

fn with_app_state(
    app_state: AppState,
) -> impl Filter<Extract = (AppState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || app_state.clone())
}

async fn handle_developer_mention(
    request: models::DiscordMentionRequest,
    app_state: AppState,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut agent = app_state.developer_agent.lock().await;

    match agent.execute_autonomous_task(&request.message).await {
        Ok(task_result) => {
            let response = create_discord_response(&task_result);
            Ok(warp::reply::json(&response))
        }
        Err(error) => {
            let error_response = models::DiscordAgentResponse {
                message: format!("Sorry, I encountered an error: {}", error),
                suggested_actions: vec!["Try rephrasing your request".to_string()],
                can_execute: false,
                requires_followup: false,
                pending_context: None,
            };
            Ok(warp::reply::json(&error_response))
        }
    }
}

fn create_discord_response(task_result: &models::TaskResult) -> models::DiscordAgentResponse {
    match task_result {
        models::TaskResult::Completed { task, language, .. } => {
            models::DiscordAgentResponse {
                message: format!(
                    "🚀 Great! I've completed your **{}** project!\n\n\
                    **Task**: {}\n\
                    **Language**: {}\n\n\
                    The code is ready and includes comprehensive tests! 📚",
                    language.display_name(),
                    task,
                    language.display_name()
                ),
                suggested_actions: vec![
                    "Create another project".to_string(),
                    "Add more features".to_string(),
                    "Deploy to production".to_string(),
                ],
                can_execute: true,
                requires_followup: false,
                pending_context: None,
            }
        }
        models::TaskResult::RequiresLanguageClarity { clarification_message, .. } => {
            models::DiscordAgentResponse {
                message: clarification_message.clone(),
                suggested_actions: vec![
                    "Specify Python".to_string(),
                    "Choose Rust".to_string(),
                    "Select TypeScript".to_string(),
                ],
                can_execute: false,
                requires_followup: true,
                pending_context: None,
            }
        }
    }
}
```

## Testing Strategy

### Automated Testing Setup

```bash
# Create test structure
mkdir -p tests/{unit,integration,e2e}

# Unit tests for core components
cat > tests/unit/claude_client_test.rs << 'EOF'
use spiral_core::claude::{ClaudeCodeClient, ClaudeConfig, ProgrammingLanguage};

#[tokio::test]
async fn test_claude_client_initialization() {
    let config = ClaudeConfig {
        api_key: "test-key".to_string(),
        base_url: Some("https://api.anthropic.com".to_string()),
        model: None,
    };

    let client = ClaudeCodeClient::new(config);
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_language_detection() {
    let detector = spiral_core::agents::LanguageInferenceEngine::new();

    let context = detector.infer_language("create a FastAPI application", None).await;
    assert_eq!(context.detected_language, Some(ProgrammingLanguage::Python));
    assert!(context.confidence > 0.8);
}
EOF

# Integration test for Discord bot
cat > discord-bot/tests/integration.test.ts << 'EOF'
import { DevBot } from '../src/agents/dev-bot';
import axios from 'axios';

// Mock axios for testing
jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

describe('DevBot Integration', () => {
    let devBot: DevBot;

    beforeEach(() => {
        devBot = new DevBot('http://localhost:8080');
        mockedAxios.post.mockClear();
    });

    test('should handle mention correctly', async () => {
        const mockResponse = {
            data: {
                message: 'Great! I\'ll create that Python application!',
                can_execute: true,
                requires_followup: false
            }
        };

        mockedAxios.post.mockResolvedValue(mockResponse);

        // Would need to mock Discord message object
        // This is a simplified test structure
        expect(true).toBe(true); // Placeholder
    });
});
EOF
```

### Development Scripts

```bash
# Create development scripts
mkdir scripts

cat > scripts/dev.sh << 'EOF'
#!/bin/bash
set -e

echo "🚀 Starting Spiral Core development environment..."

# Start Rust backend in background
echo "Starting Rust backend..."
RUST_LOG=info cargo run &
RUST_PID=$!

# Wait for backend to start
sleep 3

# Start Discord bot
echo "Starting Discord bot..."
cd discord-bot
npm run dev &
DISCORD_PID=$!

# Wait for interrupt
echo "✅ Development environment running!"
echo "Rust backend PID: $RUST_PID"
echo "Discord bot PID: $DISCORD_PID"
echo "Press Ctrl+C to stop all services"

trap "kill $RUST_PID $DISCORD_PID" EXIT
wait
EOF

chmod +x scripts/dev.sh

cat > scripts/test.sh << 'EOF'
#!/bin/bash
set -e

echo "🧪 Running Spiral Core test suite..."

# Test Rust backend
echo "Testing Rust backend..."
cargo test

# Test Discord bot
echo "Testing Discord bot..."
cd discord-bot
npm test

echo "✅ All tests passed!"
EOF

chmod +x scripts/test.sh
```

## Deployment Configuration

### Docker Setup

```dockerfile
# Dockerfile.rust
FROM rust:1.75 AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/spiral-core /usr/local/bin/spiral-core

EXPOSE 8080
CMD ["spiral-core"]
```

```dockerfile
# discord-bot/Dockerfile
FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY src ./src
COPY tsconfig.json ./

RUN npm run build

USER node
CMD ["npm", "start"]
```

```yaml
# docker-compose.yml
version: "3.8"

services:
  spiral-backend:
    build:
      context: .
      dockerfile: Dockerfile.rust
    ports:
      - "8080:8080"
    environment:
      - CLAUDE_API_KEY=${CLAUDE_API_KEY}
      - RUST_LOG=info
    restart: unless-stopped

  discord-bot:
    build:
      context: ./discord-bot
    environment:
      - DISCORD_TOKEN=${DISCORD_TOKEN}
      - RUST_BACKEND_URL=http://spiral-backend:8080
    depends_on:
      - spiral-backend
    restart: unless-stopped

  # For future phases
  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis-data:/data
    restart: unless-stopped

volumes:
  redis-data:
```

## Phase 1 Validation

### Success Criteria Testing

````bash
# Manual validation checklist
cat > PHASE1_VALIDATION.md << 'EOF'
# Phase 1 Validation Checklist

## Discord Bot Integration ✅
- [ ] Bot responds to @SpiralDev mentions within 2 seconds
- [ ] Bot handles unknown commands gracefully
- [ ] Bot provides helpful error messages
- [ ] Bot shows typing indicator during processing

## Language Detection ✅
- [ ] Detects Python from "FastAPI", "Django", "Flask" keywords
- [ ] Detects Rust from "Rust", "Cargo", "Actix" keywords
- [ ] Detects TypeScript from "TypeScript", "NestJS" keywords
- [ ] Requests clarification for ambiguous requests (>85% accuracy)

## Code Generation ✅
- [ ] Generates runnable Python FastAPI applications
- [ ] Includes comprehensive test suites
- [ ] Produces proper project structure
- [ ] Handles error cases gracefully

## System Performance ✅
- [ ] Memory usage <2GB total
- [ ] Response time <3 seconds average
- [ ] No memory leaks during extended operation
- [ ] Graceful handling of Claude API rate limits

## Integration Points ✅
- [ ] Discord bot communicates with Rust backend
- [ ] Rust backend integrates with Claude API
- [ ] Error handling works end-to-end
- [ ] Logging provides useful debugging information

## Test Commands
```bash
# Test language detection
@SpiralDev create a FastAPI todo application

# Test language clarification
@SpiralDev build me an API

# Test error handling
@SpiralDev [intentionally malformed request]

# Test resource constraints
@SpiralDev [make 10 rapid requests]
````

EOF

```

## Phase 1 Completion Criteria

Before proceeding to Phase 2, ensure:

1. **Discord Integration Works**: Bot responds to mentions and handles basic conversations
2. **Language Detection**: >85% accuracy for common frameworks and clear clarification requests
3. **Code Generation**: Produces runnable, tested applications in at least Python and Rust
4. **Performance**: System runs within 2GB memory constraint with <3s response times
5. **Error Handling**: Graceful degradation and helpful error messages
6. **Testing**: Comprehensive test suite with >80% code coverage
7. **Documentation**: Clear setup instructions and API documentation

## Next Steps

Once Phase 1 is complete and validated:

1. **Phase 2 Planning**: Project Manager Agent and GitHub integration
2. **Performance Optimization**: Based on Phase 1 metrics and bottlenecks
3. **Security Hardening**: Authentication and input validation
4. **Monitoring Setup**: Metrics collection and alerting

## Integration Points

This Phase 1 implementation serves as the foundation for:
- [Project Manager Agent](CLAUDE-agents-pm.md) coordination in Phase 2
- [GitHub Integration](CLAUDE-integrations-github.md) for automated PR creation
- [Discord Integration](CLAUDE-integrations-discord.md) enhancement with additional agents

## Related Documentation

- See [Coding Standards](CLAUDE-core-coding-standards.md) for implementation guidelines
- See [Claude Code Integration](CLAUDE-integrations-claude-code.md) for API details
- See [Developer Agent](CLAUDE-agents-developer.md) for agent-specific patterns
```
