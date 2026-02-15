//! ext-session-lock: lock screen protocol.
//!
//! Allows creating lock screens that take exclusive control of outputs.

/// State of the session lock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LockState {
    /// Lock has been acknowledged by the compositor.
    /// All outputs should now show lock surfaces.
    Locked,
    /// Lock request was finished (session ended or lock dismissed).
    Finished,
}

/// Configuration for a lock surface on a specific output.
#[derive(Debug, Clone)]
pub struct LockSurfaceConfig {
    /// Desired width (usually matches output resolution).
    pub width: u32,
    /// Desired height (usually matches output resolution).
    pub height: u32,
}

impl LockSurfaceConfig {
    /// Create a lock surface config matching output dimensions.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}
