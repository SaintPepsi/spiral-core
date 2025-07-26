//! 🎯 SPIRAL CORE CONSTANTS: System-wide configuration values
//! DECISION ARCHAEOLOGY: Each constant includes reasoning for its specific value
//! AUDIT: Verify these values align with deployment constraints and user expectations

// 🌐 API CONFIGURATION
/// 📅 ANTHROPIC API VERSION: Locked to stable API version
/// Why: "2023-06-01" provides stable feature set with latest capabilities  
/// Alternative: Latest version (rejected: potential breaking changes in production)
pub const ANTHROPIC_API_VERSION: &str = "2023-06-01";

// ⚙️ TASK PROCESSING CONFIGURATION
/// ⏱️ TASK POLLING INTERVAL: Balance between responsiveness and CPU usage
/// Why: 100ms provides near-real-time feel without excessive CPU overhead
/// Alternative: 50ms (rejected: 2x CPU usage), 500ms (rejected: sluggish UX)
/// Calculation: ~10 polls/second = reasonable for human-perceived responsiveness
pub const TASK_POLL_INTERVAL_MS: u64 = 100;

/// 📊 DEFAULT TIME ESTIMATE: Conservative baseline for task complexity estimation
/// Why: 30min baseline balances underestimation risk with user expectations
/// Research: Most coding tasks fall in 15-60min range, 30min is safe middle ground
/// Alternative: 15min (rejected: frequent underestimation), 60min (rejected: discourages use)
pub const DEFAULT_TIME_ESTIMATE_MINUTES: u32 = 30;

/// 🚦 MAX QUEUE SIZE: Memory protection for 8GB VPS deployment
/// Why: 1000 tasks ≈ 1MB RAM (1KB avg task) provides safety margin
/// Calculation: 8GB total - 2GB OS - 4GB app = 2GB buffer ÷ 1KB = 2M tasks theoretical
/// Conservative: 1K tasks allows for larger tasks and system overhead
/// Alternative: 10K (rejected: potential OOM), 100 (rejected: too restrictive)
pub const MAX_QUEUE_SIZE: usize = 1000;

/// 📚 MAX STORED TASKS: Historical data retention vs memory usage balance
/// Why: 10K tasks provides good audit trail without memory pressure
/// Retention: ~1 week of high activity (10K tasks ÷ 24 hours ÷ 60 minutes = ~7 tasks/min)
/// Alternative: 50K (rejected: memory pressure), 1K (rejected: insufficient history)
pub const MAX_STORED_TASKS: usize = 10000;

/// 📋 MAX STORED RESULTS: Balance debugging capability with memory efficiency
/// Why: 5K results = smaller than tasks (results have completion status only)
/// Reasoning: Results are accessed less frequently than task status
/// Alternative: Same as tasks (rejected: results less critical), 1K (rejected: too limited)
pub const MAX_STORED_RESULTS: usize = 5000;

/// 🧹 CLEANUP INTERVAL: Memory management frequency vs overhead balance
/// Why: 5min (300s) provides regular cleanup without constant overhead
/// Impact: Cleanup runs ~288 times/day (acceptable CPU usage)
/// Alternative: 1min (rejected: too frequent), 15min (rejected: memory buildup risk)
pub const CLEANUP_INTERVAL_SECS: u64 = 300;

// 💬 DISCORD INTEGRATION CONFIGURATION
/// ✂️ DISCORD MESSAGE TRUNCATION: Discord 2000 char limit with safety buffer
/// Why: 1000 chars provides full context while staying well under Discord limit
/// Safety: 50% of Discord limit prevents message rejection from formatting overhead
/// Alternative: 2000 (rejected: risk of rejection), 500 (rejected: insufficient context)
pub const DISCORD_MESSAGE_TRUNCATE_LENGTH: usize = 1000;

/// 🏷️ DISCORD TASK ID DISPLAY: Human-readable vs uniqueness balance
/// Why: 8 chars provides reasonable uniqueness (36^8 = 2.8×10^12 combinations)
/// UX: Short enough for Discord display, long enough to avoid collisions
/// Alternative: 4 chars (rejected: collision risk), 16 chars (rejected: too long for display)
pub const DISCORD_TASK_ID_DISPLAY_LENGTH: usize = 8;

// 🔧 CODE PROCESSING CONFIGURATION
/// 📝 CODE SNIPPET TRUNCATION: AI context limit vs processing accuracy balance
/// Why: 500 chars captures most function signatures and key context
/// Reasoning: Longer snippets diminish in relevance, increase token cost
/// Alternative: 1000 (rejected: token cost), 250 (rejected: insufficient context)
pub const CODE_SNIPPET_TRUNCATION_LENGTH: usize = 500;

/// 👁️ TASK DESCRIPTION PREVIEW: UI display vs information density balance
/// Why: 50 chars shows task essence without UI clutter
/// UX: Fits comfortably in most UI components, provides meaningful preview
/// Alternative: 100 (rejected: UI overflow), 25 (rejected: insufficient information)
pub const TASK_DESCRIPTION_PREVIEW_LENGTH: usize = 50;

// 🛡️ DEFAULT FALLBACKS: Graceful degradation when analysis fails
/// 🎯 DEFAULT PROGRAMMING SKILL: Generic fallback for task analysis
/// Why: "programming" is universally applicable when specific skills unclear
/// Safety: Prevents empty skill arrays that could cause UI/logic issues
pub const DEFAULT_PROGRAMMING_SKILL: &str = "programming";

/// 📋 DEFAULT IMPLEMENTATION APPROACH: Safe methodology when analysis incomplete
/// Why: "Incremental development" promotes good practices in unclear situations
/// Philosophy: Encourages testing and validation even for simple tasks
pub const DEFAULT_IMPLEMENTATION_APPROACH: &str =
    "Incremental development with testing and validation";

/// ⚠️ DEFAULT IMPLEMENTATION CHALLENGE: Neutral challenge assessment fallback
/// Why: "Standard requirements" avoids over/under-stating complexity
/// Psychology: Neutral framing prevents panic or overconfidence
pub const DEFAULT_IMPLEMENTATION_CHALLENGE: &str = "Standard implementation requirements";
