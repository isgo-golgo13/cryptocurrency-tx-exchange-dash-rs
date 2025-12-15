//! # dash-websocket
//!
//! WebSocket client with automatic reconnection and message handling.
//! Uses Strategy pattern for reconnection backoff policies.

pub mod client;

pub use client::*;

/// Default WebSocket server URL
pub const DEFAULT_WS_URL: &str = "ws://127.0.0.1:3001/ws";

// ============================================================================
// STRATEGY PATTERN: Reconnection Policy
// ============================================================================

/// Strategy trait for reconnection backoff
pub trait ReconnectPolicy: Send + Sync + Clone {
    /// Calculate delay in milliseconds for given attempt number (0-indexed)
    fn delay_ms(&self, attempt: u32) -> u32;
    
    /// Check if should attempt reconnection
    fn should_reconnect(&self, attempt: u32) -> bool;
    
    /// Reset the policy (called on successful connection)
    fn reset(&mut self);
}

/// Exponential backoff reconnection policy
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    /// Initial delay before first reconnection (ms)
    pub initial_delay_ms: u32,
    /// Maximum delay between attempts (ms)
    pub max_delay_ms: u32,
    /// Multiplier for each subsequent attempt
    pub multiplier: f64,
    /// Maximum number of attempts (0 = unlimited)
    pub max_attempts: u32,
    /// Add random jitter to delay
    pub jitter: bool,
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self {
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            multiplier: 1.5,
            max_attempts: 0, // Unlimited
            jitter: true,
        }
    }
}

impl ExponentialBackoff {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn initial_delay(mut self, ms: u32) -> Self {
        self.initial_delay_ms = ms;
        self
    }

    pub fn max_delay(mut self, ms: u32) -> Self {
        self.max_delay_ms = ms;
        self
    }

    pub fn multiplier(mut self, m: f64) -> Self {
        self.multiplier = m;
        self
    }

    pub fn max_attempts(mut self, n: u32) -> Self {
        self.max_attempts = n;
        self
    }

    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Aggressive reconnection (for trading dashboards)
    pub fn aggressive() -> Self {
        Self {
            initial_delay_ms: 500,
            max_delay_ms: 5000,
            multiplier: 1.2,
            max_attempts: 0,
            jitter: true,
        }
    }

    /// Conservative reconnection (for background services)
    pub fn conservative() -> Self {
        Self {
            initial_delay_ms: 2000,
            max_delay_ms: 60000,
            multiplier: 2.0,
            max_attempts: 10,
            jitter: true,
        }
    }
}

impl ReconnectPolicy for ExponentialBackoff {
    fn delay_ms(&self, attempt: u32) -> u32 {
        let base_delay = self.initial_delay_ms as f64 * self.multiplier.powi(attempt as i32);
        let mut delay = (base_delay as u32).min(self.max_delay_ms);

        // Add jitter (±20%)
        if self.jitter {
            let jitter_range = delay / 5;
            // Simple pseudo-random based on attempt number
            let jitter = ((attempt * 7919) % (jitter_range * 2 + 1)) as i32 - jitter_range as i32;
            delay = (delay as i32 + jitter).max(100) as u32;
        }

        delay
    }

    fn should_reconnect(&self, attempt: u32) -> bool {
        self.max_attempts == 0 || attempt < self.max_attempts
    }

    fn reset(&mut self) {
        // ExponentialBackoff is stateless, nothing to reset
    }
}

/// Linear backoff reconnection policy
#[derive(Debug, Clone)]
pub struct LinearBackoff {
    pub initial_delay_ms: u32,
    pub increment_ms: u32,
    pub max_delay_ms: u32,
    pub max_attempts: u32,
}

impl Default for LinearBackoff {
    fn default() -> Self {
        Self {
            initial_delay_ms: 1000,
            increment_ms: 1000,
            max_delay_ms: 10000,
            max_attempts: 10,
        }
    }
}

impl ReconnectPolicy for LinearBackoff {
    fn delay_ms(&self, attempt: u32) -> u32 {
        (self.initial_delay_ms + self.increment_ms * attempt).min(self.max_delay_ms)
    }

    fn should_reconnect(&self, attempt: u32) -> bool {
        self.max_attempts == 0 || attempt < self.max_attempts
    }

    fn reset(&mut self) {}
}

/// Constant delay reconnection policy (simple retry)
#[derive(Debug, Clone)]
pub struct ConstantDelay {
    pub delay_ms: u32,
    pub max_attempts: u32,
}

impl Default for ConstantDelay {
    fn default() -> Self {
        Self {
            delay_ms: 3000,
            max_attempts: 5,
        }
    }
}

impl ReconnectPolicy for ConstantDelay {
    fn delay_ms(&self, _attempt: u32) -> u32 {
        self.delay_ms
    }

    fn should_reconnect(&self, attempt: u32) -> bool {
        self.max_attempts == 0 || attempt < self.max_attempts
    }

    fn reset(&mut self) {}
}

// ============================================================================
// WEBSOCKET CONFIGURATION
// ============================================================================

/// WebSocket client configuration
#[derive(Debug, Clone)]
pub struct WsConfig {
    pub url: String,
    pub reconnect_policy: ExponentialBackoff,
    /// Heartbeat interval in milliseconds (0 = disabled)
    pub heartbeat_interval_ms: u32,
    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u32,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            url: DEFAULT_WS_URL.to_string(),
            reconnect_policy: ExponentialBackoff::default(),
            heartbeat_interval_ms: 30000,
            connect_timeout_ms: 10000,
        }
    }
}

impl WsConfig {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    pub fn with_policy(mut self, policy: ExponentialBackoff) -> Self {
        self.reconnect_policy = policy;
        self
    }

    pub fn heartbeat(mut self, interval_ms: u32) -> Self {
        self.heartbeat_interval_ms = interval_ms;
        self
    }

    pub fn timeout(mut self, timeout_ms: u32) -> Self {
        self.connect_timeout_ms = timeout_ms;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let policy = ExponentialBackoff {
            initial_delay_ms: 1000,
            max_delay_ms: 10000,
            multiplier: 2.0,
            max_attempts: 5,
            jitter: false,
        };

        assert_eq!(policy.delay_ms(0), 1000);
        assert_eq!(policy.delay_ms(1), 2000);
        assert_eq!(policy.delay_ms(2), 4000);
        assert_eq!(policy.delay_ms(3), 8000);
        assert_eq!(policy.delay_ms(4), 10000); // Capped at max
    }

    #[test]
    fn test_should_reconnect() {
        let policy = ExponentialBackoff {
            max_attempts: 3,
            ..Default::default()
        };

        assert!(policy.should_reconnect(0));
        assert!(policy.should_reconnect(2));
        assert!(!policy.should_reconnect(3));
    }

    #[test]
    fn test_linear_backoff() {
        let policy = LinearBackoff {
            initial_delay_ms: 1000,
            increment_ms: 500,
            max_delay_ms: 5000,
            max_attempts: 10,
        };

        assert_eq!(policy.delay_ms(0), 1000);
        assert_eq!(policy.delay_ms(1), 1500);
        assert_eq!(policy.delay_ms(2), 2000);
        assert_eq!(policy.delay_ms(10), 5000); // Capped
    }
}
