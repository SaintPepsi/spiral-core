use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject all requests
    HalfOpen, // Testing if service recovered
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening circuit
    pub failure_threshold: u32,
    /// Duration to wait before attempting recovery
    pub timeout_duration: Duration,
    /// Number of successful requests needed to close circuit from half-open
    pub success_threshold: u32,
    /// Time window for counting failures
    pub failure_window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout_duration: Duration::from_secs(60),
            success_threshold: 3,
            failure_window: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Circuit breaker for Claude Code API protection
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    last_state_change: Arc<RwLock<Instant>>,
    total_requests: Arc<AtomicU64>,
    total_failures: Arc<AtomicU64>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            last_state_change: Arc::new(RwLock::new(Instant::now())),
            total_requests: Arc::new(AtomicU64::new(0)),
            total_failures: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Check if request should be allowed
    pub async fn should_allow_request(&self) -> bool {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        let current_state = *self.state.read().await;

        match current_state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has elapsed
                let last_change = *self.last_state_change.read().await;
                if last_change.elapsed() >= self.config.timeout_duration {
                    // Transition to half-open
                    self.transition_to_half_open().await;
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                true
            }
        }
    }

    /// Record successful request
    pub async fn record_success(&self) {
        let current_state = *self.state.read().await;

        match current_state {
            CircuitState::HalfOpen => {
                let count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                debug!(
                    "Circuit breaker success count: {}/{}",
                    count, self.config.success_threshold
                );

                if count >= self.config.success_threshold {
                    self.transition_to_closed().await;
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success in closed state
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::Open => {
                // Shouldn't happen, but log it
                warn!("Success recorded while circuit is open");
            }
        }
    }

    /// Record failed request
    pub async fn record_failure(&self) {
        self.total_failures.fetch_add(1, Ordering::Relaxed);

        let current_state = *self.state.read().await;

        match current_state {
            CircuitState::Closed => {
                // Check if failures are within time window
                let mut last_failure = self.last_failure_time.write().await;
                let now = Instant::now();

                if let Some(last_time) = *last_failure {
                    if now.duration_since(last_time) > self.config.failure_window {
                        // Reset counter if outside window
                        self.failure_count.store(1, Ordering::Relaxed);
                    } else {
                        let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                        debug!(
                            "Circuit breaker failure count: {}/{}",
                            count, self.config.failure_threshold
                        );

                        if count >= self.config.failure_threshold {
                            self.transition_to_open().await;
                        }
                    }
                } else {
                    self.failure_count.store(1, Ordering::Relaxed);
                }

                *last_failure = Some(now);
            }
            CircuitState::HalfOpen => {
                // Single failure in half-open immediately opens circuit
                self.transition_to_open().await;
            }
            CircuitState::Open => {
                // Already open, just update last failure time
                let mut last_failure = self.last_failure_time.write().await;
                *last_failure = Some(Instant::now());
            }
        }
    }

    /// Transition to open state
    async fn transition_to_open(&self) {
        let mut state = self.state.write().await;
        let previous_state = *state;
        *state = CircuitState::Open;

        let mut last_change = self.last_state_change.write().await;
        *last_change = Instant::now();

        self.success_count.store(0, Ordering::Relaxed);

        warn!(
            "Circuit breaker opened (was {:?}). Total requests: {}, Total failures: {}",
            previous_state,
            self.total_requests.load(Ordering::Relaxed),
            self.total_failures.load(Ordering::Relaxed)
        );
    }

    /// Transition to half-open state
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        *state = CircuitState::HalfOpen;

        let mut last_change = self.last_state_change.write().await;
        *last_change = Instant::now();

        self.success_count.store(0, Ordering::Relaxed);
        self.failure_count.store(0, Ordering::Relaxed);

        info!("Circuit breaker transitioned to half-open");
    }

    /// Transition to closed state
    async fn transition_to_closed(&self) {
        let mut state = self.state.write().await;
        let previous_state = *state;
        *state = CircuitState::Closed;

        let mut last_change = self.last_state_change.write().await;
        *last_change = Instant::now();

        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);

        info!(
            "Circuit breaker closed (was {:?}). Service recovered.",
            previous_state
        );
    }

    /// Get current circuit state
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Get circuit breaker metrics
    pub async fn get_metrics(&self) -> CircuitBreakerMetrics {
        let last_change = *self.last_state_change.read().await;
        CircuitBreakerMetrics {
            state: *self.state.read().await,
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_failures: self.total_failures.load(Ordering::Relaxed),
            last_state_change_seconds: last_change.elapsed().as_secs(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitBreakerMetrics {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub total_requests: u64,
    pub total_failures: u64,
    pub last_state_change_seconds: u64,
}
