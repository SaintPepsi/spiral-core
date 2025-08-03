//! Example of using Claude Code validation in the self-update pipeline
//!
//! This example shows how to integrate Claude Code with custom prompts
//! into your validation workflow.

use spiral_core::discord::self_update::{SelfUpdateRequest, UpdateStatus, UpdateValidator};
use spiral_core::error::Result;
use tracing::info;

/// Example: Run standard validation then optionally spawn Claude Code
async fn validate_with_optional_claude(request: &SelfUpdateRequest) -> Result<()> {
    // Step 1: Run standard 5-step validation
    info!("Running standard validation pipeline...");
    UpdateValidator::validate_changes().await?;

    // Step 2: Optionally spawn Claude Code with custom prompt
    if should_run_claude_validation(request) {
        info!("Spawning Claude Code for additional validation...");

        let custom_prompt = format!(
            r#"
            Review the recent changes for update: {}
            
            Update description: {}
            
            Please check for:
            1. Security vulnerabilities
            2. Performance regressions
            3. Breaking API changes
            4. Test coverage gaps
            5. Documentation needs
            
            Focus on high-impact issues only.
            "#,
            request.codename, request.description
        );

        // Here you would actually spawn Claude Code
        spawn_claude_with_prompt(&custom_prompt).await?;
    }

    Ok(())
}

/// Determine if Claude validation should run
fn should_run_claude_validation(request: &SelfUpdateRequest) -> bool {
    // Example criteria for running Claude validation:
    // - High-risk updates
    // - Security-related changes
    // - Architecture changes
    // - User specifically requested it

    request.description.contains("security")
        || request.description.contains("auth")
        || request.description.contains("architecture")
        || request.description.contains("@claude") // User can request Claude review
}

/// Spawn Claude Code with a custom prompt
async fn spawn_claude_with_prompt(prompt: &str) -> Result<()> {
    info!("Claude Code prompt:\n{}", prompt);

    // Integration options:

    // Option 1: Use Claude Code CLI
    // let output = std::process::Command::new("claude-code")
    //     .arg("--prompt")
    //     .arg(prompt)
    //     .output()?;

    // Option 2: Use Claude Code API
    // let client = ClaudeCodeClient::new(api_key);
    // let response = client.execute_prompt(prompt).await?;

    // Option 3: Use Task API with specific agent
    // let task = Task {
    //     subagent_type: "security-inquisitor",
    //     description: "Security review",
    //     prompt: prompt.to_string(),
    // };
    // let result = client.create_task(task).await?;

    // For now, just log what would happen
    info!("Would execute Claude Code with provided prompt");

    Ok(())
}

/// Example: Different prompts for different scenarios
mod prompt_templates {
    pub fn security_review(files: &[String]) -> String {
        format!(
            r#"
            Security Review for Modified Files
            
            Files changed:
            {}
            
            Perform OWASP Top 10 analysis and check for:
            - Input validation
            - Authentication/authorization
            - Injection vulnerabilities
            - Sensitive data handling
            - Security misconfigurations
            "#,
            files.join("\n")
        )
    }

    pub fn performance_review(description: &str) -> String {
        format!(
            r#"
            Performance Impact Analysis
            
            Change description: {}
            
            Analyze for:
            - Algorithm complexity changes
            - Memory allocation patterns
            - I/O bottlenecks
            - Concurrency issues
            - Cache efficiency
            "#,
            description
        )
    }

    pub fn breaking_change_review(api_files: &[String]) -> String {
        format!(
            r#"
            Breaking Change Detection
            
            API files modified:
            {}
            
            Check for:
            - Removed public functions
            - Changed function signatures
            - Modified struct fields
            - Altered trait definitions
            - Semantic versioning violations
            "#,
            api_files.join("\n")
        )
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Example update request
    let request = SelfUpdateRequest {
        id: "example-001".to_string(),
        codename: "security-update".to_string(),
        description: "Update authentication system".to_string(),
        user_id: 123456789,
        channel_id: 987654321,
        message_id: 111222333,
        combined_messages: vec!["Update authentication system".to_string()],
        timestamp: chrono::Utc::now().to_rfc3339(),
        retry_count: 0,
        status: UpdateStatus::Queued,
    };

    // Run validation with optional Claude
    match validate_with_optional_claude(&request).await {
        Ok(_) => info!("✅ Validation completed successfully"),
        Err(e) => eprintln!("❌ Validation failed: {}", e),
    }

    // Example: Use specific prompt templates
    let security_prompt = prompt_templates::security_review(&vec![
        "src/auth/handler.rs".to_string(),
        "src/api/auth.rs".to_string(),
    ]);

    info!("Security review prompt:\n{}", security_prompt);

    Ok(())
}
