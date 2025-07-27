use crate::{
    agents::{Agent, AgentOrchestrator, SoftwareDeveloperAgent},
    claude_code::ClaudeCodeClient,
    discord::message_state_manager::{MessageStateManager, MessageStateConfig},
    models::{AgentType, Priority, Task},
    Result, SpiralError,
};
use serenity::{
    async_trait,
    model::{
        channel::Message, 
        gateway::Ready,
        guild::Role,
        id::GuildId,
        permissions::Permissions,
    },
    prelude::*,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Discord message length limit for safety
const MAX_MESSAGE_LENGTH: usize = 4000;
/// Maximum response output length to prevent Discord message limits
const MAX_OUTPUT_RESPONSE: usize = 1950; // Closer to Discord's 2000 char limit

/// 🧠 USER INTENT: Classification of user request types
#[derive(Debug, Clone, PartialEq)]
enum UserIntent {
    /// Information queries: "what projects exist?", "show me files", "list directories"
    InformationQuery,
    /// Development tasks: "create a game", "build an app", "implement feature"
    Development,
    /// Modification tasks: "fix this bug", "update the code", "improve performance"
    Modification,
    /// Analysis tasks: "review this code", "explain how this works", "analyze the structure"
    Analysis,
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
    start_time: Instant,
    stats: Arc<Mutex<BotStats>>,
    mention_regex: Regex,
    // Message handling
    message_state_manager: Arc<MessageStateManager>,
}

#[derive(Debug, Clone, Default)]
struct BotStats {
    dev_tasks_completed: u64,
    pm_tasks_completed: u64,
    qa_tasks_completed: u64,
    total_tasks_failed: u64,
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
        personality_traits: &["meticulous", "thorough", "safety-focused", "detail-oriented"],
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
    ) -> Result<Self> {
        // Pattern matches: @SpiralDev, @SpiralPM, @SpiralQA, etc. (user mentions)
        // Also matches role mentions: <@&role_id>
        let mention_regex = Regex::new(r"@Spiral(\w+)|<@&(\d+)>").map_err(|e| SpiralError::Agent {
            message: format!("Invalid mention regex: {e}"),
        })?;
        
        // Initialize message state manager
        let message_state_manager = Arc::new(MessageStateManager::new(MessageStateConfig::default()));
        
        Ok(Self {
            developer_agent: Some(Arc::new(developer_agent)),
            claude_client: Some(Arc::new(claude_client)),
            orchestrator: None,
            start_time: Instant::now(),
            stats: Arc::new(Mutex::new(BotStats::default())),
            mention_regex,
            message_state_manager,
        })
    }

    /// 🎛️ ORCHESTRATOR MODE: Create bot with orchestrator integration (full system)
    pub async fn new_with_orchestrator(orchestrator: Arc<AgentOrchestrator>) -> Result<Self> {
        // Pattern matches: @SpiralDev, @SpiralPM, @SpiralQA, etc. (user mentions)
        // Also matches role mentions: <@&role_id>
        let mention_regex = Regex::new(r"@Spiral(\w+)|<@&(\d+)>").map_err(|e| SpiralError::Agent {
            message: format!("Invalid mention regex: {e}"),
        })?;
        
        // Initialize message state manager
        let message_state_manager = Arc::new(MessageStateManager::new(MessageStateConfig::default()));
        
        // Start background cleanup task for message state manager
        let cleanup_manager = message_state_manager.clone();
        cleanup_manager.start_cleanup_task();
        
        Ok(Self {
            developer_agent: None,
            claude_client: None,
            orchestrator: Some(orchestrator),
            start_time: Instant::now(),
            stats: Arc::new(Mutex::new(BotStats::default())),
            mention_regex,
            message_state_manager,
        })
    }

    /// 🎭 ROLE MANAGEMENT: Create agent persona roles in Discord server
    pub async fn create_agent_roles(&self, ctx: &Context, guild_id: GuildId) -> Result<Vec<Role>> {
        let mut created_roles = Vec::new();
        
        info!("[SpiralConstellation] Creating agent persona roles in guild {}", guild_id);
        
        let personas = [
            (&AgentPersona::DEVELOPER, 0x00ff00), // Green
            (&AgentPersona::PROJECT_MANAGER, 0x0066ff), // Blue  
            (&AgentPersona::QUALITY_ASSURANCE, 0xff6600), // Orange
            (&AgentPersona::DECISION_MAKER, 0x9900ff), // Purple
            (&AgentPersona::CREATIVE_INNOVATOR, 0xff0099), // Pink
            (&AgentPersona::PROCESS_COACH, 0x00ffff), // Cyan
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
                    info!("[SpiralConstellation] Created role: {} ({})", persona.name, role.id);
                    created_roles.push(role);
                }
                Err(e) => {
                    warn!("[SpiralConstellation] Failed to create role {}: {}", persona.name, e);
                }
            }
        }
        
        Ok(created_roles)
    }

    /// 🔍 ROLE DETECTION: Find agent role by name in guild
    pub async fn find_agent_role(&self, ctx: &Context, guild_id: GuildId, persona_name: &str) -> Option<Role> {
        match guild_id.roles(&ctx.http).await {
            Ok(roles) => {
                roles.values()
                    .find(|role| role.name == persona_name)
                    .cloned()
            }
            Err(e) => {
                warn!("[SpiralConstellation] Failed to fetch roles: {}", e);
                None
            }
        }
    }

    /// 🎯 ROLE ASSIGNMENT: Give user an agent persona role
    pub async fn assign_agent_role(&self, ctx: &Context, guild_id: GuildId, user_id: serenity::model::id::UserId, persona_name: &str) -> Result<()> {
        if let Some(role) = self.find_agent_role(ctx, guild_id, persona_name).await {
            match guild_id.member(&ctx.http, user_id).await {
                Ok(member) => {
                    if let Err(e) = member.add_role(&ctx.http, role.id).await {
                        warn!("[SpiralConstellation] Failed to assign role {} to user {}: {}", persona_name, user_id, e);
                        return Err(SpiralError::Discord(Box::new(e)));
                    } else {
                        info!("[SpiralConstellation] Assigned role {} to user {}", persona_name, user_id);
                    }
                }
                Err(e) => {
                    warn!("[SpiralConstellation] Failed to get member {}: {}", user_id, e);
                    return Err(SpiralError::Discord(Box::new(e)));
                }
            }
        } else {
            // Role doesn't exist, try to create it
            if let Ok(roles) = self.create_agent_roles(ctx, guild_id).await {
                if let Some(new_role) = roles.iter().find(|r| r.name == persona_name) {
                    if let Ok(member) = guild_id.member(&ctx.http, user_id).await {
                        if let Err(e) = member.add_role(&ctx.http, new_role.id).await {
                            warn!("[SpiralConstellation] Failed to assign new role {} to user {}: {}", persona_name, user_id, e);
                            return Err(SpiralError::Discord(Box::new(e)));
                        } else {
                            info!("[SpiralConstellation] Created and assigned role {} to user {}", persona_name, user_id);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// 🎭 PERSONA DETECTION: Determine which agent persona to use based on mentions or role mentions
    async fn detect_agent_persona(&self, content: &str, msg: &Message, ctx: &Context) -> Option<AgentType> {
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
                    "create" | "creative" | "innovate" => return Some(AgentType::CreativeInnovator),
                    "coach" | "process" => return Some(AgentType::ProcessCoach),
                    "king" | "spiralking" | "lordgenome" => return Some(AgentType::SpiralKing),
                    _ => {}
                }
            }
        }
        
        // Fallback: detect based on content keywords
        if content_lower.contains("code") || content_lower.contains("implement") || content_lower.contains("function") {
            Some(AgentType::SoftwareDeveloper)
        } else if content_lower.contains("status") || content_lower.contains("project") || content_lower.contains("timeline") {
            Some(AgentType::ProjectManager)
        } else if content_lower.contains("test") || content_lower.contains("quality") || content_lower.contains("bug") {
            Some(AgentType::QualityAssurance)
        } else {
            // Default to developer agent if unclear
            Some(AgentType::SoftwareDeveloper)
        }
    }

    /// 🎯 TASK CREATION: Create task with agent persona context
    fn create_task_with_persona(&self, content: &str, agent_type: AgentType, context: MessageContext, intent: UserIntent) -> Task {
        let persona = AgentPersona::for_agent_type(&agent_type);
        
        // Enhance the description based on user intent
        let enhanced_description = match intent {
            UserIntent::InformationQuery => {
                format!("INFORMATION QUERY: {}. Please provide a clear, informative response about the current state of the workspace/project. Focus on listing, showing, or describing what exists rather than creating new code.", content)
            },
            UserIntent::Development => {
                format!("DEVELOPMENT TASK: {}. Please implement, create, or build the requested functionality following best practices.", content)
            },
            UserIntent::Modification => {
                format!("MODIFICATION TASK: {}. Please update, fix, or improve the existing code/project as requested.", content)
            },
            UserIntent::Analysis => {
                format!("ANALYSIS TASK: {}. Please review, explain, or analyze the requested code/project providing insights and explanations.", content)
            },
        };
        
        let mut task = Task::new(agent_type, enhanced_description, Priority::Medium);
        
        // Add Discord context
        task = task
            .with_context("discord_channel_id".to_string(), context.channel_id.to_string())
            .with_context("discord_message_id".to_string(), context.message_id.to_string())
            .with_context("discord_author_id".to_string(), context.author_id.to_string())
            .with_context("agent_persona".to_string(), persona.name.to_string())
            .with_context("persona_traits".to_string(), persona.personality_traits.join(","))
            .with_context("user_intent".to_string(), format!("{:?}", intent));
            
        if let Some(guild_id) = context.guild_id {
            task = task.with_context("discord_guild_id".to_string(), guild_id.to_string());
        }
        
        task
    }

    /// 🎭 PERSONA RESPONSE: Format response in the agent's persona style
    fn format_persona_response(&self, agent_type: &AgentType, result: &crate::models::TaskResult) -> String {
        let persona = AgentPersona::for_agent_type(agent_type);
        
        match &result.result {
            crate::models::TaskExecutionResult::Success { 
                output, 
                files_created, 
                files_modified 
            } => {
                let mut response = String::new();
                
                // Persona-specific header
                response.push_str(&format!("{} **{}**\n", persona.emoji, persona.name));
                response.push_str(&format!("{}\n\n", persona.completion_style));
                
                // For SoftwareDeveloper, provide concise summary instead of full output
                match agent_type {
                    AgentType::SoftwareDeveloper => {
                        // Extract key information and provide concise summary
                        let summary = self.extract_dev_summary(output, files_created, files_modified);
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
                        response.push_str(&format!("\n\n📁 **Files Created:** {}", files_created.len()));
                        for file in files_created.iter().take(3) {
                            response.push_str(&format!("\n• `{}`", file));
                        }
                        if files_created.len() > 3 {
                            response.push_str(&format!("\n• ... and {} more", files_created.len() - 3));
                        }
                    }
                    
                    if !files_modified.is_empty() {
                        response.push_str(&format!("\n\n✏️ **Files Modified:** {}", files_modified.len()));
                        for file in files_modified.iter().take(3) {
                            response.push_str(&format!("\n• `{}`", file));
                        }
                        if files_modified.len() > 3 {
                            response.push_str(&format!("\n• ... and {} more", files_modified.len() - 3));
                        }
                    }
                }
                
                // Persona-specific footer
                response.push_str(&format!("\n\n*—{} @ SpiralConstellation*", persona.name));
                
                response
            }
            crate::models::TaskExecutionResult::Failure { error, partial_output } => {
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

    /// 🧠 INTENT CLASSIFICATION: Use Claude Code to determine the type of action requested
    async fn classify_user_intent(&self, content: &str) -> UserIntent {
        // If we have access to Claude Code client, use it for classification
        if let Some(claude_client) = &self.claude_client {
            let classification_prompt = format!(
                "User is making this query: \"{}\"\n\n\
                What is the intent? Choose EXACTLY ONE from these options:\n\
                - InformationQuery: User wants to know/see/list what exists (\"what projects?\", \"show files\", \"list directories\", \"what's the status?\")\n\
                - Development: User wants to create/build/implement something new (\"create a game\", \"build an app\", \"make a function\")\n\
                - Modification: User wants to fix/update/change existing code (\"fix this bug\", \"update the code\", \"improve performance\")\n\
                - Analysis: User wants explanation/review of existing code (\"explain this\", \"how does it work?\", \"review the code\")\n\n\
                Respond with ONLY the intent name (e.g., \"InformationQuery\"). No other text.", 
                content
            );

            // Use Claude Code for classification via generate_code with a simple request
            use crate::claude_code::CodeGenerationRequest;
            use std::collections::HashMap;
            
            let request = CodeGenerationRequest {
                language: "text".to_string(),
                description: classification_prompt,
                context: HashMap::new(),
                existing_code: None,
                requirements: vec![],
                session_id: None,
            };
            
            match claude_client.generate_code(request).await {
                Ok(result) => {
                    // Try explanation first, then code field
                    let response_text = if !result.explanation.trim().is_empty() {
                        result.explanation.trim()
                    } else {
                        result.code.trim()
                    };
                    
                    info!("[SpiralConstellation] Claude classified intent as: '{}'", response_text);
                    
                    match response_text {
                        "InformationQuery" => return UserIntent::InformationQuery,
                        "Development" => return UserIntent::Development,
                        "Modification" => return UserIntent::Modification,
                        "Analysis" => return UserIntent::Analysis,
                        _ => {
                            warn!("[SpiralConstellation] Unknown intent classification: '{}', defaulting to Development", response_text);
                        }
                    }
                }
                Err(e) => {
                    warn!("[SpiralConstellation] Failed to classify intent with Claude: {}, falling back to pattern matching", e);
                }
            }
        }
        
        // Fallback: Simple pattern matching when Claude Code is unavailable
        let content_lower = content.to_lowercase();
        
        // Information/query requests
        if (content_lower.contains("what") || content_lower.starts_with("list") || 
            content_lower.starts_with("show") || content_lower.starts_with("check")) &&
           (content_lower.contains("projects") || content_lower.contains("files") || 
            content_lower.contains("directories") || content_lower.contains("status") ||
            content_lower.contains("exists")) {
            return UserIntent::InformationQuery;
        }
        
        // Development requests
        if content_lower.contains("create") || content_lower.contains("build") ||
           content_lower.contains("make") || content_lower.contains("implement") {
            return UserIntent::Development;
        }
        
        // Modification requests
        if content_lower.contains("fix") || content_lower.contains("update") ||
           content_lower.contains("modify") || content_lower.contains("improve") {
            return UserIntent::Modification;
        }
        
        // Analysis requests
        if content_lower.contains("analyze") || content_lower.contains("review") ||
           content_lower.contains("explain") || content_lower.contains("how does") {
            return UserIntent::Analysis;
        }
        
        // Default to development for ambiguous cases
        UserIntent::Development
    }
    
    /// 📋 CONCISE DEV SUMMARY: Extract key information for developer responses
    fn extract_dev_summary(&self, output: &str, files_created: &[String], files_modified: &[String]) -> String {
        // For information queries or short responses, return the full output
        if output.len() <= MAX_OUTPUT_RESPONSE || 
           output.to_lowercase().contains("current projects") ||
           output.to_lowercase().contains("projects in") ||
           output.to_lowercase().contains("workspace") ||
           output.starts_with("Here") ||
           !output.contains("Features:") {
            
            // For informational responses, just return with minimal formatting
            let clean_output = if output.len() > MAX_OUTPUT_RESPONSE {
                format!("{}...\n\n*(Output truncated - showing first {} characters)*", 
                       &output[..MAX_OUTPUT_RESPONSE-50], MAX_OUTPUT_RESPONSE-50)
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
            if trimmed.is_empty() { continue; }
            
            // Skip architecture/technical details
            if trimmed.to_lowercase().contains("architecture") || 
               trimmed.to_lowercase().contains("solid principles") ||
               trimmed.to_lowercase().contains("dry principle") ||
               trimmed.to_lowercase().contains("sid naming") {
                break;
            }
            
            // Look for key features or summary sections
            if trimmed.starts_with("Key Features:") || 
               trimmed.starts_with("Features:") ||
               trimmed.starts_with("Summary") {
                found_key_features = true;
                summary.push_str(&format!("**{}**\n", trimmed));
                continue;
            }
            
            // Capture feature list items or summary content
            if found_key_features && (trimmed.starts_with("•") || trimmed.starts_with("-") || trimmed.starts_with("*")) {
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
            if line.to_lowercase().contains("how to run") || line.to_lowercase().contains("to run:") {
                summary.push_str("\n**🚀 How to Run:**\n");
                // Capture next few lines that look like instructions
                for instruction_line in lines.iter().skip(i + 1).take(3) {
                    let trimmed = instruction_line.trim();
                    if trimmed.is_empty() { break; }
                    if trimmed.starts_with("•") || trimmed.starts_with("-") || 
                       trimmed.starts_with("1.") || trimmed.starts_with("2.") ||
                       trimmed.contains("npm") || trimmed.contains("cargo") || trimmed.contains("run") {
                        summary.push_str(&format!("• {}\n", trimmed.trim_start_matches(&['•', '-', '1', '2', '.', ' '])));
                    }
                }
                break;
            }
        }
        
        // If summary is too short, add a basic completion message
        if summary.trim().is_empty() {
            summary = "Task completed successfully! Check the files for implementation details.\n".to_string();
        }
        
        summary
    }

    /// 💬 HELPFUL ERROR: Format user-friendly error messages with solutions
    fn format_helpful_error_message(&self, error: &crate::SpiralError, persona: &AgentPersona) -> String {
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
        if error_lower.contains("claude") && (error_lower.contains("connection") || error_lower.contains("unavailable")) {
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
        if error_lower.contains("api") && (error_lower.contains("key") || error_lower.contains("auth") || error_lower.contains("unauthorized")) {
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
        if error_lower.contains("permission") || error_lower.contains("workspace") || error_lower.contains("directory") {
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

    /// 🎮 COMMAND HANDLER: Handle special bot commands
    async fn handle_special_commands(&self, content: &str, msg: &Message, ctx: &Context) -> Option<String> {
        let content_lower = content.to_lowercase();
        
        // Role creation command
        if content_lower.contains("!spiral setup roles") || content_lower.contains("!spiral create roles") {
            if let Some(guild_id) = msg.guild_id {
                match self.create_agent_roles(ctx, guild_id).await {
                    Ok(roles) => {
                        let role_list = roles.iter()
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
            let role_name = content_lower.strip_prefix("!spiral join ").unwrap_or("").trim();
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
                match self.assign_agent_role(ctx, guild_id, msg.author.id, persona_name).await {
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
                            self.find_agent_role(ctx, guild_id, persona_name).await
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
                • `!spiral help` - Show this help\n\
                • `!spiral join <role>` - Join an agent role\n\
                • `!spiral setup roles` - Create agent roles (admin)\n\n\
                *Each persona responds with unique personality and expertise!* 🌟".to_string()
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
        Self {
            bot: Arc::new(bot),
        }
    }
}

#[async_trait]
impl EventHandler for ConstellationBotHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore bot messages
        if msg.author.bot {
            return;
        }
        
        // Validate message length to prevent DoS
        if msg.content.len() > MAX_MESSAGE_LENGTH {
            warn!("[SpiralConstellation] Message too long: {} chars from user {}", msg.content.len(), msg.author.id);
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
        if let Some(command_response) = self.bot.handle_special_commands(&msg.content, &msg, &ctx).await {
            if let Err(e) = msg.reply(&ctx.http, command_response).await {
                warn!("[SpiralConstellation] Failed to send command response: {}", e);
            }
            return;
        }
        
        // Detect which agent persona to use
        let agent_type = match self.bot.detect_agent_persona(&msg.content, &msg, &ctx).await {
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
            let response = format!("{} **{}**\n{}", persona.emoji, persona.name, persona.random_greeting());
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
        
        // Step 2: Classify intent through Claude Code
        let intent = self.bot.classify_user_intent(&cleaned_content).await;
        info!("[SpiralConstellation] Classified intent as: {:?}", intent);
        
        // Step 3: Respond with intended action
        let action_description = match intent {
            UserIntent::InformationQuery => format!("🔍 **Analyzing Workspace**\nI'll inspect your workspace to find and list existing projects."),
            UserIntent::Development => format!("🚀 **Development Task**\nI'll create/build the requested functionality."),
            UserIntent::Modification => format!("🔧 **Modification Task**\nI'll update/fix the existing code as requested."),
            UserIntent::Analysis => format!("📊 **Analysis Task**\nI'll review and analyze the requested code/project."),
        };
        
        let intent_response = format!(
            "{} **{}**\n{}\n\n📝 **Request:** {}\n\n{}",
            persona.emoji,
            persona.name,
            action_description,
            if cleaned_content.len() > 100 {
                format!("{}...", &cleaned_content[..100])
            } else {
                cleaned_content.clone()
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
        let task = self.bot.create_task_with_persona(&cleaned_content, agent_type.clone(), context, intent.clone());
        let task_id = task.id.clone();
        
        info!("[SpiralConstellation] Created {} task: {}", persona.name, task_id);
        
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
                            warn!("[SpiralConstellation] Failed to submit task to orchestrator: {}", e);
                            let error_message = self.bot.format_helpful_error_message(&e, &persona);
                            if let Err(reply_err) = msg.reply(&ctx.http, error_message).await {
                                warn!("[SpiralConstellation] Failed to send error message: {}", reply_err);
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
                                    if cleaned_content.len() > 100 {
                                        format!("{}...", &cleaned_content[..100])
                                    } else {
                                        cleaned_content.clone()
                                    },
                                    progress_emojis[emoji_index],
                                    start_time.elapsed().as_secs()
                                );
                                
                                if let Some(ref mut msg_ref) = intent_msg {
                                    let _ = msg_ref.edit(&ctx.http, serenity::builder::EditMessage::new().content(progress_response)).await;
                                }
                                
                                last_update = std::time::Instant::now();
                            }
                            
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        }
                    };
                    
                    match tokio::time::timeout(timeout_duration, poll_future).await {
                        Ok(Ok(result)) => {
                            info!("[SpiralConstellation] {} task {} completed via orchestrator", persona.name, task_id);
                            
                            // Update success stats
                            {
                                let mut stats = self.bot.stats.lock().await;
                                stats.dev_tasks_completed += 1;
                                stats.current_persona = None;
                            }
                            
                            self.bot.format_persona_response(&agent_type, &result)
                        }
                        Ok(Err(e)) => {
                            warn!("[SpiralConstellation] {} task {} failed via orchestrator: {}", persona.name, task_id, e);
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
                                let error_message = self.bot.format_helpful_error_message(&timeout_error, &persona);
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
                        let content_clone = cleaned_content.clone();
                        
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
                                    let _ = intent_message.edit(&ctx_clone.http, serenity::builder::EditMessage::new().content(progress_response)).await;
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
                                    info!("[SpiralConstellation] {} task {} completed", persona.name, task_id);
                                    
                                    // Update success stats
                                    {
                                        let mut stats = self.bot.stats.lock().await;
                                        stats.dev_tasks_completed += 1;
                                        stats.current_persona = None;
                                    }
                                    
                                    self.bot.format_persona_response(&agent_type, &result)
                                }
                                Err(e) => {
                                    warn!("[SpiralConstellation] {} task {} failed: {}", persona.name, task_id, e);
                                    
                                    // Update failure stats
                                    {
                                        let mut stats = self.bot.stats.lock().await;
                                        stats.total_tasks_failed += 1;
                                        stats.current_persona = None;
                                    }
                                    
                                    // Provide helpful error messages based on error type
                                    let error_message = self.bot.format_helpful_error_message(&e, &persona);
                                    error_message
                                }
                            }
                        },
                        Err(_timeout) => {
                            // Cancel progress updates
                            progress_task.abort();
                            
                            warn!("[SpiralConstellation] {} task {} timed out after 90 seconds", persona.name, task_id);
                            
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
                            let error_message = self.bot.format_helpful_error_message(&timeout_error, &persona);
                            error_message
                        }
                    }
                } else {
                        // Neither orchestrator nor direct agent available
                        let config_error = crate::SpiralError::Agent {
                            message: "Bot not properly configured - no execution method available".to_string(),
                        };
                        let error_message = self.bot.format_helpful_error_message(&config_error, &persona);
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
                    persona.personality_traits.iter()
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
                if cleaned_content.len() > 100 {
                    format!("{}...", &cleaned_content[..100])
                } else {
                    cleaned_content.clone()
                },
                result
            );
            
            if let Err(e) = intent_message.edit(&ctx.http, serenity::builder::EditMessage::new().content(final_response)).await {
                warn!("[SpiralConstellation] Failed to edit intent message: {}", e);
                // Fallback: send as new reply if edit fails
                if let Err(e2) = msg.reply(&ctx.http, result).await {
                    warn!("[SpiralConstellation] Failed to send fallback result: {}", e2);
                }
            }
        } else {
            // Fallback: send as reply if we don't have the intent message
            if let Err(e) = msg.reply(&ctx.http, result).await {
                warn!("[SpiralConstellation] Failed to send result: {}", e);
            }
        }
    }
    
    async fn ready(&self, _: Context, ready: Ready) {
        info!("🌌 SpiralConstellation bot {} is connected and ready!", ready.user.name);
        info!("Available personas: SpiralDev, SpiralPM, SpiralQA, SpiralDecide, SpiralCreate, SpiralCoach");
        info!("Role support: Discord roles can be created with '!spiral setup roles'");
        info!("Usage: @SpiralDev, role mentions, or !spiral join <role>");
        
        let stats = self.bot.stats.lock().await;
        info!("Bot statistics: {} dev tasks completed", stats.dev_tasks_completed);
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
        use serenity::{Client, all::GatewayIntents};
        
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