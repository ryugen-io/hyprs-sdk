use std::fmt;

use wayland_client::{EventQueue, QueueHandle};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

use super::dispatch::LayerShellState;
use super::types::{LayerSurfaceConfig, LayerSurfaceHandle};

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

        // Output name events arrive asynchronously after binding; a second roundtrip
        // collects them so we can resolve output names in create_surface.
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

        // Layer surfaces can target a specific output; resolve name to proxy now
        // so the compositor places the surface on the correct monitor.
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

        // Layer shell requires a wl_surface to attach the layer role to.
        let surface = compositor.create_surface(qh, ());

        // Binding a layer surface assigns the layer role; the compositor uses
        // layer + namespace to determine stacking order and exclusion zones.
        let layer_surface = layer_shell.get_layer_surface(
            &surface,
            output,
            config.layer.to_protocol(),
            config.namespace.clone(),
            qh,
            (),
        );

        // All geometry properties must be set before the first commit so the
        // compositor can calculate the correct configure dimensions.
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

        // The protocol requires an initial bufferless commit to signal the compositor
        // that setup is complete; it responds with a configure event containing the
        // negotiated dimensions.
        surface.commit();

        // Clear stale configure state from any previous surface creation so we
        // only read the configure event for this new surface.
        state.configure_width = None;
        state.configure_height = None;
        state.configure_serial = None;
        state.surface_closed = false;

        // The configure event arrives asynchronously after the initial commit;
        // roundtrip ensures it is dispatched before we read the negotiated size.
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // The protocol requires acknowledging each configure before attaching
        // buffers; without ack the compositor will not display the surface.
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
