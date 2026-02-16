//! wlr-output-management: monitor configuration protocol.
//!
//! Provides [`OutputManagementClient`] for querying and configuring
//! output (monitor) properties via the `zwlr_output_manager_v1` protocol.

use std::fmt;

use wayland_client::protocol::{wl_output, wl_registry};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, WEnum, event_created_child};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1, zwlr_output_configuration_v1, zwlr_output_head_v1,
    zwlr_output_manager_v1, zwlr_output_mode_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// A display mode supported by an output.
#[derive(Debug, Clone, PartialEq)]
pub struct OutputMode {
    /// Horizontal resolution in pixels.
    pub width: i32,
    /// Vertical resolution in pixels.
    pub height: i32,
    /// Refresh rate in millihertz (e.g., 60000 = 60 Hz).
    pub refresh: i32,
    /// Whether this is the output's preferred mode.
    pub preferred: bool,
}

impl OutputMode {
    /// Convert the refresh rate from millihertz to hertz.
    #[must_use]
    pub fn refresh_hz(&self) -> f64 {
        self.refresh as f64 / 1000.0
    }
}

/// Describes the current state of an output (monitor/display).
#[derive(Debug, Clone, Default)]
pub struct OutputHead {
    /// Connector name (e.g., "DP-1", "HDMI-A-1").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Physical width in millimeters.
    pub physical_width: i32,
    /// Physical height in millimeters.
    pub physical_height: i32,
    /// Supported display modes.
    pub modes: Vec<OutputMode>,
    /// Whether the output is currently enabled.
    pub enabled: bool,
    /// Index into `modes` for the currently active mode.
    pub current_mode: Option<usize>,
    /// Horizontal position in the global compositor space.
    pub position_x: i32,
    /// Vertical position in the global compositor space.
    pub position_y: i32,
    /// Output scale factor.
    pub scale: f64,
    /// Output transform (rotation/reflection).
    pub transform: i32,
    /// Manufacturer name.
    pub make: String,
    /// Model name.
    pub model: String,
    /// Serial number.
    pub serial_number: String,
}

/// A configuration entry for applying output settings.
#[derive(Debug, Clone)]
pub struct OutputConfigEntry {
    /// Connector name to configure.
    pub name: String,
    /// Whether to enable or disable the output.
    pub enabled: bool,
    /// Desired display mode, if changing.
    pub mode: Option<OutputMode>,
    /// Desired horizontal position, if changing.
    pub position_x: Option<i32>,
    /// Desired vertical position, if changing.
    pub position_y: Option<i32>,
    /// Desired scale factor, if changing.
    pub scale: Option<f64>,
    /// Desired transform, if changing.
    pub transform: Option<i32>,
}

/// Client for the `zwlr_output_manager_v1` protocol.
///
/// Queries and configures output (monitor) properties such as resolution,
/// position, scale, and enabled state.
///
/// # Example
///
/// ```no_run
/// use hypr_sdk::protocols::connection::WaylandConnection;
/// use hypr_sdk::protocols::output_management::OutputManagementClient;
///
/// let wl = WaylandConnection::connect().unwrap();
/// let client = OutputManagementClient::connect(&wl).unwrap();
///
/// for head in client.heads() {
///     println!("{}: {}x{}", head.name,
///         head.modes.get(head.current_mode.unwrap_or(0))
///             .map(|m| format!("{}x{}", m.width, m.height))
///             .unwrap_or_default(),
///         if head.enabled { "enabled" } else { "disabled" });
/// }
/// ```
pub struct OutputManagementClient {
    state: OutputManagementState,
    event_queue: EventQueue<OutputManagementState>,
}

impl OutputManagementClient {
    /// Connect to the output management protocol.
    ///
    /// Binds `zwlr_output_manager_v1`, discovers all outputs and their
    /// supported modes, and queries current configuration.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `zwlr_output_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("zwlr_output_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "zwlr_output_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<OutputManagementState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = OutputManagementState::new();

