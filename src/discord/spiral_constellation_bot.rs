use crate::{
    agents::{Agent, SoftwareDeveloperAgent},
    claude_code::ClaudeCodeClient,
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
use tracing::{debug, error, info, warn};

/// Discord message length limit for safety
const MAX_MESSAGE_LENGTH: usize = 4000;
/// Maximum response output length to prevent Discord message limits
const MAX_OUTPUT_RESPONSE: usize = 1800;
use regex::Regex;

/// 🌌 SPIRAL CONSTELLATION BOT: Single Discord bot with dynamic agent personas
/// ARCHITECTURE DECISION: One bot, multiple personalities based on mention context
/// Why: Simpler deployment, dynamic persona switching, maintains agent identity feel
/// Alternative: Multiple Discord applications (rejected: deployment complexity)
pub struct SpiralConstellationBot {
    developer_agent: Arc<SoftwareDeveloperAgent>,
    claude_client: Arc<ClaudeCodeClient>,
    start_time: Instant,
    stats: Arc<Mutex<BotStats>>,
    mention_regex: Regex,
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
    pub greeting: &'static str,
    pub working_message: &'static str,
    pub completion_style: &'static str,
    pub error_style: &'static str,
    pub personality_traits: &'static [&'static str],
}

impl AgentPersona {
    pub const DEVELOPER: Self = Self {
        name: "SpiralDev",
        emoji: "🚀",
        greeting: "Ready to code! What can I build for you?",
        working_message: "⚡ Working on your code...",
        completion_style: "✅ Code generated successfully!",
        error_style: "❌ Code generation failed:",
        personality_traits: &["technical", "precise", "solution-focused", "efficient"],
    };
    
    pub const PROJECT_MANAGER: Self = Self {
        name: "SpiralPM",
        emoji: "📋",
        greeting: "Let me analyze the project status for you",
        working_message: "🔍 Gathering project information...",
        completion_style: "📊 Analysis complete!",
        error_style: "⚠️ Analysis failed:",
        personality_traits: &["strategic", "organized", "comprehensive", "collaborative"],
    };
    
    pub const QUALITY_ASSURANCE: Self = Self {
        name: "SpiralQA",
        emoji: "🔍",
        greeting: "Time for a thorough quality review!",
        working_message: "🧪 Running quality checks...",
        completion_style: "✅ Quality review complete!",
        error_style: "🚨 Quality check failed:",
        personality_traits: &["meticulous", "thorough", "safety-focused", "detail-oriented"],
    };
    
    pub const DECISION_MAKER: Self = Self {
        name: "SpiralDecide",
        emoji: "🎯",
        greeting: "Let me help you make the right decision",
        working_message: "🤔 Analyzing options...",
        completion_style: "⚡ Decision analysis ready!",
        error_style: "❓ Decision analysis failed:",
        personality_traits: &["analytical", "decisive", "logical", "balanced"],
    };
    
    pub const CREATIVE_INNOVATOR: Self = Self {
        name: "SpiralCreate",
        emoji: "✨",
        greeting: "Ready to explore creative solutions!",
        working_message: "🎨 Innovating...",
        completion_style: "🌟 Creative solution ready!",
        error_style: "💥 Innovation failed:",
        personality_traits: &["creative", "innovative", "experimental", "visionary"],
    };
    
    pub const PROCESS_COACH: Self = Self {
        name: "SpiralCoach",
        emoji: "🧘",
        greeting: "Let's optimize your process together",
        working_message: "🔄 Analyzing workflow...",
        completion_style: "🎯 Process improvement ready!",
        error_style: "🔧 Process analysis failed:",
        personality_traits: &["supportive", "methodical", "improvement-focused", "wise"],
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
        }
    }
}

