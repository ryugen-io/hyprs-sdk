//! wlr-layer-shell: create surfaces that are layers of the desktop.
//!
//! Provides [`LayerShellClient`] for creating layer surfaces (panels,
//! taskbars, overlays, lock screens) via the `zwlr_layer_shell_v1` protocol.
//!
//! The client handles surface creation and the configure/ack lifecycle.
//! To display content, attach a buffer to the returned `wl_surface` handle
//! after the initial configure event.
//!
//! # Example
//!
//! ```no_run
//! use hypr_sdk::protocols::connection::WaylandConnection;
//! use hypr_sdk::protocols::layer_shell::{
//!     LayerShellClient, LayerSurfaceConfig, ShellLayer, Anchor,
//! };
//!
//! let wl = WaylandConnection::connect().unwrap();
//! let mut client = LayerShellClient::connect(&wl).unwrap();
//!
//! let config = LayerSurfaceConfig {
//!     layer: ShellLayer::Top,
//!     namespace: "my-panel".into(),
//!     width: 0,
//!     height: 48,
//!     anchor: Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
//!     exclusive_zone: 48,
//!     ..Default::default()
//! };
//!
//! let surface = client.create_surface(&config, None).unwrap();
//! println!("Configured size: {}x{}", surface.width, surface.height);
//! ```

use std::fmt;
use std::ops::BitOr;

use wayland_client::protocol::{wl_compositor, wl_output, wl_registry, wl_surface};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// The layer a surface should be placed on.
///
/// Layers are rendered in order from background to overlay, with each
/// layer stacking above the previous one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ShellLayer {
    /// Behind all other surfaces (e.g., desktop wallpaper).
    Background = 0,
    /// Below normal windows (e.g., desktop widgets).
    Bottom = 1,
    /// Above normal windows (e.g., taskbars, panels).
    Top = 2,
    /// Above everything else (e.g., lock screens, notifications).
    Overlay = 3,
}

impl ShellLayer {
    /// Convert a raw protocol value to a `ShellLayer`.
    ///
    /// Returns `None` for unrecognized values.
    #[must_use]
    pub fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Background),
            1 => Some(Self::Bottom),
            2 => Some(Self::Top),
            3 => Some(Self::Overlay),
            _ => None,
        }
    }

    fn to_protocol(self) -> zwlr_layer_shell_v1::Layer {
        match self {
            Self::Background => zwlr_layer_shell_v1::Layer::Background,
            Self::Bottom => zwlr_layer_shell_v1::Layer::Bottom,
            Self::Top => zwlr_layer_shell_v1::Layer::Top,
            Self::Overlay => zwlr_layer_shell_v1::Layer::Overlay,
        }
    }
}

/// Edge anchoring bitmask for layer surfaces.
///
/// Anchoring a surface to opposite edges (e.g., left and right) causes
/// it to stretch to fill that dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Anchor(u32);

impl Anchor {
    /// Anchor to the top edge.
    pub const TOP: Self = Self(1);
    /// Anchor to the bottom edge.
    pub const BOTTOM: Self = Self(2);
    /// Anchor to the left edge.
    pub const LEFT: Self = Self(4);
    /// Anchor to the right edge.
    pub const RIGHT: Self = Self(8);

    /// Returns `true` if no edges are anchored.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all bits in `other` are set in `self`.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Returns `true` if this anchor describes a horizontal bar
    /// (anchored to left, right, and exactly one of top/bottom).
    #[must_use]
    pub fn is_horizontal_bar(self) -> bool {
        self.contains(Self::LEFT)
            && self.contains(Self::RIGHT)
            && (self.contains(Self::TOP) || self.contains(Self::BOTTOM))
            && !(self.contains(Self::TOP) && self.contains(Self::BOTTOM))
    }

    /// Returns `true` if this anchor describes a vertical bar
    /// (anchored to top, bottom, and exactly one of left/right).
    #[must_use]
    pub fn is_vertical_bar(self) -> bool {
        self.contains(Self::TOP)
            && self.contains(Self::BOTTOM)
            && (self.contains(Self::LEFT) || self.contains(Self::RIGHT))
            && !(self.contains(Self::LEFT) && self.contains(Self::RIGHT))
    }

    fn to_protocol(self) -> zwlr_layer_surface_v1::Anchor {
        zwlr_layer_surface_v1::Anchor::from_bits_truncate(self.0)
    }
}

