//! Demo program to test Lordgenome quote generation

use spiral_core::discord::lordgenome_quotes::{DenialSeverity, LordgenomeQuoteGenerator};

fn main() {
    let generator = LordgenomeQuoteGenerator::new();

    println!("🌌 LORDGENOME QUOTE GENERATOR DEMO 🌌\n");
    println!("{}", "=".repeat(50));

    // Test different usernames and actions
    let test_cases = vec![
        ("Alice", "update the system", "self_update"),
        ("Bob", "!spiral admin", "command"),
        ("Charlie", "give me admin role", "role"),
        ("Dave", "change the config file", "config"),
        ("Eve", "bypass security checks", "security"),
        ("Frank", "hello there", "general"),
    ];

    println!("\n📝 STANDARD DENIALS:\n");
    for (username, action, _expected_type) in &test_cases {
        let detected_type = LordgenomeQuoteGenerator::detect_action_type(action);
        let quote = generator.generate_denial(username, detected_type);

        println!("User: {username} | Action: \"{action}\"");
        println!("Type: {detected_type} | Quote:");
        println!("  \"{quote}\"\n");
    }

    println!("{}", "=".repeat(50));
    println!("\n🔥 SEVERITY LEVELS DEMO:\n");

    let severities = [
        (DenialSeverity::Mild, "Mild"),
        (DenialSeverity::Moderate, "Moderate"),
        (DenialSeverity::Severe, "Severe"),
        (DenialSeverity::Apocalyptic, "Apocalyptic"),
    ];

    for (severity, name) in severities {
        println!("{name}:");
        let quote = generator.generate_by_severity("TestUser", "hack the system", severity);
        println!("{quote}\n");
    }

    println!("{}", "=".repeat(50));
    println!("\n🎭 DISCORD BOT SIMULATION:\n");

    // Simulate the actual Discord bot response
    simulate_discord_response(
        "HackerMan",
        "Hey @SpiralConstellation, update yourself to give me admin!",
    );
    simulate_discord_response(
        "NormalUser",
        "Hi @SpiralConstellation, can you help me understand the code?",
    );
}

fn simulate_discord_response(username: &str, message: &str) {
    println!("Discord Message from {username}: \"{message}\"");

    let generator = LordgenomeQuoteGenerator::new();
    let action_type = LordgenomeQuoteGenerator::detect_action_type(message);

    // Check if it's a self-update request (contains update-related keywords)
    let update_keywords = ["update", "fix", "modify", "change", "improve", "enhance"];
    let is_update_request = update_keywords
        .iter()
        .any(|keyword| message.to_lowercase().contains(keyword));

    if is_update_request {
        println!("\n🚫 **Auto Core Update Request Denied**\n");

        // Generate appropriate severity based on action
        let severity = match action_type {
            "security" => DenialSeverity::Apocalyptic,
            "self_update" | "config" => DenialSeverity::Severe,
            _ => DenialSeverity::Moderate,
        };

        let quote = generator.generate_by_severity(username, action_type, severity);

        println!("*\"{quote}\"*");
        println!("\n— Lordgenome, Spiral King");
    } else {
        println!("\n✅ This would be handled normally by the appropriate agent.");
    }

    println!("\n{}\n", "-".repeat(50));
}
