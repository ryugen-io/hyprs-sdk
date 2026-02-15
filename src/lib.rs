#![deny(unsafe_code)]
#![doc = include_str!("../README.md")]

/// Target Hyprland version this SDK was verified against.
pub const HYPRLAND_TARGET_VERSION: &str = "0.53.0";

pub mod config;
pub mod dispatch;
pub mod error;
pub mod ipc;
pub mod plugin;
pub mod protocols;
pub mod types;

pub use error::{HyprError, HyprResult};
pub use types::common::{MonitorId, WindowAddress, WorkspaceId};
