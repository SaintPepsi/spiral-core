use spiral_core::discord::startup::start_discord_with_orchestrator;
use spiral_core::{
    agents::AgentOrchestrator,
    api::ApiServer,
    config::Config,
    monitoring::{MonitoringConfig, SystemMonitor},
    security,
};
use std::sync::Arc;
use tokio::signal;
use tracing::{debug, error, info, warn, Level};

/// 🚀 SPIRAL CORE MAIN ENTRY POINT
/// DECISION: Graceful startup/shutdown with proper resource management
/// Why: Ensure clean state transitions and prevent data corruption
/// Alternative: Simple crash on exit (rejected: loses in-flight work)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 📊 STARTUP PHASE 1: Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    info!("Starting Spiral Core Agent Orchestration System");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    info!("PID: {}", std::process::id());

    // 📊 STARTUP PHASE 2: Load and validate configuration
    info!("Loading configuration...");
    let config = match Config::load() {
        Ok(cfg) => {
            info!("Configuration loaded successfully");
            cfg
        }
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(anyhow::Error::from(e));
        }
    };

    // 📊 STARTUP PHASE 3: Perform startup validations
    perform_startup_validations(&config).await?;

    // 📊 STARTUP PHASE 4: Initialize core components
    info!("Initializing agent orchestrator...");
    let orchestrator = match AgentOrchestrator::new(config.clone()).await {
        Ok(orch) => {
            info!("Agent orchestrator initialized successfully");
            Arc::new(orch)
        }
        Err(e) => {
            error!("Failed to initialize agent orchestrator: {}", e);
            return Err(anyhow::Error::from(e));
        }
    };

    // 🤖 STARTUP PHASE 4.5: Initialize Discord integration (optional)
    let discord_handle = if !config.discord.token.is_empty() {
        info!("[Main] Discord token detected, preparing Discord integration...");
        debug!("[Main] Discord token length: {}", config.discord.token.len());
        
        let config_clone = config.clone();
        let orchestrator_clone = orchestrator.clone();

        info!("[Main] Spawning Discord integration task...");
        Some(tokio::spawn(async move {
            info!("[Main] Discord integration task started");
            match start_discord_with_orchestrator(config_clone, orchestrator_clone).await {
                Ok(()) => {
                    info!("[Main] Discord integration completed successfully");
                },
                Err(e) => {
                    error!("[Main] Discord integration failed: {}", e);
                    error!("[Main] Discord error details: {:?}", e);
                }
            }
        }))
    } else {
        warn!("[Main] Discord token not provided - Discord integration disabled");
        None
    };

    // 🔧 STARTUP PHASE 4.6: Initialize system monitoring
    info!("Initializing system monitoring...");
    let system_monitor = SystemMonitor::new(MonitoringConfig::default());

    // Register components for monitoring (Claude client will be registered by the orchestrator)
    if let Err(e) = system_monitor.start_monitoring().await {
        error!("Failed to start system monitoring: {}", e);
        return Err(anyhow::Error::from(e));
    }
    info!("System monitoring initialized successfully");

    let system_monitor = Arc::new(system_monitor);

    info!("Initializing API server...");
    let api_server = match ApiServer::new(config.clone(), orchestrator.clone()) {
        Ok(server) => {
            info!("API server initialized successfully");
            server.with_system_monitor(system_monitor.clone())
        }
        Err(e) => {
            error!("Failed to initialize API server: {}", e);
            return Err(anyhow::Error::from(e));
        }
    };

    // 📊 STARTUP PHASE 5: Setup graceful shutdown handler
    let shutdown_signal = setup_shutdown_handler();

    info!("Spiral Core startup complete - all systems operational");

    // 🔄 MAIN EXECUTION LOOP: Run services with graceful shutdown
    tokio::select! {
        result = orchestrator.run() => {
            if let Err(e) = result {
                error!("Agent orchestrator failed: {}", e);
            }
            info!("Agent orchestrator stopped");
        }
        result = api_server.run() => {
            if let Err(e) = result {
                error!("API server failed: {}", e);
            }
            info!("API server stopped");
        }
        _result = async {
            if let Some(handle) = discord_handle {
                handle.await.unwrap_or_else(|e| {
                    error!("Discord task panicked: {}", e);
                });
            } else {
                // If no Discord integration, just wait forever
                std::future::pending::<()>().await;
            }
        } => {
            info!("Discord integration stopped");
        }
        _ = shutdown_signal => {
            info!("Shutdown signal received, initiating graceful shutdown...");
        }
    }

    // 📊 SHUTDOWN PHASE: Clean up resources
    perform_graceful_shutdown(orchestrator).await;

    info!("Spiral Core shutdown complete");
    Ok(())
}

