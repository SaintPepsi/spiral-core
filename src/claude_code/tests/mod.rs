/// 🧪 CLAUDE CODE TESTS: External AI integration testing
/// AUDIT CHECKPOINT: Critical external dependency validation
/// Focus: Security, reliability, cost control
mod unit;

// 🔄 INTEGRATION TESTS: Future expansion for real API testing
// mod integration;  // Enable when mock server is implemented

/// 🛡️ SECURITY TEST SUITE: Comprehensive security validation
/// CRITICAL: AI integrations have unique security considerations
pub mod security;

/// 🧪 MOCK IMPLEMENTATIONS: Test doubles for controlled testing
/// Why: Enable deterministic testing without external dependencies
pub mod mock;
