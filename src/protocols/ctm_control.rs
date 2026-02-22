//! hyprland-ctm-control: color transform matrix for outputs.
//!
//! Provides [`CtmControlClient`] for applying 3x3 color correction matrices
//! to outputs via the `hyprland_ctm_control_manager_v1` protocol.
//!
//! # Example
//!
//! ```no_run
//! use hypr_sdk::protocols::connection::WaylandConnection;
//! use hypr_sdk::protocols::ctm_control::{CtmControlClient, ColorTransformMatrix};
//!
//! let wl = WaylandConnection::connect().unwrap();
//! let mut client = CtmControlClient::connect(&wl).unwrap();
//!
//! // Apply a red-tinted night mode to the first output
//! let matrix = ColorTransformMatrix::scale(1.0, 0.7, 0.5);
//! let output_name = &client.outputs()[0];
//! client.set_ctm(output_name, &matrix).unwrap();
//! client.commit().unwrap();
//! ```

use std::fmt;

use wayland_client::protocol::{wl_output, wl_registry};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols_hyprland::ctm_control::v1::client::hyprland_ctm_control_manager_v1;

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// A 3x3 color transform matrix.
///
/// Applied to output pixel colors as: `[R', G', B'] = matrix * [R, G, B]`.
/// Row-major order: `[r0c0, r0c1, r0c2, r1c0, r1c1, r1c2, r2c0, r2c1, r2c2]`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorTransformMatrix {
    /// 9 matrix elements in row-major order.
    pub elements: [f64; 9],
}

impl ColorTransformMatrix {
    /// Identity matrix (no color transformation).
    pub const IDENTITY: Self = Self {
        elements: [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    };

    /// Create a matrix that scales RGB channels independently.
    #[must_use]
    pub fn scale(r: f64, g: f64, b: f64) -> Self {
        Self {
            elements: [r, 0.0, 0.0, 0.0, g, 0.0, 0.0, 0.0, b],
        }
    }

    /// Create a grayscale conversion matrix using standard luminance weights.
    #[must_use]
    pub fn grayscale() -> Self {
        Self {
            elements: [
                0.2126, 0.7152, 0.0722, 0.2126, 0.7152, 0.0722, 0.2126, 0.7152, 0.0722,
            ],
        }
    }
}

impl Default for ColorTransformMatrix {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Client for the `hyprland_ctm_control_manager_v1` protocol.
///
/// Applies color transform matrices to compositor outputs. The matrix is
/// applied to pixel colors before display.
pub struct CtmControlClient {
    state: CtmControlState,
    event_queue: EventQueue<CtmControlState>,
}

impl CtmControlClient {
    /// Connect to the CTM control manager.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `hyprland_ctm_control_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("hyprland_ctm_control_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "hyprland_ctm_control_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<CtmControlState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = CtmControlState::new();

        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "hyprland_ctm_control_manager_v1".into(),
            ));
        }

        // Output name events arrive on the wl_output objects bound in the previous
        // roundtrip; a second roundtrip collects them so we can identify outputs by name.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self { state, event_queue })
    }

    /// All output names available for CTM control.
    #[must_use]
    pub fn outputs(&self) -> Vec<String> {
        self.state
            .outputs
            .iter()
            .filter_map(|o| o.name.clone())
            .collect()
    }

    /// Set the color transform matrix for an output.
    ///
    /// Changes are staged until [`commit`](Self::commit) is called.
    ///
    /// # Errors
    ///
    /// Returns an error if the output is not found.
    pub fn set_ctm(&self, output_name: &str, matrix: &ColorTransformMatrix) -> HyprResult<()> {
        let output_entry = self
            .state
            .outputs
            .iter()
            .find(|o| o.name.as_deref() == Some(output_name))
            .ok_or_else(|| {
                HyprError::WaylandDispatch(format!("output not found: {output_name}"))
            })?;

        let manager = self.state.manager.as_ref().ok_or_else(|| {
            HyprError::ProtocolNotSupported("hyprland_ctm_control_manager_v1".into())
        })?;

        let e = &matrix.elements;
        manager.set_ctm_for_output(
            &output_entry.output,
            e[0],
            e[1],
            e[2],
            e[3],
            e[4],
            e[5],
            e[6],
            e[7],
            e[8],
        );

        Ok(())
    }

    /// Commit all staged CTM changes.
    ///
    /// # Errors
    ///
    /// Returns an error if dispatch fails.
    pub fn commit(&mut self) -> HyprResult<()> {
        let manager = self.state.manager.as_ref().ok_or_else(|| {
            HyprError::ProtocolNotSupported("hyprland_ctm_control_manager_v1".into())
        })?;

        manager.commit();

        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }
}

impl fmt::Debug for CtmControlClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CtmControlClient")
            .field("outputs", &self.state.outputs.len())
            .finish()
    }
}

// ── Internal state ──────────────────────────────────────────────────────────
// Tracks the CTM manager and discovered outputs with their names.

struct CtmControlState {
    manager: Option<hyprland_ctm_control_manager_v1::HyprlandCtmControlManagerV1>,
    outputs: Vec<OutputEntry>,
}

struct OutputEntry {
    global_name: u32,
    output: wl_output::WlOutput,
    name: Option<String>,
}

impl CtmControlState {
    fn new() -> Self {
        Self {
            manager: None,
            outputs: Vec::new(),
        }
    }
}

// ── Dispatch implementations ────────────────────────────────────────────────
// wayland-client requires a Dispatch impl for every object type on the
// event queue.

impl Dispatch<wl_registry::WlRegistry, ()> for CtmControlState {
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
                        global_name: name,
                        output,
                        name: None,
                    });
                }
                "hyprland_ctm_control_manager_v1" if state.manager.is_none() => {
                    let mgr = registry
                        .bind::<hyprland_ctm_control_manager_v1::HyprlandCtmControlManagerV1, (), Self>(
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

impl Dispatch<wl_output::WlOutput, u32> for CtmControlState {
    fn event(
        state: &mut Self,
        _proxy: &wl_output::WlOutput,
        event: wl_output::Event,
        data: &u32,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let global_name = *data;
        if let Some(entry) = state
            .outputs
            .iter_mut()
            .find(|o| o.global_name == global_name)
            && let wl_output::Event::Name { name } = event
        {
            entry.name = Some(name);
        }
    }
}

impl Dispatch<hyprland_ctm_control_manager_v1::HyprlandCtmControlManagerV1, ()>
    for CtmControlState
{
    fn event(
        _state: &mut Self,
        _proxy: &hyprland_ctm_control_manager_v1::HyprlandCtmControlManagerV1,
        _event: hyprland_ctm_control_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client; CTM manager v1 is request-only.
    }
}
