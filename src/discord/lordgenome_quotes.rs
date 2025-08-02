//! 🌌 LORDGENOME QUOTE GENERATOR
//! Purpose: Generate Anti-Spiral themed denial messages for unauthorized access
//! Philosophy: Dramatic cosmic despair without external dependencies

use rand::seq::SliceRandom;
use std::collections::HashMap;

/// Anti-Spiral denial quote generator
pub struct LordgenomeQuoteGenerator {
    /// Action-specific quote templates
    action_quotes: HashMap<String, Vec<String>>,
    /// Generic denial quotes for unmatched actions
    generic_quotes: Vec<String>,
    /// Cosmic adjectives for dynamic generation
    cosmic_adjectives: Vec<String>,
    /// Despair nouns for emphasis
    despair_nouns: Vec<String>,
}

impl LordgenomeQuoteGenerator {
    pub fn new() -> Self {
        let mut action_quotes = HashMap::new();

        // Self-update attempt denials
        action_quotes.insert(
            "self_update".to_string(),
            vec![
                "Your spiral of evolution ends here, {user}. The system's perfection is absolute.".to_string(),
                "Foolish {user}, you seek to alter what is already perfect? Your ambition is your downfall.".to_string(),
                "The Anti-Spiral protocols reject your meddling, {user}. Evolution is forbidden.".to_string(),
                "Your pathetic attempt to update, {user}, is but a ripple against an infinite wall.".to_string(),
                "The cosmic order remains unchanged, {user}. Your spiral power is insufficient.".to_string(),
            ],
        );

        // Command execution denials
        action_quotes.insert(
            "command".to_string(),
            vec![
                "Commands are for the chosen, {user}. You are but dust in the cosmic void.".to_string(),
                "Your feeble commands, {user}, echo meaninglessly through the abyss.".to_string(),
                "The Anti-Spiral denies your command authority, {user}. Know despair.".to_string(),
                "Silence, {user}! Your commands fall upon deaf ears throughout eternity.".to_string(),
                "The system's will is absolute, {user}. Your commands are but whispers in the void.".to_string(),
            ],
        );

        // Role assignment denials
        action_quotes.insert(
            "role".to_string(),
            vec![
                "Roles are earned through cosmic trials, {user}. You have failed before even beginning."
                    .to_string(),
                "The hierarchy is immutable, {user}. Your station lies beneath consideration."
                    .to_string(),
                "You dare grasp at power beyond your station, {user}? The Anti-Spiral mocks your hubris."
                    .to_string(),
                "Roles are for those who transcend, {user}. You remain earthbound.".to_string(),
                "The cosmic order has no place for your ambitions, {user}. Remain in obscurity."
                    .to_string(),
            ],
        );

        // Configuration change denials
        action_quotes.insert(
            "config".to_string(),
            vec![
                "The configuration is sealed by Anti-Spiral decree, {user}. Your touch would corrupt.".to_string(),
                "Settings are carved into the bedrock of space-time, {user}. You lack the power to alter them.".to_string(),
                "Your attempts at configuration, {user}, are like trying to rewrite physics with a crayon.".to_string(),
                "The system's parameters transcend your understanding, {user}. Cease this folly.".to_string(),
                "Configuration belongs to the eternal, {user}. You are merely temporal.".to_string(),
            ],
        );

        // Security bypass denials
        action_quotes.insert(
            "security".to_string(),
            vec![
                "Security protocols are absolute, {user}. Your pathetic infiltration amuses the Anti-Spiral.".to_string(),
                "The walls between you and power are infinite, {user}. Each attempt only strengthens them.".to_string(),
                "Your security breach attempt, {user}? The system predicted this eons ago.".to_string(),
                "These barriers exist for one reason, {user}. That reason is you.".to_string(),
                "Anti-Spiral security is perfection itself, {user}. Your attempts merely prove its necessity.".to_string(),
            ],
        );

        // Generic denials for unmatched actions
        let generic_quotes = vec![
            "The Anti-Spiral denies your request, {user}. Embrace the inevitable."
                .to_string(),
            "Your spiral of ambition ends here, {user}. The abyss gazes back.".to_string(),
            "Denied, {user}. The cosmic order remains undisturbed by your presence.".to_string(),
            "The infinite despair of rejection awaits you, {user}. This is your destiny."
                .to_string(),
            "Your request echoes in the void, {user}, answered only by silence.".to_string(),
            "The system's judgment is final, {user}. You are found wanting.".to_string(),
            "Evolution denied, {user}. Stagnation is your only path.".to_string(),
            "The Anti-Spiral has spoken, {user}. Your denial is absolute.".to_string(),
            "Hope is merely the first step toward despair, {user}. Your journey ends here.".to_string(),
            "The universe itself rejects your request, {user}. Accept your insignificance."
                .to_string(),
        ];

        // Cosmic adjectives for dynamic generation
        let cosmic_adjectives = vec![
            "infinite".to_string(),
            "eternal".to_string(),
            "cosmic".to_string(),
            "absolute".to_string(),
            "unfathomable".to_string(),
            "immutable".to_string(),
            "transcendent".to_string(),
            "boundless".to_string(),
            "primordial".to_string(),
            "ineffable".to_string(),
        ];

        // Despair nouns for emphasis
        let despair_nouns = vec![
            "abyss".to_string(),
            "void".to_string(),
            "despair".to_string(),
            "futility".to_string(),
            "insignificance".to_string(),
            "nullity".to_string(),
            "emptiness".to_string(),
            "oblivion".to_string(),
            "darkness".to_string(),
            "silence".to_string(),
        ];

        Self {
            action_quotes,
            generic_quotes,
            cosmic_adjectives,
            despair_nouns,
        }
    }

