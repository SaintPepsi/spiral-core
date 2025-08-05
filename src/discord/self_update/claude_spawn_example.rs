//! Example of spawning Claude Code with a custom prompt
//!
//! This demonstrates how to spawn a Claude Code instance with a custom prompt
//! for code validation or other tasks.

use crate::error::{Result, SpiralError};
use std::process::Command;
use tracing::info;

/// Example function to spawn Claude Code with a custom prompt
///
/// This shows the pattern for running Claude Code as a subprocess with:
/// - A custom prompt/task description
/// - File paths to analyze
/// - Specific validation instructions
pub async fn spawn_claude_code_example(custom_prompt: &str) -> Result<String> {
    info!("[ClaudeSpawn] Spawning Claude Code with custom prompt");

    // Example 1: Simple prompt execution
    // In a real implementation, you would use the Claude Code CLI or API
    let simple_example = format!(
        r#"
        # Example Claude Code Invocation
        
        ## Prompt:
        {custom_prompt}
        
        ## Expected Response Format:
        - Validation findings
        - Recommendations
        - Security concerns
        "#
    );

    info!("[ClaudeSpawn] Would execute: {simple_example}");

    // Example 2: With specific agent and files
    let agent_example = spawn_with_agent(
        "ty-lee-precision-tester",
        "Review these files for pressure points",
        vec!["src/main.rs", "src/lib.rs"],
    )
    .await?;

    // Example 3: Security-focused review
    let security_example = spawn_security_review(
        vec!["src/auth/handler.rs", "src/api/endpoints.rs"],
        "Check for OWASP Top 10 vulnerabilities",
    )
    .await?;

    Ok(format!(
        "Claude Code Examples:\n1. Simple: {simple_example}\n2. Agent: {agent_example}\n3. Security: {security_example}"
    ))
}

/// Example: Spawn Claude Code with a specific agent
async fn spawn_with_agent(agent_type: &str, description: &str, files: Vec<&str>) -> Result<String> {
    // In a real implementation, this would use the Claude Code Task API:
    // ```
    // let task_result = claude_client.create_task(Task {
    //     subagent_type: agent_type,
    //     description: description,
    //     prompt: format!("Files to review:\n{}", files.join("\n")),
    // }).await?;
    // ```

    let files_str = files.join(",");
    let command_example =
        format!("claude-code --agent {agent_type} --task '{description}' --files {files_str}");

    info!("[ClaudeSpawn] Agent command: {command_example}");
    let file_count = files.len();
    Ok(format!(
        "Would run agent {agent_type} on {file_count} files"
    ))
}

/// Example: Security-focused Claude Code invocation
async fn spawn_security_review(files: Vec<&str>, security_focus: &str) -> Result<String> {
    let files_list = files.join("\n");
    let security_prompt = format!(
        r#"
        Perform security review with focus: {security_focus}
        
        Files to analyze:
        {files_list}
        
        Check for:
        - Input validation issues
        - Authentication bypasses
        - Authorization flaws
        - Injection vulnerabilities
        - Sensitive data exposure
        "#
    );

    info!("[ClaudeSpawn] Security review prompt: {security_prompt}");
    Ok("Security review would be performed".to_string())
}

/// Example: Using Claude Code for architectural review
pub async fn spawn_architecture_review(
    changed_files: Vec<String>,
    custom_instructions: &str,
) -> Result<String> {
    let files_list = changed_files.join("\n");
    let prompt = format!(
        r#"
        Architectural Review Request
        
        Changed files:
        {files_list}
        
        Custom instructions:
        {custom_instructions}
        
        Please analyze:
        1. Architectural patterns and consistency
        2. Long-term maintainability concerns
        3. Performance implications
        4. Security architecture
        5. Testing strategy alignment
        "#
    );

    // Example of how you might invoke Claude Code CLI
    // This is a conceptual example - adapt to your actual Claude Code integration
    let claude_command = Command::new("claude-code")
        .args([
            "--mode",
            "review",
            "--prompt",
            &prompt,
            "--output-format",
            "json",
        ])
        .output();

    match claude_command {
        Ok(output) => {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout);
                Ok(response.to_string())
            } else {
                let error = String::from_utf8_lossy(&output.stderr);
                Err(SpiralError::SystemError(format!(
                    "Claude Code failed: {error}"
                )))
            }
        }
        Err(e) => Err(SpiralError::SystemError(format!(
            "Failed to spawn Claude Code: {e}"
        ))),
    }
}

/// Minimal example for custom prompt execution
pub async fn simple_claude_spawn(prompt: &str) -> Result<()> {
    info!("[ClaudeSpawn] Executing custom prompt");

    // This is where you would integrate with Claude Code API/CLI
    // For now, just log what would be executed
    info!("[ClaudeSpawn] Prompt: {prompt}");

    // Example integration points:
    // 1. HTTP API: POST to Claude Code endpoint
    // 2. CLI: Execute `claude-code --prompt "..."`
    // 3. SDK: Use claude_code::Client::execute_prompt(prompt)

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_spawn() {
        let result = simple_claude_spawn("Test prompt").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_spawn_examples() {
        let result = spawn_claude_code_example("Validate this code for security issues").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Claude Code Examples"));
    }
}