        // Registry roundtrip: bind manager.
        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "zwlr_output_manager_v1".into(),
            ));
        }

        // Second roundtrip: receive head events from manager.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Third roundtrip: receive head property events + mode events + done.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Fourth roundtrip: ensure all mode properties are received.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self { state, event_queue })
    }

    /// All output heads and their current state.
    #[must_use]
    pub fn heads(&self) -> Vec<OutputHead> {
        self.state
            .heads
            .iter()
            .filter(|h| !h.finished)
            .map(|h| {
                let modes: Vec<OutputMode> = h
                    .modes
                    .iter()
                    .filter(|m| !m.finished)
                    .map(|m| OutputMode {
                        width: m.width,
                        height: m.height,
                        refresh: m.refresh,
                        preferred: m.preferred,
                    })
                    .collect();

                // Find current mode index by matching the proxy.
                let current_mode = h.current_mode_proxy.as_ref().and_then(|current| {
                    h.modes
                        .iter()
                        .filter(|m| !m.finished)
                        .position(|m| m.proxy == *current)
                });

                OutputHead {
                    name: h.name.clone(),
                    description: h.description.clone(),
                    physical_width: h.physical_width,
                    physical_height: h.physical_height,
                    modes,
                    enabled: h.enabled,
                    current_mode,
                    position_x: h.position_x,
                    position_y: h.position_y,
                    scale: h.scale,
                    transform: h.transform,
                    make: h.make.clone(),
                    model: h.model.clone(),
                    serial_number: h.serial_number.clone(),
                }
            })
            .collect()
    }

    /// Apply a configuration to outputs.
    ///
    /// Creates a configuration object, sets properties for each entry,
    /// and applies it. The compositor may reject the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is rejected or dispatch fails.
    pub fn apply(&mut self, entries: &[OutputConfigEntry]) -> HyprResult<()> {
        let Self { state, event_queue } = self;
        let qh = event_queue.handle();

        let manager = state
            .manager
            .as_ref()
            .ok_or_else(|| HyprError::ProtocolNotSupported("zwlr_output_manager_v1".into()))?;

        let config = manager.create_configuration(state.serial, &qh, ());

        for entry in entries {
            let head = state
                .heads
                .iter()
                .find(|h| h.name == entry.name && !h.finished)
                .ok_or_else(|| {
                    HyprError::WaylandDispatch(format!("output head not found: {}", entry.name))
                })?;

            if entry.enabled {
                let config_head = config.enable_head(&head.proxy, &qh, ());

                if let Some(ref mode) = entry.mode {
                    // Find matching mode proxy.
                    if let Some(mode_entry) = head.modes.iter().find(|m| {
                        m.width == mode.width
                            && m.height == mode.height
                            && m.refresh == mode.refresh
                    }) {
                        config_head.set_mode(&mode_entry.proxy);
                    } else {
                        config_head.set_custom_mode(mode.width, mode.height, mode.refresh);
                    }
                }

                if let Some(x) = entry.position_x
                    && let Some(y) = entry.position_y
                {
                    config_head.set_position(x, y);
                }

                if let Some(scale) = entry.scale {
                    config_head.set_scale(scale);
                }

                if let Some(transform) = entry.transform {
                    config_head.set_transform(transform_from_i32(transform));
                }
            } else {
                config.disable_head(&head.proxy);
            }
        }

        config.apply();

        state.config_result = None;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        match state.config_result {
            Some(ConfigResult::Succeeded) => Ok(()),
            Some(ConfigResult::Failed) => Err(HyprError::WaylandDispatch(
                "output configuration rejected by compositor".into(),
            )),
            Some(ConfigResult::Cancelled) => Err(HyprError::WaylandDispatch(
                "output configuration cancelled (outdated serial)".into(),
            )),
            None => {
                // Extra roundtrip in case result wasn't received yet.
                event_queue
                    .roundtrip(state)
                    .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
                match state.config_result {
                    Some(ConfigResult::Succeeded) => Ok(()),
                    Some(ConfigResult::Failed) => Err(HyprError::WaylandDispatch(
                        "output configuration rejected by compositor".into(),
                    )),
                    _ => Err(HyprError::WaylandDispatch(
                        "no configuration result received".into(),
                    )),
                }
            }
        }
    }

    /// Test a configuration without applying it.
    ///
    /// Returns `Ok(())` if the compositor would accept the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration would be rejected.
    pub fn test(&mut self, entries: &[OutputConfigEntry]) -> HyprResult<()> {
        let Self { state, event_queue } = self;
        let qh = event_queue.handle();

        let manager = state
            .manager
            .as_ref()
            .ok_or_else(|| HyprError::ProtocolNotSupported("zwlr_output_manager_v1".into()))?;

        let config = manager.create_configuration(state.serial, &qh, ());

        for entry in entries {
            let head = state
                .heads
                .iter()
                .find(|h| h.name == entry.name && !h.finished)
                .ok_or_else(|| {
                    HyprError::WaylandDispatch(format!("output head not found: {}", entry.name))
                })?;

            if entry.enabled {
                let config_head = config.enable_head(&head.proxy, &qh, ());
                if let Some(ref mode) = entry.mode {
                    if let Some(mode_entry) = head.modes.iter().find(|m| {
                        m.width == mode.width
                            && m.height == mode.height
                            && m.refresh == mode.refresh
                    }) {
                        config_head.set_mode(&mode_entry.proxy);
                    } else {
                        config_head.set_custom_mode(mode.width, mode.height, mode.refresh);
                    }
                }
                if let Some(x) = entry.position_x
                    && let Some(y) = entry.position_y
                {
                    config_head.set_position(x, y);
                }
                if let Some(scale) = entry.scale {
                    config_head.set_scale(scale);
                }
                if let Some(transform) = entry.transform {
                    config_head.set_transform(transform_from_i32(transform));
                }
            } else {
                config.disable_head(&head.proxy);
            }
        }

        config.test();

        state.config_result = None;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        match state.config_result {
            Some(ConfigResult::Succeeded) => Ok(()),
            Some(ConfigResult::Failed) => Err(HyprError::WaylandDispatch(
                "output configuration test failed".into(),
            )),
            Some(ConfigResult::Cancelled) => Err(HyprError::WaylandDispatch(
                "output configuration cancelled (outdated serial)".into(),
            )),
            None => Err(HyprError::WaylandDispatch(
                "no configuration result received".into(),
            )),
        }
    }

    /// Re-dispatch events to update output state.
    pub fn refresh(&mut self) -> HyprResult<()> {
        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
        Ok(())
    }
}

