//! wlr-output-power-management: DPMS control for outputs.
//!
//! Provides [`OutputPowerClient`] for controlling output power states
//! (on/off, i.e. DPMS) via the `zwlr_output_power_manager_v1` protocol.

use std::fmt;

use wayland_client::protocol::{wl_output, wl_registry};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, WEnum};
use wayland_protocols_wlr::output_power_management::v1::client::{
    zwlr_output_power_manager_v1, zwlr_output_power_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// Power mode for an output (DPMS state).
///
/// Values match the `zwlr_output_power_v1::mode` protocol enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PowerMode {
    /// Output is powered off (DPMS standby/suspend/off).
    Off = 0,
    /// Output is powered on and displaying content.
    On = 1,
}

impl PowerMode {
    /// Convert a raw protocol value to a `PowerMode`.
    #[must_use]
    pub fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Off),
            1 => Some(Self::On),
            _ => None,
        }
    }

    fn from_protocol(mode: WEnum<zwlr_output_power_v1::Mode>) -> Option<Self> {
        match mode {
            WEnum::Value(zwlr_output_power_v1::Mode::Off) => Some(Self::Off),
            WEnum::Value(zwlr_output_power_v1::Mode::On) => Some(Self::On),
            _ => None,
        }
    }
}

impl fmt::Display for PowerMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::On => write!(f, "on"),
            Self::Off => write!(f, "off"),
        }
    }
}

/// An output with its current power state.
#[derive(Debug, Clone)]
pub struct OutputPowerEntry {
    /// Output name (e.g. "DP-1", "HDMI-A-1").
    pub name: String,
    /// Current power mode.
    pub mode: PowerMode,
    /// Whether the power control for this output has failed.
    pub failed: bool,
}

/// Client for the `zwlr_output_power_manager_v1` protocol.
///
/// Controls DPMS (power on/off) for compositor outputs.
///
/// # Example
///
/// ```no_run
/// use hypr_sdk::protocols::connection::WaylandConnection;
/// use hypr_sdk::protocols::output_power::{OutputPowerClient, PowerMode};
///
/// let wl = WaylandConnection::connect().unwrap();
/// let mut client = OutputPowerClient::connect(&wl).unwrap();
///
/// for output in client.outputs() {
///     println!("{}: {}", output.name, output.mode);
/// }
/// ```
pub struct OutputPowerClient {
    state: OutputPowerState,
    event_queue: EventQueue<OutputPowerState>,
}

impl OutputPowerClient {
    /// Connect to the output power manager.
    ///
    /// Binds the `zwlr_output_power_manager_v1` global, discovers all
    /// outputs, and queries their current power state.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `zwlr_output_power_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        // Fail fast before spending time on roundtrips if the compositor lacks this protocol.
        if !wl.has_protocol("zwlr_output_power_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "zwlr_output_power_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<OutputPowerState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = OutputPowerState::new();

        // Wayland events arrive asynchronously; roundtrip ensures all outputs and the
        // manager global are bound before we use them.
        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Output name events arrive on the wl_output objects bound in the previous
        // roundtrip; a second roundtrip collects them so we can identify outputs by name.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        let manager = state.manager.as_ref().ok_or_else(|| {
            HyprError::ProtocolNotSupported("zwlr_output_power_manager_v1".into())
        })?;

        // Each output needs its own power control handle to query/set its DPMS state
        // independently.
        for output_entry in &state.outputs {
            let control = manager.get_output_power(&output_entry.output, &qh, output_entry.name);
            state.powers.push(PowerControlEntry {
                output_global_name: output_entry.name,
                mode: None,
                failed: false,
                _control: control,
            });
        }

        // The compositor sends the initial power mode asynchronously after the control
        // object is created; roundtrip ensures we have it before returning.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self { state, event_queue })
    }

    /// All outputs and their current power states.
    #[must_use]
    pub fn outputs(&self) -> Vec<OutputPowerEntry> {
        self.state
            .outputs
            .iter()
            .filter_map(|out| {
                let power = self
                    .state
                    .powers
                    .iter()
                    .find(|p| p.output_global_name == out.name)?;
                Some(OutputPowerEntry {
                    name: out.output_name.clone().unwrap_or_default(),
                    mode: power.mode.unwrap_or(PowerMode::On),
                    failed: power.failed,
                })
            })
            .collect()
    }

    /// Set the power mode for an output by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the output is not found or if dispatching fails.
    pub fn set_mode(&mut self, output_name: &str, mode: PowerMode) -> HyprResult<()> {
        let output_entry = self
            .state
            .outputs
            .iter()
            .find(|o| o.output_name.as_deref() == Some(output_name))
            .ok_or_else(|| {
                HyprError::WaylandDispatch(format!("output not found: {output_name}"))
            })?;

        let power = self
            .state
            .powers
            .iter()
            .find(|p| p.output_global_name == output_entry.name)
            .ok_or_else(|| {
                HyprError::WaylandDispatch(format!("no power control for: {output_name}"))
            })?;

        if power.failed {
            return Err(HyprError::WaylandDispatch(format!(
                "power control failed for: {output_name}"
            )));
        }

        let protocol_mode = match mode {
            PowerMode::Off => zwlr_output_power_v1::Mode::Off,
            PowerMode::On => zwlr_output_power_v1::Mode::On,
        };
        power._control.set_mode(protocol_mode);

        // Roundtrip to ensure the compositor has processed the mode change and any
        // resulting mode/failed events have been delivered.
        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Re-dispatch events to update output power states.
    ///
    /// # Errors
    ///
    /// Returns an error if event dispatch fails.
    pub fn refresh(&mut self) -> HyprResult<()> {
        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
        Ok(())
    }
}

