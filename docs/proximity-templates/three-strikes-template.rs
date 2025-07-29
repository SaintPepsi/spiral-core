//! 🔧 THREE-STRIKES ABSTRACTION TEMPLATE
//! Based on spiral-core 3-strikes extractions: language_detection.rs & task_utils.rs
//! Pattern: Track duplication, extract on 3rd occurrence, maintain single responsibility

// 🔧 UTILITY MODULES: Extracted via 3-strikes abstraction rule
// Strike 1: [Location of first duplication]
// Strike 2: [Location of second duplication]  
// Strike 3: [Location of third duplication] → EXTRACTED to [module_name]
pub mod extracted_utility;

// Example from spiral-core:
// 🔧 UTILITY MODULES: Extracted via 3-strikes abstraction rule
pub mod language_detection; // Strike 1: developer.rs, Strike 2: orchestrator.rs, Strike 3: api.rs → EXTRACTED
pub mod task_utils;         // Strike 1: developer.rs, Strike 2: orchestrator.rs, Strike 3: api.rs → EXTRACTED

/// 📋 3-STRIKES MONITORING TEMPLATE
/// Track patterns approaching extraction threshold
/// DECISION: Monitor duplication, prepare for extraction at 3rd occurrence
/// Why: Prevents premature abstraction while catching genuine reuse patterns
/// Alternative: Extract immediately (rejected: premature optimization), never extract (rejected: code duplication)

// 🔍 PATTERNS TO MONITOR (Examples from spiral-core)

// 📊 ERROR HANDLING PATTERNS (2 strikes - watching for 3rd)
// Strike 1: src/agents/developer.rs - error result creation
// Strike 2: src/api/mod.rs - error response formatting  
// Strike 3: [PENDING] - if appears again, extract to error_utils module
// Pattern: Consistent error formatting and result creation

// 📝 MESSAGE FORMATTING PATTERNS (2 strikes)  
// Strike 1: src/discord/spiral_constellation_bot.rs - Discord message formatting
// Strike 2: src/api/mod.rs - API response formatting
// Strike 3: [PENDING] - watch for 3rd occurrence to extract message_utils module
// Pattern: Structured message creation with truncation and formatting

/// 🎯 EXTRACTION CRITERIA CHECKLIST
/// Use this checklist before extracting to new module:
///
/// ✅ Pattern appears 3+ times across different modules
/// ✅ Pattern has consistent interface/behavior
/// ✅ Pattern has single, clear responsibility  
/// ✅ Extraction reduces coupling (doesn't increase dependencies)
/// ✅ Pattern is stable (not changing frequently)
/// ✅ Module can be comprehensively tested in isolation

//! 🔧 EXTRACTED UTILITY MODULE TEMPLATE
//! Single responsibility, well-tested, clear documentation

/// 🎯 [UTILITY PURPOSE]: [Brief description of single responsibility]
/// EXTRACTION RATIONALE: [Why this was extracted, citing 3 occurrences]
/// RESPONSIBILITY: [Exactly what this module handles]
/// NOT RESPONSIBLE FOR: [What this module explicitly does NOT handle]
pub struct ExtractedUtility {
    // Keep state minimal and focused
}

impl ExtractedUtility {
    /// 🏗️ CONSTRUCTOR: [Purpose of constructor]
    /// DECISION: [Why this constructor pattern]
    /// Why: [Reasoning for initialization approach]
    pub fn new(/* minimal parameters */) -> Self {
        // Initialization logic with decision comments
        Self {
            // Minimal state
        }
    }

    /// 🔧 CORE FUNCTIONALITY: [What this method does]
    /// DESIGN: [Why this method signature/approach]
    /// Why: [Reasoning for this specific implementation]
    /// Alternative: [What was rejected and why]
    pub fn core_method(&self, input: InputType) -> Result<OutputType, Error> {
        // Implementation with decision archaeology
        // 🧠 DECISION: [Implementation choice]
        // Why: [Reasoning]
        unimplemented!("Implementation here")
    }
}

// 🧪 COMPREHENSIVE TEST COVERAGE
// DECISION: Test extracted utilities thoroughly since they're reused
// Why: Bugs in utilities affect multiple call sites
// Alternative: Light testing (rejected: multiplies debugging effort)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_functionality() {
        // Test the primary use case that justified extraction
        // Based on the 3 original occurrences
    }

    #[test]  
    fn test_edge_cases() {
        // Test boundary conditions
        // Include cases from all 3 original call sites
    }

    #[test]
    fn test_error_conditions() {
        // Test failure modes
        // Ensure consistent error handling across all use cases
    }
}

// 📊 EXTRACTION SUCCESS METRICS
// Track these metrics to validate extraction decision:
// 1. Lines of code reduction across original call sites
// 2. Bug reduction (centralized fixes)
// 3. Consistency improvement (single implementation)
// 4. Test coverage improvement (focused testing)
// 5. Maintenance effort reduction

// 🔄 POST-EXTRACTION MONITORING
// Watch for:
// 1. New call sites adopting the utility (validation of need)
// 2. Pressure to add unrelated functionality (scope creep)
// 3. Multiple similar utilities emerging (need for higher-level abstraction)
// 4. Utility becoming overly complex (may need splitting)

// Pattern Notes for Successful 3-Strikes Extraction:
// 1. Wait for genuine 3rd occurrence (not forced)
// 2. Extract minimal, focused responsibility
// 3. Maintain clear documentation of extraction reasoning
// 4. Comprehensive testing of extracted utility
// 5. Monitor post-extraction for scope creep
// 6. Keep extraction threshold discipline for future patterns
// 7. Document what the utility does NOT handle (scope boundaries)