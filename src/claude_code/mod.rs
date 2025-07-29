pub mod circuit_breaker;
mod cli_client;
mod command_builder;

pub use cli_client::{
    ClaudeCodeCliClient as ClaudeCodeClient, CodeGenerationRequest, CodeGenerationResult,
    FileCreation, FileModification, TaskAnalysis,
};
pub use command_builder::{ClaudeCommandBuilder, OutputFormat, PermissionMode, SessionMode};

// 🧪 TEST MODULE: Comprehensive testing for external AI integration
#[cfg(test)]
mod tests;

// 🔧 PUBLIC TEST UTILITIES: For integration testing from other modules
// #[cfg(test)]
// pub use tests::security::*;  // Disabled until security module has exports
