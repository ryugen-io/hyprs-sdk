//! Internal state and Wayland dispatch implementations for output management.

use wayland_client::protocol::{wl_output, wl_registry};
use wayland_client::{Connection, Dispatch, QueueHandle, WEnum, event_created_child};
use wayland_protocols_wlr::output_management::v1::client::{
    zwlr_output_configuration_head_v1, zwlr_output_configuration_v1, zwlr_output_head_v1,
    zwlr_output_manager_v1, zwlr_output_mode_v1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ConfigResult {
    Succeeded,
    Failed,
    Cancelled,
}

pub(super) struct OutputManagementState {
    pub manager: Option<zwlr_output_manager_v1::ZwlrOutputManagerV1>,
    pub heads: Vec<HeadEntry>,
    pub serial: u32,
    pub config_result: Option<ConfigResult>,
}

pub(super) struct HeadEntry {
    pub proxy: zwlr_output_head_v1::ZwlrOutputHeadV1,
    pub name: String,
    pub description: String,
    pub physical_width: i32,
    pub physical_height: i32,
    pub modes: Vec<ModeEntry>,
    pub enabled: bool,
    pub current_mode_proxy: Option<zwlr_output_mode_v1::ZwlrOutputModeV1>,
    pub position_x: i32,
    pub position_y: i32,
    pub scale: f64,
    pub transform: i32,
    pub make: String,
    pub model: String,
    pub serial_number: String,
    pub finished: bool,
}

pub(super) struct ModeEntry {
    pub proxy: zwlr_output_mode_v1::ZwlrOutputModeV1,
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
    pub preferred: bool,
    pub finished: bool,
}

impl OutputManagementState {
    pub fn new() -> Self {
        Self {
            manager: None,
            heads: Vec::new(),
            serial: 0,
            config_result: None,
        }
    }
}

pub(super) fn transform_from_i32(val: i32) -> wl_output::Transform {
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
