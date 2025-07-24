use anyhow::Result;
use regex::Regex;
use crate::workspace::TaskWorkspace;
use std::sync::atomic::{AtomicU32, Ordering};
use std::io;
use chrono::Utc;

// Global counter for unique agent IDs
static AGENT_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Default container name for consistent reuse
pub const DEFAULT_CONTAINER_NAME: &str = "vscode-agent-default";

/// Generate a unique agent name with timestamp and counter
pub fn generate_agent_name() -> String {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let counter = AGENT_COUNTER.fetch_add(1, Ordering::SeqCst);
    let hostname = gethostname::gethostname()
        .into_string()
        .unwrap_or_else(|_| "unknown".to_string())
        .chars()
        .take(8)
        .collect::<String>();
    
    format!("vscode-agent-{}-{}-{}", hostname, timestamp, counter)
}

/// Generate a shorter agent name for containers (Docker has name length limits)
pub fn generate_container_name() -> String {
    let counter = AGENT_COUNTER.fetch_add(1, Ordering::SeqCst);
    let hostname = gethostname::gethostname()
        .into_string()
        .unwrap_or_else(|_| "host".to_string())
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(6)
        .collect::<String>();
    
    format!("agent-{}-{}", hostname, counter)
}

/// Check if we're already running inside a container
fn is_running_in_container() -> bool {
    // Check for common container indicators
    
    // 1. Check if /.dockerenv exists (Docker containers)
    if std::path::Path::new("/.dockerenv").exists() {
        return true;
    }
    
    // 2. Check if we're in a dev container (VS Code dev containers set this)
    if std::env::var("REMOTE_CONTAINERS").is_ok() || 
       std::env::var("CODESPACES").is_ok() ||
       std::env::var("VSCODE_REMOTE_CONTAINERS_SESSION").is_ok() {
        return true;
    }
    
    // 3. Check cgroup (Linux containers)
    if let Ok(cgroup_content) = std::fs::read_to_string("/proc/1/cgroup") {
        if cgroup_content.contains("docker") || cgroup_content.contains("containerd") {
            return true;
        }
    }
    
    // 4. Check if hostname suggests we're in a container
    if let Ok(hostname) = std::env::var("HOSTNAME") {
        if hostname.len() == 12 && hostname.chars().all(|c| c.is_ascii_hexdigit()) {
            // Looks like a Docker container ID
            return true;
        }
    }
    
    false
}

pub async fn ask_chat_agent(prompt: &str) -> Result<String> {
    println!("🤖 Asking VS Code Chat Agent: {}", prompt);

    // Simple container-based execution for all requests
    ask_chat_agent_containerized(prompt).await
}

fn is_parallel_mode() -> bool {
    // Check if we're running in parallel mode (multiple agents)
    std::env::var("VSCODE_AGENT_PARALLEL").as_deref() == Ok("true") ||
    std::env::var("SPIRAL_PARALLEL_AGENTS").as_deref() == Ok("true")
}

async fn is_docker_available() -> bool {
    tokio::process::Command::new("docker")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

async fn is_vscode_running() -> bool {
    tokio::process::Command::new("pgrep")
        .arg("Code")
        .output()
        .await
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false)
}

async fn should_use_container() -> bool {
    // If we're already in a container, no need for another container
    if is_running_in_container() {
        println!("📦 Already in container - using direct VS Code access");
        return false;
    }
    
    // Check if Docker is available
    if let Ok(output) = tokio::process::Command::new("docker")
        .arg("--version")
        .output()
        .await 
    {
        if output.status.success() {
            println!("🐳 Docker detected - checking for conflicts");
            
            // Check if VS Code is currently running (which could cause conflicts)
            if let Ok(output) = tokio::process::Command::new("pgrep")
                .arg("Code")
                .output()
                .await 
            {
                if output.status.success() && !output.stdout.is_empty() {
                    println!("⚠️  VS Code is running - using container mode for isolation");
                    return true;
                }
            }
            
            println!("✅ No VS Code conflicts detected - direct mode should work");
            return false;
        }
    }
    
    
    false
}

