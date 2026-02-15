//! ext-idle-notify + idle-inhibit: idle detection and prevention.
//!
//! Detect when the user is idle and prevent idle timeouts.

use std::time::Duration;

/// Configuration for an idle notification.
#[derive(Debug, Clone)]
pub struct IdleNotificationConfig {
    /// How long the user must be idle before notification fires.
    pub timeout: Duration,
}

impl IdleNotificationConfig {
    /// Create with timeout in seconds.
    #[must_use]
    pub fn from_secs(secs: u64) -> Self {
        Self {
            timeout: Duration::from_secs(secs),
        }
    }

    /// Create with timeout in milliseconds.
    #[must_use]
    pub fn from_millis(millis: u64) -> Self {
        Self {
            timeout: Duration::from_millis(millis),
        }
    }

    /// Timeout in milliseconds (for the Wayland protocol).
    #[must_use]
    pub fn timeout_ms(&self) -> u32 {
        self.timeout.as_millis() as u32
    }
}

/// Current idle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdleState {
    /// User is active (has interacted recently).
    Active,
    /// User is idle (no interaction for the configured timeout).
    Idle,
}
