//! Wayland protocol client bindings.
//!
//! Client-side bindings for Hyprland-specific and wlr protocols.
//! Requires the `wayland` feature flag.

#[cfg(feature = "wayland")]
pub mod connection;
#[cfg(feature = "wayland")]
pub mod data_control;
#[cfg(feature = "wayland")]
pub mod foreign_toplevel;
#[cfg(feature = "wayland")]
pub mod gamma_control;
#[cfg(feature = "wayland")]
pub mod idle;
#[cfg(feature = "wayland")]
pub mod layer_shell;
#[cfg(feature = "wayland")]
pub mod output_management;
#[cfg(feature = "wayland")]
pub mod output_power;
#[cfg(feature = "wayland")]
pub mod screencopy;
#[cfg(feature = "wayland")]
pub mod session_lock;
#[cfg(feature = "wayland")]
pub mod virtual_keyboard;
#[cfg(feature = "wayland")]
pub mod virtual_pointer;
