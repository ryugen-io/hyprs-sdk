//! Blocking (synchronous) IPC client.
//!
//! Mirrors the async [`super::HyprlandClient`] API using standard library
//! blocking I/O. Requires the `blocking` feature flag.

use std::path::PathBuf;

use crate::dispatch::DispatchCmd;
use crate::error::{HyprError, HyprResult};
use crate::ipc::commands::{self, Flags};
use crate::ipc::instance::Instance;
use crate::ipc::socket;
use crate::types::layer::LayersResponse;
use crate::types::monitor::Monitor;
use crate::types::window::Window;
use crate::types::workspace::Workspace;

/// Blocking IPC client for a single Hyprland instance.
///
/// Provides the same API as [`super::HyprlandClient`] but uses synchronous I/O.
/// Requires the `blocking` feature flag.
#[derive(Debug, Clone)]
pub struct BlockingClient {
    socket1: PathBuf,
}

impl BlockingClient {
    /// Create a client from a discovered instance.
    #[must_use]
    pub fn from_instance(instance: &Instance) -> Self {
        Self {
            socket1: instance.socket1_path(),
        }
    }

    /// Create a client for the current Hyprland session.
    pub fn current() -> HyprResult<Self> {
        let instance = crate::ipc::instance::current_instance()?;
        Ok(Self::from_instance(&instance))
    }

    // -- Raw request ----------------------------------------------------------

    /// Send a raw command and return the response string.
    pub fn request(&self, command: &str) -> HyprResult<String> {
        socket::blocking::request(&self.socket1, command)
    }

    /// Send a raw command built with flags.
    pub fn request_flagged(&self, flags: Flags, command: &str) -> HyprResult<String> {
        let wire = commands::flagged_pub(flags, command);
        self.request(&wire)
    }

    // -- Action commands ------------------------------------------------------

    fn action(&self, command: &str) -> HyprResult<()> {
        let response = self.request(command)?;
        if response.trim() == "ok" {
            Ok(())
        } else {
            Err(HyprError::Command(response))
        }
    }

    /// Dispatch a compositor action by name and args.
    pub fn dispatch(&self, dispatcher: &str, args: &str) -> HyprResult<()> {
        self.action(&commands::dispatch(dispatcher, args))
    }

    /// Dispatch a typed command from the [`dispatch`](crate::dispatch) module.
    pub fn dispatch_cmd(&self, cmd: DispatchCmd) -> HyprResult<()> {
        self.dispatch(cmd.name, &cmd.args)
    }

    /// Set a configuration keyword at runtime.
    pub fn keyword(&self, key: &str, value: &str) -> HyprResult<()> {
        self.action(&commands::keyword(key, value))
    }

    /// Reload configuration.
    pub fn reload(&self, args: &str) -> HyprResult<()> {
        self.action(&commands::reload(args))
    }

    /// Activate kill mode.
    pub fn kill(&self) -> HyprResult<()> {
        self.action(&commands::kill())
    }

    /// Reload shader programs.
    pub fn reload_shaders(&self) -> HyprResult<()> {
        self.action(&commands::reload_shaders())
    }

    /// Set cursor theme and size.
    pub fn set_cursor(&self, theme: &str, size: u32) -> HyprResult<()> {
        self.action(&commands::set_cursor(theme, size))
    }

    /// Switch XKB keyboard layout.
    pub fn switch_xkb_layout(&self, device: &str, cmd: &str) -> HyprResult<()> {
        self.action(&commands::switch_xkb_layout(device, cmd))
    }

    /// Set error message display.
    pub fn set_error(&self, message: &str) -> HyprResult<()> {
        self.action(&commands::set_error(message))
    }

    /// Create a notification.
    pub fn notify(&self, icon: i32, time_ms: u32, color: &str, message: &str) -> HyprResult<()> {
        self.action(&commands::notify(icon, time_ms, color, message))
    }

    /// Dismiss notifications.
    pub fn dismiss_notify(&self, count: i32) -> HyprResult<()> {
        self.action(&commands::dismiss_notify(count))
    }

    /// Output/monitor configuration command.
    pub fn output(&self, args: &str) -> HyprResult<()> {
        self.action(&commands::output(args))
    }

