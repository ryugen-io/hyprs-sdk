#![deny(unsafe_code)]
#![doc = include_str!("../README.md")]

/// Hyprland version detected from the system at build time (via pkg-config).
/// Falls back to a manually set version if pkg-config is unavailable.
pub const HYPRLAND_TARGET_VERSION: &str = match option_env!("HYPRLAND_SYSTEM_VERSION") {
    Some(v) => v,
    None => "0.54.1",
};

pub mod config;
pub mod dispatch;
pub mod error;
pub mod hyprpm;
pub mod ipc;
pub mod plugin;
pub mod protocols;
pub mod types;

pub use error::{HyprError, HyprResult};
pub use types::common::{MonitorId, WindowAddress, WorkspaceId};
