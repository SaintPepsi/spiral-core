use clap::{Parser, Subcommand};

mod copilot_chat;
mod workspace;

#[derive(Parser)]
#[command(name = "vscode-agent")]
#[command(about = "Ultra-simple VS Code chat agent automation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate code using VS Code chat agent
    Dev {
        /// What you want to build
        task: String,
    },
    /// Show generated workspaces
    List,
    /// Clean up old workspaces
    Clean,
    /// Check VS Code chat agent setup
    Check,
    /// Run integration tests with VS Code chat agent
    Test,
    /// Manage containerized VS Code instances
    Container {
        #[command(subcommand)]
        action: ContainerAction,
    },
}

#[derive(Subcommand)]
enum ContainerAction {
    /// Create a new VS Code agent container
    Create,
    /// Start existing container
    Start,
    /// Stop running container
    Stop,
    /// Remove container
    Remove,
    /// Show container status
    Status,
    /// Setup GitHub Copilot in container
    Setup,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dev { task } => {
            run_chat_agent_task(&task).await?;
        }
        Commands::List => {
            workspace::list_workspaces().await?;
        }
        Commands::Clean => {
            workspace::cleanup_old_workspaces().await?;
        }
        Commands::Check => {
            check_chat_agent_setup().await?;
        }
        Commands::Test => {
            run_integration_tests().await?;
        }
        Commands::Container { action } => {
            run_container_command(action).await?;
        }
    }

    Ok(())
}

async fn run_chat_agent_task(task: &str) -> Result<(), anyhow::Error> {
    println!("🤖 VS Code Chat Agent: {}", task);
    println!("⚡ Using `code chat --mode='agent'` - the simplest possible approach!");

    // Step 1: Create workspace
    let workspace = workspace::create_task_workspace(task).await?;
    println!("📁 Workspace: {}", workspace.path.display());

    // Step 2: Create appropriate prompt based on task complexity
    let prompt = if is_simple_function_task(task) {
        copilot_chat::create_simple_function_prompt(task)
    } else {
        copilot_chat::create_rust_project_prompt(task)
    };

    // Step 3: Ask VS Code chat agent to generate code
    let generated_code = copilot_chat::ask_chat_agent(&prompt).await?;
    println!("🧠 Received {} characters of generated code", generated_code.len());

    // Step 4: Parse and write the code to files
    let files_created = copilot_chat::write_code_to_workspace(&workspace, &generated_code).await?;
    println!("📝 Created {} files:", files_created.len());
    for file in &files_created {
        println!("   {}", file);
    }

    // Step 5: Build and test
    let build_result = workspace::build_and_test(&workspace).await?;
    println!("📊 Results:");
    println!("   Build: {}", if build_result.success { "✅ PASSED" } else { "❌ FAILED" });
    println!("   Tests: {} passed, {} failed", build_result.tests_passed, build_result.tests_failed);

    if !build_result.success {
        println!("   Errors:");
        for error in &build_result.errors {
            println!("     {}", error);
        }
    }

    println!("\n🎉 Task completed!");
    println!("📂 Code location: {}", workspace.path.display());

    Ok(())
}

fn is_simple_function_task(task: &str) -> bool {
    let task_lower = task.to_lowercase();
    task_lower.starts_with("simple:")
}

async fn check_chat_agent_setup() -> Result<(), anyhow::Error> {
    println!("🔍 Checking VS Code Chat Agent Setup:");

    // Check VS Code CLI
    match tokio::process::Command::new("code")
        .arg("--version")
        .output()
        .await
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("✅ VS Code CLI: {}", version.lines().next().unwrap_or("installed"));
        }
        Ok(_) => {
            println!("❌ VS Code CLI: Installed but not working");
            return Err(anyhow::anyhow!("VS Code CLI installation issue"));
        }
        Err(_) => {
            println!("❌ VS Code CLI: Not available");
            println!("   Install VS Code and ensure 'code' command is in PATH");
            return Err(anyhow::anyhow!("VS Code CLI not available"));
        }
    }

    // Test chat agent functionality
    match copilot_chat::test_chat_agent().await {
        Ok(_) => println!("✅ VS Code Chat Agent: Working"),
        Err(e) => {
            println!("❌ VS Code Chat Agent: {}", e);
            println!("   Make sure you have GitHub Copilot enabled in VS Code");
            return Err(e);
        }
    }

    println!("\n🚀 VS Code Chat Agent ready!");
    println!("   This is the simplest possible Copilot automation!");

    Ok(())
}