    /// Plugin management command.
    pub fn plugin(&self, operation: &str) -> HyprResult<String> {
        self.request(&commands::plugin(operation))
    }

    /// Execute a batch of commands.
    pub fn batch(&self, cmds: &[String]) -> HyprResult<String> {
        self.request(&commands::batch(cmds))
    }

    // -- Text queries ---------------------------------------------------------

    /// Get splash screen message.
    pub fn splash(&self) -> HyprResult<String> {
        self.request(&commands::splash())
    }

    /// Get current submap name.
    pub fn submap(&self) -> HyprResult<String> {
        self.request(&commands::submap())
    }

    /// Get system information.
    pub fn system_info(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::system_info(flags))
    }

    /// Get rolling log output.
    pub fn rolling_log(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::rolling_log(flags))
    }

    /// Get version info.
    pub fn version(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::version(flags))
    }

    /// Get lock state.
    pub fn locked(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::locked(flags))
    }

    /// Get command descriptions.
    pub fn descriptions(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::descriptions(flags))
    }

    /// Get a window property.
    pub fn get_prop(&self, window: &str, property: &str, flags: Flags) -> HyprResult<String> {
        self.request(&commands::get_prop(window, property, flags))
    }

    /// Get a configuration option value.
    pub fn get_option(&self, name: &str, flags: Flags) -> HyprResult<String> {
        self.request(&commands::get_option(name, flags))
    }

    /// Get window decorations.
    pub fn decorations(&self, window: &str, flags: Flags) -> HyprResult<String> {
        self.request(&commands::decorations(window, flags))
    }

    // -- Typed JSON queries ---------------------------------------------------

    /// Query all monitors (JSON-deserialized).
    pub fn monitors_typed(&self) -> HyprResult<Vec<Monitor>> {
        let raw = self.request(&commands::monitors(Flags::json()))?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query all clients/windows (JSON-deserialized).
    pub fn clients_typed(&self) -> HyprResult<Vec<Window>> {
        let raw = self.request(&commands::clients(Flags::json()))?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query all workspaces (JSON-deserialized).
    pub fn workspaces_typed(&self) -> HyprResult<Vec<Workspace>> {
        let raw = self.request(&commands::workspaces(Flags::json()))?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query the active workspace (JSON-deserialized).
    pub fn active_workspace_typed(&self) -> HyprResult<Workspace> {
        let raw = self.request(&commands::active_workspace(Flags::json()))?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query the active window (JSON-deserialized).
    pub fn active_window_typed(&self) -> HyprResult<Window> {
        let raw = self.request(&commands::active_window(Flags::json()))?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query all layer surfaces (JSON-deserialized).
    pub fn layers_typed(&self) -> HyprResult<LayersResponse> {
        let raw = self.request(&commands::layers(Flags::json()))?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    // -- Raw queries with flags -----------------------------------------------

    /// Query monitors with custom flags.
    pub fn monitors(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::monitors(flags))
    }

    /// Query clients with custom flags.
    pub fn clients(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::clients(flags))
    }

    /// Query workspaces with custom flags.
    pub fn workspaces(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::workspaces(flags))
    }

    /// Query the active workspace with custom flags.
    pub fn active_workspace(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::active_workspace(flags))
    }

    /// Query the active window with custom flags.
    pub fn active_window(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::active_window(flags))
    }

    /// Query layers with custom flags.
    pub fn layers(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::layers(flags))
    }

    /// Query workspace rules with custom flags.
    pub fn workspace_rules(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::workspace_rules(flags))
    }

    /// Query all keybindings with custom flags.
    pub fn binds(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::binds(flags))
    }

    /// Query all input devices with custom flags.
    pub fn devices(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::devices(flags))
    }

    /// Query cursor position with custom flags.
    pub fn cursor_pos(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::cursor_pos(flags))
    }

    /// Query global shortcuts with custom flags.
    pub fn global_shortcuts(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::global_shortcuts(flags))
    }

    /// Query animations with custom flags.
    pub fn animations(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::animations(flags))
    }

    /// Query layouts with custom flags.
    pub fn layouts(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::layouts(flags))
    }

    /// Query config errors with custom flags.
    pub fn config_errors(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::config_errors(flags))
    }
}
