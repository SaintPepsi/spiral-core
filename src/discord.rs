use crate::{
    agents::AgentOrchestrator,
    config::DiscordConfig,
    models::{AgentType, DiscordMessage, Priority, Task},
    Result, SpiralError,
};
use regex::Regex;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

pub struct DiscordBot {
    config: DiscordConfig,
    orchestrator: Arc<AgentOrchestrator>,
    mention_regex: Regex,
}

impl DiscordBot {
    pub fn new(config: DiscordConfig, orchestrator: Arc<AgentOrchestrator>) -> Result<Self> {
        let mention_regex =
            Regex::new(&config.agent_mention_pattern).map_err(|e| SpiralError::Agent {
                message: format!("Invalid agent mention regex: {e}"),
            })?;

        Ok(Self {
            config,
            orchestrator,
            mention_regex,
        })
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting Discord bot");

        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let handler = Handler {
            config: self.config.clone(),
            orchestrator: self.orchestrator.clone(),
            mention_regex: self.mention_regex.clone(),
        };

        let mut client = Client::builder(&self.config.token, intents)
            .event_handler(handler)
            .await
            .map_err(|e| SpiralError::Discord(Box::new(e)))?;

        info!("Discord bot connected, starting event loop");
        if let Err(why) = client.start().await {
            error!("Discord client error: {why:?}");
            return Err(SpiralError::Discord(Box::new(why)));
        }

        Ok(())
    }

    fn extract_agent_mentions(&self, content: &str) -> Vec<AgentType> {
        let mut mentions = Vec::new();

        for cap in self.mention_regex.captures_iter(content) {
            if let Some(agent_name) = cap.get(1) {
                if let Some(agent_type) = AgentType::from_mention(agent_name.as_str()) {
                    mentions.push(agent_type);
                }
            }
        }

        mentions
    }

    fn parse_priority(&self, content: &str) -> Priority {
        let content_lower = content.to_lowercase();

        if content_lower.contains("critical") || content_lower.contains("urgent") {
            Priority::Critical
        } else if content_lower.contains("high") || content_lower.contains("important") {
            Priority::High
        } else if content_lower.contains("low") || content_lower.contains("minor") {
            Priority::Low
        } else {
            Priority::Medium
        }
    }

    fn clean_message_content(&self, content: &str) -> String {
        let content = self.mention_regex.replace_all(content, "").to_string();

        let priority_patterns = [
            r"(?i)\b(critical|urgent|high|important|low|minor)\b",
            &format!(r"(?i){}", regex::escape(&self.config.command_prefix)),
        ];

        let mut cleaned = content;
        for pattern in &priority_patterns {
            if let Ok(re) = Regex::new(pattern) {
                cleaned = re.replace_all(&cleaned, "").to_string();
            }
        }

        cleaned.trim().to_string()
    }

    async fn create_tasks_from_message(&self, discord_msg: &DiscordMessage) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();
        let priority = self.parse_priority(&discord_msg.content);
        let cleaned_content = self.clean_message_content(&discord_msg.content);

        if cleaned_content.is_empty() {
            return Err(SpiralError::Agent {
                message: "No actionable content found in message".to_string(),
            });
        }

        if discord_msg.mentioned_agents.is_empty() {
            tasks.push(
                Task::new(AgentType::SoftwareDeveloper, cleaned_content, priority)
                    .with_context(
                        "discord_channel_id".to_string(),
                        discord_msg.channel_id.to_string(),
                    )
                    .with_context(
                        "discord_message_id".to_string(),
                        discord_msg.message_id.to_string(),
                    )
                    .with_context(
                        "discord_author_id".to_string(),
                        discord_msg.author_id.to_string(),
                    ),
            );
        } else {
            for agent_type in &discord_msg.mentioned_agents {
                let task = Task::new(
                    agent_type.clone(),
                    cleaned_content.clone(),
                    priority.clone(),
                )
                .with_context(
                    "discord_channel_id".to_string(),
                    discord_msg.channel_id.to_string(),
                )
                .with_context(
                    "discord_message_id".to_string(),
                    discord_msg.message_id.to_string(),
                )
                .with_context(
                    "discord_author_id".to_string(),
                    discord_msg.author_id.to_string(),
                );

                tasks.push(task);
            }
        }

        Ok(tasks)
    }
}

struct Handler {
    config: DiscordConfig,
    orchestrator: Arc<AgentOrchestrator>,
    mention_regex: Regex,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let should_process = msg.content.starts_with(&self.config.command_prefix)
            || self.mention_regex.is_match(&msg.content)
            || msg.guild_id.is_none();

        if !should_process {
            return;
        }

        debug!("Processing Discord message: {}", msg.id);

        let bot = DiscordBot {
            config: self.config.clone(),
            orchestrator: self.orchestrator.clone(),
            mention_regex: self.mention_regex.clone(),
        };

        let mentioned_agents = bot.extract_agent_mentions(&msg.content);

        let discord_msg = DiscordMessage {
            content: msg.content.clone(),
            author_id: msg.author.id.get(),
            channel_id: msg.channel_id.get(),
            mentioned_agents,
            message_id: msg.id.get(),
        };

        match bot.create_tasks_from_message(&discord_msg).await {
            Ok(tasks) => {
                let mut task_ids = Vec::new();

                for task in tasks {
                    match self.orchestrator.submit_task(task.clone()).await {
                        Ok(task_id) => {
                            task_ids.push(task_id);
                            info!("Submitted task {} for agent {:?}", task.id, task.agent_type);
                        }
                        Err(e) => {
                            error!("Failed to submit task: {}", e);
                            if let Err(send_err) = msg
                                .reply(&ctx.http, format!("❌ Failed to submit task: {e}"))
                                .await
                            {
                                error!("Failed to send error reply: {}", send_err);
                            }
                            continue;
                        }
                    }
                }

                if !task_ids.is_empty() {
                    let response = if task_ids.len() == 1 {
                        format!("🚀 Task submitted: `{}`", task_ids[0])
                    } else {
                        format!(
                            "🚀 {} tasks submitted: {}",
                            task_ids.len(),
                            task_ids
                                .iter()
                                .map(|id| format!(
                                    "`{}`",
                                    &id[..crate::constants::DISCORD_TASK_ID_DISPLAY_LENGTH]
                                ))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    };

                    if let Err(e) = msg.reply(&ctx.http, response).await {
                        error!("Failed to send confirmation reply: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create tasks from message: {}", e);
                if let Err(send_err) = msg
                    .reply(
                        &ctx.http,
                        "❌ Unable to process your request. Please check your message format.",
                    )
                    .await
                {
                    error!("Failed to send error reply: {}", send_err);
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("Discord bot {} is connected and ready!", ready.user.name);

        let status_summary = self.orchestrator.get_all_agent_statuses().await;
        info!(
            "Available agents: {:?}",
            status_summary.keys().collect::<Vec<_>>()
        );
    }
}