async fn run_integration_tests() -> Result<(), anyhow::Error> {
    println!("🧪 Running VS Code Chat Agent Integration Tests");

    let test_cases = vec![
        "Simple: Create a function that adds two numbers",
        "Simple: Write a function to calculate the factorial of a number",
        "Simple: Create a simple calculator with basic operations",
        "Simple: Write a function that checks if a string is a palindrome",
        "Simple: Create a function that finds the maximum value in a vector",
    ];

    let mut passed = 0;
    let mut failed = 0;

    for (i, test_case) in test_cases.iter().enumerate() {
        println!("\n📋 Test {}/{}: {}", i + 1, test_cases.len(), test_case);

        match run_single_integration_test(test_case).await {
            Ok(_) => {
                println!("   ✅ PASSED");
                passed += 1;
            }
            Err(e) => {
                println!("   ❌ FAILED: {}", e);
                failed += 1;
            }
        }
    }

    println!("\n📊 Integration Test Results:");
    println!("   ✅ Passed: {}", passed);
    println!("   ❌ Failed: {}", failed);
    println!("   📈 Success Rate: {:.1}%", (passed as f64 / test_cases.len() as f64) * 100.0);

    if failed == 0 {
        println!("\n🎉 All integration tests passed!");
        Ok(())
    } else {
        anyhow::bail!("{} integration tests failed", failed)
    }
}

async fn run_single_integration_test(task: &str) -> Result<(), anyhow::Error> {
    // Create temporary workspace
    let temp_workspace = workspace::create_task_workspace(&format!("test_{}", task)).await?;

    // Ensure cleanup
    let cleanup_path = temp_workspace.path.clone();
    let _cleanup_guard = scopeguard::guard((), |_| {
        if cleanup_path.exists() {
            let _ = std::fs::remove_dir_all(&cleanup_path);
        }
    });

    // Generate appropriate prompt
    let prompt = if is_simple_function_task(task) {
        copilot_chat::create_simple_function_prompt(task)
    } else {
        copilot_chat::create_rust_project_prompt(task)
    };

    // Get code from chat agent
    let generated_code = copilot_chat::ask_chat_agent(&prompt).await?;

    // Write to workspace
    copilot_chat::write_code_to_workspace(&temp_workspace, &generated_code).await?;

    // Test that it builds and passes tests
    let build_result = workspace::build_and_test(&temp_workspace).await?;

    if !build_result.success {
        anyhow::bail!("Generated code failed to build or test: {:?}", build_result.errors);
    }

    // Additional validation: ensure at least one test passed
    if build_result.tests_passed == 0 {
        anyhow::bail!("No tests were found or executed");
    }

    Ok(())
}

async fn run_container_command(action: ContainerAction) -> Result<(), anyhow::Error> {
    match action {
        ContainerAction::Create => {
            println!("🏗️  Creating VS Code agent container...");
            copilot_chat::ensure_default_agent_container().await?;
            println!("✅ Container created successfully");
            println!("💡 Next steps:");
            println!("   1. Run: vscode-agent container setup");
            println!("   2. Configure GitHub Copilot in the container");
            println!("   3. Set VSCODE_AGENT_USE_CONTAINER=true");
        }
        ContainerAction::Start => {
            println!("▶️  Starting VS Code agent container...");
            copilot_chat::start_container(copilot_chat::DEFAULT_CONTAINER_NAME).await?;
            println!("✅ Container started successfully");
        }
        ContainerAction::Stop => {
            println!("⏹️  Stopping VS Code agent container...");
            copilot_chat::stop_container(copilot_chat::DEFAULT_CONTAINER_NAME).await?;
            println!("✅ Container stopped successfully");
        }
        ContainerAction::Remove => {
            println!("🗑️  Removing VS Code agent container...");
            copilot_chat::remove_container(copilot_chat::DEFAULT_CONTAINER_NAME).await?;
            println!("✅ Container removed successfully");
        }
        ContainerAction::Status => {
            println!("📊 VS Code agent container status:");
            
            let status = copilot_chat::get_container_status(copilot_chat::DEFAULT_CONTAINER_NAME).await?;
            if status.trim().is_empty() {
                println!("   ❌ No container found");
                println!("   Run: vscode-agent container create");
            } else {
                println!("{}", status);
            }
        }
        ContainerAction::Setup => {
            println!("⚙️  Setting up GitHub Copilot in container...");
            println!("   Installing Copilot extension...");
            
            match copilot_chat::setup_copilot_in_container(copilot_chat::DEFAULT_CONTAINER_NAME).await {
                Ok(()) => {
                    println!("✅ Copilot extension installed");
                    println!("💡 Next steps:");
                    println!("   1. Run: docker exec -it {} code", copilot_chat::DEFAULT_CONTAINER_NAME);
                    println!("   2. Sign in to GitHub Copilot");
                    println!("   3. Set VSCODE_AGENT_USE_CONTAINER=true");
                }
                Err(e) => {
                    println!("⚠️  Extension install may have failed: {}", e);
                    println!("💡 Manual setup:");
                    println!("   1. Run: docker exec -it {} code", copilot_chat::DEFAULT_CONTAINER_NAME);
                    println!("   2. Install GitHub Copilot extension manually");
                    println!("   3. Sign in to GitHub Copilot");
                }
            }
        }
    }
    
    Ok(())
}
