//! Plugin FFI for writing Hyprland plugins in Rust.
//!
//! This module provides types, hook event definitions, raw FFI declarations,
//! lifecycle macros, and safe wrappers for building Hyprland plugins as
//! Rust shared libraries.
//!
//! # Architecture
//!
//! Hyprland's plugin API uses `extern "C"` linkage but passes C++ objects
//! (`std::string`, `std::function`, `std::any`). A C++ bridge shim is
//! required between Rust code and the actual HyprlandAPI functions.
//!
//! This module provides:
//! - **`types`** — Pure Rust types modeling plugin API concepts
//! - **`hooks`** — Strongly-typed enum of all 50 hook events
//! - **`ffi`** — Raw `extern "C"` function declarations (needs C++ bridge)
//! - **`lifecycle`** — `hyprland_plugin!` macro for entry point generation
//! - **`api`** — Safe wrappers for hooks, hyprctl, notifications, version
//! - **`config`** — Plugin config value registration and retrieval
//! - **`dispatcher`** — Custom dispatcher registration with RAII guards
//! - **`layout`** — Custom window layout trait and registration
//! - **`decoration`** — Custom window decoration trait and registration

#[allow(unsafe_code)]
pub mod api;
#[allow(unsafe_code)]
pub mod config;
#[allow(unsafe_code)]
pub mod decoration;
#[allow(unsafe_code)]
pub mod dispatcher;
#[allow(unsafe_code)]
pub mod ffi;
pub mod hooks;
#[allow(unsafe_code)]
pub mod layout;
#[allow(unsafe_code)]
pub mod lifecycle;
#[allow(unsafe_code)]
pub mod types;

pub use hooks::HookEvent;
pub use types::*;

// Flatten the public API so users can `use hyprs_sdk::plugin::*` without navigating submodule paths.
pub use api::{
    Color, FunctionHookHandle, HookCallback, HookCallbackGuard, HyprCtlCommandGuard,
    HyprCtlCommandHandler, add_notification, add_notification_v2, reload_config,
};
pub use config::{ConfigDefault, ConfigValueHandle, KeywordHandler, KeywordHandlerOptions};
pub use decoration::{
    DecorationEdges, DecorationFlags, DecorationHandle, DecorationLayer, DecorationPositionPolicy,
    DecorationPositioningInfo, DecorationType, WindowDecoration, WindowHandle,
};
pub use dispatcher::{DispatcherFn, DispatcherGuard};
pub use layout::{Direction, Layout, LayoutHandle, RectCorner};