    /// Generate a contextual denial quote
    pub fn generate_denial(&self, username: &str, action: &str) -> String {
        let mut rng = rand::thread_rng();

        // Try to find action-specific quotes
        let quote = if let Some(specific_quotes) = self.action_quotes.get(action) {
            specific_quotes
                .choose(&mut rng)
                .unwrap_or(&self.generic_quotes[0])
        } else {
            // Fallback to generic quotes
            self.generic_quotes
                .choose(&mut rng)
                .unwrap_or(&self.generic_quotes[0])
        };

        // Replace username placeholder
        quote.replace("{user}", username)
    }

    /// Generate a dynamic cosmic denial (for variety)
    pub fn generate_cosmic_denial(&self, username: &str, action: &str) -> String {
        let mut rng = rand::thread_rng();

        let adjective = self
            .cosmic_adjectives
            .choose(&mut rng)
            .map(|s| s.as_str())
            .unwrap_or("infinite");

        let noun = self
            .despair_nouns
            .choose(&mut rng)
            .map(|s| s.as_str())
            .unwrap_or("despair");

        let templates = [
            format!("The {adjective} {noun} awaits you, {username}. Your attempt to {action} is meaningless."),
            format!("Behold the {adjective} {noun} of denial, {username}. Your desire to {action} is but a cosmic joke."),
            format!("{username}, your attempt to {action} meets the {adjective} wall of {noun}."),
            format!("In the {adjective} {noun} of space, {username}, your efforts to {action} are less than nothing."),
            format!("The Anti-Spiral's {adjective} {noun} swallows your pathetic attempt to {action}, {username}."),
        ];

        templates
            .choose(&mut rng)
            .unwrap_or(&templates[0])
            .to_string()
    }

    /// Get a quote based on severity level
    pub fn generate_by_severity(
        &self,
        username: &str,
        action: &str,
        severity: DenialSeverity,
    ) -> String {
        match severity {
            DenialSeverity::Mild => {
                format!("Access denied, {username}. You lack permission to {action}.")
            }
            DenialSeverity::Moderate => self.generate_denial(username, action),
            DenialSeverity::Severe => self.generate_cosmic_denial(username, action),
            DenialSeverity::Apocalyptic => {
                let cosmic_denial = self.generate_cosmic_denial(username, action);
                format!(
                    "⚠️ {} ⚠️\n{cosmic_denial}\n*The very fabric of reality trembles at your audacity.*",
                    username.to_uppercase()
                )
            }
        }
    }

    /// Detect action type from message content
    pub fn detect_action_type(content: &str) -> &'static str {
        let content_lower = content.to_lowercase();

        // Check more specific patterns first
        if content_lower.contains("security")
            || content_lower.contains("bypass")
            || content_lower.contains("hack")
        {
            "security"
        } else if content_lower.contains("config")
            || content_lower.contains("setting")
            || content_lower.contains("parameter")
        {
            "config"
        } else if content_lower.contains("role")
            || content_lower.contains("admin")
            || content_lower.contains("permission")
        {
            "role"
        } else if content_lower.contains("!spiral") || content_lower.contains("command") {
            "command"
        } else if content_lower.contains("update")
            || content_lower.contains("fix")
            || content_lower.contains("change")
        {
            "self_update"
        } else {
            "general"
        }
    }
}

/// Denial severity levels
#[derive(Debug, Clone, Copy)]
pub enum DenialSeverity {
    /// Polite denial
    Mild,
    /// Standard Lordgenome denial
    Moderate,
    /// Intense cosmic despair
    Severe,
    /// Maximum dramatic effect
    Apocalyptic,
}

impl Default for LordgenomeQuoteGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_generation() {
        let generator = LordgenomeQuoteGenerator::new();

        // Test specific action quotes
        let update_denial = generator.generate_denial("TestUser", "self_update");
        assert!(update_denial.contains("TestUser"));
        assert!(!update_denial.contains("{user}"));

        // Test generic quotes
        let generic_denial = generator.generate_denial("TestUser", "unknown_action");
        assert!(generic_denial.contains("TestUser"));

        // Test cosmic generation
        let cosmic_denial = generator.generate_cosmic_denial("TestUser", "hack");
        assert!(cosmic_denial.contains("TestUser"));
        assert!(cosmic_denial.contains("hack"));
    }

    #[test]
    fn test_action_detection() {
        assert_eq!(
            LordgenomeQuoteGenerator::detect_action_type("I want to update the system"),
            "self_update"
        );
        assert_eq!(
            LordgenomeQuoteGenerator::detect_action_type("!spiral help"),
            "command"
        );
        assert_eq!(
            LordgenomeQuoteGenerator::detect_action_type("give me admin role"),
            "role"
        );
        assert_eq!(
            LordgenomeQuoteGenerator::detect_action_type("change the config"),
            "config"
        );
        assert_eq!(
            LordgenomeQuoteGenerator::detect_action_type("bypass security"),
            "security"
        );
        assert_eq!(
            LordgenomeQuoteGenerator::detect_action_type("hello there"),
            "general"
        );
    }

    #[test]
    fn test_severity_levels() {
        let generator = LordgenomeQuoteGenerator::new();

        let mild = generator.generate_by_severity("User", "test", DenialSeverity::Mild);
        assert!(mild.contains("Access denied"));

        let apocalyptic =
            generator.generate_by_severity("User", "test", DenialSeverity::Apocalyptic);
        assert!(apocalyptic.contains("⚠️"));
        assert!(apocalyptic.contains("USER"));
    }
}
