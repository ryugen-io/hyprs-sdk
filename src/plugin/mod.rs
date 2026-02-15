//! Plugin FFI for writing Hyprland plugins in Rust.
//!
//! This module provides types, hook event definitions, raw FFI declarations,
//! and lifecycle macros for building Hyprland plugins as Rust shared libraries.
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

#[allow(unsafe_code)]
pub mod ffi;
pub mod hooks;
#[allow(unsafe_code)]
pub mod lifecycle;
#[allow(unsafe_code)]
pub mod types;

pub use hooks::HookEvent;
pub use types::*;
