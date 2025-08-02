//! # Spiral Core
//!
//! Spiral Core is an AI agent orchestration system that creates specialized AI agents
//! which collaborate through Claude Code integration to build tools and manage complex workflows.
//!
//! ## Architecture
//!
//! The system consists of:
//! - **Agent Orchestrator**: Manages and coordinates multiple specialized agents
//! - **Claude Code Integration**: Primary AI engine for code generation and analysis
//! - **Discord Bot Service**: Human interaction interface
//! - **GitHub Integration**: Automated repository management
//! - **Session Management**: Secure session handling for agents and users
//!
//! ## Key Features
//!
//! - Multi-agent orchestration with specialized roles
//! - Safe self-update capabilities with rollback
//! - Comprehensive security validation
//! - Resource-efficient design for deployment on modest hardware
//!
//! ## Usage
//!
//! The system is typically run as a standalone service that integrates with Discord
//! for human interaction and Claude Code for AI capabilities.

/// Agent orchestration and management
pub mod agents;
/// HTTP API server and endpoints
pub mod api;
/// Authentication and authorization
pub mod auth;
/// Claude Code client integration
pub mod claude_code;
/// System configuration
pub mod config;
/// System-wide constants
pub mod constants;
/// Discord bot integration
pub mod discord;
/// Error types and handling
pub mod error;
/// Core data models
pub mod models;
/// System monitoring and metrics
pub mod monitoring;
/// Rate limiting functionality
pub mod rate_limit;
/// Security utilities and API key management
pub mod security;
/// Session management for agents and users
pub mod session;
/// Input validation and sanitization
pub mod validation;

#[cfg(test)]
mod tests;

pub use error::{Result, SpiralError};
