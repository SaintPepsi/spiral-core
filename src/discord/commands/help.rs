use super::{get_commands_by_auth, CommandHandler};
use crate::discord::spiral_constellation_bot::SpiralConstellationBot;
use serenity::{model::channel::Message, prelude::Context};
use tracing::info;

pub struct HelpCommand {
    // Help command doesn't need state for now
}

impl Default for HelpCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpCommand {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate comprehensive help information (compact version)
    fn generate_help_content(&self, is_authorized: bool) -> String {
        let mut help_text = String::new();

        help_text.push_str("🌌 **Spiral Core Help**\n\n");

        // Quick start
        help_text.push_str("**Quick Start**\n");
        help_text.push_str("• Mention agents: `@<AgentName> <your request>`\n");
        help_text.push_str("• Join roles: `!spiral roles join <AgentName>`\n\n");

        // Core commands
        help_text.push_str("**Commands**\n");
        help_text.push_str("• `!spiral help` - This help\n");
        help_text.push_str("• `!spiral commands` - Command list\n");
        help_text.push_str("• `!spiral roles join <name>` - Join agent role\n");
        help_text.push_str("• `!spiral roles setup` - Create roles\n");
        help_text.push_str("• `!spiral ratelimit` - Check limits\n\n");

        // Admin commands (only if authorized)
        if is_authorized {
            help_text.push_str("**Admin**\n");
            help_text.push_str("• `!spiral admin` - Dashboard\n");
            help_text.push_str("• `!spiral security stats` - Metrics\n");
            help_text.push_str("• `!spiral debug` - Debug (reply to msg)\n\n");
        }

        // Agent list - dynamically loaded from registry
        // 🏗️ ARCHITECTURE DECISION: Dynamic agent list from registry
        // Why: Single source of truth for available agents
        // Alternative: Hardcoded list (rejected: violates DRY)
        help_text.push_str("**Agent Information**\n");
        help_text.push_str("Agents register dynamically as they become available.\n");
        help_text.push_str("Use `!spiral roles list` to see current agents.\n\n");

        help_text.push_str("Use `!spiral commands` for full list");

        help_text
    }

    /// Generate concise command list from static definitions
    fn generate_commands_list(&self, is_authorized: bool) -> String {
        let mut commands_text = String::new();
        commands_text.push_str("🎮 **Spiral Commands**\n\n");

        // General commands (always available)
        let public_commands = get_commands_by_auth(false);
        if !public_commands.is_empty() {
            commands_text.push_str("**🌐 General Commands**\n");
            for command in public_commands {
                commands_text.push_str(&format!(
                    "• `{}` - {}\n",
                    command.prefix, command.description
                ));
            }
            commands_text.push('\n');
        }

        // Admin commands (only for authorized users)
        if is_authorized {
            let admin_commands = get_commands_by_auth(true);
            if !admin_commands.is_empty() {
                commands_text.push_str("**🔐 Admin Commands**\n");
                for command in admin_commands {
                    commands_text.push_str(&format!(
                        "• `{}` - {}\n",
                        command.prefix, command.description
                    ));
                }
                commands_text.push('\n');
            }
        }

        commands_text.push_str("*Use `!spiral help` for detailed usage information* 💡");
        commands_text
    }
}

impl CommandHandler for HelpCommand {
    async fn handle(
        &self,
        content: &str,
        msg: &Message,
        _ctx: &Context,
        bot: &SpiralConstellationBot,
    ) -> Option<String> {
        const COMMANDS_PREFIX: &str = "!spiral commands";
        const HELP_PREFIX: &str = "!spiral help";

        let content_lower = content.to_lowercase();
        let is_authorized = bot.is_authorized_user(msg.author.id.get());

        // Match command type using const patterns
        match content_lower.as_str() {
            cmd if cmd.starts_with(COMMANDS_PREFIX) => {
                info!(
                    "[HelpCommand] Commands list for user {} ({})",
                    msg.author.name,
                    msg.author.id.get()
                );
                Some(self.generate_commands_list(is_authorized))
            }
            cmd if cmd.starts_with(HELP_PREFIX) || cmd == "help" => {
                info!(
                    "[HelpCommand] Detailed help for user {} ({})",
                    msg.author.name,
                    msg.author.id.get()
                );
                Some(self.generate_help_content(is_authorized))
            }
            _ => None,
        }
    }

    fn command_prefix(&self) -> &str {
        "!spiral help"
    }

    fn description(&self) -> &str {
        "Comprehensive help system with command references and usage examples"
    }
}
