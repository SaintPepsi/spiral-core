use crate::discord::spiral_constellation_bot::SpiralConstellationBot;
use serenity::{model::channel::Message, prelude::Context};

pub mod admin;
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
];

/// Command router for handling all !spiral commands
pub struct CommandRouter {
    pub admin: admin::AdminCommand,
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

        let content_lower = content.to_lowercase();

        // Find matching command from static definitions
        for command_info in AVAILABLE_COMMANDS {
            if content_lower.starts_with(&command_info.prefix.to_lowercase()) {
                // Route to appropriate handler based on command name
                return match command_info.name {
                    "admin" => self.admin.handle(content, msg, ctx, bot).await,
                    "debug" => self.debug.handle(content, msg, ctx, bot).await,
                    "debug progress" => self.debug_progress.handle(content, msg, ctx, bot).await,
                    "help" => self.help.handle(content, msg, ctx, bot).await,
                    "ratelimit" => self.rate_limit.handle(content, msg, ctx, bot).await,
                    "roles" => self.roles.handle(content, msg, ctx, bot).await,
                    "security" => self.security.handle(content, msg, ctx, bot).await,
                    "update" => self.self_update.handle(content, msg, ctx, bot).await,
                    _ => None,
                };
            }
        }

        None
    }
}
