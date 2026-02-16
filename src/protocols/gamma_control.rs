//! wlr-gamma-control: adjust gamma tables for outputs.
//!
//! Provides [`GammaControlClient`] for setting gamma/brightness on
//! outputs via the `zwlr_gamma_control_manager_v1` protocol.

use std::fmt;
use std::io::Write;
use std::os::fd::AsFd;

use wayland_client::protocol::{wl_output, wl_registry};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols_wlr::gamma_control::v1::client::{
    zwlr_gamma_control_manager_v1, zwlr_gamma_control_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// A gamma lookup table with red, green, and blue ramp channels.
///
/// Each channel contains `size` entries of `u16` values representing
/// the gamma ramp. The table can be serialized to bytes for submission
/// to the compositor via the wlr-gamma-control protocol.
#[derive(Debug, Clone)]
pub struct GammaTable {
    /// Number of entries per channel.
    pub size: u32,
    /// Red channel ramp values.
    pub red: Vec<u16>,
    /// Green channel ramp values.
    pub green: Vec<u16>,
    /// Blue channel ramp values.
    pub blue: Vec<u16>,
}

impl GammaTable {
    /// Create an identity gamma table (linear ramp from 0 to `u16::MAX`).
    #[must_use]
    pub fn identity(size: u32) -> Self {
        let ramp: Vec<u16> = (0..size)
            .map(|i| {
                if size <= 1 {
                    u16::MAX
                } else {
                    ((i as u64 * u16::MAX as u64) / (size as u64 - 1)) as u16
                }
            })
            .collect();
        Self {
            size,
            red: ramp.clone(),
            green: ramp.clone(),
            blue: ramp,
        }
    }

    /// Serialize the gamma table to bytes in native-endian format.
    ///
    /// The layout is: all red values, then all green values, then all blue values,
    /// each as native-endian `u16`.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.size as usize * 6);
        for &v in &self.red {
            buf.extend_from_slice(&v.to_ne_bytes());
        }
        for &v in &self.green {
            buf.extend_from_slice(&v.to_ne_bytes());
        }
        for &v in &self.blue {
            buf.extend_from_slice(&v.to_ne_bytes());
        }
        buf
    }

    /// Create a gamma table with uniform brightness adjustment.
    ///
    /// `brightness` is clamped to `[0.0, 1.0]`, where `1.0` is identity
    /// and `0.0` is full black.
    #[must_use]
    pub fn with_brightness(size: u32, brightness: f64) -> Self {
        let mut table = Self::identity(size);
        let factor = brightness.clamp(0.0, 1.0);
        for v in table
            .red
            .iter_mut()
            .chain(table.green.iter_mut())
            .chain(table.blue.iter_mut())
        {
            *v = (*v as f64 * factor) as u16;
        }
        table
    }

    /// Create a gamma table with gamma correction applied.
    ///
    /// `gamma` values greater than 1.0 darken midtones, values less than
    /// 1.0 brighten midtones. A value of 1.0 produces an identity table.
    #[must_use]
    pub fn with_gamma(size: u32, gamma: f64) -> Self {
        let mut table = Self::identity(size);
        for v in table
            .red
            .iter_mut()
            .chain(table.green.iter_mut())
            .chain(table.blue.iter_mut())
        {
            let normalized = *v as f64 / u16::MAX as f64;
            *v = (normalized.powf(gamma) * u16::MAX as f64) as u16;
        }
        table
    }
}

/// An output with its gamma control state.
#[derive(Debug, Clone)]
pub struct GammaControlEntry {
    /// Output name (e.g. "DP-1").
    pub name: String,
    /// Number of gamma ramp entries supported by this output.
    pub gamma_size: u32,
    /// Whether the gamma control for this output has failed.
    pub failed: bool,
}

/// Client for the `zwlr_gamma_control_manager_v1` protocol.
///
/// Adjusts gamma tables (brightness, color temperature) for outputs.
///
/// # Example
///
/// ```no_run
/// use hypr_sdk::protocols::connection::WaylandConnection;
/// use hypr_sdk::protocols::gamma_control::{GammaControlClient, GammaTable};
///
/// let wl = WaylandConnection::connect().unwrap();
/// let mut client = GammaControlClient::connect(&wl).unwrap();
///
/// for entry in client.outputs() {
///     println!("{}: {} gamma entries", entry.name, entry.gamma_size);
/// }
///
/// // Set 80% brightness on first output
/// if let Some(entry) = client.outputs().first().cloned() {
///     let table = GammaTable::with_brightness(entry.gamma_size, 0.8);
///     client.set_gamma(&entry.name, &table).unwrap();
/// }
/// ```
pub struct GammaControlClient {
    state: GammaControlState,
    event_queue: EventQueue<GammaControlState>,
}

impl GammaControlClient {
    /// Connect to the gamma control manager.
    ///
    /// Binds `zwlr_gamma_control_manager_v1`, discovers outputs, and
    /// queries their gamma table sizes.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `zwlr_gamma_control_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("zwlr_gamma_control_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "zwlr_gamma_control_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<GammaControlState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = GammaControlState::new();

