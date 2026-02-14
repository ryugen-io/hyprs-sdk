//! High-level IPC client for a running Hyprland instance.
//!
//! Wraps Socket1 (request/response) with both raw and typed APIs.

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

/// IPC client for a single Hyprland instance.
///
/// Provides both raw string APIs and typed JSON-deserialized queries.
#[derive(Debug, Clone)]
pub struct HyprlandClient {
    socket1: PathBuf,
    socket2: PathBuf,
}

impl HyprlandClient {
    /// Create a client from a discovered instance.
    #[must_use]
    pub fn from_instance(instance: &Instance) -> Self {
        Self {
            socket1: instance.socket1_path(),
            socket2: instance.socket2_path(),
        }
    }

    /// Create a client for the current Hyprland session.
    ///
    /// Uses `$HYPRLAND_INSTANCE_SIGNATURE` to find the instance.
    pub fn current() -> HyprResult<Self> {
        let instance = crate::ipc::instance::current_instance()?;
        Ok(Self::from_instance(&instance))
    }

    /// Path to Socket1 (request/response).
    #[must_use]
    pub fn socket1_path(&self) -> &std::path::Path {
        &self.socket1
    }

    /// Path to Socket2 (event stream).
    #[must_use]
    pub fn socket2_path(&self) -> &std::path::Path {
        &self.socket2
    }

    // -- Raw request ----------------------------------------------------------

    /// Send a raw command and return the response string.
    ///
    /// Use this for commands where you want plain text output,
    /// or for commands not yet covered by typed methods.
    pub async fn request(&self, command: &str) -> HyprResult<String> {
        socket::request(&self.socket1, command).await
    }

    /// Send a raw command built with flags.
    pub async fn request_flagged(&self, flags: Flags, command: &str) -> HyprResult<String> {
        let wire = commands::flagged_pub(flags, command);
        self.request(&wire).await
    }

    // -- Action commands (return ok/error) ------------------------------------

    /// Send an action command and check for success.
    ///
    /// Returns `Ok(())` if Hyprland responds with "ok",
    /// otherwise returns `HyprError::Command` with the error text.
    async fn action(&self, command: &str) -> HyprResult<()> {
        let response = self.request(command).await?;
        if response.trim() == "ok" {
            Ok(())
        } else {
            Err(HyprError::Command(response))
        }
    }

    /// Dispatch a compositor action by name and args.
    pub async fn dispatch(&self, dispatcher: &str, args: &str) -> HyprResult<()> {
        self.action(&commands::dispatch(dispatcher, args)).await
    }

    /// Dispatch a typed command from the [`dispatch`](crate::dispatch) module.
    pub async fn dispatch_cmd(&self, cmd: DispatchCmd) -> HyprResult<()> {
        self.dispatch(cmd.name, &cmd.args).await
    }

    /// Set a configuration keyword at runtime.
    pub async fn keyword(&self, key: &str, value: &str) -> HyprResult<()> {
        self.action(&commands::keyword(key, value)).await
    }

    /// Reload configuration.
    pub async fn reload(&self, args: &str) -> HyprResult<()> {
        self.action(&commands::reload(args)).await
    }

    /// Activate kill mode (click to kill a window).
    pub async fn kill(&self) -> HyprResult<()> {
        self.action(&commands::kill()).await
    }

    /// Reload shader programs.
    pub async fn reload_shaders(&self) -> HyprResult<()> {
        self.action(&commands::reload_shaders()).await
    }

    /// Set cursor theme and size.
    pub async fn set_cursor(&self, theme: &str, size: u32) -> HyprResult<()> {
        self.action(&commands::set_cursor(theme, size)).await
    }

    /// Switch XKB keyboard layout.
    pub async fn switch_xkb_layout(&self, device: &str, cmd: &str) -> HyprResult<()> {
        self.action(&commands::switch_xkb_layout(device, cmd)).await
    }

    /// Set error message display (empty to disable).
    pub async fn set_error(&self, message: &str) -> HyprResult<()> {
        self.action(&commands::set_error(message)).await
    }

    /// Create a notification.
    pub async fn notify(
        &self,
        icon: i32,
        time_ms: u32,
        color: &str,
        message: &str,
    ) -> HyprResult<()> {
        self.action(&commands::notify(icon, time_ms, color, message))
            .await
    }

    /// Dismiss notifications.
    pub async fn dismiss_notify(&self, count: i32) -> HyprResult<()> {
        self.action(&commands::dismiss_notify(count)).await
    }

    /// Output/monitor configuration command.
    pub async fn output(&self, args: &str) -> HyprResult<()> {
        self.action(&commands::output(args)).await
    }

    /// Plugin management command.
    pub async fn plugin(&self, operation: &str) -> HyprResult<String> {
        self.request(&commands::plugin(operation)).await
    }

