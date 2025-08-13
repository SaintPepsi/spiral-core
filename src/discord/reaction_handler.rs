//! Flexible reaction handler system for Discord bot
//!
//! This module provides a callback-based reaction handling system that follows
//! SOLID principles and promotes code reusability.

#![allow(clippy::type_complexity)]

use serenity::{
    model::{channel::Reaction, user::User},
    prelude::*,
};
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Type alias for reaction callback functions
/// Takes: (context, reaction, user) and returns a future that resolves to Result<(), String>
pub type ReactionCallback = Arc<
    dyn Fn(Context, Reaction, User) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync,
>;

/// Handler for a specific reaction with optional conditions
pub struct ReactionHandler {
    /// The emoji to handle
    pub emoji: String,
    /// Optional check if this handler should process the reaction
    pub condition: Option<Arc<dyn Fn(&Reaction, &User) -> bool + Send + Sync>>,
    /// The callback to execute
    pub callback: ReactionCallback,
    /// Whether authorization is required for this reaction
    pub requires_auth: bool,
    /// Description of what this handler does
    pub description: String,
}

/// Manages reaction handlers for the Discord bot
pub struct ReactionHandlerManager {
    /// Map of emoji to handlers
    handlers: Arc<RwLock<HashMap<String, Vec<ReactionHandler>>>>,
}

impl ReactionHandlerManager {
    /// Create a new reaction handler manager
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a reaction handler
    pub async fn register_handler(&self, emoji: impl Into<String>, handler: ReactionHandler) {
        let emoji = emoji.into();
        info!(
            "[ReactionHandlerManager] Registering handler for emoji '{}': {}",
            emoji, handler.description
        );

        let mut handlers = self.handlers.write().await;
        handlers.entry(emoji).or_insert_with(Vec::new).push(handler);
    }

    /// Register a simple reaction handler without conditions
    pub async fn register_simple(
        &self,
        emoji: impl Into<String>,
        description: impl Into<String>,
        requires_auth: bool,
        callback: impl Fn(Context, Reaction, User) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
            + Send
            + Sync
            + 'static,
    ) {
        let handler = ReactionHandler {
            emoji: emoji.into(),
            condition: None,
            callback: Arc::new(callback),
            requires_auth,
            description: description.into(),
        };

        self.register_handler(handler.emoji.clone(), handler).await;
    }

    /// Register a conditional reaction handler
    pub async fn register_conditional(
        &self,
        emoji: impl Into<String>,
        description: impl Into<String>,
        requires_auth: bool,
        condition: impl Fn(&Reaction, &User) -> bool + Send + Sync + 'static,
        callback: impl Fn(Context, Reaction, User) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
            + Send
            + Sync
            + 'static,
    ) {
        let handler = ReactionHandler {
            emoji: emoji.into(),
            condition: Some(Arc::new(condition)),
            callback: Arc::new(callback),
            requires_auth,
            description: description.into(),
        };

        self.register_handler(handler.emoji.clone(), handler).await;
    }

    /// Handle a reaction
    pub async fn handle_reaction(
        &self,
        ctx: Context,
        reaction: Reaction,
        user: User,
        is_authorized: bool,
    ) -> bool {
        let emoji = match &reaction.emoji {
            serenity::model::channel::ReactionType::Unicode(name) => name.clone(),
            _ => return false, // Skip custom emojis
        };

        debug!(
            "[ReactionHandlerManager] Handling reaction '{}' from user {}",
            emoji, user.id
        );

        let handlers = self.handlers.read().await;
        if let Some(emoji_handlers) = handlers.get(&emoji) {
            for handler in emoji_handlers {
                // Check authorization if required
                if handler.requires_auth && !is_authorized {
                    debug!(
                        "[ReactionHandlerManager] Skipping handler '{}' - user not authorized",
                        handler.description
                    );
                    continue;
                }

                // Check condition if present
                if let Some(condition) = &handler.condition {
                    if !condition(&reaction, &user) {
                        debug!(
                            "[ReactionHandlerManager] Skipping handler '{}' - condition not met",
                            handler.description
                        );
                        continue;
                    }
                }

                // Execute the callback
                info!(
                    "[ReactionHandlerManager] Executing handler '{}' for emoji '{}'",
                    handler.description, emoji
                );

                match (handler.callback)(ctx.clone(), reaction.clone(), user.clone()).await {
                    Ok(()) => {
                        info!(
                            "[ReactionHandlerManager] Handler '{}' completed successfully",
                            handler.description
                        );
                        return true;
                    }
                    Err(e) => {
                        warn!(
                            "[ReactionHandlerManager] Handler '{}' failed: {}",
                            handler.description, e
                        );
                        // Continue to next handler
                    }
                }
            }
        }

        false
    }