        // Registry roundtrip: bind outputs + manager.
        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Second roundtrip: receive output name events.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        let manager = state.manager.as_ref().ok_or_else(|| {
            HyprError::ProtocolNotSupported("zwlr_gamma_control_manager_v1".into())
        })?;

        // Create gamma control per output.
        for output_entry in &state.outputs {
            let control = manager.get_gamma_control(&output_entry.output, &qh, output_entry.name);
            state.gammas.push(GammaEntry {
                output_global_name: output_entry.name,
                gamma_size: None,
                failed: false,
                control,
            });
        }

        // Roundtrip to receive gamma_size events.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self { state, event_queue })
    }

    /// All outputs and their gamma control state.
    #[must_use]
    pub fn outputs(&self) -> Vec<GammaControlEntry> {
        self.state
            .outputs
            .iter()
            .filter_map(|out| {
                let gamma = self
                    .state
                    .gammas
                    .iter()
                    .find(|g| g.output_global_name == out.name)?;
                Some(GammaControlEntry {
                    name: out.output_name.clone().unwrap_or_default(),
                    gamma_size: gamma.gamma_size.unwrap_or(0),
                    failed: gamma.failed,
                })
            })
            .collect()
    }

    /// Set the gamma table for an output.
    ///
    /// The table's `size` must match the output's `gamma_size`. The table
    /// is written to a temporary file descriptor and sent to the compositor.
    ///
    /// # Errors
    ///
    /// Returns an error if the output is not found, gamma control has
    /// failed, or the fd operation fails.
    pub fn set_gamma(&mut self, output_name: &str, table: &GammaTable) -> HyprResult<()> {
        let output_entry = self
            .state
            .outputs
            .iter()
            .find(|o| o.output_name.as_deref() == Some(output_name))
            .ok_or_else(|| {
                HyprError::WaylandDispatch(format!("output not found: {output_name}"))
            })?;

        let gamma = self
            .state
            .gammas
            .iter()
            .find(|g| g.output_global_name == output_entry.name)
            .ok_or_else(|| {
                HyprError::WaylandDispatch(format!("no gamma control for: {output_name}"))
            })?;

        if gamma.failed {
            return Err(HyprError::WaylandDispatch(format!(
                "gamma control failed for: {output_name}"
            )));
        }

        // Write gamma table to a temporary file.
        let bytes = table.to_bytes();
        let mut tmpfile = tempfile().map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
        tmpfile
            .write_all(&bytes)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Send the fd to the compositor.
        gamma.control.set_gamma(tmpfile.as_fd());

        // Roundtrip to process.
        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Re-dispatch events to update gamma control state.
    pub fn refresh(&mut self) -> HyprResult<()> {
        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
        Ok(())
    }
}

impl fmt::Debug for GammaControlClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GammaControlClient")
            .field("outputs", &self.state.outputs.len())
            .finish()
    }
}

// ── Temporary file helper ────────────────────────────────────────────

/// Create a temporary file using memfd_create or a temp directory.
fn tempfile() -> std::io::Result<std::fs::File> {
    // Try to use a temp file in $XDG_RUNTIME_DIR or /tmp.
    let path = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let path =
        std::path::PathBuf::from(path).join(format!("hypr-sdk-gamma-{}", std::process::id()));
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    // Delete file immediately — fd stays valid.
    let _ = std::fs::remove_file(&path);
    Ok(file)
}

// ── Internal state ───────────────────────────────────────────────────

struct GammaControlState {
    manager: Option<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1>,
    outputs: Vec<OutputEntry>,
    gammas: Vec<GammaEntry>,
}

struct OutputEntry {
    name: u32,
    output: wl_output::WlOutput,
    output_name: Option<String>,
}

struct GammaEntry {
    output_global_name: u32,
    gamma_size: Option<u32>,
    failed: bool,
    control: zwlr_gamma_control_v1::ZwlrGammaControlV1,
}

impl GammaControlState {
    fn new() -> Self {
        Self {
            manager: None,
            outputs: Vec::new(),
            gammas: Vec::new(),
        }
    }
}

// ── Dispatch implementations ─────────────────────────────────────────

impl Dispatch<wl_registry::WlRegistry, ()> for GammaControlState {
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
                "zwlr_gamma_control_manager_v1" if state.manager.is_none() => {
                    let mgr = registry
                        .bind::<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1, (), Self>(
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

impl Dispatch<wl_output::WlOutput, u32> for GammaControlState {
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

impl Dispatch<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1, ()> for GammaControlState {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1,
        _event: zwlr_gamma_control_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Manager has no events.
    }
}

impl Dispatch<zwlr_gamma_control_v1::ZwlrGammaControlV1, u32> for GammaControlState {
    fn event(
        state: &mut Self,
        _proxy: &zwlr_gamma_control_v1::ZwlrGammaControlV1,
        event: zwlr_gamma_control_v1::Event,
        data: &u32,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let global_name = *data;
        if let Some(entry) = state
            .gammas
            .iter_mut()
            .find(|g| g.output_global_name == global_name)
        {
            match event {
                zwlr_gamma_control_v1::Event::GammaSize { size } => {
                    entry.gamma_size = Some(size);
                }
                zwlr_gamma_control_v1::Event::Failed => {
                    entry.failed = true;
                }
                _ => {}
            }
        }
    }
}