impl BitOr for Anchor {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Keyboard interactivity mode for a layer surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u32)]
pub enum KeyboardInteractivity {
    /// The surface does not receive keyboard focus.
    #[default]
    None = 0,
    /// The surface receives exclusive keyboard focus when mapped.
    Exclusive = 1,
    /// The surface receives keyboard focus on demand (e.g., when clicked).
    OnDemand = 2,
}

impl KeyboardInteractivity {
    fn to_protocol(self) -> zwlr_layer_surface_v1::KeyboardInteractivity {
        match self {
            Self::None => zwlr_layer_surface_v1::KeyboardInteractivity::None,
            Self::Exclusive => zwlr_layer_surface_v1::KeyboardInteractivity::Exclusive,
            Self::OnDemand => zwlr_layer_surface_v1::KeyboardInteractivity::OnDemand,
        }
    }
}

/// Configuration for creating a layer surface.
#[derive(Debug, Clone)]
pub struct LayerSurfaceConfig {
    /// The layer to place the surface on.
    pub layer: ShellLayer,
    /// Application-defined namespace (e.g., "panel", "taskbar").
    pub namespace: String,
    /// Desired width (0 means the compositor decides).
    pub width: u32,
    /// Desired height (0 means the compositor decides).
    pub height: u32,
    /// Edge anchoring.
    pub anchor: Anchor,
    /// Size of the exclusive zone in pixels, or -1 for auto.
    pub exclusive_zone: i32,
    /// Keyboard interactivity mode.
    pub keyboard_interactivity: KeyboardInteractivity,
    /// Top margin in pixels.
    pub margin_top: i32,
    /// Bottom margin in pixels.
    pub margin_bottom: i32,
    /// Left margin in pixels.
    pub margin_left: i32,
    /// Right margin in pixels.
    pub margin_right: i32,
}

impl Default for LayerSurfaceConfig {
    fn default() -> Self {
        Self {
            layer: ShellLayer::Top,
            namespace: String::new(),
            width: 0,
            height: 0,
            anchor: Anchor::default(),
            exclusive_zone: 0,
            keyboard_interactivity: KeyboardInteractivity::None,
            margin_top: 0,
            margin_bottom: 0,
            margin_left: 0,
            margin_right: 0,
        }
    }
}

/// A created layer surface with its configure state.
#[derive(Debug)]
pub struct LayerSurfaceHandle {
    /// The underlying `wl_surface`. Attach a buffer and commit to display content.
    pub wl_surface: wl_surface::WlSurface,
    /// The layer surface protocol handle.
    pub layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    /// Configured width from the compositor (after first configure).
    pub width: u32,
    /// Configured height from the compositor (after first configure).
    pub height: u32,
    /// Whether the surface has been closed by the compositor.
    pub closed: bool,
}

/// Client for the `zwlr_layer_shell_v1` protocol.
///
/// Creates layer surfaces for panels, taskbars, overlays, and lock screens.
/// The configure/ack lifecycle is handled automatically. Users should
/// attach a buffer to the returned `wl_surface` to display content.
pub struct LayerShellClient {
    state: LayerShellState,
    event_queue: EventQueue<LayerShellState>,
    qh: QueueHandle<LayerShellState>,
}

