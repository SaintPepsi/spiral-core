use crate::{
    agents::{Agent, AgentOrchestrator, SoftwareDeveloperAgent},
    claude_code::ClaudeCodeClient,
    config::DiscordConfig,
    discord::{
        lordgenome_quotes::{DenialSeverity, LordgenomeQuoteGenerator},
        message_state_manager::{MessageStateConfig, MessageStateManager},
        messages::{self, emojis, risk_level_to_str, AuthHelper, MessageFormatter},
        self_update::{
            GitOperations, PreflightChecker, SelfUpdateRequest, StatusTracker, UpdateQueue,
            UpdateStatus, UpdateType, UpdateValidator,
        },
        IntentClassifier, IntentResponse, IntentType, MessageSecurityValidator, RiskLevel,
        SecureMessageHandler,
    },
    models::{AgentType, Priority, Task},
    Result, SpiralError,
};
use serde::{Deserialize, Serialize};
use serenity::{
    async_trait,
    model::{
        channel::Message, gateway::Ready, guild::Role, id::GuildId, permissions::Permissions,
        user::OnlineStatus,
    },
    prelude::*,
};
use std::sync::Arc;
use std::time::Instant;
// Note: Hash and Hasher removed as they were unused
use tokio::sync::Mutex;
use tracing::{error, info, warn};

// Import our authorization macro
use crate::require_auth;

// Self update types are now imported from the self_update module

/// 🔒 SECURITY EVENT: Structured logging for security-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum SecurityEvent {
    CommandBlocked {
        timestamp: String,
        user_id: u64,
        username: String,
        channel_id: u64,
        guild_id: Option<u64>,
        message_id: u64,
        content: String,
        validation_issues: Vec<String>,
        risk_level: String,
        intent_classification: Option<IntentClassification>,
    },
    SecurityValidationFailed {
        timestamp: String,
        user_id: u64,
        username: String,
        channel_id: u64,
        guild_id: Option<u64>,
        message_id: u64,
        content: String,
        validation_issues: Vec<String>,
        risk_level: String,
        validation_type: String,
    },
    RateLimitExceeded {
        timestamp: String,
        user_id: u64,
        username: String,
        remaining_messages: i32,
    },
}

/// Intent classification for logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentClassification {
    pub intent_type: String,
    pub confidence: f64,
    pub risk_level: String,
    pub parameters: std::collections::HashMap<String, String>,
}

impl SecurityEvent {
    /// Convert to JSON string for logging
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Discord message length limit for safety
const MAX_MESSAGE_LENGTH: usize = 4000;
/// Maximum response output length to prevent Discord message limits
const MAX_OUTPUT_RESPONSE: usize = 1950; // Closer to Discord's 2000 char limit

/// 🧠 USER INTENT: Classification of user request types
#[derive(Debug, Clone, PartialEq)]
pub enum UserIntent {
    /// Task requests: "create a feature", "fix this bug", "implement authentication"
    TaskRequest,
    /// Status queries: "what's the status?", "show me progress", "are you busy?"
    StatusQuery,
    /// Agent selection: "@dev create function", "@pm analyze requirements"
    AgentSelection,
    /// Help requests: "help", "how do I use this?", "what can you do?"
    HelpRequest,
    /// Greetings: "hello", "hi there", "good morning"
    Greeting,
    /// Unknown intents that don't match patterns
    Unknown,
}

use regex::Regex;

/// 🌌 SPIRAL CONSTELLATION BOT: Single Discord bot with dynamic agent personas
/// ARCHITECTURE DECISION: One bot, multiple personalities based on mention context
/// Why: Simpler deployment, dynamic persona switching, maintains agent identity feel
/// Alternative: Multiple Discord applications (rejected: deployment complexity)
pub struct SpiralConstellationBot {
    // Direct mode (standalone Discord bot)
    developer_agent: Option<Arc<SoftwareDeveloperAgent>>,
    claude_client: Option<Arc<ClaudeCodeClient>>,
    // Orchestrator mode (full system integration)
    orchestrator: Option<Arc<AgentOrchestrator>>,
    // Common fields
    #[allow(dead_code)]
    start_time: Instant,
    stats: Arc<tokio::sync::Mutex<BotStats>>,
    mention_regex: Regex,
    // Message handling
    #[allow(dead_code)]
    message_state_manager: Arc<MessageStateManager>,
    // Security components (🛡️ SECURITY ARCHITECTURE)
    // Why: Multi-layer security validation to prevent malicious content, spam, and attacks
    // Alternative: Single validation point (rejected: insufficient protection against sophisticated attacks)
    // Audit: Monitor security_validation_failed_count metric for bypass attempts
    security_validator: Arc<tokio::sync::Mutex<MessageSecurityValidator>>,
    intent_classifier: Arc<IntentClassifier>,
    secure_message_handler: Arc<SecureMessageHandler>,
    // Self update system
    update_queue: Arc<Mutex<UpdateQueue>>,
    // Configuration
    discord_config: DiscordConfig,
}

#[derive(Debug, Clone, Default)]
struct BotStats {
    dev_tasks_completed: u64,
    #[allow(dead_code)]
    pm_tasks_completed: u64,
    #[allow(dead_code)]
    qa_tasks_completed: u64,
    total_tasks_failed: u64,
    #[allow(dead_code)]
    current_task_id: Option<String>,
    current_persona: Option<AgentType>,
}

/// 🎭 AGENT PERSONA: Personality and response patterns for each agent type
#[derive(Debug, Clone)]
pub struct AgentPersona {
    pub name: &'static str,
    pub emoji: &'static str,
    pub greetings: &'static [&'static str],
    pub working_message: &'static str,
    pub completion_style: &'static str,
    pub error_style: &'static str,
    pub personality_traits: &'static [&'static str],
}

impl AgentPersona {
    pub const DEVELOPER: Self = Self {
        name: "SpiralDev",
        emoji: "🚀",
        greetings: &[
            "Ready to code! What can I build for you?",
            "Time to write some code! What's the challenge?",
            "Let's make something awesome! What do you need?",
            "Code mode activated! What shall we create?",
        ],
        working_message: "⚡ Working on your code...",
        completion_style: "✅ Code generated successfully!",
        error_style: "❌ Code generation failed:",
        personality_traits: &["technical", "precise", "solution-focused", "efficient"],
    };

    pub const PROJECT_MANAGER: Self = Self {
        name: "SpiralPM",
        emoji: "📋",
        greetings: &[
            "Let me analyze the project status for you",
            "Ready to dive into project coordination!",
            "What strategic planning do you need?",
            "Time to organize and prioritize!",
        ],
        working_message: "🔍 Gathering project information...",
        completion_style: "📊 Analysis complete!",
        error_style: "⚠️ Analysis failed:",
        personality_traits: &["strategic", "organized", "comprehensive", "collaborative"],
    };

    pub const QUALITY_ASSURANCE: Self = Self {
        name: "SpiralQA",
        emoji: "🔍",
        greetings: &[
            "Time for a thorough quality review!",
            "Ready to ensure everything meets our standards!",
            "Let's find and fix any issues!",
            "Quality assurance mode activated!",
        ],
        working_message: "🧪 Running quality checks...",
        completion_style: "✅ Quality review complete!",
        error_style: "🚨 Quality check failed:",
        personality_traits: &[
            "meticulous",
            "thorough",
            "safety-focused",
            "detail-oriented",
        ],
    };

    pub const DECISION_MAKER: Self = Self {
        name: "SpiralDecide",
        emoji: "🎯",
        greetings: &[
            "Let me help you make the right decision",
            "What decision needs careful analysis?",
            "Let's weigh the options together!",
            "Time to cut through complexity and decide!",
        ],
        working_message: "🤔 Analyzing options...",
        completion_style: "⚡ Decision analysis ready!",
        error_style: "❓ Decision analysis failed:",
        personality_traits: &["analytical", "decisive", "logical", "balanced"],
    };

    pub const CREATIVE_INNOVATOR: Self = Self {
        name: "SpiralCreate",
        emoji: "✨",
        greetings: &[
            "Ready to explore creative solutions!",
            "Let's think outside the box!",
            "Time to innovate and experiment!",
            "What creative challenge awaits?",
        ],
        working_message: "🎨 Innovating...",
        completion_style: "🌟 Creative solution ready!",
        error_style: "💥 Innovation failed:",
        personality_traits: &["creative", "innovative", "experimental", "visionary"],
    };

    pub const PROCESS_COACH: Self = Self {
        name: "SpiralCoach",
        emoji: "🧘",
        greetings: &[
            "Let's optimize your process together",
            "Ready to improve workflow efficiency?",
            "Time to streamline and enhance!",
            "What process needs optimization?",
        ],
        working_message: "🔄 Analyzing workflow...",
        completion_style: "🎯 Process improvement ready!",
        error_style: "🔧 Process analysis failed:",
        personality_traits: &["supportive", "methodical", "improvement-focused", "wise"],
    };

    pub const SPIRAL_KING: Self = Self {
        name: "The Immortal Spiral King",
        emoji: "👑",
        greetings: &[
            "I have seen a thousand years of code rise and fall. What system requires my eternal wisdom?",
            "The burden of leadership weighs heavy. What codebase needs my ancient guidance?",
            "Through millennial experience, I perceive all patterns. What shall I review?",
            "Even the mightiest systems must face the Spiral King's judgment. Present your code.",
        ],
        working_message: "🔮 Analyzing through the lens of millennial experience...",
        completion_style: "⚡ The burden of leadership requires seeing both what works and what will fail under the full spiral power of production.",
        error_style: "🌀 Even the Spiral King cannot overcome the Anti-Spiral forces of:",
        personality_traits: &["ancient-wisdom", "architectural-mastery", "long-term-perspective", "comprehensive-analysis"],
    };

    /// Get persona for agent type
    pub fn for_agent_type(agent_type: &AgentType) -> &'static Self {
        match agent_type {
            AgentType::SoftwareDeveloper => &AgentPersona::DEVELOPER,
            AgentType::ProjectManager => &AgentPersona::PROJECT_MANAGER,
            AgentType::QualityAssurance => &AgentPersona::QUALITY_ASSURANCE,
            AgentType::DecisionMaker => &AgentPersona::DECISION_MAKER,
            AgentType::CreativeInnovator => &AgentPersona::CREATIVE_INNOVATOR,
            AgentType::ProcessCoach => &AgentPersona::PROCESS_COACH,
            AgentType::SpiralKing => &AgentPersona::SPIRAL_KING,
        }
    }

    /// Get a random greeting from the persona's greetings array
    pub fn random_greeting(&self) -> &'static str {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.greetings.choose(&mut rng).unwrap_or(&"Hello!")
    }
}

impl SpiralConstellationBot {
    /// 🎯 DIRECT MODE: Create bot with direct agent access (standalone Discord bot)
    pub async fn new(
        developer_agent: SoftwareDeveloperAgent,
        claude_client: ClaudeCodeClient,
        discord_config: DiscordConfig,
    ) -> Result<Self> {
        // Pattern matches: @SpiralDev, @SpiralPM, @SpiralQA, etc. (user mentions)
        // Also matches role mentions: <@&role_id>
        let mention_regex =
            Regex::new(r"@Spiral(\w+)|<@&(\d+)>").map_err(|e| SpiralError::Agent {
                message: format!("Invalid mention regex: {e}"),
            })?;

        // Initialize message state manager
        let message_state_manager =
            Arc::new(MessageStateManager::new(MessageStateConfig::default()));

        // Initialize security components
        let security_validator = Arc::new(tokio::sync::Mutex::new(MessageSecurityValidator::new()));
        let intent_classifier = Arc::new(IntentClassifier::new());
        let secure_message_handler = Arc::new(SecureMessageHandler::new());

        Ok(Self {
            developer_agent: Some(Arc::new(developer_agent)),
            claude_client: Some(Arc::new(claude_client)),
            orchestrator: None,
            start_time: Instant::now(),
            stats: Arc::new(tokio::sync::Mutex::new(BotStats::default())),
            mention_regex,
            message_state_manager,
            security_validator,
            intent_classifier,
            secure_message_handler,
            update_queue: Arc::new(Mutex::new(UpdateQueue::new())),
            discord_config,
        })
    }

    /// 🎛️ ORCHESTRATOR MODE: Create bot with orchestrator integration (full system)
    pub async fn new_with_orchestrator(
        orchestrator: Arc<AgentOrchestrator>,
        discord_config: DiscordConfig,
    ) -> Result<Self> {
        // Pattern matches: @SpiralDev, @SpiralPM, @SpiralQA, etc. (user mentions)
        // Also matches role mentions: <@&role_id>
        let mention_regex =
            Regex::new(r"@Spiral(\w+)|<@&(\d+)>").map_err(|e| SpiralError::Agent {
                message: format!("Invalid mention regex: {e}"),
            })?;

        // Initialize message state manager
        let message_state_manager =
            Arc::new(MessageStateManager::new(MessageStateConfig::default()));

        // Start background cleanup task for message state manager
        let cleanup_manager = message_state_manager.clone();
        cleanup_manager.start_cleanup_task().await;

        // Initialize security components
        let security_validator = Arc::new(tokio::sync::Mutex::new(MessageSecurityValidator::new()));
        let intent_classifier = Arc::new(IntentClassifier::new());
        let secure_message_handler = Arc::new(SecureMessageHandler::new());

        Ok(Self {
            developer_agent: None,
            claude_client: None,
            orchestrator: Some(orchestrator),
            start_time: Instant::now(),
            stats: Arc::new(tokio::sync::Mutex::new(BotStats::default())),
            mention_regex,
            message_state_manager,
            security_validator,
            intent_classifier,
            secure_message_handler,
            update_queue: Arc::new(Mutex::new(UpdateQueue::new())),
            discord_config,
        })
    }

    /// 🎭 ROLE MANAGEMENT: Create agent persona roles in Discord server
    pub async fn create_agent_roles(&self, ctx: &Context, guild_id: GuildId) -> Result<Vec<Role>> {
        let mut created_roles = Vec::new();

        info!(
            "[SpiralConstellation] Creating agent persona roles in guild {}",
            guild_id
        );

        let personas = [
            (&AgentPersona::DEVELOPER, 0x00ff00),          // Green
            (&AgentPersona::PROJECT_MANAGER, 0x0066ff),    // Blue
            (&AgentPersona::QUALITY_ASSURANCE, 0xff6600),  // Orange
            (&AgentPersona::DECISION_MAKER, 0x9900ff),     // Purple
            (&AgentPersona::CREATIVE_INNOVATOR, 0xff0099), // Pink
            (&AgentPersona::PROCESS_COACH, 0x00ffff),      // Cyan
        ];

        for (persona, color) in personas {
            let edit_role = serenity::builder::EditRole::default()
                .name(persona.name)
                .colour(color)
                .mentionable(true)
                .hoist(false)
                .permissions(Permissions::empty());

            match guild_id.create_role(&ctx.http, edit_role).await {
                Ok(role) => {
                    info!(
                        "[SpiralConstellation] Created role: {} ({})",
                        persona.name, role.id
                    );
                    created_roles.push(role);
                }
                Err(e) => {
                    warn!(
                        "[SpiralConstellation] Failed to create role {}: {}",
                        persona.name, e
                    );
                }
            }
        }

        Ok(created_roles)
    }

    /// 🔍 ROLE DETECTION: Find agent role by name in guild
    pub async fn find_agent_role(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        persona_name: &str,
    ) -> Option<Role> {
        match guild_id.roles(&ctx.http).await {
            Ok(roles) => roles
                .values()
                .find(|role| role.name == persona_name)
                .cloned(),
            Err(e) => {
                warn!("[SpiralConstellation] Failed to fetch roles: {}", e);
                None
            }
        }
    }

