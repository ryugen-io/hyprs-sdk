//! wlr-output-management: monitor configuration protocol.
//!
//! Provides [`OutputManagementClient`] for querying and configuring
//! output (monitor) properties via the `zwlr_output_manager_v1` protocol.

mod client;
mod dispatch;
mod types;

pub use client::OutputManagementClient;
pub use types::*;
