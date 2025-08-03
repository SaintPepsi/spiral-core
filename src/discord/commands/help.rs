use super::{CommandHandler, get_commands_by_auth};
use crate::discord::spiral_constellation_bot::SpiralConstellationBot;
use serenity::{model::channel::Message, prelude::Context};
use tracing::info;

pub struct HelpCommand {
    // Help command doesn't need state for now
}

impl HelpCommand {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate comprehensive help information
    fn generate_help_content(&self, is_authorized: bool) -> String {
        let mut help_text = String::new();

        help_text.push_str("🌌 **SpiralConstellation Help Center**\n\n");

        // Basic usage section
        help_text.push_str("**🚀 Quick Start**\n");
        help_text.push_str("• Mention agents: @SpiralDev create a function\n");
        help_text.push_str("• Join agent roles: `!spiral join SpiralDev`\n");
        help_text.push_str("• Get agent role: `!spiral join SpiralKing`\n");
        help_text.push_str("• Setup server roles: `!spiral setup roles`\n\n");

        // Core commands
        help_text.push_str("**🎮 Core Commands**\n");
        help_text.push_str("• `!spiral help` - Show this detailed help\n");
        help_text.push_str("• `!spiral commands` - Show concise command list\n");
        help_text.push_str("• `!spiral join <role>` - Join an agent role\n");
        help_text.push_str("• `!spiral setup roles` - Create agent roles\n");
        help_text.push_str("• `!spiral ratelimit` - Check your rate limit status\n");
        help_text.push_str("• `!spiral update help` - Learn about the self-update system\n\n");

        // Authorization-specific commands
        if is_authorized {
            help_text.push_str("**🔐 Admin Commands** (Authorized Access)\n");
            help_text.push_str("• `!spiral admin` - Admin dashboard with system overview\n");
            help_text.push_str("• `!spiral security stats` - View security metrics\n");
            help_text.push_str("• `!spiral security reset` - Reset security metrics\n");
            help_text.push_str("• `!spiral security report` - Generate security report\n");
            help_text
                .push_str("• `!spiral debug` - Debug any issue (reply to problematic message)\n");
            help_text.push_str("• `!spiral ratelimit @user` - Check user's rate limit\n");
            help_text.push_str("• `!spiral reset ratelimit @user` - Reset user's rate limit\n\n");
        }

        // Agent personas section
        help_text.push_str("**🤖 Available Agent Personas**\n");
        help_text.push_str("• **SpiralDev** 💻 - Code generation and development\n");
        help_text.push_str("• **SpiralPM** 📋 - Project management and planning\n");
        help_text.push_str("• **SpiralQA** 🔍 - Quality assurance and testing\n");
        help_text.push_str("• **SpiralKing** 👑 - Leadership and decision making\n");
        help_text.push_str("• **SpiralDecide** ⚖️ - Analysis and recommendations\n");
        help_text.push_str("• **SpiralCreate** 🎨 - Creative solutions and innovation\n");
        help_text.push_str("• **SpiralCoach** 🏃 - Process optimization and guidance\n\n");

        // Usage examples
        help_text.push_str("**💡 Usage Examples**\n");
        help_text.push_str("• `@SpiralDev create a REST API for user management`\n");
        help_text.push_str("• `@SpiralPM help me plan this feature`\n");
        help_text.push_str("• `@SpiralQA review this code for issues`\n");
        help_text.push_str("• `!spiral join SpiralDev` (to get the developer role)\n\n");

        // System information
        help_text.push_str("**ℹ️ System Information**\n");
        help_text.push_str("• **Security**: Universal authorization required for all commands\n");
        help_text.push_str("• **Rate Limiting**: Active to prevent abuse\n");
        help_text.push_str("• **Privacy**: Messages processed securely\n");
        help_text.push_str("• **Logging**: Security events are logged for audit\n\n");

        // Footer
        help_text.push_str("**🆘 Need More Help?**\n");
        help_text.push_str("• Use `!spiral commands` for a quick command reference\n");
        help_text.push_str("• Check the project documentation for detailed guides\n");
        help_text.push_str("• Report issues through the proper channels\n\n");

        help_text.push_str("*Spiral Core - AI Agent Orchestration System* 🌌");

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
                commands_text.push_str(&format!("• `{}` - {}\n", command.prefix, command.description));
            }
            commands_text.push_str("\n");
        }

        // Admin commands (only for authorized users)
        if is_authorized {
            let admin_commands = get_commands_by_auth(true);
            if !admin_commands.is_empty() {
                commands_text.push_str("**🔐 Admin Commands**\n");
                for command in admin_commands {
                    commands_text.push_str(&format!("• `{}` - {}\n", command.prefix, command.description));
                }
                commands_text.push_str("\n");
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
                    msg.author.name, msg.author.id.get()
                );
                Some(self.generate_commands_list(is_authorized))
            }
            cmd if cmd.starts_with(HELP_PREFIX) || cmd == "help" => {
                info!(
                    "[HelpCommand] Detailed help for user {} ({})",
                    msg.author.name, msg.author.id.get()
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
