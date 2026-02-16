//! IPC client for communicating with a running Hyprland instance.
//!
//! Covers Socket1 (request/response) and Socket2 (event stream).

pub mod client;
pub mod commands;
pub mod events;
pub mod instance;
pub mod responses;
pub mod socket;

#[cfg(feature = "blocking")]
pub mod blocking;

pub use client::HyprlandClient;
pub use commands::Flags;
pub use events::{Event, EventStream};

#[cfg(feature = "blocking")]
pub use blocking::BlockingClient;
