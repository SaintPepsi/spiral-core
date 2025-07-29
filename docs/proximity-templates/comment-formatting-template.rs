//! 📝 COMMENT FORMATTING TEMPLATE
//! Based on spiral-core consistent comment patterns across all modules
//! Pattern: Structured decision comments with emoji categories and consistent format

// 🎯 DECISION COMMENT STANDARD FORMAT
/// 🧠 DECISION: [What was decided - be specific]
/// Why: [Primary reasoning with concrete details]
/// Alternative: [What was rejected and specific reason why]
/// Audit: [How to verify this decision is working correctly]
/// Impact: [Performance/security/maintenance implications]

// Example from spiral-core:
/// 🧠 AGENT COORDINATION DECISION: Using ClaudeCodeClient as shared intelligence engine
/// Why: Centralizes API management, rate limiting, and response handling across agents
/// Alternative: Individual clients per agent (rejected: increases complexity, API overhead)
/// Audit: Check claude_code.rs:45-60 for client initialization patterns

// 📊 EMOJI CATEGORIES FOR STRUCTURED COMMENTS

// 🧠 DECISION - Major architectural or implementation choices
// 🎯 PURPOSE - What this code accomplishes
// 🔧 CONFIGURATION - Setup and configuration decisions
// 📊 PHASE/STEP - Sequential process steps
// 🛡️ SECURITY - Security-related decisions and audit points
// 🔒 CONCURRENCY - Threading, async, locking decisions
// 🎭 ALGORITHM - Algorithm choice and complexity analysis
// 🏗️ ARCHITECTURE - System design and component relationships
// 📝 DOCUMENTATION - Why certain documentation approaches
// 🧪 TESTING - Test strategy and coverage decisions
// 🔍 MONITORING - Health checks and observability
// 🚀 PERFORMANCE - Optimization decisions and trade-offs
// 📋 PATTERN - Design pattern applications
// 🔄 LIFECYCLE - Startup, shutdown, resource management
// 🧹 CLEANUP - Resource cleanup and memory management

// 📝 COMMENT PLACEMENT PATTERNS

// Module-level decisions (at top of file):
//! 🎯 [MODULE PURPOSE]: [What this module accomplishes]
//! ARCHITECTURE DECISION: [Major design choice for this module]
//! Why: [Reasoning for module design]
//! Alternative: [Rejected approaches]

// Function-level decisions (above function):
/// 🎯 [FUNCTION PURPOSE]: [What this function accomplishes]  
/// ALGORITHM DECISION: [Choice of algorithm/approach]
/// Why: [Reasoning for this implementation]
/// Alternative: [Rejected approaches and why]
/// Performance: [Big O or performance characteristics]
pub fn example_function() {}

// Inline decisions (within function body):
fn example_with_inline_decisions() {
    // 🔒 LOCKING STRATEGY: Why this specific lock order
    // Why: Prevents deadlock by consistent lock acquisition order
    // Alternative: Different lock order (rejected: deadlock risk)
    let _guard1 = lock1.lock().await;
    let _guard2 = lock2.lock().await;
    
    // 🧠 ALGORITHM CHOICE: Why this specific approach
    // Why: O(log n) performance for frequent lookups
    // Alternative: Linear search (rejected: O(n) too slow for scale)
    let result = binary_search(&data, &key);
}

// 🛡️ SECURITY AUDIT CHECKPOINT PATTERN
/// 🛡️ SECURITY AUDIT CHECKPOINT: [What security property is verified]
/// CRITICAL: [Security requirement being enforced]
/// VALIDATION: [How compliance is checked]
/// THREAT MODEL: [What attacks this prevents]
fn security_checkpoint_example() {
    // 🛡️ INPUT VALIDATION: Prevent injection attacks
    // CRITICAL: All user input must be sanitized before processing
    // VALIDATION: Regex pattern matching + length limits
    // THREAT MODEL: Prevents SQL injection, XSS, command injection
}

// 📊 CONSTANTS WITH CALCULATION ARCHAEOLOGY
/// 🧮 CALCULATION: [Mathematical reasoning for this value]
/// Why: [Specific calculation and constraints]
/// Alternative: [Other values considered and why rejected]
/// Research: [External standards or research referenced]
pub const CALCULATED_VALUE: usize = 1000; // Based on 8GB VPS: 8GB - 2GB OS - 4GB app = 2GB ÷ 1KB per item = 2M theoretical, 1K conservative

// 🔄 PATTERN MONITORING COMMENTS
// 🔍 PATTERN WATCH: [Pattern being monitored for 3-strikes]
// Strike 1: [First occurrence location]
// Strike 2: [Second occurrence location]  
// Strike 3: [Pending] - watch for extraction opportunity

// 📋 COMPLEX LOGIC EXPLANATION PATTERN
fn complex_logic_example() {
    // 🎭 COMPLEX ALGORITHM: Step-by-step explanation
    // Step 1: [What this step does and why]
    let step1_result = first_operation();
    
    // Step 2: [What this step does and why] 
    // DECISION: [Why this specific approach for step 2]
    let step2_result = second_operation(step1_result);
    
    // Step 3: [Final step reasoning]
    // Why: [Reasoning for final operation]
    final_operation(step2_result)
}

// 🧪 TEST DECISION ARCHAEOLOGY
#[cfg(test)]
mod tests {
    /// 🧪 TEST STRATEGY: [What this test validates]
    /// COVERAGE: [What scenarios are covered]
    /// Why: [Why this specific test approach]
    /// Alternative: [Other testing approaches rejected]
    #[test]
    fn test_with_decision_archaeology() {
        // 🎯 TEST FOCUS: [Specific behavior being tested]
        // Why: [Why this test case is important]
        // Edge Case: [What edge cases are covered]
        
        // Test implementation
        assert!(true);
    }
}

// 📝 DOCUMENTATION COMMENTS VS DECISION COMMENTS

// Use /// for API documentation (what the code does):
/// Calculates the optimal task assignment for available agents.
/// 
/// # Arguments
/// * `tasks` - List of pending tasks to assign
/// * `agents` - Available agents with their current status
/// 
/// # Returns
/// Mapping of tasks to optimal agents
pub fn calculate_assignment(tasks: &[Task], agents: &[Agent]) -> HashMap<TaskId, AgentId> {
    // Use // for decision archaeology (why the code does it this way):
    // 🎭 ASSIGNMENT ALGORITHM: Hungarian algorithm for optimal matching
    // Why: Guarantees globally optimal assignment in O(n³) time
    // Alternative: Greedy assignment (rejected: suboptimal results)
    // Trade-off: Higher CPU cost for better task distribution
    unimplemented!()
}

// 🎯 COMMENT QUALITY CHECKLIST
// ✅ Does the comment explain WHY, not just WHAT?
// ✅ Are alternatives considered and rejection reasons given?
// ✅ Is the decision specific and actionable?
// ✅ Can someone audit/verify the decision is working?
// ✅ Are performance/security implications mentioned?
// ✅ Is the comment close to the relevant code?
// ✅ Does the emoji category match the comment type?

// 🚫 ANTI-PATTERNS TO AVOID
// ❌ Obvious comments: `i++; // increment i`
// ❌ Outdated comments that don't match code
// ❌ Comments that just repeat the function name
// ❌ Decision comments without reasoning
// ❌ Missing alternatives consideration
// ❌ Comments far from relevant code
// ❌ Inconsistent emoji usage
// ❌ Comments without audit/verification guidance