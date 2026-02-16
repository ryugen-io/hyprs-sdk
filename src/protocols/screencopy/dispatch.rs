//! Internal state and Wayland dispatch implementations for screencopy.

use wayland_client::protocol::{wl_buffer, wl_output, wl_registry, wl_shm, wl_shm_pool};
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1,
};

use super::types::PixelFormat;

pub(super) struct ScreencopyState {
    pub manager: Option<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1>,
    pub shm: Option<wl_shm::WlShm>,
    pub outputs: Vec<OutputEntry>,
    pub frame_proxy: Option<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1>,
    pub frame_buffer_info: Option<BufferInfo>,
    pub frame_flags: Option<u32>,
    pub frame_ready: bool,
    pub frame_failed: bool,
}

pub(super) struct OutputEntry {
    pub global_name: u32,
    pub output: wl_output::WlOutput,
    pub name: Option<String>,
}

#[derive(Clone, Copy)]
pub(super) struct BufferInfo {
    pub format: PixelFormat,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
}

impl ScreencopyState {
    pub fn new() -> Self {
        Self {
            manager: None,
            shm: None,
            outputs: Vec::new(),
            frame_proxy: None,
            frame_buffer_info: None,
            frame_flags: None,
            frame_ready: false,
            frame_failed: false,
        }
    }
}

// ── Dispatch implementations ─────────────────────────────────────────

impl Dispatch<wl_registry::WlRegistry, ()> for ScreencopyState {
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
                "zwlr_screencopy_manager_v1" if state.manager.is_none() => {
                    let mgr = registry
                        .bind::<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, (), Self>(
                            name,
                            version.min(3),
                            qh,
                            (),
                        );
                    state.manager = Some(mgr);
                }
                "wl_shm" if state.shm.is_none() => {
                    let shm =
                        registry.bind::<wl_shm::WlShm, (), Self>(name, version.min(1), qh, ());
                    state.shm = Some(shm);
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

impl Dispatch<wl_output::WlOutput, u32> for ScreencopyState {
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

impl Dispatch<wl_shm::WlShm, ()> for ScreencopyState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm::WlShm,
        _event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for ScreencopyState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for ScreencopyState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_buffer::WlBuffer,
        _event: wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1, ()> for ScreencopyState {
    fn event(
        _state: &mut Self,
        _proxy: &zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
        _event: zwlr_screencopy_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1, ()> for ScreencopyState {
    fn event(
        state: &mut Self,
        proxy: &zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1,
        event: zwlr_screencopy_frame_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_screencopy_frame_v1::Event::Buffer {
                format,
                width,
                height,
                stride,
            } => {
                if let Some(pf) = PixelFormat::from_wl_format(format) {
                    state.frame_buffer_info = Some(BufferInfo {
                        format: pf,
                        width,
                        height,
                        stride,
                    });
                }
                state.frame_proxy = Some(proxy.clone());
            }
            zwlr_screencopy_frame_v1::Event::Flags { flags } => {
                state.frame_flags = Some(flags.into());
            }
            zwlr_screencopy_frame_v1::Event::Ready { .. } => {
                state.frame_ready = true;
            }
            zwlr_screencopy_frame_v1::Event::Failed => {
                state.frame_failed = true;
            }
            _ => {}
        }
    }
}
