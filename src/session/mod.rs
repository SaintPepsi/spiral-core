//! Session Management Module for Spiral Core
//!
//! This module provides session management capabilities for the Spiral Core orchestration system.
//! It follows SOLID principles, DRY principle, and SID naming conventions.
//!
//! # Architecture
//!
//! The session module is designed with:
//! - Single Responsibility: Each component has one clear purpose
//! - Open/Closed: Extensible through traits, closed for modification
//! - Liskov Substitution: Implementations are interchangeable
//! - Interface Segregation: Small, focused interfaces
//! - Dependency Inversion: Depends on abstractions, not concretions
//!
//! # Example
//!
//! ```no_run
//! use spiral_core::session::{SessionManager, SessionConfig, InMemorySessionStore};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a session manager with default config
//! let store = InMemorySessionStore::new();
//! let config = SessionConfig::default();
//! let manager = SessionManager::new(store, config);
//!
//! // Create a new session
//! let session = manager.create_session("user123".to_string()).await?;
//! println!("Created session: {}", session.id);
//!
//! // Validate and extend session
//! let validated = manager.validate_session(&session.id).await?;
//! assert_eq!(validated.state, spiral_core::session::SessionState::Active);
//! # Ok(())
//! # }
//! ```
//!
//! # Submodules
//!
//! - `agent_sessions` - Agent-specific session management extensions

pub mod agent_sessions;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::{Result, SpiralError};

/// Session configuration with sensible defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum session duration before automatic expiration
    pub max_duration: Duration,
    /// Maximum number of concurrent sessions per user
    pub max_concurrent: usize,
    /// Enable automatic session extension on activity
    pub auto_extend: bool,
    /// Duration to extend session on activity
    pub extend_duration: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_duration: Duration::hours(24),
            max_concurrent: 5,
            auto_extend: true,
            extend_duration: Duration::hours(1),
        }
    }
}

/// Represents a unique session in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: Uuid,
    /// User identifier associated with this session
    pub user_id: String,
    /// Session creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Session expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Session metadata for extensibility
    pub metadata: HashMap<String, String>,
    pub state: SessionState,
}

/// Session lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session is active and valid
    Active,
    /// Session has been suspended
    Suspended,
    /// Session has expired
    Expired,
    /// Session has been terminated
    Terminated,
}

/// Session storage trait for dependency inversion
#[async_trait::async_trait]
pub trait SessionStore: Send + Sync {
    /// Create a new session
    async fn create(&self, session: Session) -> Result<()>;

    /// Retrieve a session by ID
    async fn get(&self, id: &Uuid) -> Result<Option<Session>>;

    /// Update an existing session
    async fn update(&self, session: Session) -> Result<()>;

    /// Delete a session
    async fn delete(&self, id: &Uuid) -> Result<()>;

    /// List all sessions for a user
    async fn list_by_user(&self, user_id: &str) -> Result<Vec<Session>>;

    /// Clean up expired sessions
    async fn cleanup_expired(&self) -> Result<usize>;
}

/// In-memory session store implementation
pub struct InMemorySessionStore {
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemorySessionStore {
    /// Create a new in-memory session store
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl SessionStore for InMemorySessionStore {
    async fn create(&self, session: Session) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        // Validate session doesn't already exist
        if sessions.contains_key(&session.id) {
            return Err(SpiralError::Validation(
                "Session already exists".to_string(),
            ));
        }

        sessions.insert(session.id, session);
        Ok(())
    }

    async fn get(&self, id: &Uuid) -> Result<Option<Session>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(id).cloned())
    }

    async fn update(&self, session: Session) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        // Validate session exists
        if !sessions.contains_key(&session.id) {
            return Err(SpiralError::NotFound("Session not found".to_string()));
        }

        sessions.insert(session.id, session);
        Ok(())
    }

    async fn delete(&self, id: &Uuid) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        if sessions.remove(id).is_none() {
            return Err(SpiralError::NotFound("Session not found".to_string()));
        }

        Ok(())
    }

    async fn list_by_user(&self, user_id: &str) -> Result<Vec<Session>> {
        let sessions = self.sessions.read().await;

        let user_sessions: Vec<Session> = sessions
            .values()
            .filter(|s| s.user_id == user_id)
            .cloned()
            .collect();

        Ok(user_sessions)
    }

    async fn cleanup_expired(&self) -> Result<usize> {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();

        let expired_ids: Vec<Uuid> = sessions
            .iter()
            .filter(|(_, s)| s.expires_at < now || s.state == SessionState::Expired)
            .map(|(id, _)| *id)
            .collect();

        let count = expired_ids.len();

        for id in expired_ids {
            sessions.remove(&id);
        }

        Ok(count)
    }
}

