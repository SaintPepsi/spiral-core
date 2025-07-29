//! 🚀 STARTUP/SHUTDOWN DECISION TEMPLATE  
//! Based on spiral-core/src/main.rs - comprehensive lifecycle management
//! Pattern: Every phase includes decision reasoning and audit checkpoints

use anyhow::Result;
use tokio::signal;
use tracing::{error, info, warn};

/// 🚀 [SYSTEM NAME] MAIN ENTRY POINT
/// DECISION: Graceful startup/shutdown with proper resource management
/// Why: Ensure clean state transitions and prevent data corruption
/// Alternative: Simple crash on exit (rejected: loses in-flight work)
/// Audit: Verify all resources properly cleaned up
#[tokio::main]
async fn main() -> Result<()> {
    // 📊 STARTUP PHASE 1: Initialize logging
    // DECISION: Debug level during development, configurable in production
    // Why: Need detailed tracing for AI system debugging
    // Alternative: Info level (rejected: insufficient detail for AI coordination)
    setup_logging()?;

    info!("Starting [SYSTEM_NAME] v{}", env!("CARGO_PKG_VERSION"));
    info!("PID: {}", std::process::id());

    // 📊 STARTUP PHASE 2: Load and validate configuration
    // DECISION: Fail fast on configuration errors
    // Why: Better to crash at startup than have subtle runtime failures
    // Alternative: Default values for missing config (rejected: silent misconfiguration)
    let config = load_and_validate_config().await?;

    // 📊 STARTUP PHASE 3: Perform startup validations
    // AUDIT CHECKPOINT: Critical system prerequisites verification
    perform_startup_validations(&config).await?;

    // 📊 STARTUP PHASE 4: Initialize core components
    // DECISION: Explicit initialization order with error handling
    // Why: Dependencies must be available before dependent services start
    // Alternative: Parallel initialization (rejected: complex dependency management)
    let core_service = initialize_core_service(&config).await?;

    // 📊 STARTUP PHASE 5: Setup graceful shutdown handler
    // DECISION: Handle multiple shutdown signals for cross-platform compatibility
    // Why: SIGTERM (containers), SIGINT (Ctrl+C), platform differences
    // Alternative: Only handle SIGTERM (rejected: poor development experience)
    let shutdown_signal = setup_shutdown_handler();

    info!("[SYSTEM_NAME] startup complete - all systems operational");

    // 🔄 MAIN EXECUTION LOOP: Run services with graceful shutdown
    // DECISION: tokio::select! for concurrent service management
    // Why: All services run concurrently, any failure triggers shutdown
    // Alternative: Sequential service startup (rejected: reduces availability)
    tokio::select! {
        result = core_service.run() => {
            handle_service_result("core_service", result).await;
        }
        result = optional_service.run() => {
            handle_service_result("optional_service", result).await;
        }
        _ = shutdown_signal => {
            info!("Shutdown signal received, initiating graceful shutdown...");
        }
    }

    // 📊 SHUTDOWN PHASE: Clean up resources  
    // AUDIT CHECKPOINT: Ensure all resources are properly released
    perform_graceful_shutdown(core_service).await;

    info!("[SYSTEM_NAME] shutdown complete");
    Ok(())
}

/// 🛡️ STARTUP VALIDATION: Ensure system prerequisites are met
/// AUDIT CHECKPOINT: Critical startup checks before service initialization
/// DECISION: Comprehensive validation with specific error messages
/// Why: Clear diagnostics for deployment issues
/// Alternative: Minimal checks (rejected: hard to debug failures)
async fn perform_startup_validations(config: &Config) -> Result<()> {
    info!("Performing startup validations...");

    // 🔍 EXTERNAL DEPENDENCY CHECK: Verify required services/files
    // DECISION: Check file existence with metadata validation
    // Why: Permissions and accessibility matter, not just existence
    // Alternative: Basic file exists check (rejected: doesn't catch permission issues)
    validate_external_dependencies(config).await?;

    // 🔐 SECURITY VALIDATION: Ensure cryptographically secure setup
    // CRITICAL: Authentication is always enabled
    // DECISION: Generate secure keys if missing, never run without auth
    // Why: Security by default, fail closed not open
    // Alternative: Allow no-auth mode (rejected: security risk)
    validate_security_configuration(config).await?;

    // 📁 WORKSPACE VALIDATION: Verify write permissions
    // DECISION: Check permissions early to avoid runtime failures
    // Why: Better to fail at startup than during task execution
    // Alternative: Create directories on-demand (rejected: unclear error points)
    validate_workspace_permissions(config).await?;

    info!("All startup validations passed");
    Ok(())
}

/// 🛑 SHUTDOWN HANDLER: Setup signal handling for graceful shutdown
/// DECISION: Handle multiple shutdown signals for cross-platform compatibility
/// Why: Docker sends SIGTERM, developers use Ctrl+C (SIGINT)
/// Alternative: Only handle one signal type (rejected: platform limitations)
fn setup_shutdown_handler() -> impl std::future::Future<Output = ()> {
    async {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                info!("Received Ctrl+C signal");
            }
            _ = terminate => {
                info!("Received SIGTERM signal");
            }
        }
    }
}

/// 🧹 GRACEFUL SHUTDOWN: Clean up resources before exit
/// AUDIT CHECKPOINT: Ensure all resources are properly released
/// DECISION: Timeout-based shutdown with force-kill fallback
/// Why: Prevent hanging on unresponsive components
/// Alternative: Wait indefinitely (rejected: deployment issues)
async fn perform_graceful_shutdown(service: CoreService) {
    info!("Beginning graceful shutdown sequence...");

    // 🕐 SHUTDOWN TIMEOUT: Balance cleanup time vs restart speed
    // DECISION: 30s timeout provides reasonable cleanup time
    // Why: Complex AI systems need time to save state, but not too long for deployments
    // Alternative: 60s (rejected: too slow for rolling deployments), 10s (rejected: insufficient for cleanup)
    let shutdown_timeout = std::time::Duration::from_secs(30);

    info!("Waiting for in-flight operations to complete (max {}s)...", shutdown_timeout.as_secs());
    
    tokio::time::timeout(shutdown_timeout, wait_for_operations_completion(&service))
        .await
        .unwrap_or_else(|_| {
            warn!("Timeout waiting for operations to complete, forcing shutdown");
        });

    // 🧹 RESOURCE CLEANUP: Specific cleanup steps
    // DECISION: Best-effort cleanup with error logging
    // Why: Shutdown should always succeed, log failures for debugging
    // Alternative: Fail shutdown on cleanup errors (rejected: causes deployment issues)
    perform_cleanup_tasks(&service).await;

    // 📝 FINAL LOGGING: Flush any remaining logs
    // DECISION: Short delay to ensure log delivery
    // Why: Important for debugging shutdown issues
    // Alternative: Immediate exit (rejected: loses important debug info)
    info!("Flushing logs...");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

// Pattern Notes:
// 1. Every phase has 📊 emoji + phase description
// 2. Decisions include reasoning for startup/shutdown choices
// 3. Audit checkpoints at critical validation points
// 4. Timeout values include calculation reasoning
// 5. Cross-platform considerations documented
// 6. Security-first design (fail closed, not open)
// 7. Resource cleanup with error tolerance
// 8. Comprehensive logging for debugging