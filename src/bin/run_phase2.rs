//! Run only Phase 2 (Core Rust Compliance Checks) of the validation pipeline
//!
//! Run with: cargo run --bin run_phase2 [--no-claude]
//!
//! This runs the ACTUAL Phase 2 validation with retry logic and Claude agent fixes.
//! Uses the SOLID Phase2Executor - no mocking, no unnecessary dependencies.
//!
//! Options:
//!   --no-claude    Run without Claude Code integration (no automatic fixes)

use spiral_core::config::ClaudeCodeConfig;
use spiral_core::discord::self_update::pipeline::{ComplianceCheck, Phase2Executor};
use spiral_core::error::Result;
use std::env;
use std::time::Instant;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let use_claude = !args.contains(&"--no-claude".to_string());

    println!("🚀 Starting Phase 2 Validation (Core Rust Compliance Checks)");
    println!("{}", "=".repeat(60));
    println!("This runs the ACTUAL Phase 2 validation pipeline with:");
    println!("  • Real compliance checks");
    println!("  • Retry logic (up to 3 attempts per check)");
    println!("  • Auto-fix for formatting (cargo fmt)");
    println!(
        "  • Claude Code CLI integration ({})",
        if use_claude {
            "ENABLED by default"
        } else {
            "DISABLED via --no-claude"
        }
    );
    println!("  • SOLID principles - truly independent execution");
    println!("{}", "=".repeat(60));

    // Create a Phase 2 executor - no Phase 1 dependencies!
    let mut executor = if use_claude {
        match create_claude_executor().await {
            Ok(exec) => {
                println!("\n✅ Claude Code CLI configured successfully");
                println!("   Automatic fixes will be attempted for failed checks");
                exec
            }
            Err(e) => {
                println!("\n⚠️  Failed to configure Claude Code CLI: {e}");
                println!("   Continuing without automatic Claude fixes");
                println!("   Hint: Ensure 'claude' is in PATH or set CLAUDE_BINARY_PATH");
                Phase2Executor::new()
            }
        }
    } else {
        println!("\n⚠️  Claude Code CLI integration: DISABLED");
        println!("   Running with --no-claude flag");
        println!("   Only auto-fix (cargo fmt) will be attempted");
        Phase2Executor::new()
    };

    info!("Running Phase 2 Core Rust Compliance Checks independently...");

    // Run Phase 2 validation - truly independent per SOLID principles
    let start = Instant::now();
    let phase2_result = executor.execute().await?;
    let duration = start.elapsed();

    // Display results
    println!("\n📋 Phase 2 Results (Core Rust Compliance Checks):");
    println!("{}", "=".repeat(60));

    println!("\n1️⃣ Compilation Check:");
    print_compliance_check(&phase2_result.checks.compilation);

    println!("\n2️⃣ Test Suite:");
    print_compliance_check(&phase2_result.checks.tests);

    println!("\n3️⃣ Formatting (cargo fmt):");
    print_compliance_check(&phase2_result.checks.formatting);

    println!("\n4️⃣ Clippy Lints:");
    print_compliance_check(&phase2_result.checks.clippy);

    println!("\n5️⃣ Documentation Build:");
    print_compliance_check(&phase2_result.checks.docs);

    println!("\n{}", "=".repeat(60));
    println!("⏱️  Total duration: {:.2}s", duration.as_secs_f32());

    // Check if any retries triggered (would loop back to Phase 1 in full pipeline)
    let total_retries = phase2_result.checks.compilation.retries
        + phase2_result.checks.tests.retries
        + phase2_result.checks.formatting.retries
        + phase2_result.checks.clippy.retries
        + phase2_result.checks.docs.retries;

    if total_retries > 0 {
        println!("\n🔄 Total retries across all checks: {total_retries}");
        println!("⚠️  In the full pipeline, ANY retry would trigger a loop back to Phase 1!");
    }

    // Check overall result
    let all_passed = phase2_result.checks.compilation.passed
        && phase2_result.checks.tests.passed
        && phase2_result.checks.formatting.passed
        && phase2_result.checks.clippy.passed
        && phase2_result.checks.docs.passed;

    if all_passed {
        println!("\n✅ All Phase 2 checks PASSED!");
        if total_retries > 0 {
            println!("⚠️  However, {total_retries} retries were needed.");
            println!("    In full pipeline, this would trigger Phase 1 re-validation.");
        }
        std::process::exit(0);
    } else {
        println!("\n❌ Some Phase 2 checks FAILED.");

        // List which checks failed
        let mut failed_checks = vec![];
        if !phase2_result.checks.compilation.passed {
            failed_checks.push("Compilation");
        }
        if !phase2_result.checks.tests.passed {
            failed_checks.push("Tests");
        }
        if !phase2_result.checks.formatting.passed {
            failed_checks.push("Formatting");
        }
        if !phase2_result.checks.clippy.passed {
            failed_checks.push("Clippy");
        }
        if !phase2_result.checks.docs.passed {
            failed_checks.push("Documentation");
        }

        println!("\n❌ Failed checks: {}", failed_checks.join(", "));

        println!("\n💡 Next steps:");
        println!("  1. Fix the issues manually, or");
        println!("  2. Ensure 'claude' is in PATH for automatic fixes, or");
        println!("  3. Run 'cargo fmt' to fix formatting issues");
        println!("  4. Run with --no-claude to skip Claude integration");

        std::process::exit(1);
    }
}

/// Create a Phase2Executor with Claude Code CLI support
/// 🏗️ ARCHITECTURE DECISION: Use Claude Code CLI for validation fixes
/// Why: Claude can understand and fix complex compilation/test errors
/// Alternative: Manual fixes only (rejected: slower iteration)
/// Audit: Verify Claude binary path and permissions
async fn create_claude_executor() -> Result<Phase2Executor> {
    // Build Claude Code config from environment
    let config = ClaudeCodeConfig {
        claude_binary_path: env::var("CLAUDE_BINARY_PATH").ok(),
        working_directory: env::var("CLAUDE_WORKING_DIR").ok(),
        timeout_seconds: env::var("CLAUDE_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .unwrap_or(300),
        permission_mode: env::var("CLAUDE_PERMISSION_MODE")
            .unwrap_or_else(|_| "acceptEdits".to_string()),
        allowed_tools: env::var("CLAUDE_ALLOWED_TOOLS")
            .unwrap_or_else(|_| "Edit,Write,Read,Bash,MultiEdit,Glob,Grep,TodoWrite".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect(),
        workspace_cleanup_after_hours: 24,
        max_workspace_size_mb: 500,
    };

    Phase2Executor::with_claude(config).await
}

/// Print compliance check result with detailed information
fn print_compliance_check(check: &ComplianceCheck) {
    println!(
        "  Status: {}",
        if check.passed {
            "✅ PASSED"
        } else {
            "❌ FAILED"
        }
    );

    if check.retries > 0 {
        println!(
            "  Retries: {} (auto-fix or Claude attempted)",
            check.retries
        );
    }

    if let Some(errors) = &check.errors {
        if !errors.is_empty() {
            println!("  Issues found:");
            for error in errors {
                // Split by newlines and display each line
                let lines: Vec<&str> = error.lines().collect();
                for (i, line) in lines.iter().take(10).enumerate() {
                    if i == 0 {
                        println!("    • {line}");
                    } else {
                        println!("      {line}");
                    }
                }
                if lines.len() > 10 {
                    println!("      ... ({} more lines)", lines.len() - 10);
                }
            }
        }
    }
}
