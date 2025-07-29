//! 🧪 DISCORD BOT TESTS
//! DECISION: Comprehensive test coverage for 1393-line Discord integration
//! Why: Large component without tests represents critical risk
//! Alternative: Minimal testing (rejected: too much business logic to leave uncovered)
//! Audit: Verify all critical bot functions work correctly

mod intent_classification_tests;
mod message_security_tests;
mod persona_tests;
mod security_integration_tests;
mod security_tests;

// Test modules are internal - no need to re-export them