impl fmt::Debug for OutputManagementClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OutputManagementClient")
            .field("heads", &self.state.heads.len())
            .finish()
    }
}

// ── Internal state ───────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigResult {
    Succeeded,
    Failed,
    Cancelled,
}

struct OutputManagementState {
    manager: Option<zwlr_output_manager_v1::ZwlrOutputManagerV1>,
    heads: Vec<HeadEntry>,
    serial: u32,
    config_result: Option<ConfigResult>,
}

struct HeadEntry {
    proxy: zwlr_output_head_v1::ZwlrOutputHeadV1,
    name: String,
    description: String,
    physical_width: i32,
    physical_height: i32,
    modes: Vec<ModeEntry>,
    enabled: bool,
    current_mode_proxy: Option<zwlr_output_mode_v1::ZwlrOutputModeV1>,
    position_x: i32,
    position_y: i32,
    scale: f64,
    transform: i32,
    make: String,
    model: String,
    serial_number: String,
    finished: bool,
}

struct ModeEntry {
    proxy: zwlr_output_mode_v1::ZwlrOutputModeV1,
    width: i32,
    height: i32,
    refresh: i32,
    preferred: bool,
    finished: bool,
}

impl OutputManagementState {
    fn new() -> Self {
        Self {
            manager: None,
            heads: Vec::new(),
            serial: 0,
            config_result: None,
        }
    }
}

fn transform_from_i32(val: i32) -> wl_output::Transform {
    match val {
        1 => wl_output::Transform::_90,
        2 => wl_output::Transform::_180,
        3 => wl_output::Transform::_270,
        4 => wl_output::Transform::Flipped,
        5 => wl_output::Transform::Flipped90,
        6 => wl_output::Transform::Flipped180,
        7 => wl_output::Transform::Flipped270,
        _ => wl_output::Transform::Normal,
    }
}

// ── Dispatch implementations ─────────────────────────────────────────

impl Dispatch<wl_registry::WlRegistry, ()> for OutputManagementState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
            && interface == "zwlr_output_manager_v1"
            && state.manager.is_none()
        {
            let mgr = registry.bind::<zwlr_output_manager_v1::ZwlrOutputManagerV1, (), Self>(
                name,
                version.min(4),
                qh,
                (),
            );
            state.manager = Some(mgr);
        }
    }
}

impl Dispatch<zwlr_output_manager_v1::ZwlrOutputManagerV1, ()> for OutputManagementState {
    fn event(
        state: &mut Self,
        _proxy: &zwlr_output_manager_v1::ZwlrOutputManagerV1,
        event: zwlr_output_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_output_manager_v1::Event::Head { head } => {
                state.heads.push(HeadEntry {
                    proxy: head,
                    name: String::new(),
                    description: String::new(),
                    physical_width: 0,
                    physical_height: 0,
                    modes: Vec::new(),
                    enabled: false,
                    current_mode_proxy: None,
                    position_x: 0,
                    position_y: 0,
                    scale: 1.0,
                    transform: 0,
                    make: String::new(),
                    model: String::new(),
                    serial_number: String::new(),
                    finished: false,
                });
            }
            zwlr_output_manager_v1::Event::Done { serial } => {
                state.serial = serial;
            }
            zwlr_output_manager_v1::Event::Finished => {
                // Manager going away.
            }
            _ => {}
        }
    }

    event_created_child!(OutputManagementState, zwlr_output_manager_v1::ZwlrOutputManagerV1, [
        // Opcode 0 = head event creates a new head object.
        0 => (zwlr_output_head_v1::ZwlrOutputHeadV1, ()),
    ]);
}