/// Get information about the current execution environment
pub async fn get_execution_mode_info() -> String {
    let in_container = is_running_in_container();
    let docker_available = tokio::process::Command::new("docker")
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false);
    
    let vscode_running = tokio::process::Command::new("pgrep")
        .arg("Code")
        .output()
        .await
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false);

    if in_container {
        format!(
            "🏠 Execution Mode: CONTAINER-DIRECT\n\
            📦 Running inside container (optimal for isolation)\n\
            🤖 VS Code and Copilot available directly\n\
            ✅ No conflicts possible with host VS Code"
        )
    } else if vscode_running && docker_available {
        format!(
            "🏠 Execution Mode: HOST-CONTAINERIZED\n\
            ⚠️  VS Code running on host (potential conflicts)\n\
            🐳 Docker available for isolation\n\
            💡 Recommendation: Use containerized mode"
        )
    } else if vscode_running {
        format!(
            "🏠 Execution Mode: HOST-DIRECT (RISKY)\n\
            ⚠️  VS Code running on host (will conflict!)\n\
            ❌ Docker not available for isolation\n\
            💡 Recommendation: Close VS Code or run in dev container"
        )
    } else {
        format!(
            "🏠 Execution Mode: HOST-DIRECT\n\
            ✅ No VS Code conflicts detected\n\
            💻 Direct mode should work fine\n\
            💡 For best isolation, consider using dev container"
        )
    }
}

/// Show current execution strategy and recommendations
pub async fn show_execution_strategy() -> Result<()> {
    let in_container = is_running_in_container();
    let docker_available = is_docker_available().await;
    let vscode_running = is_vscode_running().await;
    let parallel_mode = is_parallel_mode();
    
    println!("🔍 VS Code Agent Execution Analysis:");
    println!("   📦 In Container: {}", if in_container { "✅ Yes" } else { "❌ No" });
    println!("   🐳 Docker Available: {}", if docker_available { "✅ Yes" } else { "❌ No" });
    println!("   💻 VS Code Running: {}", if vscode_running { "⚠️  Yes (potential conflicts)" } else { "✅ No" });
    println!("   🔄 Parallel Mode: {}", if parallel_mode { "⚠️  Yes (needs isolation)" } else { "✅ No" });
    
    println!("\n🎯 Selected Strategy: Container Isolation");
    println!("   🐳 Each agent gets its own VS Code container");
    println!("   ✅ Perfect isolation - no conflicts possible");
    println!("   ⚠️  Requires Docker and container setup");
    
    println!("\n💡 Recommendations:");
    if !docker_available {
        println!("   🐳 Install Docker to use VS Code agents");
        println!("   📝 Docker is required for containerized execution");
    } else {
        println!("   ✅ Setup is optimal for VS Code agents");
    }
    
    println!("\n🔧 Environment Variables:");
    println!("   export VSCODE_AGENT_KEEP_CONTAINERS=true  # Keep containers for reuse");
    println!("   export SPIRAL_PARALLEL_AGENTS=true        # Enable parallel mode");
    
    Ok(())
}