/// Session manager for handling session lifecycle
pub struct SessionManager<S: SessionStore> {
    store: S,
    config: SessionConfig,
}

impl<S: SessionStore> SessionManager<S> {
    /// Create a new session manager
    pub fn new(store: S, config: SessionConfig) -> Self {
        Self { store, config }
    }

    /// Create a new session for a user
    pub async fn create_session(&self, user_id: String) -> Result<Session> {
        // Check concurrent session limit
        let existing = self.store.list_by_user(&user_id).await?;
        let active_count = existing
            .iter()
            .filter(|s| s.state == SessionState::Active)
            .count();

        if active_count >= self.config.max_concurrent {
            return Err(SpiralError::Validation(format!(
                "Maximum concurrent sessions ({}) reached",
                self.config.max_concurrent
            )));
        }

        let now = Utc::now();
        let session = Session {
            id: Uuid::new_v4(),
            user_id,
            created_at: now,
            last_activity: now,
            expires_at: now + self.config.max_duration,
            metadata: HashMap::new(),
            state: SessionState::Active,
        };

        self.store.create(session.clone()).await?;
        Ok(session)
    }

    /// Validate and optionally extend a session
    pub async fn validate_session(&self, id: &Uuid) -> Result<Session> {
        let session = self
            .store
            .get(id)
            .await?
            .ok_or_else(|| SpiralError::NotFound("Session not found".to_string()))?;

        // Check if session is expired
        if session.expires_at < Utc::now() {
            let mut expired = session.clone();
            expired.state = SessionState::Expired;
            self.store.update(expired).await?;
            return Err(SpiralError::Validation("Session expired".to_string()));
        }

        // Check if session is active
        if session.state != SessionState::Active {
            return Err(SpiralError::Validation(format!(
                "Session is not active: {:?}",
                session.state
            )));
        }

        // Auto-extend if enabled
        if self.config.auto_extend {
            let mut updated = session.clone();
            let now = Utc::now();
            updated.last_activity = now;
            // Extend from the current expiry, not from now
            // This ensures the session always gets longer, never shorter
            let new_expiry = updated.expires_at + self.config.extend_duration;
            // But cap it at max_duration from now
            let max_expiry = now + self.config.max_duration;
            updated.expires_at = new_expiry.min(max_expiry);
            self.store.update(updated.clone()).await?;
            return Ok(updated);
        }

        Ok(session)
    }

    /// Terminate a session
    pub async fn terminate_session(&self, id: &Uuid) -> Result<()> {
        let mut session = self
            .store
            .get(id)
            .await?
            .ok_or_else(|| SpiralError::NotFound("Session not found".to_string()))?;

        session.state = SessionState::Terminated;
        self.store.update(session).await
    }

    /// Suspend a session
    pub async fn suspend_session(&self, id: &Uuid) -> Result<()> {
        let mut session = self
            .store
            .get(id)
            .await?
            .ok_or_else(|| SpiralError::NotFound("Session not found".to_string()))?;

        if session.state != SessionState::Active {
            return Err(SpiralError::Validation(
                "Only active sessions can be suspended".to_string(),
            ));
        }

        session.state = SessionState::Suspended;
        self.store.update(session).await
    }

    /// Resume a suspended session
    pub async fn resume_session(&self, id: &Uuid) -> Result<Session> {
        let mut session = self
            .store
            .get(id)
            .await?
            .ok_or_else(|| SpiralError::NotFound("Session not found".to_string()))?;

        if session.state != SessionState::Suspended {
            return Err(SpiralError::Validation(
                "Only suspended sessions can be resumed".to_string(),
            ));
        }

        // Check if still valid
        if session.expires_at < Utc::now() {
            session.state = SessionState::Expired;
            self.store.update(session).await?;
            return Err(SpiralError::Validation("Session expired".to_string()));
        }

        session.state = SessionState::Active;
        session.last_activity = Utc::now();
        self.store.update(session.clone()).await?;
        Ok(session)
    }

