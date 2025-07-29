use crate::{
    agents::{Agent, AgentOrchestrator, SoftwareDeveloperAgent},
    claude_code::ClaudeCodeClient,
    config::DiscordConfig,
    discord::{
        message_state_manager::{MessageStateConfig, MessageStateManager},
        IntentClassifier, IntentResponse, IntentType, MessageSecurityValidator, RiskLevel,
        SecureMessageHandler,
    },
    models::{AgentType, Priority, Task},
    Result, SpiralError,
};
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
use tracing::{info, warn};

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
        cleanup_manager.start_cleanup_task();

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
                format!("INFORMATION QUERY: {}. Please provide a clear, informative response about the current state of the workspace/project. Focus on listing, showing, or describing what exists rather than creating new code.", content)
            }
            UserIntent::TaskRequest => {
                format!("DEVELOPMENT TASK: {}. Please implement, create, or build the requested functionality following best practices.", content)
            }
            UserIntent::AgentSelection => {
                format!("AGENT-SPECIFIC TASK: {}. Please execute this task with the selected agent's specific expertise and capabilities.", content)
            }
            UserIntent::HelpRequest => {
                format!("HELP REQUEST: {}. Please provide helpful information about usage, capabilities, or guidance as requested.", content)
            }
            UserIntent::Greeting => {
                format!(
                    "GREETING: {}. Please respond appropriately to the user's greeting.",
                    content
                )
            }
            UserIntent::Unknown => {
                format!("GENERAL REQUEST: {}. Please interpret and respond to this request appropriately.", content)
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
            .with_context("user_intent".to_string(), format!("{:?}", intent));

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
                            response.push_str(&format!("\n• `{}`", file));
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
                            response.push_str(&format!("\n• `{}`", file));
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
                summary.push_str(&format!("**{}**\n", trimmed));
                continue;
            }

            // Capture feature list items or summary content
            if found_key_features
                && (trimmed.starts_with("•")
                    || trimmed.starts_with("-")
                    || trimmed.starts_with("*"))
            {
                summary.push_str(&format!("{}\n", trimmed));
            } else if found_key_features && !trimmed.is_empty() && !trimmed.contains(":") {
                summary.push_str(&format!("{}\n", trimmed));
            } else if !found_summary && !trimmed.contains(":") && trimmed.len() > 20 {
                // Capture main description if no structured summary found
                summary.push_str(&format!("{}\n\n", trimmed));
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
                            trimmed.trim_start_matches(&['•', '-', '1', '2', '.', ' '])
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
        // Validate command input for security
        let validation_result = self.secure_message_handler.validate_command_input(content);
        if !validation_result.is_valid {
            warn!(
                "[SpiralConstellation] Command validation failed: {:?}",
                validation_result.issues
            );
            return Some(format!(
                "⚠️ Command blocked: {}",
                validation_result.issues.join(", ")
            ));
        }

        let content_lower = content.to_lowercase();

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
                        return Some(format!("❌ Failed to create roles: {}", e));
                    }
                }
            } else {
                return Some("❌ Role creation only works in servers, not DMs.".to_string());
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
                _ => return Some(format!("❓ Unknown role: `{}`. Available: SpiralDev, SpiralPM, SpiralQA, SpiralDecide, SpiralCreate, SpiralCoach, SpiralKing", role_name))
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
                        return Some(format!("❌ Failed to assign role: {}", e));
                    }
                }
            } else {
                return Some("❌ Role assignment only works in servers, not DMs.".to_string());
            }
        }

        // Security stats command (authorized users only)
        if content_lower.starts_with("!spiral security stats") {
            // Check authorized user permission
            if !self.is_authorized_user(msg.author.id.get()) {
                return Some(
                    "🚫 This command requires authorization. Contact an administrator.".to_string(),
                );
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
            if !self.is_authorized_user(msg.author.id.get()) {
                return Some(
                    "🚫 This command requires authorization. Contact an administrator.".to_string(),
                );
            }

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
            if !self.is_authorized_user(msg.author.id.get()) {
                return Some(
                    "🚫 This command requires authorization. Contact an administrator.".to_string(),
                );
            }

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
                        "✅ Rate limit reset for <@{}>\nThey can now send messages again.",
                        uid
                    ));
                } else {
                    return Some("❌ Invalid user ID or mention format.".to_string());
                }
            }
        }

        // Security report command (authorized users only)
        if content_lower.starts_with("!spiral security report") {
            // Check authorized user permission
            if !self.is_authorized_user(msg.author.id.get()) {
                return Some(
                    "🚫 This command requires authorization. Contact an administrator.".to_string(),
                );
            }

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
                • `!spiral ratelimit` - Check your rate limit status\n\n\
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
            warn!(
                "[SpiralConstellation] Security validation failed for user {}: {:?}",
                msg.author.id, validation_result.issues
            );
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

        // Handle special commands first
        if let Some(command_response) = self
            .bot
            .handle_special_commands(&msg.content, &msg, &ctx)
            .await
        {
            if let Err(e) = msg.reply(&ctx.http, command_response).await {
                warn!(
                    "[SpiralConstellation] Failed to send command response: {}",
                    e
                );
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
            UserIntent::StatusQuery => format!("🔍 **Analyzing Workspace**\nI'll inspect your workspace to find and list existing projects."),
            UserIntent::TaskRequest => format!("🚀 **Development Task**\nI'll create/build the requested functionality."),
            UserIntent::AgentSelection => format!("🎯 **Agent-Specific Task**\nI'll handle this with the selected agent's expertise."),
            UserIntent::HelpRequest => format!("❓ **Help Request**\nI'll provide the information you need."),
            UserIntent::Greeting => format!("👋 **Greeting**\nNice to meet you!"),
            UserIntent::Unknown => format!("🔄 **Processing Request**\nI'll handle your request appropriately."),
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
                            let error_message = self.bot.format_helpful_error_message(&e, &persona);
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
                            let error_message = self.bot.format_helpful_error_message(&e, &persona);
                            error_message
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
                                let error_message = self
                                    .bot
                                    .format_helpful_error_message(&timeout_error, &persona);
                                error_message
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
                                    let error_message =
                                        self.bot.format_helpful_error_message(&e, &persona);
                                    error_message
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
                            let error_message = self
                                .bot
                                .format_helpful_error_message(&timeout_error, &persona);
                            error_message
                        }
                    }
                } else {
                    // Neither orchestrator nor direct agent available
                    let config_error = crate::SpiralError::Agent {
                        message: "Bot not properly configured - no execution method available"
                            .to_string(),
                    };
                    let error_message = self
                        .bot
                        .format_helpful_error_message(&config_error, &persona);
                    error_message
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
                        .map(|trait_name| format!("• {}", trait_name))
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
            | GatewayIntents::GUILDS;
        // Commented out for now: | GatewayIntents::GUILD_MEMBERS;  // Requires "Server Members Intent"

        let handler = ConstellationBotHandler::new(self.bot);

        let mut client = Client::builder(&self.token, intents)
            .event_handler(handler)
            .await
            .map_err(|e| SpiralError::Discord(Box::new(e)))?;

        info!("[SpiralConstellation] Discord client created, starting...");

        if let Err(e) = client.start().await {
            return Err(SpiralError::Discord(Box::new(e)));
        }

        Ok(())
    }
}
