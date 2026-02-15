//! Wayland protocol client bindings.
//!
//! Client-side bindings for Hyprland-specific and wlr protocols.
//! Requires the `wayland` feature flag.

#[cfg(feature = "wayland")]
pub mod connection;
#[cfg(feature = "wayland")]
pub mod gamma_control;
#[cfg(feature = "wayland")]
pub mod output_management;
#[cfg(feature = "wayland")]
pub mod output_power;
#[cfg(feature = "wayland")]
pub mod screencopy;
