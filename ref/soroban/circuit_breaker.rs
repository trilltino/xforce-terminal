use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests pass through
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Time to wait before attempting to close circuit
    pub timeout: Duration,
    /// Number of successes needed in half-open state to close
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
        }
    }
}

#[derive(Debug)]
struct CircuitBreakerState {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
}

/// Circuit breaker for RPC failures - prevents cascading failures
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        info!("🔌 Initializing circuit breaker (failure threshold: {})", config.failure_threshold);

        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
            })),
        }
    }

    /// Check if request should be allowed
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        // Check if circuit is open
        {
            let state = self.state.read().await;
            match state.state {
                CircuitState::Open => {
                    // Check if timeout has elapsed
                    if let Some(last_failure) = state.last_failure_time {
                        if last_failure.elapsed() < self.config.timeout {
                            return Err(CircuitBreakerError::CircuitOpen);
                        }
                    }
                    // Timeout elapsed, transition to half-open
                    drop(state);
                    self.transition_to_half_open().await;
                }
                _ => {}
            }
        }

        // Execute the function
        match f.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(CircuitBreakerError::RequestFailed(error))
            }
        }
    }

    /// Record a successful call
    async fn on_success(&self) {
        let mut state = self.state.write().await;

        match state.state {
            CircuitState::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= self.config.success_threshold {
                    info!("✅ Circuit breaker closing (success threshold reached)");
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.last_failure_time = None;
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
            }
            CircuitState::Open => {
                // Should not happen, but reset if it does
                state.failure_count = 0;
            }
        }
    }

    /// Record a failed call
    async fn on_failure(&self) {
        let mut state = self.state.write().await;

        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());
        state.success_count = 0; // Reset success count

        if state.failure_count >= self.config.failure_threshold {
            if state.state != CircuitState::Open {
                warn!("⚠️  Circuit breaker opening (failure threshold reached: {})", state.failure_count);
                state.state = CircuitState::Open;
            }
        }
    }

    /// Transition to half-open state
    async fn transition_to_half_open(&self) {
        let mut state = self.state.write().await;
        info!("🔄 Circuit breaker transitioning to half-open state");
        state.state = CircuitState::HalfOpen;
        state.success_count = 0;
    }

    /// Get current state
    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.state
    }

    /// Get circuit breaker statistics
    pub async fn stats(&self) -> CircuitBreakerStats {
        let state = self.state.read().await;
        CircuitBreakerStats {
            state: state.state,
            failure_count: state.failure_count,
            success_count: state.success_count,
            is_open: state.state == CircuitState::Open,
        }
    }

    /// Manually reset the circuit breaker
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        info!("🔄 Manually resetting circuit breaker");
        state.state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
        state.last_failure_time = None;
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    CircuitOpen,
    RequestFailed(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::CircuitOpen => write!(f, "Circuit breaker is open"),
            CircuitBreakerError::RequestFailed(e) => write!(f, "Request failed: {}", e),
        }
    }
}

impl<E: std::error::Error> std::error::Error for CircuitBreakerError<E> {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub is_open: bool,
}

// Manual Serialize impl for CircuitState
impl serde::Serialize for CircuitState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let state_str = match self {
            CircuitState::Closed => "closed",
            CircuitState::Open => "open",
            CircuitState::HalfOpen => "half_open",
        };
        serializer.serialize_str(state_str)
    }
}
