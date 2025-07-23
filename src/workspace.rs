use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct TaskWorkspace {
    pub name: String,
    pub path: PathBuf,
    pub task: String,
}

#[derive(Debug)]
pub struct BuildResult {
    pub success: bool,
    pub tests_passed: u32,
    pub tests_failed: u32,
    pub errors: Vec<String>,
}

pub async fn create_task_workspace(task: &str) -> Result<TaskWorkspace> {
    let base_dir = std::env::var("WORKSPACE_DIR")
        .unwrap_or_else(|_| "./workspaces".to_string());

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let task_slug = task
        .chars()
        .take(30)
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    let workspace_name = format!("{}_{}", timestamp, task_slug);
    let workspace_path = PathBuf::from(base_dir).join(&workspace_name);

    tokio::fs::create_dir_all(&workspace_path).await?;

    Ok(TaskWorkspace {
        name: workspace_name,
        path: workspace_path,
        task: task.to_string(),
    })
}

pub async fn build_and_test(workspace: &TaskWorkspace) -> Result<BuildResult> {
    println!("🔧 Building project...");

    let output = tokio::process::Command::new("cargo")
        .arg("check")
        .current_dir(&workspace.path)
        .output()
        .await?;

    let check_success = output.status.success();
    let mut errors = Vec::new();

    if !check_success {
        let stderr = String::from_utf8_lossy(&output.stderr);
        errors.extend(
            stderr.lines()
                .filter(|line| line.contains("error:"))
                .map(|line| line.to_string())
                .collect::<Vec<_>>()
        );
    }

    // Run tests if build succeeded
    let (tests_passed, tests_failed) = if check_success {
        let test_output = tokio::process::Command::new("cargo")
            .arg("test")
            .current_dir(&workspace.path)
            .output()
            .await?;

        let test_stdout = String::from_utf8_lossy(&test_output.stdout);
        parse_test_results(&test_stdout)
    } else {
        (0, 0)
    };

    Ok(BuildResult {
        success: check_success && tests_failed == 0,
        tests_passed,
        tests_failed,
        errors,
    })
}

fn parse_test_results(output: &str) -> (u32, u32) {
    for line in output.lines() {
        if line.contains("test result:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let mut passed = 0;
            let mut failed = 0;

            for (i, part) in parts.iter().enumerate() {
                if part == &"passed;" && i > 0 {
                    passed = parts[i-1].parse().unwrap_or(0);
                } else if part == &"failed;" && i > 0 {
                    failed = parts[i-1].parse().unwrap_or(0);
                }
            }

            return (passed, failed);
        }
    }
    (0, 0)
}

pub async fn list_workspaces() -> Result<()> {
    let base_dir = std::env::var("WORKSPACE_DIR")
        .unwrap_or_else(|_| "./workspaces".to_string());

    println!("📁 Generated Workspaces:");
    if let Ok(mut entries) = tokio::fs::read_dir(&base_dir).await {
        let mut count = 0;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                println!("   {}", entry.file_name().to_string_lossy());
                count += 1;
            }
        }
        if count == 0 {
            println!("   No workspaces found");
        }
    } else {
        println!("   No workspaces directory found");
    }
    Ok(())
}

pub async fn cleanup_old_workspaces() -> Result<()> {
    let base_dir = std::env::var("WORKSPACE_DIR")
        .unwrap_or_else(|_| "./workspaces".to_string());

    if let Ok(mut entries) = tokio::fs::read_dir(&base_dir).await {
        let mut workspaces = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                workspaces.push(entry.path());
            }
        }

        workspaces.sort();
        let to_remove = workspaces.len().saturating_sub(5);

        for workspace in workspaces.iter().take(to_remove) {
            tokio::fs::remove_dir_all(workspace).await?;
            println!("🗑️ Removed {}", workspace.file_name().unwrap().to_string_lossy());
        }

        println!("✅ Cleaned up {} old workspaces", to_remove);
    }
    Ok(())
}
