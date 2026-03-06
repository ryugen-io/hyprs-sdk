//! High-level IPC client for a running Hyprland instance.
//!
//! Wraps Socket1 (request/response) with both raw and typed APIs.

use std::path::PathBuf;

use crate::dispatch::DispatchCmd;
use crate::error::{HyprError, HyprResult};
use crate::ipc::commands::{self, Flags};
use crate::ipc::common::{normalize_window_selector, parse_json_or_command_error};
use crate::ipc::instance::Instance;
use crate::ipc::responses;
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

    // Lowest-level API: callers who need unparsed text or want to send commands this SDK doesn't wrap yet.

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

    // Actions differ from queries: Hyprland responds with "ok" or an error string.
    // Separating them lets us return Result<()> instead of forcing callers to inspect raw text.

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

    // Some commands (splash, submap, systeminfo) only return human-readable text with no JSON mode.
    // These must stay as raw-string queries because there is no structured format to deserialize.

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
    ///
    /// `window_address` accepts either:
    /// - a selector (e.g. `address:0xabc`, `class:^(kitty)$`)
    /// - a raw window pointer address (`0xabc`), which is normalized to
    ///   `address:0xabc` for `hyprctl getprop`.
    pub async fn get_prop<P: AsRef<str>>(
        &self,
        window_address: &str,
        property: P,
        flags: Flags,
    ) -> HyprResult<String> {
        let selector = normalize_window_selector(window_address);
        self.request(&commands::get_prop(&selector, property.as_ref(), flags))
            .await
    }

    /// Get a configuration option value.
    pub async fn get_option(&self, name: &str, flags: Flags) -> HyprResult<String> {
        self.request(&commands::get_option(name, flags)).await
    }

    /// Get window decorations.
    pub async fn decorations(&self, window_address: &str, flags: Flags) -> HyprResult<String> {
        let selector = normalize_window_selector(window_address);
        self.request(&commands::decorations(&selector, flags)).await
    }

    // Typed queries force the `j` (JSON) flag and deserialize into Rust structs.
    // This is the preferred API: callers get compile-time field access instead of string parsing.

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

    /// Query Hyprland version information (JSON-deserialized).
    pub async fn version_typed(&self) -> HyprResult<responses::VersionInfo> {
        let raw = self.request(&commands::version(Flags::json())).await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query all input devices (JSON-deserialized).
    pub async fn devices_typed(&self) -> HyprResult<responses::DevicesResponse> {
        let raw = self.request(&commands::devices(Flags::json())).await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query all keybindings (JSON-deserialized).
    pub async fn binds_typed(&self) -> HyprResult<Vec<responses::Bind>> {
        let raw = self.request(&commands::binds(Flags::json())).await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query cursor position (JSON-deserialized).
    pub async fn cursor_pos_typed(&self) -> HyprResult<responses::CursorPosition> {
        let raw = self.request(&commands::cursor_pos(Flags::json())).await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query animation configurations (JSON-deserialized).
    pub async fn animations_typed(&self) -> HyprResult<responses::AnimationsResponse> {
        let raw = self.request(&commands::animations(Flags::json())).await?;
        responses::AnimationsResponse::from_json(&raw).map_err(HyprError::Json)
    }

    /// Query registered global shortcuts (JSON-deserialized).
    pub async fn global_shortcuts_typed(&self) -> HyprResult<Vec<responses::GlobalShortcutInfo>> {
        let raw = self
            .request(&commands::global_shortcuts(Flags::json()))
            .await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query workspace rules (JSON-deserialized).
    pub async fn workspace_rules_typed(&self) -> HyprResult<Vec<responses::WorkspaceRuleInfo>> {
        let raw = self
            .request(&commands::workspace_rules(Flags::json()))
            .await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query configuration errors (JSON-deserialized).
    pub async fn config_errors_typed(&self) -> HyprResult<Vec<String>> {
        let raw = self
            .request(&commands::config_errors(Flags::json()))
            .await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query session lock state (JSON-deserialized).
    pub async fn locked_typed(&self) -> HyprResult<responses::LockState> {
        let raw = self.request(&commands::locked(Flags::json())).await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query a configuration option value (JSON-deserialized).
    pub async fn get_option_typed(&self, name: &str) -> HyprResult<responses::OptionValue> {
        let raw = self
            .request(&commands::get_option(name, Flags::json()))
            .await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query window decorations (JSON-deserialized).
    pub async fn decorations_typed(
        &self,
        window_address: &str,
    ) -> HyprResult<Vec<responses::DecorationInfo>> {
        let selector = normalize_window_selector(window_address);
        let raw = self
            .request(&commands::decorations(&selector, Flags::json()))
            .await?;
        if raw.trim() == "none" {
            return Ok(Vec::new());
        }
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query all config option descriptions (JSON-deserialized).
    ///
    /// Returns metadata for every config option including type, default
    /// value, current value, and whether it was explicitly set.
    pub async fn descriptions_typed(&self) -> HyprResult<Vec<responses::ConfigDescription>> {
        let raw = self.request(&commands::descriptions(Flags::json())).await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query loaded plugins (JSON-deserialized).
    pub async fn plugin_list_typed(&self) -> HyprResult<Vec<responses::PluginInfo>> {
        let raw = self.request_flagged(Flags::json(), "plugin list").await?;
        serde_json::from_str(&raw).map_err(HyprError::Json)
    }

    /// Query a window property as a JSON value.
    ///
    /// The shape of the returned value varies by property. Common
    /// properties: `"minSize"`, `"maxSize"`, `"alpha"`, `"alphaOverride"`.
    pub async fn get_prop_value<P: AsRef<str>>(
        &self,
        window_address: &str,
        property: P,
    ) -> HyprResult<serde_json::Value> {
        let selector = normalize_window_selector(window_address);
        let raw = self
            .request(&commands::get_prop(
                &selector,
                property.as_ref(),
                Flags::json(),
            ))
            .await?;
        parse_json_or_command_error(raw)
    }

    // Raw flagged queries let callers combine flags (json, all, config, reload) and handle
    // the response themselves. Useful when the caller needs a flag combo the typed API doesn't cover.

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

    /// Query config errors with custom flags.
    pub async fn config_errors(&self, flags: Flags) -> HyprResult<String> {
        self.request(&commands::config_errors(flags)).await
    }

    // Socket2 is a separate Unix socket that pushes events (no request/response).
    // Returning the raw stream lets callers choose their own parsing/buffering strategy.

    /// Connect to Socket2 and return a raw event stream.
    ///
    /// For parsed events, wrap the stream with [`crate::ipc::events::EventStream`].
    pub async fn event_stream(&self) -> HyprResult<tokio::net::UnixStream> {
        socket::connect_event_stream(&self.socket2).await
    }
}
