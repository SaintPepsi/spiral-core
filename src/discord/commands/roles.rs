use super::CommandHandler;
use crate::discord::spiral_constellation_bot::SpiralConstellationBot;
use serenity::{
    model::{channel::Message, guild::Role, id::GuildId, permissions::Permissions, Colour},
    prelude::Context,
};
use tracing::{error, info};

pub struct RolesCommand {
    // Roles command doesn't need state for now
}

impl Default for RolesCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl RolesCommand {
    pub fn new() -> Self {
        Self {}
    }

    /// Agent persona definitions for role creation
    fn get_agent_personas() -> Vec<AgentPersona> {
        vec![
            AgentPersona {
                name: "SpiralDev".to_string(),
                emoji: "💻",
                color: Colour::from_rgb(0, 162, 232), // Blue
                description: "Code generation and development".to_string(),
            },
            AgentPersona {
                name: "SpiralPM".to_string(),
                emoji: "📋",
                color: Colour::from_rgb(34, 139, 34), // Green
                description: "Project management and planning".to_string(),
            },
            AgentPersona {
                name: "SpiralQA".to_string(),
                emoji: "🔍",
                color: Colour::from_rgb(255, 140, 0), // Orange
                description: "Quality assurance and testing".to_string(),
            },
            AgentPersona {
                name: "SpiralKing".to_string(),
                emoji: "👑",
                color: Colour::from_rgb(218, 165, 32), // Gold
                description: "Leadership and decision making".to_string(),
            },
            AgentPersona {
                name: "SpiralDecide".to_string(),
                emoji: "⚖️",
                color: Colour::from_rgb(128, 0, 128), // Purple
                description: "Analysis and recommendations".to_string(),
            },
            AgentPersona {
                name: "SpiralCreate".to_string(),
                emoji: "🎨",
                color: Colour::from_rgb(255, 20, 147), // Pink
                description: "Creative solutions and innovation".to_string(),
            },
            AgentPersona {
                name: "SpiralCoach".to_string(),
                emoji: "🏃",
                color: Colour::from_rgb(220, 20, 60), // Crimson
                description: "Process optimization and guidance".to_string(),
            },
        ]
    }

    /// Create agent roles in a Discord server
    async fn create_agent_roles(
        &self,
        ctx: &Context,
        guild_id: GuildId,
    ) -> Result<Vec<Role>, String> {
        let personas = Self::get_agent_personas();
        let mut created_roles = Vec::new();

        for persona in personas {
            use serenity::builder::EditRole;
            let role_data = EditRole::default()
                .name(&persona.name)
                .colour(persona.color.0)
                .mentionable(true)
                .permissions(Permissions::empty());

            match guild_id.create_role(&ctx.http, role_data).await {
                Ok(role) => {
                    info!(
                        "[RolesCommand] Created role: {} ({})",
                        persona.name,
                        role.id.get()
                    );
                    created_roles.push(role);
                }
                Err(e) => {
                    error!(
                        "[RolesCommand] Failed to create role {}: {}",
                        persona.name, e
                    );
                    let persona_name = &persona.name;
                    return Err(format!("Failed to create role '{persona_name}': {e}"));
                }
            }
        }

        Ok(created_roles)
    }

    /// Assign a role to a user
    async fn assign_role(&self, ctx: &Context, msg: &Message, role_name: &str) -> Option<String> {
        let guild_id = match msg.guild_id {
            Some(id) => id,
            None => {
                return Some(
                    "❌ Role assignment only works in servers, not direct messages.".to_string(),
                )
            }
        };

        // Normalize role name (add Spiral prefix if missing)
        let normalized_role_name = if role_name.to_lowercase().starts_with("spiral") {
            role_name.to_string()
        } else {
            format!("Spiral{role_name}")
        };

        // Find the role
        let roles = match guild_id.roles(&ctx.http).await {
            Ok(roles) => roles,
            Err(e) => {
                error!("[RolesCommand] Failed to fetch server roles: {}", e);
                return Some("❌ Failed to fetch server roles. Please try again.".to_string());
            }
        };

        let target_role = roles
            .values()
            .find(|role| role.name.to_lowercase() == normalized_role_name.to_lowercase());

        let role = match target_role {
            Some(role) => role,
            None => {
                let available_roles: Vec<&str> = roles
                    .values()
                    .filter(|r| r.name.starts_with("Spiral"))
                    .map(|r| r.name.as_str())
                    .collect();

                if available_roles.is_empty() {
                    return Some(
                        "❌ No Spiral agent roles found. Use `!spiral setup roles` to create them first.".to_string()
                    );
                } else {
                    return Some(format!(
                        "❌ Role '{}' not found.\n\n**Available roles:**\n{}\n\n*Use exact role names*",
                        normalized_role_name,
                        available_roles.join(", ")
                    ));
                }
            }
        };

        // Get guild member
        let member = match guild_id.member(&ctx.http, msg.author.id).await {
            Ok(member) => member,
            Err(e) => {
                error!("[RolesCommand] Failed to fetch member: {}", e);
                return Some(
                    "❌ Failed to fetch member information. Please try again.".to_string(),
                );
            }
        };

        // Check if user already has the role
        if member.roles.contains(&role.id) {
            let role_name = &role.name;
            return Some(format!("ℹ️ You already have the **{role_name}** role!"));
        }

        // Assign the role
        match member.add_role(&ctx.http, role.id).await {
            Ok(_) => {
                info!(
                    "[RolesCommand] Assigned role {} to user {} ({})",
                    role.name,
                    msg.author.name,
                    msg.author.id.get()
                );

                let role_name_assigned = &role.name;
                Some(format!(
                    "✅ **Role Assigned!**\n\nYou now have the **{role_name_assigned}** role!\n\n\
                    You can now:\n\
                    • Be mentioned with agent tasks\n\
                    • Access role-specific channels (if configured)\n\
                    • Represent this agent persona in discussions\n\n\
                    *Welcome to the Spiral team!* 🌌"
                ))
            }
            Err(e) => {
                error!("[RolesCommand] Failed to assign role: {}", e);
                Some(format!("❌ Failed to assign role: {e}"))
            }
        }
    }
}