    /// Execute a batch of commands.
    pub async fn batch(&self, cmds: &[String]) -> HyprResult<String> {
        self.request(&commands::batch(cmds)).await
    }

    // -- Text queries (return raw string) -------------------------------------

    /// Get the current splash screen message.
    pub async fn splash(&self) -> HyprResult<String> {
        self.request(&commands::splash()).await
    }

    /// Get current keybind submap name.
    pub async fn submap(&self) -> HyprResult<String> {
        self.request(&commands::submap()).await
    }

    /// Get system information.
    pub async fn system_info(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::system_info(flags)).await
    }

    /// Get rolling log output.
    pub async fn rolling_log(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::rolling_log(flags)).await
    }

    /// Get Hyprland version info.
    pub async fn version(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::version(flags)).await
    }

    /// Get lock state.
    pub async fn locked(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::locked(flags)).await
    }

    /// Get command descriptions.
    pub async fn descriptions(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::descriptions(flags)).await
    }

    /// Get a window property.
    pub async fn get_prop(
        &self,
        window_address: &str,
        property: &str,
        flags: Flags,
    ) -> HyprResult<String> {
        self.request(&commands::get_prop(window_address, property, flags))
            .await
    }

    /// Get a configuration option value.
    pub async fn get_option(&self, name: &str, flags: Flags) -> HyprResult<String> {
        self.request(&commands::get_option(name, flags)).await
    }

    /// Get window decorations.
    pub async fn decorations(&self, window_address: &str, flags: Flags) -> HyprResult<String> {
        self.request(&commands::decorations(window_address, flags))
            .await
    }

    // -- Typed JSON queries ---------------------------------------------------

    /// Query all monitors (JSON-deserialized).
    pub async fn monitors_typed(&self) -> HyprResult<Vec<Monitor>> {
        let raw = self.request(&commands::monitors(Flags::json())).await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query all clients/windows (JSON-deserialized).
    pub async fn clients_typed(&self) -> HyprResult<Vec<Window>> {
        let raw = self.request(&commands::clients(Flags::json())).await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query all workspaces (JSON-deserialized).
    pub async fn workspaces_typed(&self) -> HyprResult<Vec<Workspace>> {
        let raw = self.request(&commands::workspaces(Flags::json())).await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query the active workspace (JSON-deserialized).
    pub async fn active_workspace_typed(&self) -> HyprResult<Workspace> {
        let raw = self
            .request(&commands::active_workspace(Flags::json()))
            .await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query the active window (JSON-deserialized).
    pub async fn active_window_typed(&self) -> HyprResult<Window> {
        let raw = self
            .request(&commands::active_window(Flags::json()))
            .await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query all layer surfaces (JSON-deserialized).
    pub async fn layers_typed(&self) -> HyprResult<LayersResponse> {
        let raw = self.request(&commands::layers(Flags::json())).await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    // -- Raw queries with flags -----------------------------------------------

    /// Query monitors with custom flags.
    pub async fn monitors(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::monitors(flags)).await
    }

    /// Query clients with custom flags.
    pub async fn clients(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::clients(flags)).await
    }

    /// Query workspaces with custom flags.
    pub async fn workspaces(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::workspaces(flags)).await
    }

    /// Query the active workspace with custom flags.
    pub async fn active_workspace(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::active_workspace(flags)).await
    }

    /// Query the active window with custom flags.
    pub async fn active_window(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::active_window(flags)).await
    }

    /// Query layers with custom flags.
    pub async fn layers(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::layers(flags)).await
    }

    /// Query workspace rules with custom flags.
    pub async fn workspace_rules(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::workspace_rules(flags)).await
    }

    /// Query all keybindings with custom flags.
    pub async fn binds(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::binds(flags)).await
    }

    /// Query all input devices with custom flags.
    pub async fn devices(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::devices(flags)).await
    }

    /// Query cursor position with custom flags.
    pub async fn cursor_pos(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::cursor_pos(flags)).await
    }

    /// Query global shortcuts with custom flags.
    pub async fn global_shortcuts(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::global_shortcuts(flags)).await
    }

    /// Query animations with custom flags.
    pub async fn animations(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::animations(flags)).await
    }

    /// Query layouts with custom flags.
    pub async fn layouts(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::layouts(flags)).await
    }

    /// Query config errors with custom flags.
    pub async fn config_errors(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::config_errors(flags)).await
    }

    // -- Event stream ---------------------------------------------------------

    /// Connect to Socket2 and return a raw event stream.
    ///
    /// For parsed events, wrap the stream with [`crate::ipc::events::EventStream`].
    pub async fn event_stream(&self) -> HyprResult<tokio::net::UnixStream> {
        socket::connect_event_stream(&self.socket2).await
    }
}
