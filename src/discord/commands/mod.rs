use crate::discord::spiral_constellation_bot::SpiralConstellationBot;
use serenity::{model::channel::Message, prelude::Context};
use tracing::debug;

pub mod admin;
pub mod claude_agents;
pub mod debug;
pub mod debug_progress;
pub mod help;
pub mod rate_limit;
pub mod roles;
pub mod security;
pub mod self_update;

/// Command handler trait for all Discord commands
#[allow(async_fn_in_trait)]
pub trait CommandHandler {
    async fn handle(
        &self,
        content: &str,
        msg: &Message,
        ctx: &Context,
        bot: &SpiralConstellationBot,
    ) -> Option<String>;

    fn command_prefix(&self) -> &str;
    fn description(&self) -> &str;
}

/// Command metadata for help generation
#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: &'static str,
    pub prefix: &'static str,
    pub description: &'static str,
    pub category: CommandCategory,
    pub requires_auth: bool,
}

/// Command categories for organization
#[derive(Debug, Clone, PartialEq)]
pub enum CommandCategory {
    Admin,
    Debug,
    General,
    Roles,
    Security,
    Updates,
}

/// Get commands by category
pub fn get_commands_by_category(category: CommandCategory) -> Vec<&'static CommandInfo> {
    AVAILABLE_COMMANDS
        .iter()
        .filter(|cmd| cmd.category == category)
        .collect()
}

/// Get commands by authorization requirement
pub fn get_commands_by_auth(requires_auth: bool) -> Vec<&'static CommandInfo> {
    AVAILABLE_COMMANDS
        .iter()
        .filter(|cmd| cmd.requires_auth == requires_auth)
        .collect()
}

/// All available commands defined statically
pub const AVAILABLE_COMMANDS: &[CommandInfo] = &[
    CommandInfo {
        name: "admin",
        prefix: "!spiral admin",
        description: "System dashboard with metrics and quick actions",
        category: CommandCategory::Admin,
        requires_auth: true,
    },
    CommandInfo {
        name: "debug progress",
        prefix: "!spiral debug progress",
        description: "Demo the progress bar functionality",
        category: CommandCategory::Debug,
        requires_auth: false,
    },
    CommandInfo {
        name: "debug",
        prefix: "!spiral debug",
        description: "Debug information and security analysis",
        category: CommandCategory::Debug,
        requires_auth: true,
    },
    CommandInfo {
        name: "help",
        prefix: "!spiral help",
        description: "Show available commands and usage information",
        category: CommandCategory::General,
        requires_auth: false,
    },
    CommandInfo {
        name: "commands",
        prefix: "!spiral commands",
        description: "Show concise command list",
        category: CommandCategory::General,
        requires_auth: false,
    },
    CommandInfo {
        name: "ratelimit",
        prefix: "!spiral ratelimit",
        description: "Check and manage user rate limits",
        category: CommandCategory::Admin,
        requires_auth: true,
    },
    CommandInfo {
        name: "roles",
        prefix: "!spiral roles",
        description: "Manage Discord agent roles",
        category: CommandCategory::Roles,
        requires_auth: true,
    },
    CommandInfo {
        name: "security",
        prefix: "!spiral security",
        description: "Security metrics and analysis tools",
        category: CommandCategory::Security,
        requires_auth: true,
    },
    CommandInfo {
        name: "update",
        prefix: "!spiral update",
        description: "Self-update system information",
        category: CommandCategory::Updates,
        requires_auth: true,
    },
    // 🏗️ ARCHITECTURE DECISION: Dual command aliases for discoverability
    // Why: Users might look for "agents" or "claude-agents"
    // Alternative: Single command (rejected: reduces discoverability)
    // Trade-off: Slight duplication in command list
    // Audit: Both must route to same handler in CommandRouter::route_command
    CommandInfo {
        name: "agents",
        prefix: "!spiral agents",
        description: "List all available Claude validation and utility agents",
        category: CommandCategory::General,
        requires_auth: false,
    },
    CommandInfo {
        name: "claude-agents",
        prefix: "!spiral claude-agents",
        description: "List all available Claude validation and utility agents",
        category: CommandCategory::General,
        requires_auth: false,
    },
];

