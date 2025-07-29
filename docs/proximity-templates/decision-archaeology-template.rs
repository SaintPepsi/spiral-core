//! 🎯 DECISION ARCHAEOLOGY TEMPLATE
//! Based on spiral-core/src/constants.rs - industry-leading example
//! Pattern: Every constant includes mathematical reasoning and alternatives

// 🌐 [CATEGORY NAME] CONFIGURATION
/// 📅 [CONSTANT PURPOSE]: [Brief description of what this controls]
/// Why: [Primary reasoning with specific values/research]
/// Alternative: [What was rejected and specific reason]  
/// Calculation: [Mathematical reasoning where applicable]
/// Impact: [Performance/security/memory implications]
/// Research: [External research/standards referenced]
pub const EXAMPLE_CONSTANT: Type = value;

// Example from spiral-core (EXCEPTIONAL):
/// ⏱️ TASK POLLING INTERVAL: Balance between responsiveness and CPU usage
/// Why: 100ms provides near-real-time feel without excessive CPU overhead
/// Alternative: 50ms (rejected: 2x CPU usage), 500ms (rejected: sluggish UX)
/// Calculation: ~10 polls/second = reasonable for human-perceived responsiveness
pub const TASK_POLL_INTERVAL_MS: u64 = 100;

/// 🚦 MAX QUEUE SIZE: Memory protection for 8GB VPS deployment
/// Why: 1000 tasks ≈ 1MB RAM (1KB avg task) provides safety margin
/// Calculation: 8GB total - 2GB OS - 4GB app = 2GB buffer ÷ 1KB = 2M tasks theoretical
/// Conservative: 1K tasks allows for larger tasks and system overhead
/// Alternative: 10K (rejected: potential OOM), 100 (rejected: too restrictive)
pub const MAX_QUEUE_SIZE: usize = 1000;

// 🛡️ FALLBACK VALUES: Graceful degradation pattern
/// 🎯 [PURPOSE]: [When this fallback is used]
/// Why: [Reasoning for this specific fallback value]
/// Safety: [How this prevents system failures]
/// Philosophy: [Design principle this supports]
pub const DEFAULT_FALLBACK: &str = "safe_default_value";

// Pattern Notes:
// 1. Every constant has 📝 emoji + purpose
// 2. "Why:" explains the specific choice
// 3. "Alternative:" shows what was rejected  
// 4. Calculations include mathematical reasoning
// 5. Security/performance implications stated
// 6. Conservative defaults with safety margins
// 7. Research citations where applicable