    /// Clean up expired sessions
    pub async fn cleanup(&self) -> Result<usize> {
        self.store.cleanup_expired().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper to create a test session manager
    async fn create_test_manager() -> SessionManager<InMemorySessionStore> {
        let store = InMemorySessionStore::new();
        let config = SessionConfig {
            max_duration: Duration::hours(1),
            max_concurrent: 2,
            auto_extend: true,
            extend_duration: Duration::minutes(30),
        };
        SessionManager::new(store, config)
    }

    #[tokio::test]
    async fn test_create_session() {
        let manager = create_test_manager().await;
        let session = manager.create_session("user1".to_string()).await.unwrap();

        assert_eq!(session.user_id, "user1");
        assert_eq!(session.state, SessionState::Active);
        assert!(session.expires_at > session.created_at);
    }

    #[tokio::test]
    async fn test_concurrent_session_limit() {
        let manager = create_test_manager().await;

        // Create max concurrent sessions
        manager.create_session("user1".to_string()).await.unwrap();
        manager.create_session("user1".to_string()).await.unwrap();

        // Third should fail
        let result = manager.create_session("user1".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SpiralError::Validation(_)));
    }

    #[tokio::test]
    async fn test_validate_and_extend_session() {
        let manager = create_test_manager().await;
        let session = manager.create_session("user1".to_string()).await.unwrap();
        let original_expiry = session.expires_at;
        let original_activity = session.last_activity;

        // Sleep to ensure time has passed
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Validate should extend the session
        let validated = manager.validate_session(&session.id).await.unwrap();

        // The new expiry should be later than the original
        assert!(
            validated.expires_at >= original_expiry,
            "Expected expires_at {} to be >= original {}",
            validated.expires_at,
            original_expiry
        );
        assert!(
            validated.last_activity > original_activity,
            "Expected last_activity {} to be > original {}",
            validated.last_activity,
            original_activity
        );
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        let manager = create_test_manager().await;
        let session = manager.create_session("user1".to_string()).await.unwrap();

        // Suspend
        manager.suspend_session(&session.id).await.unwrap();
        let suspended = manager.store.get(&session.id).await.unwrap().unwrap();
        assert_eq!(suspended.state, SessionState::Suspended);

        // Validate should fail for suspended
        let validate_result = manager.validate_session(&session.id).await;
        assert!(validate_result.is_err());

        // Resume
        let resumed = manager.resume_session(&session.id).await.unwrap();
        assert_eq!(resumed.state, SessionState::Active);

        // Terminate
        manager.terminate_session(&session.id).await.unwrap();
        let terminated = manager.store.get(&session.id).await.unwrap().unwrap();
        assert_eq!(terminated.state, SessionState::Terminated);
    }

    #[tokio::test]
    async fn test_cleanup_expired_sessions() {
        let store = InMemorySessionStore::new();
        let config = SessionConfig {
            max_duration: Duration::milliseconds(100), // Very short for testing
            max_concurrent: 10,
            auto_extend: false,
            extend_duration: Duration::minutes(30),
        };
        let manager = SessionManager::new(store, config);

        // Create sessions
        let session1 = manager.create_session("user1".to_string()).await.unwrap();
        manager.create_session("user2".to_string()).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Mark one as expired manually
        let mut expired = manager.store.get(&session1.id).await.unwrap().unwrap();
        expired.state = SessionState::Expired;
        manager.store.update(expired).await.unwrap();

        // Cleanup
        let cleaned = manager.cleanup().await.unwrap();
        assert_eq!(cleaned, 2);

        // Verify cleaned up
        let remaining = manager.store.list_by_user("user1").await.unwrap();
        assert_eq!(remaining.len(), 0);
    }

    #[tokio::test]
    async fn test_session_not_found() {
        let manager = create_test_manager().await;
        let fake_id = Uuid::new_v4();

        let result = manager.validate_session(&fake_id).await;
        assert!(matches!(result.unwrap_err(), SpiralError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_session_metadata() {
        let manager = create_test_manager().await;
        let mut session = manager.create_session("user1".to_string()).await.unwrap();

        // Add metadata
        session
            .metadata
            .insert("agent".to_string(), "developer".to_string());
        session
            .metadata
            .insert("workspace".to_string(), "project-x".to_string());

        manager.store.update(session.clone()).await.unwrap();

        // Retrieve and verify
        let retrieved = manager.store.get(&session.id).await.unwrap().unwrap();
        assert_eq!(
            retrieved.metadata.get("agent"),
            Some(&"developer".to_string())
        );
        assert_eq!(
            retrieved.metadata.get("workspace"),
            Some(&"project-x".to_string())
        );
    }
}
