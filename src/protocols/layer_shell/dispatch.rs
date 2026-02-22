use wayland_client::protocol::{wl_compositor, wl_output, wl_registry, wl_surface};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::layer_shell::v1::client::{zwlr_layer_shell_v1, zwlr_layer_surface_v1};

pub(super) struct LayerShellState {
    pub(super) layer_shell: Option<zwlr_layer_shell_v1::ZwlrLayerShellV1>,
    pub(super) compositor: Option<wl_compositor::WlCompositor>,
    pub(super) outputs: Vec<OutputEntry>,
    pub(super) configure_serial: Option<u32>,
    pub(super) configure_width: Option<u32>,
    pub(super) configure_height: Option<u32>,
    pub(super) surface_closed: bool,
}

pub(super) struct OutputEntry {
    pub(super) global_name: u32,
    pub(super) output: wl_output::WlOutput,
    pub(super) name: Option<String>,
}

impl LayerShellState {
    pub(super) fn new() -> Self {
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
        // wayland-client requires a Dispatch impl for every object on the event queue;
        // the layer shell manager is request-only and never emits events.
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
        // wayland-client requires a Dispatch impl for every object on the event queue;
        // wl_compositor is request-only and never emits events.
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
        // wayland-client requires a Dispatch impl for every object on the event queue;
        // wl_surface events (enter/leave) are irrelevant for layer shell lifecycle.
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