async fn ask_chat_agent_containerized(prompt: &str) -> Result<String> {
    println!("🐳 Using containerized VS Code for parallel isolation");
    
    // Generate unique container name for this agent instance
    let container_name = format!("vscode-agent-{}", std::process::id());
    println!("🏷️  Using container: {}", container_name);
    
    // Ensure we have a fresh container for this agent
    ensure_agent_container(&container_name).await?;
    
    // Create a prompt file inside the container
    let temp_dir = std::env::temp_dir();
    let prompt_file = temp_dir.join(format!("vscode-agent-prompt-{}.md", std::process::id()));
    
    let prompt_content = format!(
        "# VS Code Agent Task (Container: {})\n\n\
        **Prompt:** {}\n\n\
        ## Instructions\n\
        1. Use GitHub Copilot to generate code for the above prompt\n\
        2. Include complete, working code with:\n\
           - Cargo.toml (if needed)\n\
           - Rust source files\n\
           - Unit tests\n\
           - Documentation\n\
        3. Save this file when you're done with the generated code\n\n\
        ## Generated Code\n\
        ```rust\n\
        // Your generated code will go here\n\
        ```\n\n\
        ## Generated Cargo.toml (if needed)\n\
        ```toml\n\
        # Your Cargo.toml content here if needed\n\
        ```\n",
        container_name, prompt
    );
    
    tokio::fs::write(&prompt_file, &prompt_content).await?;
    
    // Copy the prompt file into the container
    let container_prompt_path = "/workspace/agent-prompt.md";
    let copy_output = tokio::process::Command::new("docker")
        .arg("cp")
        .arg(&prompt_file)
        .arg(&format!("{}:{}", container_name, container_prompt_path))
        .output()
        .await?;
    
    if !copy_output.status.success() {
        let stderr = String::from_utf8_lossy(&copy_output.stderr);
        let _ = remove_container(&container_name).await;
        anyhow::bail!("Failed to copy prompt to container: {}", stderr);
    }
    
    // Open VS Code in the container with the prompt file
    println!("📝 Opening VS Code in isolated container...");
    println!("🤖 Container {} is ready - no chat conflicts possible!", container_name);
    println!("⏳ Please use Copilot to generate code, then save the file when done.");
    println!("💡 When finished, press Enter here to continue...");
    
    let vscode_output = tokio::process::Command::new("docker")
        .arg("exec")
        .arg("-it")
        .arg(&container_name)
        .arg("code")
        .arg(container_prompt_path)
        .output()
        .await?;

    if !vscode_output.status.success() {
        let stderr = String::from_utf8_lossy(&vscode_output.stderr);
        let _ = remove_container(&container_name).await;
        anyhow::bail!("Failed to open VS Code in container: {}", stderr);
    }

    // Wait for user confirmation instead of using --wait
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Copy the updated file back from the container
    let copy_back_output = tokio::process::Command::new("docker")
        .arg("cp")
        .arg(&format!("{}:{}", container_name, container_prompt_path))
        .arg(&prompt_file)
        .output()
        .await?;
    
    if !copy_back_output.status.success() {
        let stderr = String::from_utf8_lossy(&copy_back_output.stderr);
        let _ = remove_container(&container_name).await;
        anyhow::bail!("Failed to copy result from container: {}", stderr);
    }

    // Read the updated file
    let response = tokio::fs::read_to_string(&prompt_file).await?;
    
    // Clean up files
    let _ = tokio::fs::remove_file(&prompt_file).await;
    
    // Clean up container after successful use (optional - could keep for reuse)
    let keep_containers = std::env::var("VSCODE_AGENT_KEEP_CONTAINERS").as_deref() == Ok("true");
    if !keep_containers {
        let _ = remove_container(&container_name).await;
        println!("🗑️  Container {} cleaned up", container_name);
    } else {
        println!("💾 Container {} kept for reuse", container_name);
    }

    if response.trim() == prompt_content.trim() {
        anyhow::bail!("No changes detected in container. Please generate code using Copilot and save the file.");
    }

    println!("✅ Containerized chat agent completed successfully!");
    Ok(response)
}