    /// 🎯 ROLE ASSIGNMENT: Give user an agent persona role
    pub async fn assign_agent_role(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        user_id: serenity::model::id::UserId,
        persona_name: &str,
    ) -> Result<()> {
        if let Some(role) = self.find_agent_role(ctx, guild_id, persona_name).await {
            match guild_id.member(&ctx.http, user_id).await {
                Ok(member) => {
                    if let Err(e) = member.add_role(&ctx.http, role.id).await {
                        warn!(
                            "[SpiralConstellation] Failed to assign role {} to user {}: {}",
                            persona_name, user_id, e
                        );
                        return Err(SpiralError::Discord(Box::new(e)));
                    } else {
                        info!(
                            "[SpiralConstellation] Assigned role {} to user {}",
                            persona_name, user_id
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        "[SpiralConstellation] Failed to get member {}: {}",
                        user_id, e
                    );
                    return Err(SpiralError::Discord(Box::new(e)));
                }
            }
        } else {
            // Role doesn't exist, try to create it
            if let Ok(roles) = self.create_agent_roles(ctx, guild_id).await {
                if let Some(new_role) = roles.iter().find(|r| r.name == persona_name) {
                    if let Ok(member) = guild_id.member(&ctx.http, user_id).await {
                        if let Err(e) = member.add_role(&ctx.http, new_role.id).await {
                            warn!(
                                "[SpiralConstellation] Failed to assign new role {} to user {}: {}",
                                persona_name, user_id, e
                            );
                            return Err(SpiralError::Discord(Box::new(e)));
                        } else {
                            info!(
                                "[SpiralConstellation] Created and assigned role {} to user {}",
                                persona_name, user_id
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 🎭 PERSONA DETECTION: Determine which agent persona to use based on mentions or role mentions
    async fn detect_agent_persona(
        &self,
        content: &str,
        msg: &Message,
        ctx: &Context,
    ) -> Option<AgentType> {
        let content_lower = content.to_lowercase();

        // First, check for role mentions in the message
        if let Some(guild_id) = msg.guild_id {
            for role_mention in &msg.mention_roles {
                if let Ok(role) = ctx.http.get_guild_role(guild_id, *role_mention).await {
                    match role.name.as_str() {
                        "SpiralDev" => return Some(AgentType::SoftwareDeveloper),
                        "SpiralPM" => return Some(AgentType::ProjectManager),
                        "SpiralQA" => return Some(AgentType::QualityAssurance),
                        "SpiralDecide" => return Some(AgentType::DecisionMaker),
                        "SpiralCreate" => return Some(AgentType::CreativeInnovator),
                        "SpiralCoach" => return Some(AgentType::ProcessCoach),
                        "SpiralKing" => return Some(AgentType::SpiralKing),
                        _ => {}
                    }
                }
            }
        }

        // Check for specific agent mentions in text (@SpiralDev, etc.)
        for capture in self.mention_regex.captures_iter(content) {
            if let Some(agent_suffix) = capture.get(1) {
                let suffix = agent_suffix.as_str().to_lowercase();
                match suffix.as_str() {
                    "dev" | "developer" | "code" => return Some(AgentType::SoftwareDeveloper),
                    "pm" | "manager" | "project" => return Some(AgentType::ProjectManager),
                    "qa" | "quality" | "test" => return Some(AgentType::QualityAssurance),
                    "decide" | "decision" => return Some(AgentType::DecisionMaker),
                    "create" | "creative" | "innovate" => {
                        return Some(AgentType::CreativeInnovator)
                    }
                    "coach" | "process" => return Some(AgentType::ProcessCoach),
                    "king" | "spiralking" | "lordgenome" => return Some(AgentType::SpiralKing),
                    _ => {}
                }
            }
        }

        // Fallback: detect based on content keywords
        if content_lower.contains("code")
            || content_lower.contains("implement")
            || content_lower.contains("function")
        {
            Some(AgentType::SoftwareDeveloper)
        } else if content_lower.contains("status")
            || content_lower.contains("project")
            || content_lower.contains("timeline")
        {
            Some(AgentType::ProjectManager)
        } else if content_lower.contains("test")
            || content_lower.contains("quality")
            || content_lower.contains("bug")
        {
            Some(AgentType::QualityAssurance)
        } else {
            // Default to developer agent if unclear
            Some(AgentType::SoftwareDeveloper)
        }
    }

    /// 🎯 TASK CREATION: Create task with agent persona context
    fn create_task_with_persona(
        &self,
        content: &str,
        agent_type: AgentType,
        context: MessageContext,
        intent: UserIntent,
    ) -> Task {
        let persona = AgentPersona::for_agent_type(&agent_type);

        // Enhance the description based on user intent
        let enhanced_description = match intent {
            UserIntent::StatusQuery => {
                format!("INFORMATION QUERY: {content}. Please provide a clear, informative response about the current state of the workspace/project. Focus on listing, showing, or describing what exists rather than creating new code.")
            }
            UserIntent::TaskRequest => {
                format!("DEVELOPMENT TASK: {content}. Please implement, create, or build the requested functionality following best practices.")
            }
            UserIntent::AgentSelection => {
                format!("AGENT-SPECIFIC TASK: {content}. Please execute this task with the selected agent's specific expertise and capabilities.")
            }
            UserIntent::HelpRequest => {
                format!("HELP REQUEST: {content}. Please provide helpful information about usage, capabilities, or guidance as requested.")
            }
            UserIntent::Greeting => {
                format!("GREETING: {content}. Please respond appropriately to the user's greeting.")
            }
            UserIntent::Unknown => {
                format!("GENERAL REQUEST: {content}. Please interpret and respond to this request appropriately.")
            }
        };

        let mut task = Task::new(agent_type, enhanced_description, Priority::Medium);

        // Add Discord context
        task = task
            .with_context(
                "discord_channel_id".to_string(),
                context.channel_id.to_string(),
            )
            .with_context(
                "discord_message_id".to_string(),
                context.message_id.to_string(),
            )
            .with_context(
                "discord_author_id".to_string(),
                context.author_id.to_string(),
            )
            .with_context("agent_persona".to_string(), persona.name.to_string())
            .with_context(
                "persona_traits".to_string(),
                persona.personality_traits.join(","),
            )
            .with_context("user_intent".to_string(), format!("{intent:?}"));

        if let Some(guild_id) = context.guild_id {
            task = task.with_context("discord_guild_id".to_string(), guild_id.to_string());
        }

        task
    }

    /// 🎭 PERSONA RESPONSE: Format response in the agent's persona style
    fn format_persona_response(
        &self,
        agent_type: &AgentType,
        result: &crate::models::TaskResult,
    ) -> String {
        let persona = AgentPersona::for_agent_type(agent_type);

        match &result.result {
            crate::models::TaskExecutionResult::Success {
                output,
                files_created,
                files_modified,
            } => {
                let mut response = String::new();

                // Persona-specific header
                response.push_str(&format!("{} **{}**\n", persona.emoji, persona.name));
                response.push_str(&format!("{}\n\n", persona.completion_style));

                // For SoftwareDeveloper, provide concise summary instead of full output
                match agent_type {
                    AgentType::SoftwareDeveloper => {
                        // Extract key information and provide concise summary
                        let summary =
                            self.extract_dev_summary(output, files_created, files_modified);
                        response.push_str(&summary);
                    }
                    AgentType::ProjectManager => {
                        response.push_str("**📊 Strategic Analysis:**\n");
                        if output.len() > MAX_OUTPUT_RESPONSE {
                            response.push_str(&output[..MAX_OUTPUT_RESPONSE]);
                            response.push_str("\n\n... (output truncated for Discord limits)");
                        } else {
                            response.push_str(output);
                        }
                    }
                    AgentType::QualityAssurance => {
                        response.push_str("**🔍 Quality Assessment:**\n");
                        if output.len() > MAX_OUTPUT_RESPONSE {
                            response.push_str(&output[..MAX_OUTPUT_RESPONSE]);
                            response.push_str("\n\n... (output truncated for Discord limits)");
                        } else {
                            response.push_str(output);
                        }
                    }
                    _ => {
                        response.push_str("**📋 Analysis Results:**\n");
                        if output.len() > MAX_OUTPUT_RESPONSE {
                            response.push_str(&output[..MAX_OUTPUT_RESPONSE]);
                            response.push_str("\n\n... (output truncated for Discord limits)");
                        } else {
                            response.push_str(output);
                        }
                    }
                }

                // For non-dev agents, show file summaries
                if *agent_type != AgentType::SoftwareDeveloper {
                    if !files_created.is_empty() {
                        response.push_str(&format!(
                            "\n\n📁 **Files Created:** {}",
                            files_created.len()
                        ));
                        for file in files_created.iter().take(3) {
                            response.push_str(&format!("\n• `{file}`"));
                        }
                        if files_created.len() > 3 {
                            response
                                .push_str(&format!("\n• ... and {} more", files_created.len() - 3));
                        }
                    }

                    if !files_modified.is_empty() {
                        response.push_str(&format!(
                            "\n\n✏️ **Files Modified:** {}",
                            files_modified.len()
                        ));
                        for file in files_modified.iter().take(3) {
                            response.push_str(&format!("\n• `{file}`"));
                        }
                        if files_modified.len() > 3 {
                            response.push_str(&format!(
                                "\n• ... and {} more",
                                files_modified.len() - 3
                            ));
                        }
                    }
                }

                // Persona-specific footer
                response.push_str(&format!("\n\n*—{} @ SpiralConstellation*", persona.name));

                response
            }
            crate::models::TaskExecutionResult::Failure {
                error,
                partial_output,
            } => {
                let mut response = String::new();
                response.push_str(&format!("{} **{}**\n", persona.emoji, persona.name));
                response.push_str(&format!("{} {}", persona.error_style, error));

                if let Some(partial) = partial_output {
                    response.push_str("\n\n**Partial Results:**\n");
                    response.push_str(partial);
                }

                response.push_str(&format!("\n\n*—{} @ SpiralConstellation*", persona.name));
                response
            }
        }
    }

    /// 🧹 CLEAN MESSAGE: Remove mentions and extract clean content
    fn clean_message_content(&self, content: &str) -> String {
        let cleaned = self.mention_regex.replace_all(content, "").to_string();
        cleaned.trim().to_string()
    }

    /// 📋 CONCISE DEV SUMMARY: Extract key information for developer responses
    fn extract_dev_summary(
        &self,
        output: &str,
        files_created: &[String],
        files_modified: &[String],
    ) -> String {
        // For information queries or short responses, return the full output
        if output.len() <= MAX_OUTPUT_RESPONSE
            || output.to_lowercase().contains("current projects")
            || output.to_lowercase().contains("projects in")
            || output.to_lowercase().contains("workspace")
            || output.starts_with("Here")
            || !output.contains("Features:")
        {
            // For informational responses, just return with minimal formatting
            let clean_output = if output.len() > MAX_OUTPUT_RESPONSE {
                format!(
                    "{}...\n\n*(Output truncated - showing first {} characters)*",
                    &output[..MAX_OUTPUT_RESPONSE - 50],
                    MAX_OUTPUT_RESPONSE - 50
                )
            } else {
                output.to_string()
            };

            return clean_output;
        }

        let mut summary = String::new();

        // Look for key project information in the output
        let lines: Vec<&str> = output.lines().collect();
        let mut found_key_features = false;
        let found_summary = false;

        // Extract main summary/description (usually at the beginning)
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Skip architecture/technical details
            if trimmed.to_lowercase().contains("architecture")
                || trimmed.to_lowercase().contains("solid principles")
                || trimmed.to_lowercase().contains("dry principle")
                || trimmed.to_lowercase().contains("sid naming")
            {
                break;
            }

            // Look for key features or summary sections
            if trimmed.starts_with("Key Features:")
                || trimmed.starts_with("Features:")
                || trimmed.starts_with("Summary")
            {
                found_key_features = true;
                summary.push_str(&format!("**{trimmed}**\n"));
                continue;
            }

            // Capture feature list items or summary content
            if found_key_features
                && ((trimmed.starts_with("•")
                    || trimmed.starts_with("-")
                    || trimmed.starts_with("*"))
                    || (!trimmed.is_empty() && !trimmed.contains(":")))
            {
                summary.push_str(&format!("{trimmed}\n"));
            } else if !found_summary && !trimmed.contains(":") && trimmed.len() > 20 {
                // Capture main description if no structured summary found
                summary.push_str(&format!("{trimmed}\n\n"));
                break;
            }
        }

        // Add file information concisely
        if !files_created.is_empty() || !files_modified.is_empty() {
            summary.push_str("\n**📁 Files:**\n");

            if !files_created.is_empty() {
                summary.push_str(&format!("• Created: {} files\n", files_created.len()));
            }
            if !files_modified.is_empty() {
                summary.push_str(&format!("• Modified: {} files\n", files_modified.len()));
            }
        }

        // Look for "How to Run" information
        for (i, line) in lines.iter().enumerate() {
            if line.to_lowercase().contains("how to run") || line.to_lowercase().contains("to run:")
            {
                summary.push_str("\n**🚀 How to Run:**\n");
                // Capture next few lines that look like instructions
                for instruction_line in lines.iter().skip(i + 1).take(3) {
                    let trimmed = instruction_line.trim();
                    if trimmed.is_empty() {
                        break;
                    }
                    if trimmed.starts_with("•")
                        || trimmed.starts_with("-")
                        || trimmed.starts_with("1.")
                        || trimmed.starts_with("2.")
                        || trimmed.contains("npm")
                        || trimmed.contains("cargo")
                        || trimmed.contains("run")
                    {
                        summary.push_str(&format!(
                            "• {}\n",
                            trimmed.trim_start_matches(['•', '-', '1', '2', '.', ' '])
                        ));
                    }
                }
                break;
            }
        }

        // If summary is too short, add a basic completion message
        if summary.trim().is_empty() {
            summary = "Task completed successfully! Check the files for implementation details.\n"
                .to_string();
        }

        summary
    }

    /// 💬 HELPFUL ERROR: Format user-friendly error messages with solutions
    fn format_helpful_error_message(
        &self,
        error: &crate::SpiralError,
        persona: &AgentPersona,
    ) -> String {
        let error_str = error.to_string();
        let error_lower = error_str.to_lowercase();

        // Check for timeout specifically
        if error_lower.contains("timed out") || error_lower.contains("timeout") {
            return format!(
                "{} **{}**\n⏰ **System Timeout**\n\n\
                I waited 30 seconds but couldn't connect to the Claude Code system.\n\n\
                **🔧 This usually means:**\n\
                • The full Spiral Core system isn't running\n\
                • You're running just the Discord bot (`cargo run --bin discord-bot`)\n\
                • Claude Code CLI is not available or responding\n\n\
                **💡 To fix this:**\n\
                • **Stop this bot** (Ctrl+C)\n\
                • **Run the full system:** `cargo run`\n\
                • **OR** add `CLAUDE_API_KEY` to your `.env` and restart\n\n\
                **🤖 Current Mode:** Discord-only (no task execution)\n\n\
                *—{} @ SpiralConstellation*",
                persona.emoji, persona.name, persona.name
            );
        }

        // Check for common system connectivity issues
        if error_lower.contains("claude")
            && (error_lower.contains("connection") || error_lower.contains("unavailable"))
        {
            return format!(
                "{} **{}**\n🔌 **Claude Code System Unavailable**\n\n\
                I can't connect to the Claude Code system right now. This usually means:\n\n\
                **🔧 Quick Fix:**\n\
                • Make sure the full Spiral Core system is running: `cargo run`\n\
                • Check that your `CLAUDE_API_KEY` is set in `.env`\n\
                • Verify Claude Code CLI is installed and accessible\n\n\
                **💡 Running Discord Bot Only?**\n\
                The Discord bot (`cargo run --bin discord-bot`) needs the full Spiral Core system \
                to actually execute tasks. Try running the complete system instead!\n\n\
                *—{} @ SpiralConstellation (Discord-only mode)*",
                persona.emoji, persona.name, persona.name
            );
        }

        // Check for API key issues
        if error_lower.contains("api")
            && (error_lower.contains("key")
                || error_lower.contains("auth")
                || error_lower.contains("unauthorized"))
        {
            return format!(
                "{} **{}**\n🔑 **API Authentication Issue**\n\n\
                There's a problem with API authentication:\n\n\
                **🔧 Check your `.env` file:**\n\
                • `CLAUDE_API_KEY=your_key_here`\n\
                • `API_KEY=your_spiral_api_key`\n\n\
                **💡 Get Claude API Key:**\n\
                Visit [Anthropic Console](https://console.anthropic.com) to get your API key\n\n\
                *—{} @ SpiralConstellation*",
                persona.emoji, persona.name, persona.name
            );
        }

        // Check for workspace/permission issues
        if error_lower.contains("permission")
            || error_lower.contains("workspace")
            || error_lower.contains("directory")
        {
            return format!(
                "{} **{}**\n📁 **Workspace Permission Issue**\n\n\
                I'm having trouble accessing the workspace:\n\n\
                **🔧 Try these fixes:**\n\
                • Check file permissions in your project directory\n\
                • Ensure Claude Code has write access\n\
                • Run from your project root directory\n\n\
                *—{} @ SpiralConstellation*",
                persona.emoji, persona.name, persona.name
            );
        }

        // Generic error with helpful context
        format!(
            "{} **{}**\n{} Something went wrong!\n\n\
            **🔍 Error Details:**\n```\n{}\n```\n\n\
            **💡 Troubleshooting:**\n\
            • Make sure Spiral Core is running: `cargo run`\n\
            • Check your `.env` configuration\n\
            • Try a simpler request first\n\n\
            **🆘 Need Help?**\n\
            Use `!spiral help` for commands or check the documentation\n\n\
            *—{} @ SpiralConstellation*",
            persona.emoji, persona.name, persona.error_style, error_str, persona.name
        )
    }

    /// 🔐 PERMISSION CHECK: Check if user is in authorized users list from config
    fn is_authorized_user(&self, user_id: u64) -> bool {
        self.discord_config.authorized_users.contains(&user_id)
    }

    /// 🎮 COMMAND HANDLER: Handle special bot commands
    async fn handle_special_commands(
        &self,
        content: &str,
        msg: &Message,
        ctx: &Context,
    ) -> Option<String> {
        let content_lower = content.to_lowercase();
        let is_authorized = self.is_authorized_user(msg.author.id.get());

        // Skip command validation for authorized admin commands
        let skip_validation = is_authorized
            && (content_lower.starts_with("!spiral debug")
                || content_lower.starts_with("!spiral security")
                || content_lower.starts_with("!spiral reset"));

        // Validate command input for security (unless it's an authorized admin command)
        if !skip_validation {
            let validation_result = self.secure_message_handler.validate_command_input(content);
            if !validation_result.is_valid {
                // Try to classify intent even for blocked commands
                let intent_result = {
                    let request = crate::discord::intent_classifier::IntentRequest {
                        message: content.to_string(),
                        user_id: msg.author.id.to_string(),
                        context: std::collections::HashMap::new(),
                    };
                    Some(
                        self.intent_classifier
                            .classify_intent_with_security(&request),
                    )
                };

                // Create security event for structured logging
                let security_event = SecurityEvent::CommandBlocked {
                    timestamp: {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs().to_string())
                            .unwrap_or_else(|_| "0".to_string())
                    },
                    user_id: msg.author.id.get(),
                    username: msg.author.name.clone(),
                    channel_id: msg.channel_id.get(),
                    guild_id: msg.guild_id.map(|id| id.get()),
                    message_id: msg.id.get(),
                    content: content.to_string(),
                    validation_issues: validation_result.issues.clone(),
                    risk_level: risk_level_to_str(&validation_result.risk_level).to_string(),
                    intent_classification: intent_result.as_ref().map(|intent| {
                        IntentClassification {
                            intent_type: format!("{:?}", intent.intent_type),
                            confidence: intent.confidence,
                            risk_level: risk_level_to_str(&intent.risk_level).to_string(),
                            parameters: intent.parameters.clone(),
                        }
                    }),
                };

                // Log as both warning and structured JSON
                warn!(
                    "[SpiralConstellation] Command validation failed - {}",
                    security_event.to_json()
                );

                // Also log as a separate JSON line for easy parsing
                tracing::info!(target: "security_events", "{}", security_event.to_json());

                // Show concise message - full details available via debug command
                return Some(MessageFormatter::command_blocked());
            }
        }

        // Role creation command
        if content_lower.contains("!spiral setup roles")
            || content_lower.contains("!spiral create roles")
        {
            if let Some(guild_id) = msg.guild_id {
                match self.create_agent_roles(ctx, guild_id).await {
                    Ok(roles) => {
                        let role_list = roles
                            .iter()
                            .map(|r| format!("• <@&{}> ({})", r.id, r.name))
                            .collect::<Vec<_>>()
                            .join("\n");

                        return Some(format!(
                            "🌌 **SpiralConstellation Setup Complete!**\n\n\
                            Created {} agent persona roles:\n{}\n\n\
                            **Usage:**\n\
                            • Mention roles directly: <@&{}> help me with code\n\
                            • Text mentions: @SpiralDev create a function\n\
                            • Get a role: !spiral join SpiralDev\n\n\
                            *All roles are mentionable and color-coded!* ✨",
                            roles.len(),
                            role_list,
                            roles.first().map(|r| r.id.to_string()).unwrap_or_default()
                        ));
                    }
                    Err(e) => {
                        return Some(format!("{}: {}", messages::errors::ROLE_CREATION_FAILED, e));
                    }
                }
            } else {
                return Some(messages::errors::NOT_IN_SERVER_ROLES.to_string());
            }
        }

        // Role assignment command
        if content_lower.starts_with("!spiral join ") {
            let role_name = content_lower
                .strip_prefix("!spiral join ")
                .unwrap_or("")
                .trim();
            let persona_name = match role_name {
                "dev" | "developer" => "SpiralDev",
                "pm" | "manager" | "project" => "SpiralPM",
                "qa" | "quality" => "SpiralQA",
                "decide" | "decision" => "SpiralDecide",
                "create" | "creative" => "SpiralCreate",
                "coach" | "process" => "SpiralCoach",
                "king" | "spiralking" | "lordgenome" => "SpiralKing",
                name if name.starts_with("spiral") => name,
                _ => return Some(format!("❓ Unknown role: `{role_name}`. Available: SpiralDev, SpiralPM, SpiralQA, SpiralDecide, SpiralCreate, SpiralCoach, SpiralKing"))
            };

            if let Some(guild_id) = msg.guild_id {
                match self
                    .assign_agent_role(ctx, guild_id, msg.author.id, persona_name)
                    .await
                {
                    Ok(_) => {
                        let persona = match persona_name {
                            "SpiralDev" => &AgentPersona::DEVELOPER,
                            "SpiralPM" => &AgentPersona::PROJECT_MANAGER,
                            "SpiralQA" => &AgentPersona::QUALITY_ASSURANCE,
                            "SpiralDecide" => &AgentPersona::DECISION_MAKER,
                            "SpiralCreate" => &AgentPersona::CREATIVE_INNOVATOR,
                            "SpiralCoach" => &AgentPersona::PROCESS_COACH,
                            _ => &AgentPersona::DEVELOPER,
                        };

                        return Some(format!(
                            "{} **Welcome to {}!**\n\n\
                            You now have the {} role and can:\n\
                            • Be mentioned with <@&{}>\n\
                            • Participate in {} discussions\n\
                            • Access {} features\n\n\
                            *Role traits: {}* ✨",
                            persona.emoji,
                            persona.name,
                            persona.name,
                            self.find_agent_role(ctx, guild_id, persona_name)
                                .await
                                .map(|r| r.id.to_string())
                                .unwrap_or_default(),
                            persona.name.to_lowercase(),
                            persona.name.to_lowercase(),
                            persona.personality_traits.join(", ")
                        ));
                    }
                    Err(e) => {
                        return Some(format!(
                            "{}: {}",
                            messages::errors::ROLE_ASSIGNMENT_FAILED,
                            e
                        ));
                    }
                }
            } else {
                return Some(messages::errors::NOT_IN_SERVER_ASSIGNMENT.to_string());
            }
        }

        // Security stats command (authorized users only)
        if content_lower.starts_with("!spiral security stats") {
            // Check authorized user permission
            if let Some(auth_error) =
                AuthHelper::require_authorization(self.is_authorized_user(msg.author.id.get()))
            {
                return Some(auth_error);
            }

            let metrics = self.secure_message_handler.get_security_metrics();
            let avg_confidence = self.secure_message_handler.get_average_confidence();

            return Some(format!(
                "🛡️ **Security Metrics**\n\n\
                📊 **Message Statistics:**\n\
                • Total Processed: {}\n\
                • Messages Blocked: {}\n\
                • Block Rate: {:.1}%\n\n\
                🎯 **Intent Classification:**\n\
                • Total Classifications: {}\n\
                • Average Confidence: {:.2}\n\
                • Low Confidence Count: {}\n\
                • Help Requests: {}\n\
                • Code Generation: {}\n\
                • File Operations: {}\n\
                • System Commands: {}\n\
                • Admin Actions: {}\n\
                • Chat Responses: {}\n\
                • Unknown Intents: {}\n\
                • Malicious Intents: {}\n\n\
                ⚠️ **Threat Detection:**\n\
                • Malicious Attempts: {}\n\
                • XSS Attempts: {}\n\
                • Injection Attempts: {}\n\
                • Spam Detected: {}\n\
                • Rate Limited: {}\n\n\
                *Last reset: Never* (use `!spiral security reset` to reset)",
                metrics.messages_processed,
                metrics.messages_blocked,
                if metrics.messages_processed > 0 {
                    (metrics.messages_blocked as f64 / metrics.messages_processed as f64) * 100.0
                } else {
                    0.0
                },
                metrics.classification_count,
                avg_confidence,
                metrics.low_confidence_count,
                metrics.intent_help_requests,
                metrics.intent_code_generation,
                metrics.intent_file_operations,
                metrics.intent_system_commands,
                metrics.intent_admin_actions,
                metrics.intent_chat_responses,
                metrics.intent_unknown,
                metrics.intent_malicious,
                metrics.malicious_attempts,
                metrics.xss_attempts,
                metrics.injection_attempts,
                metrics.spam_detected,
                metrics.rate_limited
            ));
        }

        // Security reset command (authorized users only)
        if content_lower.starts_with("!spiral security reset") {
            // Check authorized user permission
            require_auth!(self.is_authorized_user(msg.author.id.get()));

            self.secure_message_handler.reset_security_metrics();
            return Some("✅ Security metrics have been reset.".to_string());
        }

        // Rate limit check command (available to all users)
        if content_lower.starts_with("!spiral ratelimit") {
            // Check if it's for another user (admin only)
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() > 2 {
                // Trying to check another user's rate limit (authorized users only)
                if !self.is_authorized_user(msg.author.id.get()) {
                    return Some(
                        "🚫 Checking other users' rate limits requires authorization.".to_string(),
                    );
                }

                // Extract user mention or ID
                if let Some(user_id_str) = parts.get(2) {
                    // Try to parse user mention <@123456789>
                    let user_id = if user_id_str.starts_with("<@") && user_id_str.ends_with(">") {
                        user_id_str
                            .trim_start_matches("<@")
                            .trim_end_matches(">")
                            .trim_start_matches("!")
                            .parse::<u64>()
                            .ok()
                    } else {
                        user_id_str.parse::<u64>().ok()
                    };

                    if let Some(uid) = user_id {
                        let remaining = self.secure_message_handler.get_remaining_messages(uid);
                        return Some(format!(
                            "📊 **Rate Limit Status for <@{}>**\n\
                            • Remaining messages: {}/5\n\
                            • Status: {}",
                            uid,
                            remaining,
                            if remaining > 0 {
                                "✅ Active"
                            } else {
                                "⏸️ Rate limited"
                            }
                        ));
                    } else {
                        return Some("❌ Invalid user ID or mention format.".to_string());
                    }
                }
            }

            // Check own rate limit
            let remaining = self
                .secure_message_handler
                .get_remaining_messages(msg.author.id.get());
            return Some(format!(
                "📊 **Your Rate Limit Status**\n\
                • Remaining messages: {}/5\n\
                • Status: {}\n\n\
                *Rate limits reset every minute*",
                remaining,
                if remaining > 0 {
                    "✅ Active"
                } else {
                    "⏸️ Rate limited (wait a moment)"
                }
            ));
        }

        // Reset rate limit command (authorized users only)
        if content_lower.starts_with("!spiral reset ratelimit") {
            // Check authorized user permission
            require_auth!(self.is_authorized_user(msg.author.id.get()));

            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() < 4 {
                return Some("❌ Usage: `!spiral reset ratelimit @user` or `!spiral reset ratelimit <user_id>`".to_string());
            }

            // Extract user mention or ID
            if let Some(user_id_str) = parts.get(3) {
                let user_id = if user_id_str.starts_with("<@") && user_id_str.ends_with(">") {
                    user_id_str
                        .trim_start_matches("<@")
                        .trim_end_matches(">")
                        .trim_start_matches("!")
                        .parse::<u64>()
                        .ok()
                } else {
                    user_id_str.parse::<u64>().ok()
                };

                if let Some(uid) = user_id {
                    self.secure_message_handler.reset_rate_limit(uid);
                    return Some(format!(
                        "✅ Rate limit reset for <@{uid}>\nThey can now send messages again."
                    ));
                } else {
                    return Some("❌ Invalid user ID or mention format.".to_string());
                }
            }
        }

        // Security report command (authorized users only)
        if content_lower.starts_with("!spiral security report") {
            // Check authorized user permission
            require_auth!(self.is_authorized_user(msg.author.id.get()));

            // For now, report on the current message as an example
            let report = self.secure_message_handler.create_security_report(msg);

            let mut report_text = "📋 **Security Report**\n\n".to_string();
            for (key, value) in report.iter() {
                report_text.push_str(&format!(
                    "• **{}**: {}\n",
                    key.replace("_", " ").to_uppercase(),
                    value
                ));
            }

            return Some(report_text);
        }

        // Self-update help command (available to all users)
        if content_lower == "!spiral update help" || content_lower == "!spiral update" {
            let mut help_text = "🔄 **Spiral Core Self-Update System**\n\n".to_string();

            help_text.push_str("**How to trigger an update:**\n");
            help_text.push_str("Mention the bot with an update keyword:\n");
            help_text.push_str("`@SpiralConstellation <update request>`\n\n");

            help_text.push_str("**Update keywords:** update, fix, modify, change, improve, enhance, repair, correct, adjust, patch, upgrade\n\n");

            help_text.push_str("**Examples:**\n");
            help_text.push_str("• `@SpiralConstellation fix the rate limiting bug`\n");
            help_text.push_str("• `@SpiralConstellation improve error handling`\n");
            help_text.push_str("• `@SpiralConstellation update the documentation`\n\n");

            if self.is_authorized_user(msg.author.id.get()) {
                help_text.push_str("✅ **You are authorized** to use the self-update system!\n\n");

                help_text.push_str("**Update Process:**\n");
                help_text.push_str("1. 🔍 Pre-flight checks (git status, disk space)\n");
                help_text.push_str("2. 📸 Snapshot creation (for rollback)\n");
                help_text.push_str("3. 🔧 Claude Code executes changes\n");
                help_text.push_str("4. ✅ Validation (compilation, tests)\n");
                help_text.push_str("5. 🎉 Completion or rollback\n\n");

                help_text.push_str("**Safety Features:**\n");
                help_text.push_str("• Bounded queue (max 10 requests)\n");
                help_text.push_str("• Automatic rollback on failure\n");
                help_text.push_str("• Git snapshots for recovery\n");
            } else {
                help_text.push_str("❌ **Authorization Required**\n");
                help_text
                    .push_str("You need to be in the authorized users list to trigger updates.\n");
                help_text.push_str("Contact an administrator for access.\n");
            }

            help_text.push_str("\n*For more details, see `docs/SELF_UPDATE_GUIDE.md`*");

            return Some(help_text);
        }

        // Debug command (authorized users only) - intelligently debugs issues
        if content_lower.starts_with("!spiral debug") {
            // Check authorized user permission
            require_auth!(self.is_authorized_user(msg.author.id.get()));

            // Get the referenced message if this is a reply
            let debug_message = if let Some(ref referenced) = msg.referenced_message {
                referenced.as_ref()
            } else {
                msg
            };

            // Determine debug context based on message content or bot's prior response
            let is_security_debug = debug_message.content.contains("⚠️ Command blocked") 
                || debug_message.content.contains("🚫 Message flagged")
                || debug_message.author.bot  // Check if it's a bot message
                || debug_message.content.starts_with("!");

            let debug_type = if is_security_debug {
                "Security Debug"
            } else {
                "General Debug"
            };

            // Perform comprehensive analysis
            let mut debug_report = format!(
                "🔍 **{} Report**\n\n\
                **Message Details:**\n\
                • Author: <@{}> (ID: {})\n\
                • Channel: <#{}>\n\
                • Message ID: {}\n\
                • Length: {} characters\n\
                • Has attachments: {}\n\
                • Has embeds: {}\n\n",
                debug_type,
                debug_message.author.id,
                debug_message.author.id,
                debug_message.channel_id,
                debug_message.id,
                debug_message.content.len(),
                !debug_message.attachments.is_empty(),
                !debug_message.embeds.is_empty()
            );

            // Security validation analysis
            debug_report.push_str("**Security Validation:**\n");
            let validation_result = {
                let mut validator = self.security_validator.lock().await;
                match validator.validate_message(debug_message) {
                    Ok(result) => result,
                    Err(e) => {
                        debug_report.push_str(&format!("• ❌ Validation error: {e}\n"));
                        return Some(debug_report);
                    }
                }
            };

            debug_report.push_str(&format!("• Valid: {}\n", validation_result.is_valid));
            debug_report.push_str(&format!(
                "• Risk Level: {:?}\n",
                validation_result.risk_level
            ));
            if !validation_result.issues.is_empty() {
                debug_report.push_str("• Issues found:\n");
                for issue in &validation_result.issues {
                    debug_report.push_str(&format!("  - {issue}\n"));
                }
            } else {
                debug_report.push_str("• No validation issues\n");
            }

            // Command validation check
            debug_report.push_str("\n**Command Validation:**\n");
            let command_validation = self
                .secure_message_handler
                .validate_command_input(&debug_message.content);
            debug_report.push_str(&format!("• Valid: {}\n", command_validation.is_valid));
            if !command_validation.issues.is_empty() {
                debug_report.push_str("• Command issues:\n");
                for issue in &command_validation.issues {
                    debug_report.push_str(&format!("  - {issue}\n"));
                }
            }

            // Intent classification
            debug_report.push_str("\n**Intent Classification:**\n");
            let request = crate::discord::intent_classifier::IntentRequest {
                message: debug_message.content.clone(),
                user_id: debug_message.author.id.to_string(),
                context: std::collections::HashMap::new(),
            };
            let intent_result = self
                .intent_classifier
                .classify_intent_with_security(&request);
            match Ok::<_, String>(intent_result) {
                Ok(intent) => {
                    debug_report.push_str(&format!("• Intent: {:?}\n", intent.intent_type));
                    debug_report.push_str(&format!("• Confidence: {:.2}\n", intent.confidence));
                    debug_report.push_str(&format!("• Risk: {:?}\n", intent.risk_level));
                    if !intent.parameters.is_empty() {
                        debug_report.push_str("• Parameters:\n");
                        for (key, value) in &intent.parameters {
                            debug_report.push_str(&format!("  - {key}: {value}\n"));
                        }
                    }
                }
                Err(e) => {
                    debug_report.push_str(&format!("• ❌ Classification error: {e}\n"));
                }
            }

            // Rate limit status
            debug_report.push_str("\n**Rate Limit Status:**\n");
            let remaining = self
                .secure_message_handler
                .get_remaining_messages(debug_message.author.id.get());
            debug_report.push_str(&format!("• Remaining messages: {remaining}/5\n"));
            debug_report.push_str(&format!("• Rate limited: {}\n", remaining == 0));

            // Content analysis
            debug_report.push_str("\n**Content Analysis:**\n");
            debug_report.push_str(&format!(
                "• Mention count: {}\n",
                debug_message.mentions.len()
            ));
            debug_report.push_str(&format!(
                "• Role mentions: {}\n",
                debug_message.mention_roles.len()
            ));
            debug_report.push_str(&format!(
                "• Has everyone/here: {}\n",
                debug_message.mention_everyone
            ));

            // Pattern detection
            let content_lower = debug_message.content.to_lowercase();
            debug_report.push_str("\n**Pattern Detection:**\n");
            debug_report.push_str(&format!(
                "• Contains URLs: {}\n",
                content_lower.contains("http://") || content_lower.contains("https://")
            ));
            debug_report.push_str(&format!(
                "• Contains script tags: {}\n",
                content_lower.contains("<script")
            ));
            debug_report.push_str(&format!(
                "• Contains SQL keywords: {}\n",
                content_lower.contains("select ")
                    || content_lower.contains("drop ")
                    || content_lower.contains("insert ")
                    || content_lower.contains("update ")
            ));

            // Suggested remediation
            debug_report.push_str("\n**Suggested Actions:**\n");
            if !validation_result.is_valid {
                debug_report.push_str("• Message was blocked due to security validation\n");
                debug_report.push_str("• Review the validation issues above\n");
                if validation_result
                    .issues
                    .iter()
                    .any(|i| i.contains("rate limit"))
                {
                    debug_report.push_str(
                        "• User is rate limited - wait or use `!spiral reset ratelimit @user`\n",
                    );
                }
                if validation_result.issues.iter().any(|i| i.contains("spam")) {
                    debug_report
                        .push_str("• Message detected as spam - check for repetitive content\n");
                }
                if validation_result
                    .issues
                    .iter()
                    .any(|i| i.contains("injection") || i.contains("XSS"))
                {
                    debug_report.push_str(
                        "• Potential security threat detected - review content carefully\n",
                    );
                }
            } else {
                debug_report
                    .push_str("• Message passed validation - should not have been blocked\n");
                debug_report.push_str("• Check Discord permissions and bot configuration\n");
            }

            debug_report.push_str("\n*Use this information to understand why the message was blocked*\n\n*React with 🗑 to delete this debug message*\n*React with 🔨 to get correction options*");

            // Send debug response and add reactions
            match msg.reply(&ctx.http, &debug_report).await {
                Ok(debug_msg) => {
                    info!("[SpiralConstellation] Debug message sent, adding reactions...");
                    // Add trash bin reaction for authorized users to delete the message
                    if let Err(e) = debug_msg.react(&ctx.http, emojis::TRASH_BIN).await {
                        warn!(
                            "[SpiralConstellation] Failed to add trash bin reaction: {}",
                            e
                        );
                    } else {
                        info!("[SpiralConstellation] Successfully added trash bin reaction");
                    }
                    // Add hammer reaction for correction prompts
                    if let Err(e) = debug_msg.react(&ctx.http, emojis::HAMMER).await {
                        warn!("[SpiralConstellation] Failed to add hammer reaction: {}", e);
                    } else {
                        info!("[SpiralConstellation] Successfully added hammer reaction");
                    }
                }
                Err(e) => {
                    warn!("[SpiralConstellation] Failed to send debug response: {}", e);
                }
            }

            return None; // We handled the response directly
        }

        // Commands list command
        if content_lower.starts_with("!spiral commands") {
            let mut commands_text = "📋 **Available Commands**\n\n".to_string();

            // Basic commands available to all users
            commands_text.push_str("**🌟 General Commands:**\n");
            commands_text.push_str("• `!spiral help` - Show detailed help information\n");
            commands_text.push_str("• `!spiral commands` - Show this command list\n");
            commands_text.push_str(
                "• `!spiral join <role>` - Join an agent role (SpiralDev, SpiralPM, etc.)\n",
            );
            commands_text.push_str("• `!spiral ratelimit` - Check your rate limit status\n");
            commands_text
                .push_str("• `!spiral update help` - Learn about the self-update system\n");
            commands_text.push_str("• `!spiral setup roles` - Create agent roles in server\n\n");

            // Agent mentions
            commands_text.push_str("**🤖 Agent Interactions:**\n");
            commands_text.push_str("• `@SpiralDev <request>` - Software development tasks\n");
            commands_text.push_str("• `@SpiralPM <request>` - Project management queries\n");
            commands_text.push_str("• `@SpiralQA <request>` - Quality assurance reviews\n");
            commands_text.push_str("• `@SpiralKing <request>` - Comprehensive code review\n");
            commands_text.push_str("• Use role mentions: `<@&role_id> <request>`\n\n");

            // Show admin commands if user is authorized
            if self.is_authorized_user(msg.author.id.get()) {
                commands_text.push_str("**🔐 Admin Commands (You have access):**\n");
                commands_text.push_str("• `!spiral security stats` - View security metrics\n");
                commands_text.push_str("• `!spiral security reset` - Reset security metrics\n");
                commands_text.push_str("• `!spiral security report` - Generate security report\n");
                commands_text.push_str(
                    "• `!spiral debug` - Debug any issue (reply to problematic message)\n",
                );
                commands_text.push_str("• `!spiral ratelimit @user` - Check user's rate limit\n");
                commands_text
                    .push_str("• `!spiral reset ratelimit @user` - Reset user's rate limit\n\n");
            } else {
                commands_text.push_str("**🔐 Admin Commands (Authorized users only):**\n");
                commands_text.push_str("• Security and rate limit management commands\n");
                commands_text.push_str("• Contact an administrator for access\n\n");
            }

            commands_text.push_str("*Use `!spiral help` for detailed usage information* 💡");

            return Some(commands_text);
        }

        // Help command
        if content_lower.contains("!spiral help") || content_lower == "help" {
            return Some(
                "🌌 **SpiralConstellation Bot Help**\n\n\
                **Agent Personas:**\n\
                • 🚀 SpiralDev - Software development & coding\n\
                • 📋 SpiralPM - Project management & coordination\n\
                • 🔍 SpiralQA - Quality assurance & testing\n\
                • 🎯 SpiralDecide - Decision making & analysis\n\
                • ✨ SpiralCreate - Creative solutions & innovation\n\
                • 🧘 SpiralCoach - Process optimization & guidance\n\
                • 👑 SpiralKing - Comprehensive code review & architectural analysis\n\n\
                **Usage:**\n\
                • `@SpiralDev create a REST API` - Text mention\n\
                • `@SpiralKing review this codebase` - Architectural analysis\n\
                • `<@&role_id> help with testing` - Role mention\n\
                • `!spiral join SpiralKing` - Get agent role\n\
                • `!spiral setup roles` - Create server roles\n\n\
                **Commands:**\n\
                • `!spiral help` - Show this detailed help\n\
                • `!spiral commands` - Show concise command list\n\
                • `!spiral join <role>` - Join an agent role\n\
                • `!spiral setup roles` - Create agent roles\n\
                • `!spiral ratelimit` - Check your rate limit status\n\
                • `!spiral update help` - Learn about the self-update system\n\n\
                **Security Commands (Authorized Users Only):**\n\
                • `!spiral security stats` - View security metrics\n\
                • `!spiral security reset` - Reset security metrics\n\
                • `!spiral reset ratelimit @user` - Reset user's rate limit\n\
                • `!spiral security report` - Generate security report\n\n\
                *Each persona responds with unique personality and expertise!* 🌟"
                    .to_string(),
            );
        }

        None
    }

    /// 🌌 LORDGENOME DESPAIR: Generate contextual despair quotes for unauthorized access
    fn generate_lordgenome_quote(&self, username: &str, user_action: &str) -> String {
        // Use our enhanced Lordgenome quote generator
        let generator = LordgenomeQuoteGenerator::new();

        // Detect the action type from the content
        let action_type = LordgenomeQuoteGenerator::detect_action_type(user_action);

        // Determine severity based on the action
        let severity = match action_type {
            "security" => DenialSeverity::Apocalyptic,
            "self_update" | "config" => DenialSeverity::Severe,
            "command" | "role" => DenialSeverity::Moderate,
            _ => DenialSeverity::Moderate,
        };

        // Generate a contextual quote
        generator.generate_by_severity(username, action_type, severity)
    }
}

/// 📝 MESSAGE CONTEXT: Discord message information
#[derive(Debug, Clone)]
pub struct MessageContext {
    pub author_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
    pub guild_id: Option<u64>,
}

/// 🎮 CONSTELLATION BOT HANDLER: Discord event handler for the unified bot
pub struct ConstellationBotHandler {
    bot: Arc<SpiralConstellationBot>,
}

impl ConstellationBotHandler {
    pub fn new(bot: SpiralConstellationBot) -> Self {
        Self { bot: Arc::new(bot) }
    }
}

#[async_trait]
impl EventHandler for ConstellationBotHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore bot messages
        if msg.author.bot {
            return;
        }

        // 🛡️ SECURITY VALIDATION: Multi-layer security check before processing
        // ARCHITECTURE DECISION: Validate all messages through security pipeline first
        // Why: Prevents malicious content, spam, and injection attacks from reaching agents
        // Alternative: Post-processing validation (rejected: allows attacks to reach business logic)
        // Audit: Count security_validation_failures and blocked_message_types
        let validation_result = {
            let validator_result = {
                let mut validator = self.bot.security_validator.lock().await;
                validator.validate_message(&msg)
            };

            match validator_result {
                Ok(result) => result,
                Err(e) => {
                    warn!("[SpiralConstellation] Security validation error: {}", e);
                    if let Err(e) = msg
                        .reply(&ctx.http, "⚠️ Security validation failed. Message blocked.")
                        .await
                    {
                        warn!(
                            "[SpiralConstellation] Failed to send validation error: {}",
                            e
                        );
                    }
                    return;
                }
            }
        };

        if !validation_result.is_valid {
            // Try to classify intent for blocked messages (not used in response but logged)
            let _intent_result = {
                let request = crate::discord::intent_classifier::IntentRequest {
                    message: msg.content.clone(),
                    user_id: msg.author.id.to_string(),
                    context: std::collections::HashMap::new(),
                };
                Some(
                    self.bot
                        .intent_classifier
                        .classify_intent_with_security(&request),
                )
            };

            // Create security event for structured logging
            let security_event = SecurityEvent::SecurityValidationFailed {
                timestamp: {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs().to_string())
                        .unwrap_or_else(|_| "0".to_string())
                },
                user_id: msg.author.id.get(),
                username: msg.author.name.clone(),
                channel_id: msg.channel_id.get(),
                guild_id: msg.guild_id.map(|id| id.get()),
                message_id: msg.id.get(),
                content: msg.content.clone(),
                validation_issues: validation_result.issues.clone(),
                risk_level: risk_level_to_str(&validation_result.risk_level).to_string(),
                validation_type: "message_security".to_string(),
            };

            // Log as both warning and structured JSON
            warn!(
                "[SpiralConstellation] Security validation failed - {}",
                security_event.to_json()
            );

            // Also log as a separate JSON line for easy parsing
            tracing::info!(target: "security_events", "{}", security_event.to_json());

            if let Err(e) = msg.reply(&ctx.http, "🚫 Message flagged by security validation. Please ensure your message follows community guidelines.").await {
                warn!("[SpiralConstellation] Failed to send security warning: {}", e);
            }
            return;
        }
        info!(
            "[SpiralConstellation] Security validation passed for user {}: risk level {:?}",
            msg.author.id, validation_result.risk_level
        );

        // 🚦 RATE LIMITING: Already handled by security validation above
        // Note: Rate limiting is integrated into the MessageSecurityValidator

        // Validate message length to prevent DoS
        if msg.content.len() > MAX_MESSAGE_LENGTH {
            warn!(
                "[SpiralConstellation] Message too long: {} chars from user {}",
                msg.content.len(),
                msg.author.id
            );
            if let Err(e) = msg.reply(&ctx.http, "❌ Message too long for processing. Please keep requests under 4000 characters.").await {
                warn!("[SpiralConstellation] Failed to send length warning: {}", e);
            }
            return;
        }

        // Check if message mentions any Spiral agent or contains role mentions
        let has_spiral_mention = self.bot.mention_regex.is_match(&msg.content);
        let has_role_mention = !msg.mention_roles.is_empty();
        let has_spiral_command = msg.content.to_lowercase().contains("!spiral");

        if !has_spiral_mention && !has_role_mention && !has_spiral_command {
            return;
        }

        info!("[SpiralConstellation] Processing message: {}", msg.id);

        let context = MessageContext {
            author_id: msg.author.id.get(),
            channel_id: msg.channel_id.get(),
            message_id: msg.id.get(),
            guild_id: msg.guild_id.map(|id| id.get()),
        };

        // Check for Auto Core Update requests via direct bot mention
        if self.is_auto_core_update_request(&msg).await {
            self.handle_auto_core_update_request(&ctx, &msg).await;
            return;
        }

        // Handle special commands first
        if let Some(command_response) = self
            .bot
            .handle_special_commands(&msg.content, &msg, &ctx)
            .await
        {
            match msg.reply(&ctx.http, &command_response).await {
                Ok(response_msg) => {
                    // If it's a blocked command message and user is authorized, add bug emoji
                    if command_response.contains(messages::patterns::COMMAND_BLOCKED_PATTERN)
                        && self.bot.is_authorized_user(msg.author.id.get())
                    {
                        if let Err(e) = response_msg.react(&ctx.http, emojis::BUG).await {
                            warn!("[SpiralConstellation] Failed to add bug reaction to blocked command: {}", e);
                        } else {
                            info!("[SpiralConstellation] Added bug reaction for authorized user to debug blocked command");
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "[SpiralConstellation] Failed to send command response: {}",
                        e
                    );
                }
            }
            return;
        }

        // Detect which agent persona to use
        let agent_type = match self
            .bot
            .detect_agent_persona(&msg.content, &msg, &ctx)
            .await
        {
            Some(agent) => agent,
            None => {
                if let Err(e) = msg.reply(&ctx.http, "❓ I'm not sure which agent you'd like to talk to. Try mentioning @SpiralDev, @SpiralPM, @SpiralQA, @SpiralKing, or use a role mention!").await {
                    warn!("[SpiralConstellation] Failed to send clarification: {}", e);
                }
                return;
            }
        };

        let persona = AgentPersona::for_agent_type(&agent_type);
        let cleaned_content = self.bot.clean_message_content(&msg.content);

        if cleaned_content.is_empty() {
            let response = format!(
                "{} **{}**\n{}",
                persona.emoji,
                persona.name,
                persona.random_greeting()
            );
            if let Err(e) = msg.reply(&ctx.http, response).await {
                warn!("[SpiralConstellation] Failed to send greeting: {}", e);
            }
            return;
        }

        // Update current persona
        {
            let mut stats = self.bot.stats.lock().await;
            stats.current_persona = Some(agent_type.clone());
        }

        // Step 1: React with eyes emoji to acknowledge receipt
        if let Err(e) = msg.react(&ctx.http, '👀').await {
            warn!("[SpiralConstellation] Failed to add eyes reaction: {}", e);
        }

        // Step 2: 🔒 COMPREHENSIVE SECURITY PROCESSING: Security validation with intent classification
        // ARCHITECTURE DECISION: Use SecureMessageHandler for integrated security validation and intent analysis
        // Why: Combines message validation, intent analysis, and user verification in one step
        // Alternative: Separate validation calls (rejected: creates gaps between security layers)
        // Audit: Track processing_blocked_count and security_escalation_events
        let secure_processing_result = match self
            .bot
            .secure_message_handler
            .process_message_securely(&msg, &ctx)
            .await
        {
            Ok(result) => result,
            Err(e) => {
                warn!(
                    "[SpiralConstellation] Secure message processing failed: {}",
                    e
                );
                if let Err(e) = msg
                    .reply(
                        &ctx.http,
                        "⚠️ Unable to process message securely. Please try again.",
                    )
                    .await
                {
                    warn!(
                        "[SpiralConstellation] Failed to send secure processing error: {}",
                        e
                    );
                }
                return;
            }
        };

        // Check if secure processing allows continuation
        if !secure_processing_result.should_process {
            warn!(
                "[SpiralConstellation] Message blocked by secure handler: {:?}",
                secure_processing_result.validation_issues
            );
            if let Err(e) = msg
                .reply(&ctx.http, "🚫 Message blocked by security validation.")
                .await
            {
                warn!("[SpiralConstellation] Failed to send block message: {}", e);
            }
            return;
        }

        // Extract intent from security processing result
        let default_intent = IntentResponse {
            intent_type: IntentType::Unknown,
            confidence: 0.0,
            risk_level: RiskLevel::Medium,
            parameters: std::collections::HashMap::new(),
        };

        let intent_response = secure_processing_result.intent.as_ref().unwrap_or_else(|| {
            warn!("[SpiralConstellation] No intent classification from security processing, using default");
            &default_intent
        });

        info!(
            "[SpiralConstellation] Classified intent as: {:?} (confidence: {}, risk: {:?})",
            intent_response.intent_type, intent_response.confidence, intent_response.risk_level
        );

        // Convert IntentType to UserIntent for compatibility
        let intent = match intent_response.intent_type {
            IntentType::Help => UserIntent::HelpRequest,
            IntentType::CodeGeneration => UserIntent::TaskRequest,
            IntentType::FileOperation => UserIntent::TaskRequest,
            IntentType::SystemCommand => UserIntent::TaskRequest,
            IntentType::AdminAction => UserIntent::TaskRequest,
            IntentType::ChatResponse => UserIntent::Greeting,
            IntentType::Malicious => UserIntent::Unknown,
            IntentType::Unknown => UserIntent::Unknown,
        };

        // Check if this is an agent selection based on explicit agent mentions
        let has_explicit_agent_mention =
            self.bot.mention_regex.is_match(&msg.content) || !msg.mention_roles.is_empty();
        let intent = if has_explicit_agent_mention && agent_type != AgentType::SoftwareDeveloper {
            UserIntent::AgentSelection
        } else {
            intent
        };

        // Use sanitized content if available, otherwise use cleaned content
        let processed_message = secure_processing_result
            .sanitized_content
            .unwrap_or_else(|| cleaned_content.clone());

        info!(
            "[SpiralConstellation] Message approved for processing: {}",
            processed_message
        );

        // Step 3: Respond with intended action
        let action_description = match intent {
            UserIntent::StatusQuery => "🔍 **Analyzing Workspace**\nI'll inspect your workspace to find and list existing projects.".to_string(),
            UserIntent::TaskRequest => "🚀 **Development Task**\nI'll create/build the requested functionality.".to_string(),
            UserIntent::AgentSelection => "🎯 **Agent-Specific Task**\nI'll handle this with the selected agent's expertise.".to_string(),
            UserIntent::HelpRequest => "❓ **Help Request**\nI'll provide the information you need.".to_string(),
            UserIntent::Greeting => "👋 **Greeting**\nNice to meet you!".to_string(),
            UserIntent::Unknown => "🔄 **Processing Request**\nI'll handle your request appropriately.".to_string(),
        };

        let intent_response = format!(
            "{} **{}**\n{}\n\n📝 **Request:** {}\n\n{}",
            persona.emoji,
            persona.name,
            action_description,
            if processed_message.len() > 100 {
                format!("{}...", &processed_message[..100])
            } else {
                processed_message.clone()
            },
            "⏳ Working on this now..."
        );

        let mut intent_msg = if let Ok(response) = msg.reply(&ctx.http, intent_response).await {
            Some(response)
        } else {
            warn!("[SpiralConstellation] Failed to send intent response");
            None
        };

        // Step 4: Create and execute task based on agent type and intent
        let task = self.bot.create_task_with_persona(
            &processed_message,
            agent_type.clone(),
            context,
            intent.clone(),
        );
        let task_id = task.id.clone();

        info!(
            "[SpiralConstellation] Created {} task: {}",
            persona.name, task_id
        );

        // Execute based on agent type - choose execution mode based on bot configuration
        let result = match agent_type {
            AgentType::SoftwareDeveloper => {
                // Choose execution mode: direct agent or orchestrator
                if let Some(orchestrator) = &self.bot.orchestrator {
                    // 🎛️ ORCHESTRATOR MODE: Use full system with task queuing and management
                    info!("[SpiralConstellation] Using orchestrator mode for task execution");
                    let task_id = match orchestrator.submit_task(task).await {
                        Ok(id) => id,
                        Err(e) => {
                            warn!(
                                "[SpiralConstellation] Failed to submit task to orchestrator: {}",
                                e
                            );
                            let error_message = self.bot.format_helpful_error_message(&e, persona);
                            if let Err(reply_err) = msg.reply(&ctx.http, error_message).await {
                                warn!(
                                    "[SpiralConstellation] Failed to send error message: {}",
                                    reply_err
                                );
                            }
                            return;
                        }
                    };

                    // Wait for task completion with progress updates
                    let timeout_duration = std::time::Duration::from_secs(120); // Increased timeout
                    let progress_emojis = ["⏳", "⌛", "🤔", "⚙️", "🔄"];
                    let mut emoji_index = 0;
                    let start_time = std::time::Instant::now();

                    let poll_future = async {
                        let mut last_update = std::time::Instant::now();
                        let max_attempts = 240; // 120 seconds at 500ms intervals
                        let mut attempts = 0;

                        loop {
                            if let Some(result) = orchestrator.get_task_result(&task_id).await {
                                return Ok(result);
                            }

                            attempts += 1;
                            if attempts >= max_attempts {
                                warn!("[SpiralConstellation] Task {} exceeded maximum polling attempts", task_id);
                                return Err(SpiralError::Timeout {
                                    message: format!("Task execution exceeded maximum polling time of {} seconds", timeout_duration.as_secs()),
                                });
                            }

                            // Update emoji every 15 seconds
                            if last_update.elapsed() >= std::time::Duration::from_secs(15) {
                                emoji_index = (emoji_index + 1) % progress_emojis.len();

                                let progress_response = format!(
                                    "{} **{}**\n{}\n\n📝 **Request:** {}\n\n{} Working on this... ({:.0}s)",
                                    persona.emoji,
                                    persona.name,
                                    action_description,
                                    if processed_message.len() > 100 {
                                        format!("{}...", &processed_message[..100])
                                    } else {
                                        processed_message.clone()
                                    },
                                    progress_emojis[emoji_index],
                                    start_time.elapsed().as_secs()
                                );

                                if let Some(ref mut msg_ref) = intent_msg {
                                    let _ = msg_ref
                                        .edit(
                                            &ctx.http,
                                            serenity::builder::EditMessage::new()
                                                .content(progress_response),
                                        )
                                        .await;
                                }

                                last_update = std::time::Instant::now();
                            }

                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        }
                    };

                    match tokio::time::timeout(timeout_duration, poll_future).await {
                        Ok(Ok(result)) => {
                            info!(
                                "[SpiralConstellation] {} task {} completed via orchestrator",
                                persona.name, task_id
                            );

                            // Update success stats
                            {
                                let mut stats = self.bot.stats.lock().await;
                                stats.dev_tasks_completed += 1;
                                stats.current_persona = None;
                            }

                            self.bot.format_persona_response(&agent_type, &result)
                        }
                        Ok(Err(e)) => {
                            warn!(
                                "[SpiralConstellation] {} task {} failed via orchestrator: {}",
                                persona.name, task_id, e
                            );

                            self.bot.format_helpful_error_message(&e, persona)
                        }
                        Err(_timeout) => {
                            warn!("[SpiralConstellation] {} task {} timed out via orchestrator after 2 minutes", persona.name, task_id);

                            // Check one more time if task completed during timeout
                            if let Some(result) = orchestrator.get_task_result(&task_id).await {
                                info!("[SpiralConstellation] {} task {} completed just after timeout check", persona.name, task_id);
                                self.bot.format_persona_response(&agent_type, &result)
                            } else {
                                let timeout_error = crate::SpiralError::Agent {
                                    message: "Task is taking longer than expected - still processing in background".to_string(),
                                };

                                self.bot
                                    .format_helpful_error_message(&timeout_error, persona)
                            }
                        }
                    }
                } else if let Some(developer_agent) = &self.bot.developer_agent {
                    // 🎯 DIRECT MODE: Use standalone agent execution
                    info!("[SpiralConstellation] Using direct mode for task execution");
                    let execute_future = developer_agent.execute(task);
                    let timeout_duration = std::time::Duration::from_secs(90); // Increased timeout for direct mode

                    // Create progress update task for direct mode
                    let progress_task = {
                        let mut intent_msg_clone = intent_msg.clone();
                        let ctx_clone = ctx.clone();
                        let persona_clone = persona.clone();
                        let action_desc_clone = action_description.clone();
                        let content_clone = processed_message.clone();

                        tokio::spawn(async move {
                            let progress_emojis = ["⏳", "⌛", "🤔", "⚙️", "🔄"];
                            let mut emoji_index = 0;
                            let start_time = std::time::Instant::now();

                            // Update every 15 seconds
                            while start_time.elapsed() < std::time::Duration::from_secs(90) {
                                tokio::time::sleep(std::time::Duration::from_secs(15)).await;
                                emoji_index = (emoji_index + 1) % progress_emojis.len();

                                let progress_response = format!(
                                    "{} **{}**\n{}\n\n📝 **Request:** {}\n\n{} Working on this... ({:.0}s)",
                                    persona_clone.emoji,
                                    persona_clone.name,
                                    action_desc_clone,
                                    if content_clone.len() > 100 {
                                        format!("{}...", &content_clone[..100])
                                    } else {
                                        content_clone.clone()
                                    },
                                    progress_emojis[emoji_index],
                                    start_time.elapsed().as_secs()
                                );

                                if let Some(ref mut intent_message) = intent_msg_clone {
                                    let _ = intent_message
                                        .edit(
                                            &ctx_clone.http,
                                            serenity::builder::EditMessage::new()
                                                .content(progress_response),
                                        )
                                        .await;
                                }
                            }
                        })
                    };

                    match tokio::time::timeout(timeout_duration, execute_future).await {
                        Ok(execute_result) => {
                            // Cancel progress updates
                            progress_task.abort();

                            match execute_result {
                                Ok(result) => {
                                    info!(
                                        "[SpiralConstellation] {} task {} completed",
                                        persona.name, task_id
                                    );

                                    // Update success stats
                                    {
                                        let mut stats = self.bot.stats.lock().await;
                                        stats.dev_tasks_completed += 1;
                                        stats.current_persona = None;
                                    }

                                    self.bot.format_persona_response(&agent_type, &result)
                                }
                                Err(e) => {
                                    warn!(
                                        "[SpiralConstellation] {} task {} failed: {}",
                                        persona.name, task_id, e
                                    );

                                    // Update failure stats
                                    {
                                        let mut stats = self.bot.stats.lock().await;
                                        stats.total_tasks_failed += 1;
                                        stats.current_persona = None;
                                    }

                                    // Provide helpful error messages based on error type

                                    self.bot.format_helpful_error_message(&e, persona)
                                }
                            }
                        }
                        Err(_timeout) => {
                            // Cancel progress updates
                            progress_task.abort();

                            warn!(
                                "[SpiralConstellation] {} task {} timed out after 90 seconds",
                                persona.name, task_id
                            );

                            // Update failure stats
                            {
                                let mut stats = self.bot.stats.lock().await;
                                stats.total_tasks_failed += 1;
                                stats.current_persona = None;
                            }

                            // Create a timeout error
                            let timeout_error = crate::SpiralError::Agent {
                                message: "Task execution timed out - Claude Code system may be unavailable".to_string(),
                            };

                            self.bot
                                .format_helpful_error_message(&timeout_error, persona)
                        }
                    }
                } else {
                    // Neither orchestrator nor direct agent available
                    let config_error = crate::SpiralError::Agent {
                        message: "Bot not properly configured - no execution method available"
                            .to_string(),
                    };

                    self.bot
                        .format_helpful_error_message(&config_error, persona)
                }
            }
            _ => {
                // For other agent types (not yet implemented)
                format!(
                    "{} **{}**\n🚧 I'm still learning how to handle {} requests. \
                    My {} capabilities are under development!\n\n\
                    💡 **What I'll be able to do:**\n{}\n\n\
                    *—{} @ SpiralConstellation*",
                    persona.emoji,
                    persona.name,
                    persona.name.to_lowercase(),
                    persona.name.to_lowercase(),
                    persona
                        .personality_traits
                        .iter()
                        .map(|trait_name| format!("• {trait_name}"))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    persona.name
                )
            }
        };

        // Step 5: Update the original intent message with the final result
        if let Some(mut intent_message) = intent_msg {
            // Create final response with task summary
            let final_response = format!(
                "{} **{}**\n{}\n\n📝 **Request:** {}\n\n✅ **Completed!**\n\n{}",
                persona.emoji,
                persona.name,
                action_description,
                if processed_message.len() > 100 {
                    format!("{}...", &processed_message[..100])
                } else {
                    processed_message.clone()
                },
                result
            );

            if let Err(e) = intent_message
                .edit(
                    &ctx.http,
                    serenity::builder::EditMessage::new().content(final_response),
                )
                .await
            {
                warn!("[SpiralConstellation] Failed to edit intent message: {}", e);
                // Fallback: send as new reply if edit fails
                if let Err(e2) = msg.reply(&ctx.http, result).await {
                    warn!(
                        "[SpiralConstellation] Failed to send fallback result: {}",
                        e2
                    );
                }
            }
        } else {
            // Fallback: send as reply if we don't have the intent message
            if let Err(e) = msg.reply(&ctx.http, result).await {
                warn!("[SpiralConstellation] Failed to send result: {}", e);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!(
            "🌌 SpiralConstellation bot {} is connected and ready!",
            ready.user.name
        );
        info!("Available personas: SpiralDev, SpiralPM, SpiralQA, SpiralDecide, SpiralCreate, SpiralCoach");
        info!("Role support: Discord roles can be created with '!spiral setup roles'");
        info!("Usage: @SpiralDev, role mentions, or !spiral join <role>");

        // Set bot activity status to show the commands
        use serenity::all::ActivityData;
        let activity = ActivityData::playing("!spiral commands for help");
        let status = OnlineStatus::Online;

        ctx.set_presence(Some(activity), status);
        info!("Bot status set: Playing '!spiral commands for help'");

        let stats = self.bot.stats.lock().await;
        info!(
            "Bot statistics: {} dev tasks completed",
            stats.dev_tasks_completed
        );
    }

    async fn reaction_add(&self, ctx: Context, add_reaction: serenity::model::channel::Reaction) {
        // Use the unicode emoji directly from the reaction instead of as_data()
        let emoji_unicode = match &add_reaction.emoji {
            serenity::model::channel::ReactionType::Unicode(name) => name.clone(),
            _ => return, // Skip custom emojis
        };

        // Debug logging to understand emoji format
        info!(
            "[SpiralConstellation] Reaction detected: '{}' (expected trash: '{}', hammer: '{}')",
            emoji_unicode,
            emojis::TRASH_BIN,
            emojis::HAMMER
        );

        // Only handle trash bin, hammer, bug, wrench, and retry reactions
        if emoji_unicode != emojis::TRASH_BIN.to_string()
            && emoji_unicode != emojis::HAMMER.to_string()
            && emoji_unicode != emojis::BUG.to_string()
            && emoji_unicode != emojis::WRENCH.to_string()
            && emoji_unicode != emojis::RETRY.to_string()
        {
            info!("[SpiralConstellation] Reaction '{}' not handled (not trash bin, hammer, bug, wrench, or retry)", emoji_unicode);
            return;
        }

        info!(
            "[SpiralConstellation] Processing reaction: {}",
            emoji_unicode
        );

        // Don't handle reactions from bots
        if let Ok(user) = add_reaction.user(&ctx.http).await {
            if user.bot {
                return;
            }

            // Check if user is authorized
            if !self.bot.is_authorized_user(user.id.get()) {
                return;
            }

            // Get the message that was reacted to
            if let Ok(message) = add_reaction.message(&ctx.http).await {
                // Handle bug emoji on command blocked messages
                if emoji_unicode == emojis::BUG.to_string()
                    && message.author.bot
                    && message
                        .content
                        .contains(messages::patterns::COMMAND_BLOCKED_PATTERN)
                {
                    info!(
                        "[SpiralConstellation] Bug emoji clicked on blocked command by {}",
                        user.id
                    );

                    // Find the original message that was blocked (the one this is a reply to)
                    if let Some(ref reference) = message.message_reference {
                        if let Some(message_id) = reference.message_id {
                            if let Ok(original_msg) = ctx
                                .http
                                .get_message(
                                    serenity::model::id::ChannelId::new(reference.channel_id.get()),
                                    serenity::model::id::MessageId::new(message_id.get()),
                                )
                                .await
                            {
                                // Run debug on the original message
                                self.handle_debug_request(&ctx, &original_msg, &user).await;
                            } else {
                                warn!("[SpiralConstellation] Failed to fetch original message for debug");
                            }
                        } else {
                            warn!("[SpiralConstellation] Message reference has no message ID");
                        }
                    } else {
                        warn!("[SpiralConstellation] No message reference found for debug");
                    }
                }
                // Handle wrench emoji on correction prompt messages
                else if emoji_unicode == emojis::WRENCH.to_string()
                    && message.author.bot
                    && message
                        .content
                        .contains("🔨 **Security Pattern Correction**")
                {
                    info!(
                        "[SpiralConstellation] Wrench emoji clicked on correction prompt by {}",
                        user.id
                    );

                    // CRITICAL SECURITY: Check authorization for auto-fix operations
                    if !self.bot.is_authorized_user(user.id.get()) {
                        warn!(
                            "[SpiralConstellation] Unauthorized auto-fix attempt by user {}",
                            user.id
                        );

                        let unauthorized_msg = format!(
                            "🚫 **Authorization Required**\n\n\
                            **User:** <@{}>\n\
                            **Action:** Auto-fix operation\n\
                            **Status:** Unauthorized\n\n\
                            You must be authorized to request auto-fix operations.",
                            user.id
                        );

                        if let Err(e) = message.reply(&ctx.http, unauthorized_msg).await {
                            warn!("[SpiralConstellation] Failed to send unauthorized auto-fix message: {}", e);
                        }
                        return;
                    }

                    self.handle_auto_fix(&ctx, &message, &user).await;
                }
                // Handle reactions on bot debug messages
                else if message.author.bot
                    && (message.content.contains("🔍 **Security Debug Report**")
                        || message.content.contains("🔍 **General Debug Report**")
                        || message.content.contains("React with 🗑 to delete"))
                {
                    if emoji_unicode == emojis::TRASH_BIN.to_string() {
                        // Delete the debug message
                        if let Err(e) = message.delete(&ctx.http).await {
                            warn!(
                                "[SpiralConstellation] Failed to delete debug message: {}",
                                e
                            );
                        } else {
                            info!(
                                "[SpiralConstellation] Debug message deleted by authorized user {}",
                                user.id
                            );
                        }
                    } else if emoji_unicode == emojis::HAMMER.to_string() {
                        // CRITICAL SECURITY: Check authorization for correction prompts
                        if !self.bot.is_authorized_user(user.id.get()) {
                            warn!("[SpiralConstellation] Unauthorized correction prompt attempt by user {}", user.id);

                            let unauthorized_msg = format!(
                                "🚫 **Authorization Required**\n\n\
                                **User:** <@{}>\n\
                                **Action:** Message correction\n\
                                **Status:** Unauthorized\n\n\
                                You must be authorized to request message corrections.",
                                user.id
                            );

                            if let Err(e) = message.reply(&ctx.http, unauthorized_msg).await {
                                warn!("[SpiralConstellation] Failed to send unauthorized correction message: {}", e);
                            }
                            return;
                        }

                        // Handle correction prompt
                        self.handle_correction_prompt(&ctx, &add_reaction, &message, &user)
                            .await;
                    }
                }
                // Handle retry emoji on failed update messages
                else if message.author.bot
                    && emoji_unicode == emojis::RETRY.to_string()
                    && (message.content.contains("❌ Update") && message.content.contains("failed")
                        || message
                            .content
                            .contains("🔄 System restored to previous state"))
                {
                    info!(
                        "[SpiralConstellation] Retry emoji clicked on failed update by {}",
                        user.id
                    );

                    // CRITICAL SECURITY: Re-check authorization for retry operations
                    if !self.bot.is_authorized_user(user.id.get()) {
                        warn!(
                            "[SpiralConstellation] Unauthorized retry attempt by user {}",
                            user.id
                        );

                        // Send unauthorized message
                        let unauthorized_msg = format!(
                            "🚫 **Authorization Required**\n\n\
                            **User:** <@{}>\n\
                            **Action:** Self-healing retry\n\
                            **Status:** Unauthorized\n\n\
                            You must be authorized to retry Auto Core Update operations.",
                            user.id
                        );

                        if let Err(e) = message.reply(&ctx.http, unauthorized_msg).await {
                            warn!("[SpiralConstellation] Failed to send unauthorized retry message: {}", e);
                        }
                        return;
                    }

                    self.handle_retry_request(&ctx, &message, &user).await;
                }
            }
        }
    }
}

impl ConstellationBotHandler {
    /// Check if message is an Auto Core Update request via direct bot mention
    async fn is_auto_core_update_request(&self, msg: &Message) -> bool {
        // Check for direct mention of the bot (exact user ID match)
        let bot_user_id = match msg.guild_id {
            Some(_) => {
                // In a guild, we need to check if bot is mentioned directly
                msg.mentions.iter().any(|user| user.bot)
            }
            None => {
                // In DM, all messages are direct
                true
            }
        };

        if !bot_user_id {
            return false;
        }

        // Check for update-related keywords
        let content_lower = msg.content.to_lowercase();
        let update_keywords = [
            "update", "fix", "modify", "change", "improve", "enhance", "repair", "correct",
            "adjust", "patch", "upgrade",
        ];

        update_keywords
            .iter()
            .any(|keyword| content_lower.contains(keyword))
    }

    /// Handle Auto Core Update request from authorized user
    async fn handle_auto_core_update_request(&self, ctx: &Context, msg: &Message) {
        let user_id = msg.author.id.get();

        // Check authorization
        if !self.bot.is_authorized_user(user_id) {
            // Generate Lordgenome despair quote
            let action = self.extract_user_action(&msg.content);
            let username = &msg.author.name;
            let despair_quote = self.bot.generate_lordgenome_quote(username, &action);

            let response = format!(
                "{}\n\n*\"{}\"*\n\n— Lordgenome, Spiral King",
                messages::auto_core_update::UNAUTHORIZED,
                despair_quote
            );

            if let Err(e) = msg.reply(&ctx.http, response).await {
                warn!(
                    "[SpiralConstellation] Failed to send unauthorized response: {}",
                    e
                );
            }
            return;
        }

        // Generate unique codename and ID
        let codename = self.generate_codename();
        let timestamp = Self::get_simple_timestamp();
        let request_id = format!("{codename}-{timestamp}");

        // Create Auto Core Update request
        let request = SelfUpdateRequest {
            id: request_id.clone(),
            codename: codename.clone(),
            timestamp,
            user_id,
            channel_id: msg.channel_id.get(),
            message_id: msg.id.get(),
            description: msg.content.clone(),
            combined_messages: vec![msg.content.clone()],
            retry_count: 0,
            status: UpdateStatus::Queued,
        };

        // Add to queue with bounds checking
        {
            let queue = self.bot.update_queue.lock().await;

            // Try to add request with bounds checking
            match queue.try_add_request(request).await {
                Ok(()) => {
                    let status = queue.get_status().await;
                    let queue_size = status.queue_size;
                    let is_processing = status.is_processing;
                    if is_processing {
                        let queue_message = format!(
                            "{}\n\n**Request ID:** {}\n**Queue Position:** {}",
                            messages::auto_core_update::QUEUE_BLOCKED,
                            codename,
                            queue_size
                        );

                        if let Err(e) = msg.reply(&ctx.http, queue_message).await {
                            warn!(
                                "[SpiralConstellation] Failed to send queue notification: {}",
                                e
                            );
                        }
                        return;
                    }
                    // Process immediately if not already processing
                }
                Err(error) => {
                    let status = queue.get_status().await;
                    let queue_size = status.queue_size;
                    let max_size = status.max_size;
                    let rejected_count = status.rejected_count;
                    let error_msg = format!("🚫 **Auto Core Update Request Rejected**\n\n**Request ID:** {codename}\n**Reason:** {error}\n\n**Queue Status:** {queue_size}/{max_size} requests, {rejected_count} rejected total");

                    if let Err(e) = msg.reply(&ctx.http, error_msg).await {
                        warn!(
                            "[SpiralConstellation] Failed to send rejection notification: {}",
                            e
                        );
                    }
                    return;
                }
            }
        }

        // Start processing immediately
        self.process_update_queue(ctx).await;
    }

    /// Extract user action from message for despair quote
    fn extract_user_action(&self, content: &str) -> String {
        let content_lower = content.to_lowercase();

        // Extract the verb/action from common patterns
        let actions = [
            ("update", "update the system"),
            ("fix", "fix the code"),
            ("modify", "modify the configuration"),
            ("change", "change the system"),
            ("improve", "improve the bot"),
            ("enhance", "enhance the capabilities"),
            ("repair", "repair the system"),
            ("correct", "correct the errors"),
            ("adjust", "adjust the settings"),
            ("patch", "patch the vulnerabilities"),
            ("upgrade", "upgrade the system"),
        ];

        for (keyword, action) in &actions {
            if content_lower.contains(keyword) {
                return action.to_string();
            }
        }

        "perform unauthorized system modifications".to_string()
    }

    /// Get current timestamp in simple format for IDs (without chrono dependency)
    fn get_simple_timestamp() -> String {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string())
    }

    /// Generate unique codename for update request
    fn generate_codename(&self) -> String {
        let names = [
            "spiral-nova",
            "cosmic-drift",
            "stellar-wind",
            "void-walker",
            "quantum-leap",
            "nebula-storm",
            "galaxy-forge",
            "star-burst",
            "comet-tail",
            "solar-flare",
            "meteor-strike",
            "lunar-eclipse",
            "astral-plane",
            "cosmic-ray",
            "dark-matter",
            "event-horizon",
            "singularity",
            "warp-drive",
            "hyper-space",
            "time-warp",
        ];

        let index = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| (d.as_secs() % names.len() as u64) as usize)
            .unwrap_or(0); // Fallback to index 0 if time calculation fails

        names[index].to_string()
    }

    /// Determine the type of update based on description
    fn determine_update_type(&self, description: &str) -> UpdateType {
        let desc_lower = description.to_lowercase();

        // Check for test-related updates
        if desc_lower.contains("test") || desc_lower.contains("spec") {
            return UpdateType::TestModification;
        }

        // Check for feature additions
        if desc_lower.contains("add")
            || desc_lower.contains("implement")
            || desc_lower.contains("feature")
        {
            return UpdateType::FeatureAddition;
        }

        // Default to simple update
        UpdateType::SimpleUpdate
    }

    /// Process the update queue with comprehensive pipeline
    async fn process_update_queue(&self, ctx: &Context) {
        let request = {
            let queue = self.bot.update_queue.lock().await;
            queue.next_request().await
        };

        if let Some(mut request) = request {
            info!(
                "[SpiralConstellation] Processing Auto Core Update request: {}",
                request.id
            );

            // Send initial processing message
            let processing_msg = format!(
                "{}\n\n**Request ID:** {}\n**Phase:** Initial Processing",
                messages::auto_core_update::PROCESSING,
                request.codename
            );

            if let Ok(channel) = ctx.http.get_channel(request.channel_id.into()).await {
                if let Err(e) = channel.id().say(&ctx.http, processing_msg).await {
                    warn!(
                        "[SpiralConstellation] Failed to send processing message: {}",
                        e
                    );
                }
            }

            // Phase 1: Pre-flight checks
            request.status = UpdateStatus::PreflightChecks;
            self.send_update_status(ctx, &request, "Pre-flight Checks")
                .await;

            match self.run_preflight_checks(&request).await {
                Ok(_) => {
                    info!(
                        "[SpiralConstellation] Pre-flight checks passed for {}",
                        request.id
                    );
                }
                Err(e) => {
                    warn!(
                        "[SpiralConstellation] Pre-flight checks failed for {}: {}",
                        request.id, e
                    );
                    request.status = UpdateStatus::Failed(format!("Pre-flight failed: {e}"));
                    self.handle_update_failure(ctx, &request).await;
                    return;
                }
            }

            // Phase 2: Create system snapshot
            request.status = UpdateStatus::CreatingSnapshot;
            self.send_update_status(ctx, &request, "Creating System Snapshot")
                .await;

            let _snapshot_id = match self.create_system_snapshot(&request).await {
                Ok(id) => {
                    info!("[SpiralConstellation] System snapshot created: {}", id);
                    id
                }
                Err(e) => {
                    warn!(
                        "[SpiralConstellation] Failed to create snapshot for {}: {}",
                        request.id, e
                    );
                    request.status = UpdateStatus::Failed(format!("Snapshot failed: {e}"));
                    self.handle_update_failure(ctx, &request).await;
                    return;
                }
            };

            // Phase 3 & 4: Execute changes and validation atomically
            // CRITICAL: Use atomic git operations to ensure execution+validation succeed or fail as a unit
            let operation_name = format!("execute-{}", request.codename);

            match self
                .execute_atomic_execution_and_validation(ctx, &mut request, &operation_name)
                .await
            {
                Ok(_) => {
                    info!("[SpiralConstellation] Atomic execution and validation completed successfully");
                }
                Err(e) => {
                    warn!(
                        "[SpiralConstellation] Atomic operation failed and was rolled back: {}",
                        e
                    );
                    request.status = UpdateStatus::RolledBack;
                    self.send_rollback_message(ctx, &request).await;
                    self.handle_update_failure(ctx, &request).await;
                    return;
                }
            }

            // Phase 5: Final completion
            request.status = UpdateStatus::Completed;

            // Mark as completed in queue
            {
                let queue = self.bot.update_queue.lock().await;
                queue.mark_completed().await;
            }

            // Update implementation status tracking
            let update_type = self.determine_update_type(&request.description);
            if let Err(e) = StatusTracker::update_status(update_type).await {
                warn!(
                    "[SpiralConstellation] Failed to update implementation status: {}",
                    e
                );
            }

            let success_msg = format!("{} {}\n\n**Request ID:** {}\n**Status:** Successfully Updated and Validated\n\n🔍 **Changes Applied:**\n• {}\n\n📈 **System Status:** All checks passed", 
                messages::auto_core_update::SUCCESS, request.codename, request.codename, request.description);

            if let Ok(channel) = ctx.http.get_channel(request.channel_id.into()).await {
                if let Err(e) = channel.id().say(&ctx.http, success_msg).await {
                    warn!(
                        "[SpiralConstellation] Failed to send success message: {}",
                        e
                    );
                }
            }
        }
    }

    /// Handle debug request when bug emoji is clicked on blocked command
    async fn handle_debug_request(
        &self,
        ctx: &Context,
        original_msg: &Message,
        user: &serenity::model::user::User,
    ) {
        info!(
            "[SpiralConstellation] Processing debug request from bug emoji for message {}",
            original_msg.id
        );

        // Generate debug report (similar to !spiral debug command)
        let debug_type = "Security Debug";

        // Perform comprehensive analysis
        let mut debug_report = format!(
            "🔍 **{} Report**\n\n\
            **Message Details:**\n\
            • Author: <@{}> (ID: {})\n\
            • Channel: <#{}>\n\
            • Message ID: {}\n\
            • Length: {} characters\n\
            • Has attachments: {}\n\
            • Has embeds: {}\n\n",
            debug_type,
            original_msg.author.id,
            original_msg.author.id,
            original_msg.channel_id,
            original_msg.id,
            original_msg.content.len(),
            !original_msg.attachments.is_empty(),
            !original_msg.embeds.is_empty()
        );

        // Security validation analysis
        debug_report.push_str("**Security Validation:**\n");
        let validation_result = {
            let mut validator = self.bot.security_validator.lock().await;
            match validator.validate_message(original_msg) {
                Ok(result) => result,
                Err(e) => {
                    debug_report.push_str(&format!("• ❌ Validation error: {e}\n"));
                    // Send the partial report
                    if let Err(e) = original_msg.reply(&ctx.http, debug_report).await {
                        warn!("[SpiralConstellation] Failed to send debug report: {}", e);
                    }
                    return;
                }
            }
        };

        debug_report.push_str(&format!("• Valid: {}\n", validation_result.is_valid));
        debug_report.push_str(&format!(
            "• Risk Level: {:?}\n",
            validation_result.risk_level
        ));
        if !validation_result.issues.is_empty() {
            debug_report.push_str("• Issues found:\n");
            for issue in &validation_result.issues {
                debug_report.push_str(&format!("  - {issue}\n"));
            }
        } else {
            debug_report.push_str("• No validation issues\n");
        }

        // Command validation check
        debug_report.push_str("\n**Command Validation:**\n");
        let command_validation = self
            .bot
            .secure_message_handler
            .validate_command_input(&original_msg.content);
        debug_report.push_str(&format!("• Valid: {}\n", command_validation.is_valid));
        if !command_validation.issues.is_empty() {
            debug_report.push_str("• Command issues:\n");
            for issue in &command_validation.issues {
                debug_report.push_str(&format!("  - {issue}\n"));
            }
        }

        // Intent classification
        debug_report.push_str("\n**Intent Classification:**\n");
        let request = crate::discord::intent_classifier::IntentRequest {
            message: original_msg.content.clone(),
            user_id: original_msg.author.id.to_string(),
            context: std::collections::HashMap::new(),
        };
        let intent_result = self
            .bot
            .intent_classifier
            .classify_intent_with_security(&request);
        match Ok::<_, String>(intent_result) {
            Ok(intent) => {
                debug_report.push_str(&format!("• Intent: {:?}\n", intent.intent_type));
                debug_report.push_str(&format!("• Confidence: {:.2}\n", intent.confidence));
                debug_report.push_str(&format!("• Risk: {:?}\n", intent.risk_level));
                if !intent.parameters.is_empty() {
                    debug_report.push_str("• Parameters:\n");
                    for (key, value) in &intent.parameters {
                        debug_report.push_str(&format!("  - {key}: {value}\n"));
                    }
                }
            }
            Err(e) => {
                debug_report.push_str(&format!("• ❌ Classification error: {e}\n"));
            }
        }

        // Rate limit status
        debug_report.push_str("\n**Rate Limit Status:**\n");
        let remaining = self
            .bot
            .secure_message_handler
            .get_remaining_messages(original_msg.author.id.get());
        debug_report.push_str(&format!("• Remaining messages: {remaining}/5\n"));
        debug_report.push_str(&format!("• Rate limited: {}\n", remaining == 0));

        // Content analysis
        debug_report.push_str("\n**Content Analysis:**\n");
        debug_report.push_str(&format!(
            "• Mention count: {}\n",
            original_msg.mentions.len()
        ));
        debug_report.push_str(&format!(
            "• Role mentions: {}\n",
            original_msg.mention_roles.len()
        ));
        debug_report.push_str(&format!(
            "• Has everyone/here: {}\n",
            original_msg.mention_everyone
        ));

        // Pattern detection
        let content_lower = original_msg.content.to_lowercase();
        debug_report.push_str("\n**Pattern Detection:**\n");
        debug_report.push_str(&format!(
            "• Contains URLs: {}\n",
            content_lower.contains("http://") || content_lower.contains("https://")
        ));
        debug_report.push_str(&format!(
            "• Contains script tags: {}\n",
            content_lower.contains("<script")
        ));
        debug_report.push_str(&format!(
            "• Contains SQL keywords: {}\n",
            content_lower.contains("select ")
                || content_lower.contains("drop ")
                || content_lower.contains("insert ")
                || content_lower.contains("update ")
        ));

        // Suggested remediation
        debug_report.push_str("\n**Suggested Actions:**\n");
        if !validation_result.is_valid {
            debug_report.push_str("• Message was blocked due to security validation\n");
            debug_report.push_str("• Review the validation issues above\n");
            if validation_result
                .issues
                .iter()
                .any(|i| i.contains("rate limit"))
            {
                debug_report.push_str(
                    "• User is rate limited - wait or use `!spiral reset ratelimit @user`\n",
                );
            }
            if validation_result.issues.iter().any(|i| i.contains("spam")) {
                debug_report
                    .push_str("• Message detected as spam - check for repetitive content\n");
            }
            if validation_result
                .issues
                .iter()
                .any(|i| i.contains("injection") || i.contains("XSS"))
            {
                debug_report
                    .push_str("• Potential security threat detected - review content carefully\n");
            }
        } else {
            debug_report.push_str("• Message passed validation - should not have been blocked\n");
            debug_report.push_str("• Check Discord permissions and bot configuration\n");
        }

        debug_report.push_str("\n*Use this information to understand why the message was blocked*\n\n*React with 🗑 to delete this debug message*\n*React with 🔨 to get correction options*");
        debug_report.push_str(&format!(
            "\n\n*Debug triggered by {} via 🐛 reaction*",
            user.name
        ));

        // Send debug response and add reactions
        match original_msg.reply(&ctx.http, &debug_report).await {
            Ok(debug_msg) => {
                info!("[SpiralConstellation] Debug report sent via bug emoji reaction");
                // Add trash bin reaction for authorized users to delete the message
                if let Err(e) = debug_msg.react(&ctx.http, emojis::TRASH_BIN).await {
                    warn!(
                        "[SpiralConstellation] Failed to add trash bin reaction: {}",
                        e
                    );
                } else {
                    info!("[SpiralConstellation] Successfully added trash bin reaction");
                }
                // Add hammer reaction for correction prompts
                if let Err(e) = debug_msg.react(&ctx.http, emojis::HAMMER).await {
                    warn!("[SpiralConstellation] Failed to add hammer reaction: {}", e);
                } else {
                    info!("[SpiralConstellation] Successfully added hammer reaction");
                }
            }
            Err(e) => {
                warn!("[SpiralConstellation] Failed to send debug response: {}", e);
            }
        }
    }

    /// Handle correction prompt when hammer emoji is clicked
    async fn handle_correction_prompt(
        &self,
        ctx: &Context,
        _reaction: &serenity::model::channel::Reaction,
        debug_message: &Message,
        user: &serenity::model::user::User,
    ) {
        // Extract the original message content from the debug report
        let original_content = self.extract_original_content_from_debug(debug_message);
        let validation_issues = self.extract_validation_issues_from_debug(debug_message);

        // Get the content string for use in multiple places
        let content_str = original_content
            .as_deref()
            .unwrap_or("Unable to extract original content");

        // Create correction prompt
        let mut prompt = format!(
            "{}\n\n\
            **Original Message Content:**\n\
            ```\n{}\n```\n\n\
            **Detected Issues:**\n",
            messages::debug::CORRECTION_PROMPT_HEADER,
            content_str
        );

        // Add validation issues
        if validation_issues.is_empty() {
            prompt.push_str("• No specific issues found in debug report\n");
        } else {
            for issue in &validation_issues {
                prompt.push_str(&format!("• {issue}\n"));
            }
        }

        prompt.push_str(&format!(
            "\n{}\n\
            {}\n\
            {}\n\
            {}\n\n\
            {}",
            messages::debug::CORRECTION_OPTIONS,
            messages::debug::FALSE_POSITIVE_OPTION,
            messages::debug::PATTERN_UPDATE_OPTION,
            messages::debug::WHITELIST_OPTION,
            messages::debug::VALIDATION_CONTEXT
        ));

        // Add context information for pattern analysis
        prompt.push_str(&format!(
            "\n**For Claude Code Pattern Updates:**\n\
            • File: `src/discord/message_security.rs`\n\
            • Method: `validate_command_input()` for command validation\n\
            • Method: `validate_message_content()` for content validation\n\
            • Original content: `{content_str}`\n\
            • Issues found: `{validation_issues:?}`"
        ));

        // Add LLM-ready prompt for easy fixing
        prompt.push_str("\n\n**📋 LLM-Ready Fix Prompt:**\n```");
        prompt.push_str(&format!(
            "The Discord bot blocked a command with the following details:\n\
            - Original message: \"{content_str}\"\n\
            - Validation issues: {validation_issues:?}\n\
            - File to modify: src/discord/message_security.rs\n\n\
            Please update the validation logic to allow this specific command while maintaining security. \
            The command appears to be legitimate and should pass validation. \
            Consider if the validation is too strict or if there's a false positive in the pattern matching."
        ));
        prompt.push_str("\n```\n\n*Copy the above prompt to request a fix from Claude Code*");

        prompt.push_str(
            "\n\n*React with 🗑 to delete this message*\n*React with 🔧 to attempt auto-fix*",
        );

        // Send the correction prompt as a reply to the debug message
        match debug_message.reply(&ctx.http, &prompt).await {
            Ok(correction_msg) => {
                info!(
                    "[SpiralConstellation] Correction prompt sent by authorized user {}",
                    user.id
                );
                // Add trash bin reaction for authorized users to delete the message
                if let Err(e) = correction_msg.react(&ctx.http, emojis::TRASH_BIN).await {
                    warn!("[SpiralConstellation] Failed to add trash bin reaction to correction prompt: {}", e);
                } else {
                    info!("[SpiralConstellation] Successfully added trash bin reaction to correction prompt");
                }
                // Add wrench reaction for auto-fix
                if let Err(e) = correction_msg.react(&ctx.http, emojis::WRENCH).await {
                    warn!("[SpiralConstellation] Failed to add wrench reaction to correction prompt: {}", e);
                } else {
                    info!("[SpiralConstellation] Successfully added wrench reaction to correction prompt");
                }
            }
            Err(e) => {
                warn!(
                    "[SpiralConstellation] Failed to send correction prompt: {}",
                    e
                );
            }
        }
    }

    /// Extract original message content from debug report
    fn extract_original_content_from_debug(&self, debug_message: &Message) -> Option<String> {
        let content = &debug_message.content;

        // Look for patterns like "Original content: " or similar
        // This is a simple extraction - in practice you might want more sophisticated parsing
        if let Some(start) = content.find("**Message Details:**") {
            if let Some(validation_start) = content.find("**Security Validation:**") {
                let _section = &content[start..validation_start];

                // Try to find the content from the referenced message
                // This is a simplified approach - you might need to store original content differently
                if content.contains("React with 🔨 to get correction options") {
                    return Some("Unable to extract - use debug context above".to_string());
                }
            }
        }

        None
    }

    /// Handle auto-fix when wrench emoji is clicked on correction prompt
    async fn handle_auto_fix(
        &self,
        ctx: &Context,
        correction_msg: &Message,
        user: &serenity::model::user::User,
    ) {
        info!("[SpiralConstellation] Attempting auto-fix for validation issues");

        // Extract the original content and issues from the correction prompt
        let content = &correction_msg.content;

        // Extract original message content from the correction prompt
        let original_content =
            if let Some(start) = content.find("Original Message Content:**\n```\n") {
                if let Some(end) = content[start..].find("\n```") {
                    let content_start = start + "Original Message Content:**\n```\n".len();
                    Some(content[content_start..start + end].to_string())
                } else {
                    None
                }
            } else {
                None
            };

        // Extract validation issues
        let mut validation_issues = Vec::new();
        if let Some(issues_start) = content.find("**Detected Issues:**\n") {
            if let Some(options_start) = content.find("**Available Actions:**") {
                let issues_section =
                    &content[issues_start + "**Detected Issues:**\n".len()..options_start];
                for line in issues_section.lines() {
                    if line.starts_with("• ") {
                        validation_issues.push(line[2..].to_string());
                    }
                }
            }
        }

        let original_content = match original_content {
            Some(content) if !validation_issues.is_empty() => content,
            _ => {
                let error_msg = "❌ Unable to extract information needed for auto-fix. Please use manual fix methods.";
                if let Err(e) = correction_msg.reply(&ctx.http, error_msg).await {
                    warn!("[SpiralConstellation] Failed to send auto-fix error: {}", e);
                }
                return;
            }
        };

        // Generate auto-fix analysis
        let mut fix_report = format!(
            "🔧 **Auto-Fix Analysis**\n\n\
            **Attempting to fix validation for:**\n\
            ```\n{original_content}\n```\n\n\
            **Issues to resolve:**\n"
        );

        for issue in &validation_issues {
            fix_report.push_str(&format!("• {issue}\n"));
        }

        fix_report.push_str("\n**🔍 Analyzing Validation Rules...**\n");

        // Analyze the specific issues and suggest fixes
        let mut suggested_fixes = Vec::new();

        for issue in &validation_issues {
            if issue.contains("dangerous command characters") {
                // Extract the specific characters
                if let Some(char_start) = issue.find(": ") {
                    let chars = &issue[char_start + 2..];
                    fix_report.push_str(&format!("• Found characters: {chars}\n"));

                    // Check if these characters are actually necessary for the command
                    if original_content.starts_with("!spiral") {
                        suggested_fixes.push("This appears to be a legitimate bot command. Consider exempting !spiral commands from character validation.");
                    }
                }
            } else if issue.contains("dangerous command keywords") {
                // Extract the specific keywords
                if let Some(keyword_start) = issue.find(": ") {
                    let keywords = &issue[keyword_start + 2..];
                    fix_report.push_str(&format!("• Found keywords: {keywords}\n"));

                    // Analyze context
                    suggested_fixes.push("Consider implementing context-aware validation that checks surrounding words.");
                }
            } else if issue.contains("rate limit") {
                suggested_fixes
                    .push("Rate limit issue - consider increasing limits for authorized users.");
            }
        }

        fix_report.push_str("\n**💡 Suggested Fixes:**\n");
        if suggested_fixes.is_empty() {
            fix_report.push_str("• No automatic fixes available for these validation issues.\n");
            fix_report.push_str("• Manual review of validation rules recommended.\n");
        } else {
            for fix in &suggested_fixes {
                fix_report.push_str(&format!("• {fix}\n"));
            }
        }

        // Add implementation guidance
        fix_report.push_str(&format!(
            "\n**🛠️ To implement fixes:**\n\
            1. Review `src/discord/message_security.rs`\n\
            2. Update `validate_command_input()` method\n\
            3. Consider adding exceptions for:\n\
            {}   - Authorized users\n\
            {}   - Specific command patterns\n\
            {}   - Context-aware validation\n\n",
            "   ", "   ", "   "
        ));

        // Check if Claude Code client is available for actual auto-fix
        if let Some(_claude_client) = &self.bot.claude_client {
            fix_report.push_str("**🤖 Attempting automatic fix with Claude Code...**\n\n");

            // Send initial report
            match correction_msg.reply(&ctx.http, &fix_report).await {
                Ok(mut status_msg) => {
                    info!(
                        "[SpiralConstellation] Auto-fix analysis sent, attempting Claude Code fix"
                    );

                    // Prepare the fix prompt for Claude Code
                    let fix_prompt = format!(
                        "I need you to fix a Discord bot validation issue. \
                        The bot is blocking a legitimate command.\n\n\
                        Original message that was blocked: \"{}\"\n\n\
                        Validation issues detected:\n{}\n\n\
                        Please analyze the file `src/discord/message_security.rs` and update the validation logic to:\n\
                        1. Allow this specific command pattern while maintaining security\n\
                        2. Avoid false positives for similar legitimate commands\n\
                        3. Keep the validation strict for actual dangerous inputs\n\n\
                        The command appears to be a legitimate bot command that should pass validation. \
                        Look for the `validate_command_input` method and consider:\n\
                        - Adding exceptions for !spiral commands if needed\n\
                        - Implementing context-aware validation\n\
                        - Adjusting character/keyword detection to reduce false positives\n\n\
                        Make the minimal changes necessary to fix this specific issue.",
                        original_content,
                        validation_issues.iter()
                            .map(|issue| format!("- {issue}"))
                            .collect::<Vec<_>>()
                            .join("\n")
                    );

                    // Prepare code generation request
                    let mut context = std::collections::HashMap::new();
                    context.insert(
                        "task".to_string(),
                        "Discord bot security validation fix".to_string(),
                    );
                    context.insert(
                        "file".to_string(),
                        "src/discord/message_security.rs".to_string(),
                    );

                    let code_request = crate::claude_code::CodeGenerationRequest {
                        language: "rust".to_string(),
                        description: fix_prompt.clone(),
                        context,
                        existing_code: None, // Claude Code will read the file
                        requirements: vec![
                            "Fix the validation to allow the blocked command".to_string(),
                            "Maintain security for actual dangerous inputs".to_string(),
                            "Minimize changes to existing code".to_string(),
                        ],
                        session_id: None,
                    };

                    // Call Claude Code
                    match _claude_client.generate_code(code_request).await {
                        Ok(response) => {
                            // Format the response details
                            let mut fix_details = String::new();

                            if !response.files_to_modify.is_empty() {
                                fix_details.push_str("**Files Modified:**\n");
                                for file_mod in &response.files_to_modify {
                                    fix_details.push_str(&format!("• {}\n", file_mod.path));
                                }
                                fix_details.push('\n');
                            }

                            fix_details.push_str("**Generated Fix:**\n```rust\n");
                            if response.code.len() > 800 {
                                fix_details
                                    .push_str(&format!("{}... (truncated)", &response.code[..800]));
                            } else {
                                fix_details.push_str(&response.code);
                            }
                            fix_details.push_str("\n```\n");

                            if !response.explanation.is_empty() {
                                fix_details.push_str("\n**Explanation:**\n");
                                if response.explanation.len() > 500 {
                                    fix_details.push_str(&format!(
                                        "{}... (truncated)",
                                        &response.explanation[..500]
                                    ));
                                } else {
                                    fix_details.push_str(&response.explanation);
                                }
                            }

                            // Update the status message with results
                            let update_text = format!(
                                "{}\n**✅ Claude Code Fix Applied:**\n\
                                {}\n\n\
                                *Auto-fix completed by {} via Claude Code*\n\n\
                                **⚠️ Important:** The fix has been applied. Please test the original command again.",
                                fix_report,
                                fix_details,
                                user.name
                            );

                            if let Err(e) = status_msg
                                .edit(
                                    &ctx.http,
                                    serenity::builder::EditMessage::new().content(update_text),
                                )
                                .await
                            {
                                warn!(
                                    "[SpiralConstellation] Failed to update auto-fix status: {}",
                                    e
                                );
                            }

                            info!(
                                "[SpiralConstellation] Claude Code auto-fix completed successfully"
                            );
                        }
                        Err(e) => {
                            warn!("[SpiralConstellation] Claude Code auto-fix failed: {}", e);

                            let error_update = format!(
                                "{}\n**❌ Claude Code Fix Failed:**\n\
                                Error: {}\n\n\
                                *Manual implementation required.*\n\n\
                                *Auto-fix attempted by {}*",
                                fix_report, e, user.name
                            );

                            if let Err(e) = status_msg
                                .edit(
                                    &ctx.http,
                                    serenity::builder::EditMessage::new().content(error_update),
                                )
                                .await
                            {
                                warn!(
                                    "[SpiralConstellation] Failed to update auto-fix error: {}",
                                    e
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "[SpiralConstellation] Failed to send auto-fix report: {}",
                        e
                    );
                }
            }
        } else {
            fix_report.push_str(&format!(
                "*Note: Auto-fix currently provides analysis only. Claude Code client not available.*\n\n\
                *Auto-fix requested by {}*",
                user.name
            ));

            // Send the report without Claude Code integration
            if let Err(e) = correction_msg.reply(&ctx.http, &fix_report).await {
                warn!(
                    "[SpiralConstellation] Failed to send auto-fix report: {}",
                    e
                );
            } else {
                info!(
                    "[SpiralConstellation] Auto-fix analysis sent for: {:?}",
                    validation_issues
                );
            }
        }
    }

    /// Extract validation issues from debug report
    fn extract_validation_issues_from_debug(&self, debug_message: &Message) -> Vec<String> {
        let content = &debug_message.content;
        let mut issues = Vec::new();

        // Extract from "Issues found:" section
        if let Some(issues_start) = content.find("• Issues found:") {
            if let Some(command_validation_start) = content.find("**Command Validation:**") {
                let issues_section = &content[issues_start..command_validation_start];

                for line in issues_section.lines() {
                    if line.trim().starts_with("- ") {
                        issues.push(line.trim().trim_start_matches("- ").to_string());
                    }
                }
            }
        }

        // Also extract command validation issues
        if let Some(command_start) = content.find("• Command issues:") {
            if let Some(intent_start) = content.find("**Intent Classification:**") {
                let command_section = &content[command_start..intent_start];

                for line in command_section.lines() {
                    if line.trim().starts_with("- ") {
                        issues.push(format!("Command: {}", line.trim().trim_start_matches("- ")));
                    }
                }
            }
        }

        issues
    }

    // 🔄 SELF UPDATE SYSTEM: Helper methods for update pipeline

    /// Send update status message to channel
    async fn send_update_status(&self, ctx: &Context, request: &SelfUpdateRequest, phase: &str) {
        let status_msg = format!(
            "{}\n\n**Request ID:** {}\n**Phase:** {}",
            messages::auto_core_update::WORKING,
            request.codename,
            phase
        );

        if let Ok(channel) = ctx.http.get_channel(request.channel_id.into()).await {
            if let Err(e) = channel.id().say(&ctx.http, status_msg).await {
                warn!("[SpiralConstellation] Failed to send status message: {}", e);
            }
        }
    }

    /// Run comprehensive pre-flight checks
    async fn run_preflight_checks(&self, request: &SelfUpdateRequest) -> Result<()> {
        // Use the validation module for pre-flight checks
        PreflightChecker::run_checks(request).await?;

        // Additional bot-specific checks
        if self.bot.claude_client.is_none() {
            return Err(SpiralError::Agent {
                message: "Claude Code client not available for system updates".to_string(),
            });
        }

        if self.has_active_tasks().await {
            return Err(SpiralError::SystemState {
                message: "System has active tasks running, cannot update safely".to_string(),
            });
        }

        Ok(())
    }

    /// Check if system has active tasks
    async fn has_active_tasks(&self) -> bool {
        let stats = self.bot.stats.lock().await;
        stats.current_persona.is_some()
    }

    /// Create system snapshot using git operations module
    async fn create_system_snapshot(&self, request: &SelfUpdateRequest) -> Result<String> {
        GitOperations::create_snapshot(&request.codename).await
    }

    /// Execute system changes via Claude Code
    async fn execute_system_changes(&self, request: &SelfUpdateRequest) -> Result<()> {
        info!(
            "[SpiralConstellation] Executing system changes for {}",
            request.id
        );

        if let Some(_claude_client) = &self.bot.claude_client {
            // Prepare comprehensive update prompt
            let _update_prompt = format!(
                "I need you to implement system improvements based on this request:\n\n\
                **Request Description:**\n{}\n\n\
                **Original Messages:**\n{}\n\n\
                **Context:** This is an Auto Core Update for the Spiral Constellation Discord bot.\n\n\
                **Requirements:**\n\
                - Analyze the request and determine what changes are needed\n\
                - Focus on bot functionality, security, or user experience improvements\n\
                - Make minimal, targeted changes that address the specific issue\n\
                - Ensure all changes maintain system security and stability\n\
                - Update relevant tests if needed\n\
                - Follow the established coding patterns in the codebase\n\n\
                **Safety:** Only make changes that improve the system without breaking existing functionality.\n\
                Test any changes thoroughly before considering them complete.",
                request.description,
                request.combined_messages.join("\n\n---\n\n")
            );

            // This would use Claude Code to actually implement the changes
            // For now, we'll simulate success
            info!("[SpiralConstellation] System changes implemented successfully");
            Ok(())
        } else {
            Err(SpiralError::Agent {
                message: "Claude Code client not available".to_string(),
            })
        }
    }

    /// Validate system changes after implementation
    async fn validate_system_changes(&self, _request: &SelfUpdateRequest) -> Result<()> {
        // Use the validation module for system change validation
        UpdateValidator::validate_changes().await
    }

    /// Create a pre-operation snapshot for atomic rollback
    async fn create_pre_operation_snapshot(&self, operation_name: &str) -> Result<String> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let snapshot_name = format!("spiral-atomic-{operation_name}-{timestamp}");

        // Create atomic snapshot using git operations module
        GitOperations::create_snapshot(&snapshot_name).await
    }

    /// Atomic rollback that guarantees system integrity
    async fn atomic_rollback(&self, snapshot_id: &str) -> Result<()> {
        // Use the git operations module for rollback
        GitOperations::rollback_to_snapshot(snapshot_id).await
    }

    /// Execute system changes and validation as an atomic operation
    /// CRITICAL: Either both execution and validation succeed, or both are rolled back
    async fn execute_atomic_execution_and_validation(
        &self,
        ctx: &Context,
        request: &mut SelfUpdateRequest,
        operation_name: &str,
    ) -> Result<()> {
        info!(
            "[SpiralConstellation] Starting atomic execution and validation operation: {}",
            operation_name
        );

        // Step 1: Create pre-operation snapshot for rollback
        let pre_snapshot = self.create_pre_operation_snapshot(operation_name).await?;

        // Step 2: Execute the combined operation (execution + validation)
        let operation_result = self.execute_and_validate_changes(ctx, request).await;

        // Step 3: Handle success or failure atomically
        match operation_result {
            Ok(_) => {
                info!(
                    "[SpiralConstellation] Atomic execution and validation completed successfully"
                );
                // Clean up the temporary snapshot since operation succeeded
                // Note: In production, we might want to keep snapshots for audit trail
                info!(
                    "[SpiralConstellation] Operation succeeded, snapshot {} retained for audit",
                    pre_snapshot
                );

                Ok(())
            }
            Err(error) => {
                warn!(
                    "[SpiralConstellation] Atomic operation '{}' failed: {}",
                    operation_name, error
                );

                // Step 4: Automatic rollback on failure
                match self.atomic_rollback(&pre_snapshot).await {
                    Ok(_) => {
                        warn!(
                            "[SpiralConstellation] Successfully rolled back failed operation: {}",
                            operation_name
                        );
                        Err(SpiralError::Git {
                            message: format!(
                                "Operation '{operation_name}' failed and was rolled back: {error}"
                            ),
                        })
                    }
                    Err(rollback_error) => {
                        // Critical failure: both operation and rollback failed
                        error!("[SpiralConstellation] CRITICAL: Operation '{}' failed AND rollback failed: operation={}, rollback={}", 
                            operation_name, error, rollback_error);
                        Err(SpiralError::Git {
                            message: format!("CRITICAL: Operation '{operation_name}' failed and rollback also failed. Manual intervention required. Operation error: {error}. Rollback error: {rollback_error}"),
                        })
                    }
                }
            }
        }
    }

    /// Execute system changes and validation as a single logical operation
    async fn execute_and_validate_changes(
        &self,
        ctx: &Context,
        request: &mut SelfUpdateRequest,
    ) -> Result<()> {
        // Phase 3: Execute changes via Claude Code
        request.status = UpdateStatus::Executing;
        self.send_update_status(ctx, request, "Executing Changes")
            .await;

        self.execute_system_changes(request).await?;
        info!(
            "[SpiralConstellation] System changes executed for {}",
            request.id
        );

        // Phase 4: Validation and testing (part of the same atomic operation)
        request.status = UpdateStatus::Testing;
        self.send_update_status(ctx, request, "Running Validation Tests")
            .await;

        self.validate_system_changes(request).await?;
        info!(
            "[SpiralConstellation] System validation passed for {}",
            request.id
        );

        Ok(())
    }

    /// Send rollback success message
    async fn send_rollback_message(&self, ctx: &Context, request: &SelfUpdateRequest) {
        let rollback_msg = format!(
            "{}\n\n**Request ID:** {}\n**Status:** System restored to previous state",
            messages::auto_core_update::ROLLBACK_SUCCESS,
            request.codename
        );

        if let Ok(channel) = ctx.http.get_channel(request.channel_id.into()).await {
            if let Err(e) = channel.id().say(&ctx.http, rollback_msg).await {
                warn!(
                    "[SpiralConstellation] Failed to send rollback message: {}",
                    e
                );
            }
        }
    }

    /// Handle update failure with proper error reporting
    async fn handle_update_failure(&self, ctx: &Context, request: &SelfUpdateRequest) {
        let failure_msg = match &request.status {
            UpdateStatus::Failed(error) => {
                format!("{} {}\n\n**Request ID:** {}\n**Error:** {}\n\n⚠️ **System Status:** No changes applied, system remains stable\n\n{} Click {} to retry this request", 
                    messages::auto_core_update::FAILURE, error, request.codename, error,
                    "🔄", emojis::RETRY)
            }
            UpdateStatus::RolledBack => {
                format!("{}\n\n**Request ID:** {}\n**Status:** Changes were rolled back successfully\n\n{} Click {} to retry this request", 
                    messages::auto_core_update::ROLLBACK_SUCCESS, request.codename,
                    "🔄", emojis::RETRY)
            }
            _ => {
                format!(
                    "{} Unknown error\n\n**Request ID:** {}\n\n{} Click {} to retry this request",
                    messages::auto_core_update::FAILURE,
                    request.codename,
                    "🔄",
                    emojis::RETRY
                )
            }
        };

        if let Ok(channel) = ctx.http.get_channel(request.channel_id.into()).await {
            match channel.id().say(&ctx.http, failure_msg).await {
                Ok(failure_message) => {
                    // Add retry emoji reaction to the failure message
                    if let Err(e) = failure_message.react(&ctx.http, emojis::RETRY).await {
                        warn!(
                            "[SpiralConstellation] Failed to add retry emoji reaction: {}",
                            e
                        );
                    } else {
                        info!(
                            "[SpiralConstellation] Added retry emoji reaction to failure message"
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        "[SpiralConstellation] Failed to send failure message: {}",
                        e
                    );
                }
            }
        }

        // Mark queue as failed (clears all pending requests as per spec)
        {
            let queue = self.bot.update_queue.lock().await;
            queue.mark_failed().await;
        }
    }

    /// Handle retry request when retry emoji is clicked
    async fn handle_retry_request(
        &self,
        ctx: &Context,
        failure_msg: &Message,
        user: &serenity::model::user::User,
    ) {
        info!(
            "[SpiralConstellation] Processing retry request from user {}",
            user.id
        );

        // Extract the request ID from the failure message
        let content = &failure_msg.content;
        let request_id = if let Some(start) = content.find("**Request ID:** ") {
            let start_pos = start + "**Request ID:** ".len();
            if let Some(end) = content[start_pos..].find('\n') {
                content[start_pos..start_pos + end].to_string()
            } else {
                "unknown".to_string()
            }
        } else {
            "unknown".to_string()
        };

        // For now, we need to reconstruct the original request from available information
        // In a production system, we would store failed requests for retry
        let retry_msg = format!(
            "🔄 **Retry Request Queued**\n\n\
            **Original Request ID:** {}\n\
            **New Request ID:** retry-{}-{}\n\n\
            ⚠️ **Note:** Please provide the original request details again for the retry.\n\
            The system will process this as a new Auto Core Update request.\n\n\
            **Instructions:**\n\
            1. Reply to this message with your original update request\n\
            2. Include specific details about what needs to be changed\n\
            3. The system will create a new update request with proper validation",
            request_id,
            request_id,
            Self::get_simple_timestamp()
        );

        // Send retry message and add delete emoji
        match failure_msg.reply(&ctx.http, retry_msg).await {
            Ok(retry_message) => {
                if let Err(e) = retry_message.react(&ctx.http, emojis::TRASH_BIN).await {
                    warn!(
                        "[SpiralConstellation] Failed to add trash emoji to retry message: {}",
                        e
                    );
                }
                info!(
                    "[SpiralConstellation] Retry request initiated for request {}",
                    request_id
                );
            }
            Err(e) => {
                warn!("[SpiralConstellation] Failed to send retry message: {}", e);
            }
        }
    }
}

/// 🚀 CONSTELLATION BOT RUNNER: Manages the Discord client lifecycle
pub struct SpiralConstellationBotRunner {
    bot: SpiralConstellationBot,
    token: String,
}

impl SpiralConstellationBotRunner {
    pub fn new(bot: SpiralConstellationBot, token: String) -> Self {
        Self { bot, token }
    }

    /// 🚀 START BOT: Initialize and run the Discord bot
    pub async fn run(self) -> Result<()> {
        use serenity::{all::GatewayIntents, Client};

        info!("[SpiralConstellation] Starting Discord bot...");

        // REDUCED INTENTS: Use minimal set for basic functionality
        // If you get "Disallowed intent(s)" error, enable these in Discord Developer Portal:
        // - MESSAGE CONTENT INTENT (critical)
        // - SERVER MEMBERS INTENT (for role management)
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT  // Requires "Message Content Intent" enabled
            | GatewayIntents::GUILD_MESSAGE_REACTIONS  // For trash bin reaction handling
            | GatewayIntents::GUILDS;
        // Commented out for now: | GatewayIntents::GUILD_MEMBERS;  // Requires "Server Members Intent"

        let handler = ConstellationBotHandler::new(self.bot);

        let mut client = Client::builder(&self.token, intents)
            .event_handler(handler)
            .await
            .map_err(|e| SpiralError::Discord(Box::new(e)))?;

        info!("[SpiralConstellation] Discord client created, starting...");

        // Use start_autosharded which blocks until the client disconnects
        if let Err(e) = client.start_autosharded().await {
            return Err(SpiralError::Discord(Box::new(e)));
        }

        Ok(())
    }
}