impl LayerShellClient {
    /// Connect to the layer shell manager.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `zwlr_layer_shell_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("zwlr_layer_shell_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "zwlr_layer_shell_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<LayerShellState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = LayerShellState::new();

        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.layer_shell.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "zwlr_layer_shell_v1".into(),
            ));
        }

        if state.compositor.is_none() {
            return Err(HyprError::WaylandDispatch(
                "no wl_compositor available".into(),
            ));
        }

        // Second roundtrip to receive wl_output name events.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self {
            state,
            event_queue,
            qh,
        })
    }

    /// All output names.
    #[must_use]
    pub fn outputs(&self) -> Vec<String> {
        self.state
            .outputs
            .iter()
            .filter_map(|o| o.name.clone())
            .collect()
    }

    /// Create a layer surface.
    ///
    /// If `output_name` is `None`, the compositor chooses the output.
    /// The returned handle contains the configured size after the
    /// initial configure event. Attach a buffer to `handle.wl_surface`
    /// to display content.
    ///
    /// # Errors
    ///
    /// Returns an error if the manager or compositor is unavailable,
    /// or if the specified output is not found.
    pub fn create_surface(
        &mut self,
        config: &LayerSurfaceConfig,
        output_name: Option<&str>,
    ) -> HyprResult<LayerSurfaceHandle> {
        let Self {
            state,
            event_queue,
            qh,
        } = self;

        let layer_shell = state
            .layer_shell
            .as_ref()
            .ok_or_else(|| HyprError::ProtocolNotSupported("zwlr_layer_shell_v1".into()))?;
        let compositor = state
            .compositor
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no wl_compositor available".into()))?;

        // Resolve the output.
        let output = if let Some(name) = output_name {
            let entry = state
                .outputs
                .iter()
                .find(|o| o.name.as_deref() == Some(name))
                .ok_or_else(|| HyprError::WaylandDispatch(format!("output not found: {name}")))?;
            Some(&entry.output)
        } else {
            None
        };

        // Create the wl_surface.
        let surface = compositor.create_surface(qh, ());

        // Create the layer surface.
        let layer_surface = layer_shell.get_layer_surface(
            &surface,
            output,
            config.layer.to_protocol(),
            config.namespace.clone(),
            qh,
            (),
        );

        // Configure the layer surface properties.
        layer_surface.set_size(config.width, config.height);
        layer_surface.set_anchor(config.anchor.to_protocol());
        layer_surface.set_exclusive_zone(config.exclusive_zone);
        layer_surface.set_keyboard_interactivity(config.keyboard_interactivity.to_protocol());
        layer_surface.set_margin(
            config.margin_top,
            config.margin_right,
            config.margin_bottom,
            config.margin_left,
        );

        // Initial commit (no buffer) to trigger configure.
        surface.commit();

        // Reset configure state.
        state.configure_width = None;
        state.configure_height = None;
        state.configure_serial = None;
        state.surface_closed = false;

        // Roundtrip to receive the configure event.
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Ack the configure if received.
        if let Some(serial) = state.configure_serial.take() {
            layer_surface.ack_configure(serial);
        }

        let width = state.configure_width.unwrap_or(config.width);
        let height = state.configure_height.unwrap_or(config.height);

        Ok(LayerSurfaceHandle {
            wl_surface: surface,
            layer_surface,
            width,
            height,
            closed: state.surface_closed,
        })
    }
}

impl fmt::Debug for LayerShellClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LayerShellClient")
            .field("outputs", &self.state.outputs.len())
            .finish()
    }
}

// ── Internal state ───────────────────────────────────────────────────

struct LayerShellState {
    layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    compositor: Option<wl_compositor::WlCompositor>,
    outputs: Vec<OutputEntry>,
    configure_serial: Option<u32>,
    configure_width: Option<u32>,
    configure_height: Option<u32>,
    surface_closed: bool,
}

struct OutputEntry {
    global_name: u32,
    output: wl_output::WlOutput,
    name: Option<String>,
}

impl LayerShellState {
    fn new() -> Self {
        Self {
            layer_shell: None,
            compositor: None,
            outputs: Vec::new(),
            configure_serial: None,
            configure_width: None,
            configure_height: None,
            surface_closed: false,
        }
    }
}

// ── Dispatch implementations ─────────────────────────────────────────

impl Dispatch<wl_registry::WlRegistry, ()> for LayerShellState {
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
                "zwlr_layer_shell_v1" if state.layer_shell.is_none() => {
                    let shell = registry.bind::<zwlr_layer_shell_v1::ZwlrLayerShellV1, (), Self>(
                        name,
                        version.min(4),
                        qh,
                        (),
                    );
                    state.layer_shell = Some(shell);
                }
                "wl_compositor" if state.compositor.is_none() => {
                    let comp = registry.bind::<wl_compositor::WlCompositor, (), Self>(
                        name,
                        version.min(6),
                        qh,
                        (),
                    );
                    state.compositor = Some(comp);
                }
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
                _ => {}
            }
        }
    }
}

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for LayerShellState {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _event: zwlr_layer_shell_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Layer shell manager has no events.
    }
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for LayerShellState {
    fn event(
        state: &mut Self,
        _proxy: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_layer_surface_v1::Event::Configure {
                serial,
                width,
                height,
            } => {
                state.configure_serial = Some(serial);
                state.configure_width = Some(width);
                state.configure_height = Some(height);
            }
            zwlr_layer_surface_v1::Event::Closed => {
                state.surface_closed = true;
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for LayerShellState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_compositor::WlCompositor,
        _event: wl_compositor::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Compositor has no events.
    }
}

impl Dispatch<wl_surface::WlSurface, ()> for LayerShellState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_surface::WlSurface,
        _event: wl_surface::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Surface events not needed for layer shell setup.
    }
}

impl Dispatch<wl_output::WlOutput, u32> for LayerShellState {
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
