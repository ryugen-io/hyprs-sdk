//! Screencopy protocol client (`zwlr_screencopy_manager_v1`).
//!
//! Captures screenshots of compositor outputs using shared memory buffers.

mod client;
mod dispatch;
mod types;

pub use client::ScreencopyClient;
pub use types::*;
