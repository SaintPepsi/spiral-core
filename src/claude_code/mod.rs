mod cli_client;

pub use cli_client::{
    ClaudeCodeCliClient as ClaudeCodeClient, CodeGenerationRequest, CodeGenerationResult,
    FileCreation, FileModification, TaskAnalysis,
};

// 🧪 TEST MODULE: Comprehensive testing for external AI integration
#[cfg(test)]
mod tests;

// 🔧 PUBLIC TEST UTILITIES: For integration testing from other modules
// #[cfg(test)]
// pub use tests::security::*;  // Disabled until security module has exports
