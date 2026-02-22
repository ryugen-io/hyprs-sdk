//! Output management client implementation.

use std::fmt;

use wayland_client::{EventQueue, QueueHandle};
use wayland_protocols_wlr::output_management::v1::client::zwlr_output_configuration_v1;

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

use super::dispatch::{ConfigResult, OutputManagementState, transform_from_i32};
use super::types::*;

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

        // First roundtrip: Wayland globals are advertised asynchronously; roundtrip
        // ensures the output manager global is bound before creating child objects.
        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "zwlr_output_manager_v1".into(),
            ));
        }

        // Second roundtrip: the manager emits head events for each connected output
        // asynchronously after binding; roundtrip collects all head proxies.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Third roundtrip: head property events (name, description, modes) and mode
        // child objects arrive after the heads are created; roundtrip collects them.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Fourth roundtrip: mode property events (size, refresh, preferred) arrive on
        // the mode objects created in the previous roundtrip; this collects them all
        // so heads() returns complete mode information.
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

                // The protocol identifies the current mode by proxy object, but our
                // public API uses an index into the filtered modes vec for ergonomics.
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
        let config = build_config(state, &qh, entries)?;
        config.apply();
        await_config_result(state, event_queue)
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
        let config = build_config(state, &qh, entries)?;
        config.test();
        await_config_result(state, event_queue)
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

// ── Helpers ──────────────────────────────────────────────────────────
// Separated from the client impl to keep the public API surface clean;
// these functions encapsulate the protocol's configuration lifecycle.

/// Build a configuration object with all entries applied.
fn build_config(
    state: &OutputManagementState,
    qh: &QueueHandle<OutputManagementState>,
    entries: &[OutputConfigEntry],
) -> HyprResult<zwlr_output_configuration_v1::ZwlrOutputConfigurationV1> {
    let manager = state
        .manager
        .as_ref()
        .ok_or_else(|| HyprError::ProtocolNotSupported("zwlr_output_manager_v1".into()))?;

    let config = manager.create_configuration(state.serial, qh, ());

    for entry in entries {
        let head = state
            .heads
            .iter()
            .find(|h| h.name == entry.name && !h.finished)
            .ok_or_else(|| {
                HyprError::WaylandDispatch(format!("output head not found: {}", entry.name))
            })?;

        if entry.enabled {
            let config_head = config.enable_head(&head.proxy, qh, ());

            if let Some(ref mode) = entry.mode {
                if let Some(mode_entry) = head.modes.iter().find(|m| {
                    m.width == mode.width && m.height == mode.height && m.refresh == mode.refresh
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

    Ok(config)
}

/// Wait for the compositor to respond with a config result.
fn await_config_result(
    state: &mut OutputManagementState,
    event_queue: &mut EventQueue<OutputManagementState>,
) -> HyprResult<()> {
    state.config_result = None;
    event_queue
        .roundtrip(state)
        .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

    // The compositor may need an extra roundtrip to process the configuration
    // and send the succeeded/failed/cancelled response.
    if state.config_result.is_none() {
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
    }

    match state.config_result {
        Some(ConfigResult::Succeeded) => Ok(()),
        Some(ConfigResult::Failed) => Err(HyprError::WaylandDispatch(
            "output configuration rejected by compositor".into(),
        )),
        Some(ConfigResult::Cancelled) => Err(HyprError::WaylandDispatch(
            "output configuration cancelled (outdated serial)".into(),
        )),
        None => Err(HyprError::WaylandDispatch(
            "no configuration result received".into(),
        )),
    }
}
