//! System-wide lock mechanism to prevent concurrent self-updates
//!
//! This module provides a global lock that ensures only one self-update
//! can execute at a time, preventing file conflicts and corruption.

use crate::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Global system lock for self-updates
pub struct SystemLock {
    /// The actual mutex that provides exclusive access
    lock: Arc<Mutex<LockState>>,
}

/// State tracked by the system lock
#[derive(Debug, Clone)]
struct LockState {
    /// Whether an update is currently in progress
    is_locked: bool,
    /// ID of the current update holding the lock
    current_update_id: Option<String>,
    /// When the lock was acquired
    locked_at: Option<std::time::Instant>,
}

/// Token that proves the lock is held
/// The SystemLock must be used to release it
pub struct LockToken {
    pub update_id: String,
}

impl SystemLock {
    /// Create a new system lock
    pub fn new() -> Self {
        Self {
            lock: Arc::new(Mutex::new(LockState {
                is_locked: false,
                current_update_id: None,
                locked_at: None,
            })),
        }
    }

    /// Try to acquire the system lock for an update
    pub async fn try_acquire(&self, update_id: String) -> Result<Option<LockToken>> {
        let mut state = self.lock.lock().await;
        
        if state.is_locked {
            if let Some(ref current_id) = state.current_update_id {
                warn!(
                    "[SystemLock] Cannot acquire lock for {} - already held by {}",
                    update_id, current_id
                );
                
                // Check if lock has been held too long (stale lock detection)
                if let Some(locked_at) = state.locked_at {
                    let duration = locked_at.elapsed();
                    if duration > std::time::Duration::from_secs(1800) { // 30 minutes
                        warn!(
                            "[SystemLock] Lock held by {} for {} seconds - may be stale",
                            current_id,
                            duration.as_secs()
                        );
                    }
                }
            }
            return Ok(None);
        }
        
        // Acquire the lock
        state.is_locked = true;
        state.current_update_id = Some(update_id.clone());
        state.locked_at = Some(std::time::Instant::now());
        
        info!("[SystemLock] Lock acquired by update {}", update_id);
        
        Ok(Some(LockToken { update_id }))
    }
    
    /// Release a lock using its token
    pub async fn release(&self, token: LockToken) {
        let mut state = self.lock.lock().await;
        
        // Only release if this token still holds the lock
        if state.current_update_id.as_ref() == Some(&token.update_id) {
            info!("[SystemLock] Lock released by update {}", token.update_id);
            state.is_locked = false;
            state.current_update_id = None;
            state.locked_at = None;
        } else {
            warn!(
                "[SystemLock] Attempted to release lock with token {} but lock is held by {:?}",
                token.update_id, state.current_update_id
            );
        }
    }
    
    /// Force release the lock (emergency use only)
    pub async fn force_release(&self) {
        let mut state = self.lock.lock().await;
        
        if let Some(ref id) = state.current_update_id {
            warn!("[SystemLock] Force releasing lock held by {}", id);
        }
        
        state.is_locked = false;
        state.current_update_id = None;
        state.locked_at = None;
    }
    
    /// Check if the system is currently locked
    pub async fn is_locked(&self) -> bool {
        let state = self.lock.lock().await;
        state.is_locked
    }
    
    /// Get information about the current lock holder
    pub async fn current_holder(&self) -> Option<(String, std::time::Duration)> {
        let state = self.lock.lock().await;
        
        if state.is_locked {
            if let (Some(id), Some(locked_at)) = (&state.current_update_id, &state.locked_at) {
                return Some((id.clone(), locked_at.elapsed()));
            }
        }
        
        None
    }
}

impl Default for SystemLock {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_lock_acquisition() {
        let lock = SystemLock::new();
        
        // First acquisition should succeed
        let token1 = lock.try_acquire("update-1".to_string()).await.unwrap();
        assert!(token1.is_some());
        
        // Second acquisition should fail
        let token2 = lock.try_acquire("update-2".to_string()).await.unwrap();
        assert!(token2.is_none());
        
        // Release the first lock
        if let Some(t) = token1 {
            lock.release(t).await;
        }
        
        // Now acquisition should succeed
        let token3 = lock.try_acquire("update-3".to_string()).await.unwrap();
        assert!(token3.is_some());
    }
    
    #[tokio::test]
    async fn test_lock_info() {
        let lock = SystemLock::new();
        
        // No lock holder initially
        assert!(lock.current_holder().await.is_none());
        assert!(!lock.is_locked().await);
        
        // Acquire lock
        let _guard = lock.try_acquire("test-update".to_string()).await.unwrap();
        
        // Check lock info
        assert!(lock.is_locked().await);
        let holder = lock.current_holder().await;
        assert!(holder.is_some());
        
        if let Some((id, _duration)) = holder {
            assert_eq!(id, "test-update");
        }
    }
}