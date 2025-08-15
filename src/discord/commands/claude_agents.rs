use super::CommandHandler;
use crate::discord::spiral_constellation_bot::SpiralConstellationBot;
use serenity::{model::channel::Message, prelude::Context};
use std::path::Path;
use tracing::info;

/// 🏗️ ARCHITECTURE DECISION: Hardcoded Claude agent paths
/// Why: Claude agents are checked into version control at fixed locations
/// Alternative: Dynamic discovery (rejected: adds complexity without benefit)
/// Trade-off: Less flexible but more predictable and faster
/// Audit: Verify these paths exist in repository structure
const CLAUDE_DIR: &str = ".claude";
const UTILITY_AGENTS_DIR: &str = ".claude/utility-agents";
const PHASE1_AGENTS_DIR: &str = ".claude/validation-agents/phase1";
const PHASE2_AGENTS_DIR: &str = ".claude/validation-agents/phase2";
const ANALYSIS_AGENTS_DIR: &str = ".claude/validation-agents/analysis";

/// 📐 SOLID: Open-Closed Principle
/// New commands can be added without modifying existing command structure
/// Both commands route to same handler for DRY compliance
const CLAUDE_AGENTS: &str = "!spiral claude-agents";
const AGENTS_ALT: &str = "!spiral agents"; // Convenience alias for discoverability

pub struct ClaudeAgentsCommand {
    // Claude agents command doesn't need state
}

impl Default for ClaudeAgentsCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeAgentsCommand {
    pub fn new() -> Self {
        Self {}
    }

    /// 🔍 AUDIT CHECKPOINT: Claude agent listing
    /// Critical: This shows users what validation/utility agents are available
    /// Verify: Agent descriptions match actual agent capabilities in .md files
    /// Security: No sensitive paths or implementation details exposed
    fn list_claude_agents(&self) -> String {
        let mut response = "🤖 **Available Claude Agents**\n\n".to_string();

        // Utility Agents
        response.push_str("**🛠️ Utility Agents**\n");
        response.push_str("• `agent-validator` - Validates agent configuration and structure\n");
        response
            .push_str("• `claude-improver` - Automated code quality analysis and refactoring\n");
        response
            .push_str("• `documentation-lean-analyzer` - Analyzes and optimizes documentation\n");
        response.push_str("• `dry-analyzer` - Detection and elimination of code duplication\n");
        response.push_str("• `package-updater-rust` - Rust dependency management and updates\n\n");

        // Validation Agents - Phase 1
        response.push_str("**📋 Phase 1 Validation Agents**\n");
        response.push_str(
            "• `code-review-standards` - Comprehensive code review against project standards\n",
        );
        response.push_str(
            "• `comprehensive-testing` - Focus on pressure points and critical scenarios\n",
        );
        response.push_str("• `security-audit` - Identify vulnerabilities and unsafe patterns\n");
        response.push_str("• `system-integration` - Verify no regressions or breaking changes\n\n");

        // Validation Agents - Phase 2
        response.push_str("**🔧 Phase 2 Compliance Agents**\n");
        response.push_str("• `clippy-resolver` - Fixes Clippy linting issues\n");
        response.push_str("• `compilation-fixer` - Resolves compilation errors\n");
        response.push_str("• `doc-builder` - Fixes documentation build issues\n");
        response.push_str("• `formatting-fixer` - Applies cargo fmt corrections\n");
        response.push_str("• `test-failure-analyzer` - Analyzes and fixes test failures\n\n");

        // Analysis Agents
        response.push_str("**📊 Analysis Agents**\n");
        response.push_str("• `failure-analyzer` - Analyzes validation pipeline failures\n");
        response.push_str("• `success-analyzer` - Reports on successful validations\n");
        response.push_str("• `success-with-issues-analyzer` - Analyzes partial successes\n\n");

        // 🛡️ SECURITY DECISION: Filesystem access validation
        // Why: Check directory exists before attempting to read
        // Alternative: Try-catch on read operations (rejected: less clear intent)
        // Risk: None - read-only operation on known paths
        let claude_dir = Path::new(CLAUDE_DIR);
        if claude_dir.exists() {
            response.push_str("✅ **Status:** Claude agents directory found\n");

            // 📊 PERFORMANCE DECISION: Shallow directory scan
            // Why: Only count files, don't read contents (faster)
            // Alternative: Deep scan with validation (rejected: unnecessary overhead)
            // Trade-off: Count may include non-.md files
            // Audit: Verify only .md files exist in these directories
            let mut agent_count = 0;
            if let Ok(entries) = std::fs::read_dir(UTILITY_AGENTS_DIR) {
                agent_count += entries.filter_map(Result::ok).count();
            }
            if let Ok(entries) = std::fs::read_dir(PHASE1_AGENTS_DIR) {
                agent_count += entries.filter_map(Result::ok).count();
            }
            if let Ok(entries) = std::fs::read_dir(PHASE2_AGENTS_DIR) {
                agent_count += entries.filter_map(Result::ok).count();
            }
            if let Ok(entries) = std::fs::read_dir(ANALYSIS_AGENTS_DIR) {
                agent_count += entries.filter_map(Result::ok).count();
            }

            response.push_str(&format!("📁 **Total Agent Files:** {agent_count}\n\n"));
        } else {
            response.push_str("⚠️ **Warning:** Claude agents directory not found\n\n");
        }

        response.push_str("**Usage:**\n");
        response.push_str("• These agents are used during self-update validation\n");
        response.push_str("• Phase 1 agents perform quality checks\n");
        response.push_str("• Phase 2 agents fix compliance issues\n");
        response.push_str("• Analysis agents provide detailed reports\n\n");

        response
            .push_str("💡 *Use `!spiral self-update help` to learn about the validation pipeline*");

        response
    }
}

impl CommandHandler for ClaudeAgentsCommand {
    /// 🔄 DRY PATTERN: Dual command routing
    /// Both "!spiral agents" and "!spiral claude-agents" route here
    /// Why: User convenience without code duplication
    /// Audit: Ensure both commands are registered in mod.rs
    async fn handle(
        &self,
        content: &str,
        msg: &Message,
        _ctx: &Context,
        _bot: &SpiralConstellationBot,
    ) -> Option<String> {
        let content_lower = content.to_lowercase();

        // 🏗️ ARCHITECTURE DECISION: Pattern matching for command routing
        // Why: Consistent with other command handlers in the system
        // Alternative: Exact match (rejected: prevents subcommands)
        // Trade-off: Slightly slower but more flexible
        match content_lower.as_str() {
            cmd if cmd.starts_with(CLAUDE_AGENTS) || cmd.starts_with(AGENTS_ALT) => {
                // 🔍 AUDIT CHECKPOINT: User action logging
                // Critical: Track who's querying agent information
                // Security: Log user ID for audit trail
                info!(
                    "[ClaudeAgentsCommand] Listing Claude agents for user {} ({})",
                    msg.author.name,
                    msg.author.id.get()
                );
                Some(self.list_claude_agents())
            }
            _ => None,
        }
    }

    fn command_prefix(&self) -> &str {
        CLAUDE_AGENTS
    }

    fn description(&self) -> &str {
        "List all available Claude validation and utility agents"
    }
}