pub async fn ensure_agent_container(container_name: &str) -> Result<()> {
    // Check if container exists and is running
    let check_output = tokio::process::Command::new("docker")
        .arg("ps")
        .arg("-a")
        .arg("--filter")
        .arg(&format!("name={}", container_name))
        .arg("--format")
        .arg("{{.Status}}")
        .output()
        .await?;
    
    let status = String::from_utf8_lossy(&check_output.stdout);
    
    if status.trim().is_empty() {
        // Container doesn't exist, create it
        println!("🏗️  Creating VS Code agent container...");
        create_agent_container(container_name).await?;
    } else if status.contains("Exited") {
        // Container exists but is stopped, start it
        println!("▶️  Starting existing VS Code agent container...");
        let start_output = tokio::process::Command::new("docker")
            .arg("start")
            .arg(container_name)
            .output()
            .await?;
        
        if !start_output.status.success() {
            anyhow::bail!("Failed to start container: {}", String::from_utf8_lossy(&start_output.stderr));
        }
    } else {
        println!("✅ VS Code agent container is running");
    }
    
    Ok(())
}

/// Ensure the default agent container exists and is running
pub async fn ensure_default_agent_container() -> Result<()> {
    ensure_agent_container(DEFAULT_CONTAINER_NAME).await
}

/// Create a new agent container with a unique name
pub async fn create_unique_agent_container() -> Result<String> {
    let container_name = generate_container_name();
    create_agent_container(&container_name).await?;
    Ok(container_name)
}

/// Container management functions for CLI commands
pub async fn start_container(container_name: &str) -> Result<()> {
    let output = tokio::process::Command::new("docker")
        .arg("start")
        .arg(container_name)
        .output()
        .await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to start container {}: {}", container_name, stderr);
    }
    
    Ok(())
}

pub async fn stop_container(container_name: &str) -> Result<()> {
    let output = tokio::process::Command::new("docker")
        .arg("stop")
        .arg(container_name)
        .output()
        .await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to stop container {}: {}", container_name, stderr);
    }
    
    Ok(())
}

pub async fn remove_container(container_name: &str) -> Result<()> {
    // Stop first
    let _ = stop_container(container_name).await;
    
    // Then remove
    let output = tokio::process::Command::new("docker")
        .arg("rm")
        .arg(container_name)
        .output()
        .await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to remove container {}: {}", container_name, stderr);
    }
    
    Ok(())
}

