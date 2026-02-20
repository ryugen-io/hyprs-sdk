//! Safe wrappers for the Hyprland plugin API.
//!
//! Provides high-level Rust APIs for:
//! - Hook event registration
//! - HyprCtl command invocation and registration
//! - Notifications
//! - Config reload
//! - Function hooking (advanced/unstable)
//! - Version queries
mod function_hooks;
mod hooks;
mod hyprctl;
mod notify;
mod version;

pub use function_hooks::{
    FunctionHookHandle, create_function_hook, find_functions_by_name, remove_function_hook,
};
pub use hooks::{HookCallback, HookCallbackGuard, register_hook};
pub use hyprctl::{
    HyprCtlCommandGuard, HyprCtlCommandHandler, invoke_hyprctl, register_hyprctl_command,
};
pub use notify::{Color, add_notification, add_notification_v2, reload_config};
pub use version::{get_server_hash, get_version};