/// Command router for handling all !spiral commands
/// 📐 SOLID: Single Responsibility - Each command handler manages one concern
/// 🔍 AUDIT CHECKPOINT: All commands must be registered here AND in AVAILABLE_COMMANDS
pub struct CommandRouter {
    pub admin: admin::AdminCommand,
    pub claude_agents: claude_agents::ClaudeAgentsCommand,
    pub debug: debug::DebugCommand,
    pub debug_progress: debug_progress::DebugProgressCommand,
    pub help: help::HelpCommand,
    pub rate_limit: rate_limit::RateLimitCommand,
    pub roles: roles::RolesCommand,
    pub security: security::SecurityCommand,
    pub self_update: self_update::SelfUpdateCommand,
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRouter {
    pub fn new() -> Self {
        Self {
            admin: admin::AdminCommand::new(),
            claude_agents: claude_agents::ClaudeAgentsCommand::new(),
            debug: debug::DebugCommand::new(),
            debug_progress: debug_progress::DebugProgressCommand::new(),
            help: help::HelpCommand::new(),
            rate_limit: rate_limit::RateLimitCommand::new(),
            roles: roles::RolesCommand::new(),
            security: security::SecurityCommand::new(),
            self_update: self_update::SelfUpdateCommand::new(),
        }
    }

    /// Route a command to the appropriate handler using static command definitions
    pub async fn route_command(
        &self,
        content: &str,
        msg: &Message,
        ctx: &Context,
        bot: &SpiralConstellationBot,
    ) -> Option<String> {
        // FLOW: Find command match → Route to handler → Execute action
        // 1. Match against static command definitions
        // 2. Route to appropriate handler based on match
        // 3. Handler parses action and executes

        debug!("[CommandRouter] Routing command: {}", content);
        debug!(
            "[CommandRouter] From user: {} ({})",
            msg.author.name, msg.author.id
        );

        let content_lower = content.to_lowercase();

        // Find matching command from static definitions
        for command_info in AVAILABLE_COMMANDS {
            if content_lower.starts_with(&command_info.prefix.to_lowercase()) {
                debug!(
                    "[CommandRouter] Matched command: {} (prefix: {})",
                    command_info.name, command_info.prefix
                );
                debug!(
                    "[CommandRouter] Command requires auth: {}",
                    command_info.requires_auth
                );

                // Route to appropriate handler based on command name
                // 🔄 DRY PATTERN: Command name to handler mapping
                // Critical: This mapping must stay synchronized with AVAILABLE_COMMANDS
                // Audit: Verify all commands in AVAILABLE_COMMANDS have handlers here
                let result = match command_info.name {
                    "admin" => self.admin.handle(content, msg, ctx, bot).await,
                    // 📐 SOLID: Both commands use same handler (DRY principle)
                    "agents" => self.claude_agents.handle(content, msg, ctx, bot).await,
                    "claude-agents" => self.claude_agents.handle(content, msg, ctx, bot).await,
                    "debug" => self.debug.handle(content, msg, ctx, bot).await,
                    "debug progress" => self.debug_progress.handle(content, msg, ctx, bot).await,
                    "help" => self.help.handle(content, msg, ctx, bot).await,
                    "commands" => self.help.handle(content, msg, ctx, bot).await, // Help handles both
                    "ratelimit" => self.rate_limit.handle(content, msg, ctx, bot).await,
                    "roles" => self.roles.handle(content, msg, ctx, bot).await,
                    "security" => self.security.handle(content, msg, ctx, bot).await,
                    "update" => self.self_update.handle(content, msg, ctx, bot).await,
                    _ => {
                        debug!(
                            "[CommandRouter] No handler for command: {}",
                            command_info.name
                        );
                        None
                    }
                };

                debug!(
                    "[CommandRouter] Command result: {}",
                    if result.is_some() {
                        "response generated"
                    } else {
                        "no response"
                    }
                );
                return result;
            }
        }

        debug!("[CommandRouter] No matching command found for: {}", content);
        None
    }
}