pub async fn get_container_status(container_name: &str) -> Result<String> {
    let output = tokio::process::Command::new("docker")
        .arg("ps")
        .arg("-a")
        .arg("--filter")
        .arg(&format!("name={}", container_name))
        .arg("--format")
        .arg("table {{.Names}}\t{{.Status}}\t{{.Ports}}")
        .output()
        .await?;
    
    if !output.status.success() {
        anyhow::bail!("Failed to check container status");
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn setup_copilot_in_container(container_name: &str) -> Result<()> {
    let output = tokio::process::Command::new("docker")
        .arg("exec")
        .arg("-it")
        .arg(container_name)
        .arg("code")
        .arg("--install-extension")
        .arg("GitHub.copilot")
        .output()
        .await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Failed to install Copilot extension: {}", stderr));
    }
    
    Ok(())
}

async fn create_agent_container(container_name: &str) -> Result<()> {
    // Get the current project directory to find the Dockerfile
    let current_dir = std::env::current_dir()?;
    let dockerfile_path = current_dir.join("src/resources/Dockerfile.vscode-agent");
    
    // Check if Dockerfile exists
    if !dockerfile_path.exists() {
        anyhow::bail!("Dockerfile.vscode-agent not found at: {}", dockerfile_path.display());
    }
    
    // Build the image using the external Dockerfile
    println!("🔨 Building VS Code agent Docker image...");
    let build_output = tokio::process::Command::new("docker")
        .arg("build")
        .arg("-t")
        .arg("vscode-agent:latest")
        .arg("-f")
        .arg(&dockerfile_path)
        .arg(&current_dir)
        .output()
        .await?;
    
    if !build_output.status.success() {
        anyhow::bail!("Failed to build container: {}", String::from_utf8_lossy(&build_output.stderr));
    }
    
    // Run the container
    println!("🚀 Starting VS Code agent container...");
    let run_output = tokio::process::Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--name")
        .arg(container_name)
        .arg("-v")
        .arg("/tmp/vscode-agent-workspace:/workspace")
        .arg("vscode-agent:latest")
        .output()
        .await?;
    
    if !run_output.status.success() {
        anyhow::bail!("Failed to start container: {}", String::from_utf8_lossy(&run_output.stderr));
    }
    
    println!("✅ VS Code agent container created and running");
    
    println!("⚠️  Note: You'll need to configure GitHub Copilot in the container");
    println!("   Run: vscode-agent container setup");
    
    Ok(())
}

pub fn create_rust_project_prompt(task: &str) -> String {
    format!(
        "Create a complete Rust project for: {}\n\n\
        Please generate:\n\
        1. A complete Cargo.toml with appropriate dependencies\n\
        2. Implementation in src/lib.rs with proper error handling\n\
        3. Comprehensive unit tests\n\
        4. Documentation comments\n\n\
        Make it production-ready and follow Rust best practices.",
        task
    )
}

pub fn create_simple_function_prompt(task: &str) -> String {
    format!(
        "Create a simple Rust function for: {}\n\n\
        Please provide:\n\
        1. A single function implementation with proper error handling\n\
        2. Unit tests for the function\n\
        3. Documentation comments\n\n\
        Keep it simple and focused.",
        task
    )
}

pub async fn write_code_to_workspace(
    workspace: &TaskWorkspace,
    generated_code: &str,
) -> Result<Vec<String>> {
    let mut created_files = Vec::new();

    // Parse the generated code for different file types
    let files = parse_generated_files(generated_code);

    for (filename, content) in files {
        let file_path = workspace.path.join(&filename);

        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&file_path, content).await?;
        created_files.push(filename);
        println!("📝 Created: {}", file_path.display());
    }

    // If no files were parsed, create a basic structure
    if created_files.is_empty() {
        create_default_structure(workspace, generated_code).await?;
        created_files = vec!["Cargo.toml".to_string(), "src/lib.rs".to_string()];
    }

    Ok(created_files)
}

fn parse_generated_files(content: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();

    // Look for common file patterns in the response
    let cargo_toml_re = Regex::new(r"```toml\s*\n([\s\S]*?)\n```").unwrap();
    let rust_code_re = Regex::new(r"```rust\s*\n([\s\S]*?)\n```").unwrap();

    // Extract Cargo.toml
    if let Some(captures) = cargo_toml_re.captures(content) {
        if let Some(toml_content) = captures.get(1) {
            files.push(("Cargo.toml".to_string(), toml_content.as_str().to_string()));
        }
    }

    // Extract Rust code - single block with tests goes to src/lib.rs
    let rust_blocks: Vec<_> = rust_code_re.captures_iter(content).collect();
    
    for (i, captures) in rust_blocks.iter().enumerate() {
        if let Some(rust_content) = captures.get(1) {
            let content_str = rust_content.as_str();
            let filename = if rust_blocks.len() == 1 {
                // Single block always goes to src/lib.rs
                "src/lib.rs".to_string()
            } else if content_str.contains("#[cfg(test)]") && !content_str.contains("pub fn") {
                // Pure test block
                format!("tests/test_{}.rs", i)
            } else if content_str.contains("fn main") {
                "src/main.rs".to_string()
            } else {
                "src/lib.rs".to_string()
            };

            files.push((filename, content_str.to_string()));
        }
    }

    // Look for file headers (e.g., "// src/lib.rs") - simplified approach
    let lines: Vec<&str> = content.lines().collect();
    let mut current_file: Option<String> = None;
    let mut current_content = Vec::new();
    
    for line in lines {
        if line.trim().starts_with("//") && line.contains(".rs") {
            // Save previous file if we have one
            if let Some(filename) = current_file.take() {
                if !current_content.is_empty() {
                    files.push((filename, current_content.join("\n").trim().to_string()));
                }
            }
            
            // Start new file
            if let Some(filename) = line.trim().strip_prefix("//").map(|s| s.trim()) {
                if filename.ends_with(".rs") {
                    current_file = Some(filename.to_string());
                    current_content.clear();
                }
            }
        } else if current_file.is_some() {
            current_content.push(line);
        }
    }
    
    // Don't forget the last file
    if let Some(filename) = current_file {
        if !current_content.is_empty() {
            files.push((filename, current_content.join("\n").trim().to_string()));
        }
    }

    files
}