impl Dispatch<zwlr_output_head_v1::ZwlrOutputHeadV1, ()> for OutputManagementState {
    fn event(
        state: &mut Self,
        proxy: &zwlr_output_head_v1::ZwlrOutputHeadV1,
        event: zwlr_output_head_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let Some(head) = state.heads.iter_mut().find(|h| h.proxy == *proxy) {
            match event {
                zwlr_output_head_v1::Event::Name { name } => {
                    head.name = name;
                }
                zwlr_output_head_v1::Event::Description { description } => {
                    head.description = description;
                }
                zwlr_output_head_v1::Event::PhysicalSize { width, height } => {
                    head.physical_width = width;
                    head.physical_height = height;
                }
                zwlr_output_head_v1::Event::Mode { mode } => {
                    head.modes.push(ModeEntry {
                        proxy: mode,
                        width: 0,
                        height: 0,
                        refresh: 0,
                        preferred: false,
                        finished: false,
                    });
                }
                zwlr_output_head_v1::Event::Enabled { enabled } => {
                    head.enabled = enabled != 0;
                }
                zwlr_output_head_v1::Event::CurrentMode { mode } => {
                    head.current_mode_proxy = Some(mode);
                }
                zwlr_output_head_v1::Event::Position { x, y } => {
                    head.position_x = x;
                    head.position_y = y;
                }
                zwlr_output_head_v1::Event::Transform { transform } => {
                    head.transform = match transform {
                        WEnum::Value(t) => t as i32,
                        WEnum::Unknown(v) => v as i32,
                    };
                }
                zwlr_output_head_v1::Event::Scale { scale } => {
                    head.scale = scale;
                }
                zwlr_output_head_v1::Event::Make { make } => {
                    head.make = make;
                }
                zwlr_output_head_v1::Event::Model { model } => {
                    head.model = model;
                }
                zwlr_output_head_v1::Event::SerialNumber { serial_number } => {
                    head.serial_number = serial_number;
                }
                zwlr_output_head_v1::Event::Finished => {
                    head.finished = true;
                }
                _ => {}
            }
        }
    }

    event_created_child!(OutputManagementState, zwlr_output_head_v1::ZwlrOutputHeadV1, [
        // Opcode 3 = mode event creates a new mode object.
        3 => (zwlr_output_mode_v1::ZwlrOutputModeV1, ()),
    ]);
}

impl Dispatch<zwlr_output_mode_v1::ZwlrOutputModeV1, ()> for OutputManagementState {
    fn event(
        state: &mut Self,
        proxy: &zwlr_output_mode_v1::ZwlrOutputModeV1,
        event: zwlr_output_mode_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Find the mode entry across all heads.
        for head in &mut state.heads {
            if let Some(mode) = head.modes.iter_mut().find(|m| m.proxy == *proxy) {
                match event {
                    zwlr_output_mode_v1::Event::Size { width, height } => {
                        mode.width = width;
                        mode.height = height;
                    }
                    zwlr_output_mode_v1::Event::Refresh { refresh } => {
                        mode.refresh = refresh;
                    }
                    zwlr_output_mode_v1::Event::Preferred => {
                        mode.preferred = true;
                    }
                    zwlr_output_mode_v1::Event::Finished => {
                        mode.finished = true;
                    }
                    _ => {}
                }
                return;
            }
        }
    }
}

impl Dispatch<zwlr_output_configuration_v1::ZwlrOutputConfigurationV1, ()>
    for OutputManagementState
{
    fn event(
        state: &mut Self,
        _proxy: &zwlr_output_configuration_v1::ZwlrOutputConfigurationV1,
        event: zwlr_output_configuration_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_output_configuration_v1::Event::Succeeded => {
                state.config_result = Some(ConfigResult::Succeeded);
            }
            zwlr_output_configuration_v1::Event::Failed => {
                state.config_result = Some(ConfigResult::Failed);
            }
            zwlr_output_configuration_v1::Event::Cancelled => {
                state.config_result = Some(ConfigResult::Cancelled);
            }
            _ => {}
        }
    }
}

impl Dispatch<zwlr_output_configuration_head_v1::ZwlrOutputConfigurationHeadV1, ()>
    for OutputManagementState
{
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_output_configuration_head_v1::ZwlrOutputConfigurationHeadV1,
        _event: zwlr_output_configuration_head_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Configuration head has no events.
    }
}