    /// Get all registered emojis
    pub async fn registered_emojis(&self) -> Vec<String> {
        let handlers = self.handlers.read().await;
        handlers.keys().cloned().collect()
    }

    /// Clear all handlers
    pub async fn clear(&self) {
        let mut handlers = self.handlers.write().await;
        handlers.clear();
    }
}

impl Default for ReactionHandlerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper functions for creating common callbacks
pub mod callbacks {
    use super::*;

    /// Create a callback that sends a reply to the message
    pub fn reply_callback(
        response: String,
    ) -> impl Fn(Context, Reaction, User) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
    {
        move |ctx, reaction, _user| {
            let response = response.clone();
            Box::pin(async move {
                let message = reaction
                    .message(&ctx.http)
                    .await
                    .map_err(|e| format!("Failed to get message: {e}"))?;

                message
                    .reply(&ctx.http, &response)
                    .await
                    .map_err(|e| format!("Failed to send reply: {e}"))?;

                Ok(())
            })
        }
    }

    /// Create a callback that removes a specific reaction
    pub fn remove_reaction_callback(
        _emoji_to_remove: String,
    ) -> impl Fn(Context, Reaction, User) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
    {
        move |ctx, reaction, user| {
            Box::pin(async move {
                // Get the message first, then remove the reaction
                let message = reaction
                    .message(&ctx.http)
                    .await
                    .map_err(|e| format!("Failed to get message: {e}"))?;

                message
                    .delete_reaction(&ctx.http, Some(user.id), reaction.emoji.clone())
                    .await
                    .map_err(|e| format!("Failed to remove reaction: {e}"))?;

                Ok(())
            })
        }
    }

    /// Create a callback that deletes the message
    pub fn delete_message_callback(
    ) -> impl Fn(Context, Reaction, User) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
    {
        |ctx, reaction, _user| {
            Box::pin(async move {
                let message = reaction
                    .message(&ctx.http)
                    .await
                    .map_err(|e| format!("Failed to get message: {e}"))?;

                message
                    .delete(&ctx.http)
                    .await
                    .map_err(|e| format!("Failed to delete message: {e}"))?;

                Ok(())
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reaction_registration() {
        let manager = ReactionHandlerManager::new();

        manager
            .register_simple(
                "✅",
                "Test approval handler",
                true,
                |_ctx, _reaction, _user| Box::pin(async { Ok(()) }),
            )
            .await;

        let emojis = manager.registered_emojis().await;
        assert!(emojis.contains(&"✅".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_handlers_same_emoji() {
        let manager = ReactionHandlerManager::new();

        // Register two handlers for the same emoji
        manager
            .register_simple("✅", "Handler 1", false, |_ctx, _reaction, _user| {
                Box::pin(async { Ok(()) })
            })
            .await;

        manager
            .register_simple("✅", "Handler 2", false, |_ctx, _reaction, _user| {
                Box::pin(async { Ok(()) })
            })
            .await;

        let handlers = manager.handlers.read().await;
        let check_handlers = handlers.get("✅").unwrap();
        assert_eq!(check_handlers.len(), 2);
    }
}