async fn create_default_structure(workspace: &TaskWorkspace, generated_code: &str) -> Result<()> {
    // Create basic Cargo.toml
    let project_name = workspace.task
        .chars()
        .take(20)
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    let cargo_content = format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
serde = {{ version = "1.0", features = ["derive"] }}
"#, project_name);

    tokio::fs::write(workspace.path.join("Cargo.toml"), cargo_content).await?;

    // Create src directory and lib.rs with generated code
    tokio::fs::create_dir_all(workspace.path.join("src")).await?;

    let lib_content = if generated_code.contains("fn ") {
        // Use the generated code directly
        generated_code.to_string()
    } else {
        // Create a basic implementation with the generated content as comments
        format!(r#"// Generated by VS Code Chat Agent for: {}

{}

// TODO: Implement the functionality described above

pub fn placeholder() -> String {{
    "Implementation needed".to_string()
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_placeholder() {{
        assert_eq!(placeholder(), "Implementation needed");
    }}
}}
"#, workspace.task, generated_code)
    };

    tokio::fs::write(workspace.path.join("src/lib.rs"), lib_content).await?;

    Ok(())
}

pub async fn test_chat_agent() -> Result<()> {
    println!("🧪 Testing VS Code chat agent...");

    let test_prompt = "Create a simple Rust function that adds two numbers";
    
    println!("🧪 Testing with containerized VS Code isolation");
    
    // Test containerized version with unique name
    let test_container_name = format!("vscode-agent-test-{}", std::process::id());
    println!("🧪 Testing with container: {}", test_container_name);
    ensure_agent_container(&test_container_name).await?;
    let output = tokio::process::Command::new("docker")
        .arg("exec")
        .arg(&test_container_name)
        .arg("code")
        .arg("chat")
        .arg("--mode=agent")
        .arg(test_prompt)
        .output()
        .await?;
    
    // Clean up test container
    let _ = remove_container(&test_container_name).await;

    if output.status.success() {
        let response = String::from_utf8_lossy(&output.stdout);
        if !response.trim().is_empty() {
            println!("✅ Chat agent test successful");
            Ok(())
        } else {
            anyhow::bail!("Chat agent returned empty response")
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Chat agent test failed: {}", stderr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rust_code() {
        let sample_response = r##"
Here's a simple Rust function:

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
```
"##;

        let files = parse_generated_files(sample_response);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "src/lib.rs");
        assert!(files[0].1.contains("pub fn add"));
        assert!(files[0].1.contains("#[test]"));
    }

    #[test]
    fn test_parse_cargo_toml_and_rust() {
        let sample_response = r##"

Here's a complete Rust project:

```toml
[package]
name = "calculator"
version = "0.1.0"
edition = "2021"

[dependencies]
```

```rust
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiply() {
        assert_eq!(multiply(4, 5), 20);
    }
}
```
"##;

        let files = parse_generated_files(sample_response);
        assert_eq!(files.len(), 2);

        // Check Cargo.toml
        let cargo_file = files.iter().find(|(name, _)| name == "Cargo.toml");
        assert!(cargo_file.is_some());
        let (_, content) = cargo_file.unwrap();
        assert!(content.contains("[package]"));
        assert!(content.contains("calculator"));

        // Check Rust file
        let rust_file = files.iter().find(|(name, _)| name == "src/lib.rs");
        assert!(rust_file.is_some());
        let (_, content) = rust_file.unwrap();
        assert!(content.contains("pub fn multiply"));
    }

    #[test]
    fn test_create_simple_function_prompt() {
        let prompt = create_simple_function_prompt("add two numbers");
        assert!(prompt.contains("add two numbers"));
        assert!(prompt.contains("simple Rust function"));
        assert!(prompt.contains("Unit tests"));
        assert!(!prompt.contains("complete Rust project")); // Should be simple, not full project
    }

    #[test]
    fn test_create_rust_project_prompt() {
        let prompt = create_rust_project_prompt("REST API for user management");
        assert!(prompt.contains("REST API for user management"));
        assert!(prompt.contains("complete Rust project"));
        assert!(prompt.contains("Cargo.toml"));
        assert!(prompt.contains("src/lib.rs"));
    }

    #[test]
    fn test_parse_file_headers() {
        let sample_response = r##"

// src/lib.rs
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// tests/integration_tests.rs
#[test]
fn test_greet() {
    assert_eq!(greet("World"), "Hello, World!");
}
"##;

        let files = parse_generated_files(sample_response);
        
        // Should have 2 files from file headers
        let lib_file = files.iter().find(|(name, _)| name == "src/lib.rs");
        assert!(lib_file.is_some());
        assert!(lib_file.unwrap().1.contains("pub fn greet"));

        let test_file = files.iter().find(|(name, _)| name == "tests/integration_tests.rs");
        assert!(test_file.is_some());
        assert!(test_file.unwrap().1.contains("#[test]"));
    }

    #[tokio::test]
    async fn test_create_default_structure() {
        let temp_dir = std::env::temp_dir().join("vscode_agent_test");
        tokio::fs::create_dir_all(&temp_dir).await.unwrap();

        let workspace = crate::workspace::TaskWorkspace {
            name: "test".to_string(),
            path: temp_dir.clone(),
            task: "test function".to_string(),
        };

        let generated_code = "This is some generated content that doesn't contain valid Rust code.";

        create_default_structure(&workspace, generated_code).await.unwrap();

        // Check that files were created
        assert!(temp_dir.join("Cargo.toml").exists());
        assert!(temp_dir.join("src/lib.rs").exists());

        // Check Cargo.toml content
        let cargo_content = tokio::fs::read_to_string(temp_dir.join("Cargo.toml")).await.unwrap();
        assert!(cargo_content.contains("[package]"));
        assert!(cargo_content.contains("test-function")); // task should be converted to valid name

        // Check lib.rs content
        let lib_content = tokio::fs::read_to_string(temp_dir.join("src/lib.rs")).await.unwrap();
        assert!(lib_content.contains("test function")); // original task
        assert!(lib_content.contains("This is some generated content")); // original generated code
        assert!(lib_content.contains("pub fn placeholder")); // fallback function

        // Cleanup
        tokio::fs::remove_dir_all(&temp_dir).await.unwrap();
    }

    #[test]
    fn test_empty_response_parsing() {
        let empty_response = "";
        let files = parse_generated_files(empty_response);
        assert_eq!(files.len(), 0);

        let whitespace_response = "   \n\n   ";
        let files = parse_generated_files(whitespace_response);
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_multiple_rust_blocks() {
        let sample_response = r##"

Here are multiple Rust files:

```rust
// Main library code
pub fn calculate(x: i32) -> i32 {
    x * 2
}
```

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate() {
        assert_eq!(calculate(5), 10);
    }
}
```
"##;

        let files = parse_generated_files(sample_response);
        assert_eq!(files.len(), 2);

        // First should be main code
        assert_eq!(files[0].0, "src/lib.rs");
        assert!(files[0].1.contains("pub fn calculate"));

        // Second should be test file
        assert!(files[1].0.starts_with("tests/test_"));
        assert!(files[1].1.contains("#[cfg(test)]"));
    }
}