impl fmt::Debug for OutputPowerClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OutputPowerClient")
            .field("outputs", &self.state.outputs.len())
            .finish()
    }
}

// ── Internal state ──────────────────────────────────────────────────────────
// Tracks outputs, their names, and per-output power control handles.

struct OutputPowerState {
    manager: Option<zwlr_output_power_manager_v1::ZwlrOutputPowerManagerV1>,
    outputs: Vec<OutputEntry>,
    powers: Vec<PowerControlEntry>,
}

struct OutputEntry {
    name: u32,
    output: wl_output::WlOutput,
    output_name: Option<String>,
}

struct PowerControlEntry {
    output_global_name: u32,
    mode: Option<PowerMode>,
    failed: bool,
    _control: zwlr_output_power_v1::ZwlrOutputPowerV1,
}

impl OutputPowerState {
    fn new() -> Self {
        Self {
            manager: None,
            outputs: Vec::new(),
            powers: Vec::new(),
        }
    }
}

// ── Dispatch implementations ────────────────────────────────────────────────
// wayland-client requires a Dispatch impl for every object type on the
// event queue.

impl Dispatch<wl_registry::WlRegistry, ()> for OutputPowerState {
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
        {
            match interface.as_str() {
                "wl_output" => {
                    let output = registry.bind::<wl_output::WlOutput, u32, Self>(
                        name,
                        version.min(4),
                        qh,
                        name,
                    );
                    state.outputs.push(OutputEntry {
                        name,
                        output,
                        output_name: None,
                    });
                }
                "zwlr_output_power_manager_v1" if state.manager.is_none() => {
                    let mgr = registry
                        .bind::<zwlr_output_power_manager_v1::ZwlrOutputPowerManagerV1, (), Self>(
                            name,
                            version.min(1),
                            qh,
                            (),
                        );
                    state.manager = Some(mgr);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<wl_output::WlOutput, u32> for OutputPowerState {
    fn event(
        state: &mut Self,
        _proxy: &wl_output::WlOutput,
        event: wl_output::Event,
        data: &u32,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let global_name = *data;
        if let Some(entry) = state.outputs.iter_mut().find(|o| o.name == global_name)
            && let wl_output::Event::Name { name } = event
        {
            entry.output_name = Some(name);
        }
    }
}

impl Dispatch<zwlr_output_power_manager_v1::ZwlrOutputPowerManagerV1, ()> for OutputPowerState {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_output_power_manager_v1::ZwlrOutputPowerManagerV1,
        _event: zwlr_output_power_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client; this interface is request-only.
    }
}

impl Dispatch<zwlr_output_power_v1::ZwlrOutputPowerV1, u32> for OutputPowerState {
    fn event(
        state: &mut Self,
        _proxy: &zwlr_output_power_v1::ZwlrOutputPowerV1,
        event: zwlr_output_power_v1::Event,
        data: &u32,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let global_name = *data;
        if let Some(entry) = state
            .powers
            .iter_mut()
            .find(|p| p.output_global_name == global_name)
        {
            match event {
                zwlr_output_power_v1::Event::Mode { mode } => {
                    entry.mode = PowerMode::from_protocol(mode);
                }
                zwlr_output_power_v1::Event::Failed => {
                    entry.failed = true;
                }
                _ => {}
            }
        }
    }
}