/// Agent persona data structure
#[derive(Debug, Clone)]
struct AgentPersona {
    name: String,
    #[allow(dead_code)] // Used for future UI features
    emoji: &'static str,
    color: Colour,
    #[allow(dead_code)] // Used for future UI features
    description: String,
}

impl CommandHandler for RolesCommand {
    async fn handle(
        &self,
        content: &str,
        msg: &Message,
        ctx: &Context,
        _bot: &SpiralConstellationBot,
    ) -> Option<String> {
        // FLOW: Parse action → Validate context → Execute → Respond
        // 1. Parse roles action (setup, join, list, etc.)
        // 2. Validate Discord server context
        // 3. Execute the role operation
        // 4. Return formatted response

        const ROLES_SETUP: &str = "!spiral roles setup";
        const ROLES_JOIN: &str = "!spiral roles join ";

        let content_lower = content.to_lowercase();

        // Match roles command type using const patterns
        match content_lower.as_str() {
            cmd if cmd.starts_with(ROLES_SETUP) => {
                let guild_id = match msg.guild_id {
                    Some(id) => id,
                    None => {
                        return Some(
                            "❌ Role setup only works in servers, not direct messages.".to_string(),
                        );
                    }
                };

                info!(
                    "[RolesCommand] Creating agent roles for guild {} by user {} ({})",
                    guild_id.get(),
                    msg.author.name,
                    msg.author.id.get()
                );

                match self.create_agent_roles(ctx, guild_id).await {
                    Ok(roles) => {
                        let role_list = roles
                            .iter()
                            .map(|r| {
                                let role_id = r.id;
                                let role_name = &r.name;
                                format!("• <@&{role_id}> ({role_name})")
                            })
                            .collect::<Vec<_>>()
                            .join("\n");

                        let role_count = roles.len();
                        let first_role_id = roles.first().map(|r| r.id.to_string()).unwrap_or_default();
                        Some(format!(
                            "🌌 **SpiralConstellation Setup Complete!**\n\n\
                            Created {role_count} agent persona roles:\n{role_list}\n\n\
                            **Usage:**\n\
                            • Mention roles directly: <@&{first_role_id}> help me with code\n\
                            • Text mentions: @SpiralDev create a function\n\
                            • Get a role: !spiral roles join SpiralDev\n\n\
                            *All roles are mentionable and color-coded!* ✨"
                        ))
                    }
                    Err(e) => {
                        Some(format!("❌ **Role Creation Failed**\n\n{e}\n\n**Common Issues:**\n• Bot needs 'Manage Roles' permission\n• Check bot role hierarchy\n• Verify server permissions"))
                    }
                }
            }
            cmd if cmd.starts_with(ROLES_JOIN) => {
                let role_name = content_lower.strip_prefix(ROLES_JOIN).unwrap_or("").trim();

                if role_name.is_empty() {
                    return Some(
                        "❌ Usage: `!spiral roles join <role_name>`\n\nExample: `!spiral roles join SpiralDev`"
                            .to_string(),
                    );
                }

                info!(
                    "[RolesCommand] User {} ({}) joining role: {}",
                    msg.author.name,
                    msg.author.id.get(),
                    role_name
                );

                self.assign_role(ctx, msg, role_name).await
            }
            "!spiral roles" => {
                // Show help when just "!spiral roles" is called
                Some(
                    "**🌌 Spiral Roles Management**\n\n\
                    **Available commands:**\n\
                    • `!spiral roles setup` - Create all agent roles in the server\n\
                    • `!spiral roles join <name>` - Join an agent role\n\n\
                    **Example:** `!spiral roles join SpiralDev`\n\n\
                    **Available agents:**\n\
                    • SpiralDev - Code generation and development\n\
                    • SpiralPM - Project management and planning\n\
                    • SpiralQA - Quality assurance and testing\n\
                    • SpiralKing - Leadership and decision making\n\
                    • SpiralDecide - Analysis and recommendations\n\
                    • SpiralCreate - Creative solutions and innovation\n\
                    • SpiralCoach - Process optimization and guidance"
                        .to_string(),
                )
            }
            _ => None,
        }
    }

    fn command_prefix(&self) -> &str {
        "!spiral roles"
    }

    fn description(&self) -> &str {
        "Manage Discord agent roles - create roles and assign them to users"
    }
}
