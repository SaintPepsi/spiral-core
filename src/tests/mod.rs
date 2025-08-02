//! 🧪 COMPREHENSIVE TEST SUITES: System-wide testing infrastructure
//! CRITICAL: Ensures reliability across all components and scenarios
//! Why: Production systems require thorough testing of all paths
//! Alternative: Ad-hoc testing (rejected: insufficient coverage)

#[cfg(test)]
mod lifecycle_tests;

// #[cfg(test)]
// mod auto_core_update_lifecycle; // Needs refactoring to match current API

#[cfg(test)]
mod test_helpers;

#[cfg(test)]
mod precision_pressure_tests;