/// 🛡️ STARTUP VALIDATION: Ensure system prerequisites are met
/// AUDIT CHECKPOINT: Critical startup checks before service initialization
async fn perform_startup_validations(config: &Config) -> anyhow::Result<()> {
    info!("Performing startup validations...");

    // Check Claude Code CLI availability
    if let Some(claude_binary) = &config.claude_code.claude_binary_path {
        if tokio::fs::metadata(claude_binary).await.is_err() {
            error!("Claude Code binary not found at: {}", claude_binary);
            return Err(anyhow::anyhow!("Claude Code CLI not available"));
        }
    }

    // 🔐 SECURE API KEY VALIDATION: Ensure cryptographically secure API key exists
    // CRITICAL: Use config key if exists, otherwise generate secure file-based key
    // SECURITY: Authentication is always enabled
    match security::ensure_api_key_exists(config.api.api_key.as_deref()) {
        Ok(api_key) => {
            info!(
                "API key validation successful (length: {} chars)",
                api_key.len()
            );
            // Note: Don't log the actual key for security
        }
        Err(e) => {
            error!("Failed to ensure secure API key exists: {}", e);
            return Err(anyhow::anyhow!("API key security validation failed: {}", e));
        }
    }

    // Check workspace directory permissions
    let workspace_dir = config
        .claude_code
        .working_directory
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .join("claude-workspaces")
        });

    if workspace_dir.exists() {
        match tokio::fs::metadata(&workspace_dir).await {
            Ok(metadata) if !metadata.permissions().readonly() => {
                info!("Workspace directory validated: {:?}", workspace_dir);
            }
            _ => {
                error!("Workspace directory not writable: {:?}", workspace_dir);
                return Err(anyhow::anyhow!("Workspace directory not writable"));
            }
        }
    }

    // Validate Discord configuration if token is provided
    if config.discord.token.is_empty() {
        info!("Discord token not provided - Discord integration disabled");
    } else if config.discord.token == "your-discord-token" {
        warn!("Discord token appears to be placeholder - Discord integration will fail");
    } else {
        info!("Discord integration enabled");
    }

    info!("All startup validations passed");
    Ok(())
}

/// 🛑 SHUTDOWN HANDLER: Setup signal handling for graceful shutdown
/// DECISION: Handle multiple shutdown signals for cross-platform compatibility
async fn setup_shutdown_handler() {
    let ctrl_c = async {
        match signal::ctrl_c().await {
            Ok(()) => {}
            Err(e) => {
                error!("Failed to install Ctrl+C handler: {}", e);
                // Still allow the program to run, but warn about missing handler
                warn!("Graceful shutdown via Ctrl+C will not be available");
                // Wait forever since we can't handle the signal
                std::future::pending::<()>().await
            }
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(e) => {
                error!("Failed to install SIGTERM handler: {}", e);
                warn!("Graceful shutdown via SIGTERM will not be available");
                // Wait forever since we can't handle the signal
                std::future::pending::<()>().await
            }
        }
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

/// 🧹 GRACEFUL SHUTDOWN: Clean up resources before exit
/// AUDIT CHECKPOINT: Ensure all resources are properly released
async fn perform_graceful_shutdown(orchestrator: Arc<AgentOrchestrator>) {
    info!("Beginning graceful shutdown sequence...");

    // Give in-flight requests time to complete
    info!("Waiting for in-flight requests to complete (max 30s)...");
    tokio::time::timeout(
        std::time::Duration::from_secs(30),
        wait_for_tasks_completion(&orchestrator),
    )
    .await
    .unwrap_or_else(|_| {
        warn!("Timeout waiting for tasks to complete, forcing shutdown");
    });

    // Clean up Claude Code workspaces
    if let Ok(claude_client) = orchestrator.get_claude_client() {
        info!("Cleaning up Claude Code workspaces...");
        if let Err(e) = claude_client.cleanup_old_workspaces().await {
            warn!("Failed to clean up workspaces: {}", e);
        }
    }

    // Flush any remaining logs
    info!("Flushing logs...");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

/// Wait for all active tasks to complete
async fn wait_for_tasks_completion(orchestrator: &AgentOrchestrator) {
    let mut check_interval = tokio::time::interval(std::time::Duration::from_secs(1));
    let start_time = std::time::Instant::now();

    loop {
        check_interval.tick().await;

        let status = orchestrator.get_system_status().await;
        let active_tasks: usize = status.agents.values().filter(|s| s.is_busy).count();

        if active_tasks == 0 {
            info!("All tasks completed");
            break;
        }

        let elapsed = start_time.elapsed();
        info!(
            "Waiting for {} active tasks to complete ({}s elapsed)",
            active_tasks,
            elapsed.as_secs()
        );

        if elapsed > std::time::Duration::from_secs(25) {
            warn!(
                "Approaching shutdown timeout with {} tasks still active",
                active_tasks
            );
        }
    }
}
