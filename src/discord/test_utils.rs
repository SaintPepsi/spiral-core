/// 🧪 TEST UTILITIES: Proper mock message creation with realistic IDs
/// Purpose: Centralized test utilities to ensure consistent, realistic test data
/// CRITICAL: Use non-zero, realistic IDs to properly test integration paths
use serenity::model::prelude::*;

/// Generate a realistic Discord snowflake ID based on timestamp
/// Discord IDs are snowflakes: https://discord.com/developers/docs/reference#snowflakes
fn generate_snowflake(timestamp_offset_ms: u64) -> u64 {
    // Discord epoch: 2015-01-01 00:00:00 UTC = 1420070400000 ms
    const DISCORD_EPOCH: u64 = 1420070400000;

    // Current time in ms since Unix epoch
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Timestamp relative to Discord epoch
    let discord_timestamp = (now_ms - DISCORD_EPOCH + timestamp_offset_ms) << 22;

    // Add some worker/process/increment bits for uniqueness
    let worker_id = (timestamp_offset_ms % 32) << 17;
    let process_id = (timestamp_offset_ms % 32) << 12;
    let increment = timestamp_offset_ms % 4096;

    discord_timestamp | worker_id | process_id | increment
}

/// Create a realistic test message with proper snowflake IDs
pub fn create_test_message(content: &str, author_bot: bool) -> Message {
    create_test_message_with_offset(content, author_bot, 0)
}

/// Create a test message with timestamp offset for unique IDs
pub fn create_test_message_with_offset(content: &str, author_bot: bool, offset_ms: u64) -> Message {
    // Generate unique snowflake IDs
    let message_id = generate_snowflake(offset_ms);
    let channel_id = generate_snowflake(offset_ms + 1000);
    let guild_id = generate_snowflake(offset_ms + 2000);
    let user_id = generate_snowflake(offset_ms + 3000);

    // Build message using available builder pattern or struct fields
    // This is a simplified version that should work with most Serenity versions
    let mut message = Message::default();

    message.id = MessageId::new(message_id);
    message.channel_id = ChannelId::new(channel_id);
    message.guild_id = Some(GuildId::new(guild_id));
    message.content = content.to_string();
    message.timestamp = Timestamp::now();
    message.kind = MessageType::Regular;

    // Create author
    let mut author = User::default();
    author.id = UserId::new(user_id);
    author.name = if author_bot { "test_bot" } else { "test_user" }.to_string();
    author.bot = author_bot;

    message.author = author;

    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snowflake_generation() {
        let id1 = generate_snowflake(0);
        let id2 = generate_snowflake(1);
        let id3 = generate_snowflake(1000);

        // IDs should be unique
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);

        // IDs should be non-zero
        assert_ne!(id1, 0);
        assert_ne!(id2, 0);
        assert_ne!(id3, 0);

        // IDs should be large (Discord snowflakes are 64-bit)
        assert!(id1 > 1_000_000_000_000_000); // Roughly 2015+ timestamp
    }

    #[test]
    fn test_message_creation() {
        // Use different offsets to ensure unique IDs
        let msg1 = create_test_message_with_offset("Hello", false, 0);
        let msg2 = create_test_message_with_offset("World", true, 100);
        let msg3 = create_test_message_with_offset("Test", false, 5000);

        // All IDs should be unique
        assert_ne!(msg1.id, msg2.id);
        assert_ne!(msg2.id, msg3.id);
        assert_ne!(msg1.channel_id, msg2.channel_id);

        // All IDs should be non-zero
        assert_ne!(msg1.id.get(), 0);
        assert_ne!(msg1.channel_id.get(), 0);
        assert_ne!(msg1.guild_id.unwrap().get(), 0);
        assert_ne!(msg1.author.id.get(), 0);

        // Content and bot status should be correct
        assert_eq!(msg1.content, "Hello");
        assert_eq!(msg2.content, "World");
        assert!(!msg1.author.bot);
        assert!(msg2.author.bot);
    }
}