impl SpiralConstellationBot {
    pub fn new(
        developer_agent: SoftwareDeveloperAgent,
        claude_client: ClaudeCodeClient,
    ) -> Result<Self> {
        // Pattern matches: @SpiralDev, @SpiralPM, @SpiralQA, etc. (user mentions)
        // Also matches role mentions: <@&role_id>
        let mention_regex = Regex::new(r"@Spiral(\w+)|<@&(\d+)>").map_err(|e| SpiralError::Agent {
            message: format!("Invalid mention regex: {e}"),
        })?;
        
        Ok(Self {
            developer_agent: Arc::new(developer_agent),
            claude_client: Arc::new(claude_client),
            start_time: Instant::now(),
            stats: Arc::new(Mutex::new(BotStats::default())),
            mention_regex,
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
    fn create_task_with_persona(&self, content: &str, agent_type: AgentType, context: MessageContext) -> Task {
        let persona = AgentPersona::for_agent_type(&agent_type);
        
        let mut task = Task::new(agent_type, content.to_string(), Priority::Medium);
        
        // Add Discord context
        task = task
            .with_context("discord_channel_id".to_string(), context.channel_id.to_string())
            .with_context("discord_message_id".to_string(), context.message_id.to_string())
            .with_context("discord_author_id".to_string(), context.author_id.to_string())
            .with_context("agent_persona".to_string(), persona.name.to_string())
            .with_context("persona_traits".to_string(), persona.personality_traits.join(","));
            
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
                
                // Add personality flavor based on agent type
                match agent_type {
                    AgentType::SoftwareDeveloper => {
                        response.push_str("**🔧 Technical Implementation:**\n");
                    }
                    AgentType::ProjectManager => {
                        response.push_str("**📊 Strategic Analysis:**\n");
                    }
                    AgentType::QualityAssurance => {
                        response.push_str("**🔍 Quality Assessment:**\n");
                    }
                    _ => {
                        response.push_str("**📋 Analysis Results:**\n");
                    }
                }
                
                // Truncate output for Discord safety
                if output.len() > MAX_OUTPUT_RESPONSE {
                    response.push_str(&output[..MAX_OUTPUT_RESPONSE]);
                    response.push_str("\n\n... (output truncated for Discord limits)");
                } else {
                    response.push_str(output);
                }
                
                // File summaries
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
                name if name.starts_with("spiral") => name,
                _ => return Some(format!("❓ Unknown role: `{}`. Available: SpiralDev, SpiralPM, SpiralQA, SpiralDecide, SpiralCreate, SpiralCoach", role_name))
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
                • 🧘 SpiralCoach - Process optimization & guidance\n\n\
                **Usage:**\n\
                • `@SpiralDev create a REST API` - Text mention\n\
                • `<@&role_id> help with testing` - Role mention\n\
                • `!spiral join SpiralDev` - Get agent role\n\
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
                error!("[SpiralConstellation] Failed to send length warning: {}", e);
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
                error!("[SpiralConstellation] Failed to send command response: {}", e);
            }
            return;
        }
        
        // Detect which agent persona to use
        let agent_type = match self.bot.detect_agent_persona(&msg.content, &msg, &ctx).await {
            Some(agent) => agent,
            None => {
                if let Err(e) = msg.reply(&ctx.http, "❓ I'm not sure which agent you'd like to talk to. Try mentioning @SpiralDev, @SpiralPM, or @SpiralQA, or use a role mention!").await {
                    error!("[SpiralConstellation] Failed to send clarification: {}", e);
                }
                return;
            }
        };
        
        let persona = AgentPersona::for_agent_type(&agent_type);
        let cleaned_content = self.bot.clean_message_content(&msg.content);
        
        if cleaned_content.is_empty() {
            let response = format!("{} **{}**\n{}", persona.emoji, persona.name, persona.greeting);
            if let Err(e) = msg.reply(&ctx.http, response).await {
                error!("[SpiralConstellation] Failed to send greeting: {}", e);
            }
            return;
        }
        
        // Update current persona
        {
            let mut stats = self.bot.stats.lock().await;
            stats.current_persona = Some(agent_type.clone());
        }
        
        // Send immediate acknowledgment with persona
        let working_response = format!(
            "{} **{}**\n{}\n\n📝 **Request:** {}",
            persona.emoji,
            persona.name,
            persona.working_message,
            if cleaned_content.len() > 100 {
                format!("{}...", &cleaned_content[..100])
            } else {
                cleaned_content.clone()
            }
        );
        
        if let Err(e) = msg.reply(&ctx.http, working_response).await {
            error!("[SpiralConstellation] Failed to send working message: {}", e);
            return;
        }
        
        // Create and execute task based on agent type
        let task = self.bot.create_task_with_persona(&cleaned_content, agent_type.clone(), context);
        let task_id = task.id.clone();
        
        debug!("[SpiralConstellation] Created {} task: {}", persona.name, task_id);
        
        // Execute based on agent type (currently only developer is implemented)
        let result = match agent_type {
            AgentType::SoftwareDeveloper => {
                // Add timeout to prevent hanging when Claude Code system is unavailable
                let execute_future = self.bot.developer_agent.execute(task);
                let timeout_duration = std::time::Duration::from_secs(30); // 30 second timeout
                
                match tokio::time::timeout(timeout_duration, execute_future).await {
                    Ok(execute_result) => match execute_result {
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
                    },
                    Err(_timeout) => {
                        warn!("[SpiralConstellation] {} task {} timed out after 30 seconds", persona.name, task_id);
                        
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
        
        // Send the response
        if let Err(e) = msg.reply(&ctx.http, result).await {
            error!("[SpiralConstellation] Failed to send result: {}", e);
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