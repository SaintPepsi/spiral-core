use super::CommandHandler;
use crate::discord::{
    agent_registry::get_agent_registry, spiral_constellation_bot::SpiralConstellationBot,
};
use serenity::{
    model::{channel::Message, guild::Role, id::GuildId, permissions::Permissions},
    prelude::Context,
};
use tracing::{error, info, warn};

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

    /// Create agent roles in a Discord server (only if they don't exist)
    async fn create_agent_roles(
        &self,
        ctx: &Context,
        guild_id: GuildId,
    ) -> Result<Vec<Role>, String> {
        // 🏗️ ARCHITECTURE DECISION: Get personas from registry
        // Why: Single source of truth for agent definitions
        // Alternative: Local definitions (rejected: violates DRY)
        let personas = get_agent_registry().get_available_agents().await;
        let mut created_roles = Vec::new();
        let mut skipped_roles = Vec::new();

        // 🔍 AUDIT CHECKPOINT: Check existing roles first
        let existing_roles = match guild_id.roles(&ctx.http).await {
            Ok(roles) => roles,
            Err(e) => {
                error!("[RolesCommand] Failed to fetch existing roles: {}", e);
                return Err(format!("Failed to fetch existing roles: {e}"));
            }
        };

        for persona in personas {
            // Check if role already exists
            let role_exists = existing_roles
                .values()
                .any(|r| r.name.to_lowercase() == persona.name.to_lowercase());

            if role_exists {
                info!(
                    "[RolesCommand] Role {} already exists, skipping creation",
                    persona.name
                );
                skipped_roles.push(persona.name.clone());
                continue;
            }

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

        // Remove roles for agents that are no longer registered/available
        let all_agents = get_agent_registry().get_all_agents().await;
        let mut removed_roles = Vec::new();

        for role in existing_roles.values() {
            // Check if this is a Spiral role that's not in our registry
            if role.name.starts_with("Spiral") {
                let is_registered = all_agents.iter().any(|a| a.name == role.name);
                if !is_registered && role.name != "SpiralConstellation" && role.name != "Spiral" {
                    info!(
                        "[RolesCommand] Removing unregistered agent role: {}",
                        role.name
                    );
                    match guild_id.delete_role(&ctx.http, role.id).await {
                        Ok(_) => removed_roles.push(role.name.clone()),
                        Err(e) => {
                            error!("[RolesCommand] Failed to remove role {}: {}", role.name, e)
                        }
                    }
                }
            }
        }

        if !skipped_roles.is_empty() {
            info!("[RolesCommand] Skipped existing roles: {:?}", skipped_roles);
        }
        if !removed_roles.is_empty() {
            info!(
                "[RolesCommand] Removed unavailable agent roles: {:?}",
                removed_roles
            );
        }

        Ok(created_roles)
    }

    /// 🛡️ SECURITY DECISION: Clean up unauthorized Spiral roles
    /// Why: Prevent role pollution and confusion
    /// Audit: Only removes single-word Spiral* roles not in reserved list
    async fn cleanup_unauthorized_spiral_roles(
        &self,
        ctx: &Context,
        guild_id: GuildId,
    ) -> Result<(usize, Vec<String>), String> {
        let reserved_names = get_agent_registry().get_reserved_names().await;
        let mut removed_count = 0;
        let mut removed_names = Vec::new();

        // Fetch all guild roles
        let roles = match guild_id.roles(&ctx.http).await {
            Ok(roles) => roles,
            Err(e) => {
                error!("[RolesCommand] Failed to fetch roles for cleanup: {}", e);
                return Err(format!("Failed to fetch server roles: {e}"));
            }
        };

        // Identify unauthorized Spiral roles
        for role in roles.values() {
            // Check if role starts with "Spiral" (case-insensitive)
            if !role.name.to_lowercase().starts_with("spiral") {
                continue;
            }

            // Check if it's a single word (no spaces after "Spiral")
            let has_spaces = role.name.contains(' ');
            if has_spaces {
                continue; // Multi-word roles like "Spiral Team Member" are allowed
            }

            // Check if it's in the reserved list
            if reserved_names.contains(&role.name.to_lowercase()) {
                continue; // This is an official role
            }

            // This is an unauthorized single-word Spiral role
            warn!(
                "[RolesCommand] Removing unauthorized role: {} (ID: {})",
                role.name,
                role.id.get()
            );

            // Delete the role
            match guild_id.delete_role(&ctx.http, role.id).await {
                Ok(_) => {
                    info!("[RolesCommand] Successfully removed role: {}", role.name);
                    removed_names.push(role.name.clone());
                    removed_count += 1;
                }
                Err(e) => {
                    error!("[RolesCommand] Failed to remove role {}: {}", role.name, e);
                    // Continue with other roles even if one fails
                }
            }
        }

        Ok((removed_count, removed_names))
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
        const ROLES_CLEANUP: &str = "!spiral roles cleanup";
        const ROLES_LIST: &str = "!spiral roles list";

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
                        if roles.is_empty() {
                            Some("ℹ️ **No new roles created**\n\nAll available agent roles already exist or no agents are currently registered.".to_string())
                        } else {
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
                            Some(format!(
                                "🌌 **SpiralConstellation Setup Complete!**\n\n\
                                Created {role_count} agent persona role(s):\n{role_list}\n\n\
                                *Roles are created dynamically as agents become available.*"
                            ))
                        }
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
            cmd if cmd.starts_with(ROLES_CLEANUP) => {
                // 🔍 AUDIT CHECKPOINT: Admin-only operation
                let guild_id = match msg.guild_id {
                    Some(id) => id,
                    None => {
                        return Some(
                            "❌ Role cleanup only works in servers, not direct messages."
                                .to_string(),
                        );
                    }
                };

                info!(
                    "[RolesCommand] Cleaning up unauthorized roles for guild {} by user {} ({})",
                    guild_id.get(),
                    msg.author.name,
                    msg.author.id.get()
                );

                match self.cleanup_unauthorized_spiral_roles(ctx, guild_id).await {
                    Ok((count, names)) => {
                        if count == 0 {
                            Some("✅ **No unauthorized Spiral roles found!**\n\nAll existing Spiral roles are authorized constellation agents.".to_string())
                        } else {
                            let names_list = names.join(", ");
                            Some(format!(
                                "🧹 **Role Cleanup Complete!**\n\n\
                                Removed {count} unauthorized Spiral role(s):\n• {names_list}\n\n\
                                *Only official constellation agent roles are preserved.*"
                            ))
                        }
                    }
                    Err(e) => Some(format!("❌ **Cleanup Failed**\n\n{e}")),
                }
            }
            cmd if cmd.starts_with(ROLES_LIST) => {
                let guild_id = match msg.guild_id {
                    Some(id) => id,
                    None => {
                        return Some(
                            "❌ Role listing only works in servers, not direct messages."
                                .to_string(),
                        );
                    }
                };

                // List all Spiral roles
                match guild_id.roles(&ctx.http).await {
                    Ok(roles) => {
                        let mut official_roles = Vec::new();
                        let mut unauthorized_roles = Vec::new();
                        let mut custom_roles = Vec::new();

                        for role in roles.values() {
                            if !role.name.to_lowercase().starts_with("spiral") {
                                continue;
                            }

                            let is_single_word = !role.name.contains(' ');
                            let is_reserved = get_agent_registry().is_reserved(&role.name).await;

                            if is_reserved {
                                official_roles.push(format!("• {} (Official)", role.name));
                            } else if is_single_word {
                                unauthorized_roles.push(format!(
                                    "• {} ⚠️ (Unauthorized - will be removed)",
                                    role.name
                                ));
                            } else {
                                custom_roles.push(format!("• {} (Custom - allowed)", role.name));
                            }
                        }

                        let mut response = "**📋 Spiral Roles Status**\n\n".to_string();

                        if !official_roles.is_empty() {
                            response.push_str("**Official Agent Roles:**\n");
                            response.push_str(&official_roles.join("\n"));
                            response.push_str("\n\n");
                        }

                        if !custom_roles.is_empty() {
                            response.push_str("**Custom Roles (Allowed):**\n");
                            response.push_str(&custom_roles.join("\n"));
                            response.push_str("\n\n");
                        }

                        if !unauthorized_roles.is_empty() {
                            response.push_str("**Unauthorized Roles:**\n");
                            response.push_str(&unauthorized_roles.join("\n"));
                            response.push_str(
                                "\n\n*Run `!spiral roles cleanup` to remove unauthorized roles*\n",
                            );
                        }

                        if official_roles.is_empty()
                            && custom_roles.is_empty()
                            && unauthorized_roles.is_empty()
                        {
                            response.push_str(
                                "No Spiral roles found. Use `!spiral roles setup` to create them.",
                            );
                        }

                        Some(response)
                    }
                    Err(e) => {
                        error!("[RolesCommand] Failed to list roles: {}", e);
                        Some(format!("❌ Failed to fetch roles: {e}"))
                    }
                }
            }
            "!spiral roles" => {
                // Show help when just "!spiral roles" is called
                Some(
                    "**🌌 Spiral Roles Management**\n\n\
                    **Available commands:**\n\
                    • `!spiral roles setup` - Create all agent roles in the server\n\
                    • `!spiral roles join <name>` - Join an agent role\n\
                    • `!spiral roles list` - List all Spiral roles and their status\n\
                    • `!spiral roles cleanup` - Remove unauthorized Spiral roles\n\n\
                    *Roles are managed dynamically by the agent registry.*\n\
                    *Single-word Spiral roles are reserved for official agents only.*"
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
