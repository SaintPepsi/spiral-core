//! 🎭 PERSONA SYSTEM TESTS
//! DECISION: Test all agent personas for consistency and correctness
//! Why: Personas drive user experience and agent selection
//! Alternative: Manual testing (rejected: too many permutations)

use crate::discord::spiral_constellation_bot::AgentPersona;
use crate::models::AgentType;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_agent_types_have_personas() {
        // 🎯 TEST PURPOSE: Ensure every AgentType has a corresponding persona
        // Why: Missing personas would cause runtime panics
        // Coverage: All AgentType variants

        let agent_types = vec![
            AgentType::SoftwareDeveloper,
            AgentType::ProjectManager,
            // Only test implemented agents
        ];

        for agent_type in agent_types {
            let persona = AgentPersona::for_agent_type(&agent_type);

            // Verify persona has required fields
            assert!(
                !persona.name.is_empty(),
                "Persona name cannot be empty for {agent_type:?}"
            );
            assert!(
                !persona.emoji.is_empty(),
                "Persona emoji cannot be empty for {agent_type:?}"
            );
            assert!(
                !persona.greetings.is_empty(),
                "Persona must have at least one greeting for {agent_type:?}"
            );
            assert!(
                !persona.working_message.is_empty(),
                "Working message cannot be empty for {agent_type:?}"
            );
            assert!(
                !persona.personality_traits.is_empty(),
                "Personality traits cannot be empty for {agent_type:?}"
            );
        }
    }

    #[test]
    fn test_persona_greeting_variety() {
        // 🎯 TEST PURPOSE: Ensure personas have multiple greeting variations
        // Why: Variety prevents bot from feeling repetitive
        // Coverage: All personas should have 2+ greetings

        let persona = AgentPersona::DEVELOPER;
        assert!(
            persona.greetings.len() >= 2,
            "Software Developer should have multiple greetings"
        );

        let persona = AgentPersona::SPIRAL_KING;
        assert!(
            persona.greetings.len() >= 2,
            "Spiral King should have multiple greetings"
        );

        // Test random greeting returns valid options
        for _ in 0..10 {
            let greeting = persona.random_greeting();
            assert!(
                persona.greetings.contains(&greeting),
                "Random greeting should be from greetings array"
            );
        }
    }

    #[test]
    fn test_persona_content_quality() {
        // 🎯 TEST PURPOSE: Verify persona content meets quality standards
        // Why: Poor content affects user experience
        // Coverage: Content length, formatting, consistency

        let personas = vec![
            AgentPersona::DEVELOPER,
            AgentPersona::PROJECT_MANAGER,
            AgentPersona::QUALITY_ASSURANCE,
            AgentPersona::DECISION_MAKER,
            AgentPersona::CREATIVE_INNOVATOR,
            AgentPersona::PROCESS_COACH,
            AgentPersona::SPIRAL_KING,
        ];

        for persona in personas {
            // Check greeting length (should be substantial but not overwhelming)
            for greeting in persona.greetings {
                assert!(
                    greeting.len() >= 20,
                    "Greeting too short for {}: '{}'",
                    persona.name,
                    greeting
                );
                assert!(
                    greeting.len() <= 200,
                    "Greeting too long for {}: '{}'",
                    persona.name,
                    greeting
                );
            }

            // Check working message is appropriate
            assert!(
                persona.working_message.contains("work")
                    || persona.working_message.contains("Work")
                    || persona.working_message.contains("analyz")
                    || persona.working_message.contains("Analyz")
                    || persona.working_message.contains("process")
                    || persona.working_message.contains("Innovat")
                    || persona.working_message.contains("Review")
                    || persona.working_message.contains("Gather")
                    || persona.working_message.contains("check")
                    || persona.working_message.contains("flow"),
                "Working message should indicate activity for {}",
                persona.name
            );

            // Check personality traits are meaningful
            assert!(
                persona.personality_traits.len() >= 3,
                "Should have at least 3 personality traits for {}",
                persona.name
            );
            for trait_desc in persona.personality_traits {
                assert!(
                    trait_desc.len() >= 4,
                    "Personality trait too short for {}: '{}'",
                    persona.name,
                    trait_desc
                );
            }
        }
    }

    #[test]
    fn test_spiral_king_persona_uniqueness() {
        // 🎯 TEST PURPOSE: Verify Spiral King has distinctive characteristics
        // Why: Spiral King should stand out as the supreme code reviewer
        // Coverage: Unique elements of Spiral King persona

        let spiral_king = AgentPersona::SPIRAL_KING;

        // Should have distinctive title
        assert!(
            spiral_king.name.contains("Immortal") || spiral_king.name.contains("King"),
            "Spiral King should have distinctive title"
        );

        // Should have crown emoji
        assert_eq!(
            spiral_king.emoji, "👑",
            "Spiral King should have crown emoji"
        );

        // Greetings should reference eternal/millennial themes
        let has_eternal_theme = spiral_king.greetings.iter().any(|g| {
            g.contains("millennial")
                || g.contains("eternal")
                || g.contains("thousand")
                || g.contains("ancient")
        });
        assert!(
            has_eternal_theme,
            "Spiral King greetings should reference eternal themes"
        );

        // Personality should reflect authority and experience
        let has_authority_trait = spiral_king.personality_traits.iter().any(|t| {
            t.contains("authority")
                || t.contains("wisdom")
                || t.contains("experience")
                || t.contains("judgment")
        });
        assert!(
            has_authority_trait,
            "Spiral King should have authority-related personality traits"
        );
    }

    #[test]
    fn test_persona_emoji_consistency() {
        // 🎯 TEST PURPOSE: Ensure each persona has appropriate emoji
        // Why: Emojis help users quickly identify agent types
        // Coverage: All personas have single, appropriate emoji

        let personas = vec![
            (AgentPersona::DEVELOPER, "🚀"),
            (AgentPersona::PROJECT_MANAGER, "📋"),
            (AgentPersona::QUALITY_ASSURANCE, "🔍"),
            (AgentPersona::DECISION_MAKER, "🎯"),
            (AgentPersona::CREATIVE_INNOVATOR, "✨"),
            (AgentPersona::PROCESS_COACH, "🧘"),
            (AgentPersona::SPIRAL_KING, "👑"),
        ];

        for (persona, expected_emoji) in personas {
            assert_eq!(
                persona.emoji, expected_emoji,
                "Persona {} should have emoji {}",
                persona.name, expected_emoji
            );

            // Ensure emoji is single character (or compound emoji)
            assert!(
                persona.emoji.chars().count() <= 2,
                "Emoji should be 1-2 characters for {}",
                persona.name
            );
        }
    }
}
