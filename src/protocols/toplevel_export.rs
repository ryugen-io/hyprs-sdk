//! hyprland-toplevel-export: capture individual window content.
//!
//! Provides [`ToplevelExportClient`] for capturing window content via the
//! `hyprland_toplevel_export_manager_v1` protocol. Like screencopy but
//! for individual windows instead of full outputs.
//!
//! # Example
//!
//! ```no_run
//! use hypr_sdk::protocols::connection::WaylandConnection;
//! use hypr_sdk::protocols::toplevel_export::ToplevelExportClient;
//!
//! let wl = WaylandConnection::connect().unwrap();
//! let mut client = ToplevelExportClient::connect(&wl).unwrap();
//!
//! // Capture a window by its toplevel handle number
//! let frame = client.capture_toplevel(42).unwrap();
//! if let Some(data) = frame.data {
//!     println!("Captured {}x{} frame", frame.format.width, frame.format.height);
//! }
//! ```

use std::fmt;
use std::io::{Read as IoRead, Seek};
use std::os::unix::io::AsFd;

use wayland_client::protocol::{wl_buffer, wl_registry, wl_shm, wl_shm_pool};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, WEnum};
use wayland_protocols_hyprland::toplevel_export::v1::client::{
    hyprland_toplevel_export_frame_v1, hyprland_toplevel_export_manager_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// Format info for a toplevel capture frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToplevelFrameFormat {
    /// DRM fourcc format code.
    pub format: u32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Row stride in bytes.
    pub stride: u32,
}

impl ToplevelFrameFormat {
    /// Total buffer size needed in bytes.
    #[must_use]
    pub fn buffer_size(&self) -> usize {
        self.stride as usize * self.height as usize
    }
}

/// Flags for a captured toplevel frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ToplevelFrameFlags(u32);

impl ToplevelFrameFlags {
    /// The frame is vertically inverted (Y axis flipped).
    pub const Y_INVERT: Self = Self(1);

    /// Returns `true` if no flags are set.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all flags in `other` are set in `self`.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

/// A captured toplevel frame.
#[derive(Debug, Clone)]
pub struct CapturedFrame {
    /// Format of the captured data.
    pub format: ToplevelFrameFormat,
    /// Frame flags.
    pub flags: ToplevelFrameFlags,
    /// Raw pixel data (if capture succeeded).
    pub data: Option<Vec<u8>>,
}

/// Client for the `hyprland_toplevel_export_manager_v1` protocol.
///
/// Captures individual window content into shared memory buffers.
pub struct ToplevelExportClient {
    state: ToplevelExportState,
    event_queue: EventQueue<ToplevelExportState>,
    qh: QueueHandle<ToplevelExportState>,
}

impl ToplevelExportClient {
    /// Connect to the toplevel export manager.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `hyprland_toplevel_export_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("hyprland_toplevel_export_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "hyprland_toplevel_export_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<ToplevelExportState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = ToplevelExportState::new();

        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "hyprland_toplevel_export_manager_v1".into(),
            ));
        }

        if state.shm.is_none() {
            return Err(HyprError::WaylandDispatch("no wl_shm available".into()));
        }

        Ok(Self {
            state,
            event_queue,
            qh,
        })
    }

    /// Capture a toplevel window by its numeric handle.
    ///
    /// The `toplevel_handle` is the numeric window identifier
    /// (e.g., from the foreign-toplevel protocol).
    ///
    /// # Errors
    ///
    /// Returns an error if the capture fails or dispatch errors occur.
    pub fn capture_toplevel(&mut self, toplevel_handle: u32) -> HyprResult<CapturedFrame> {
        let Self {
            state,
            event_queue,
            qh,
        } = self;

        let manager = state.manager.as_ref().ok_or_else(|| {
            HyprError::ProtocolNotSupported("hyprland_toplevel_export_manager_v1".into())
        })?;

        // Clear previous capture state so events from this capture don't mix
        // with leftovers from a prior one.
        state.frame_format = None;
        state.frame_flags = None;
        state.frame_ready = false;
        state.frame_failed = false;

        // overlay_cursor = 0 excludes the hardware cursor from the capture.
        let _frame = manager.capture_toplevel(0, toplevel_handle, qh, ());

        // The compositor sends buffer format events asynchronously after frame creation;
        // roundtrip ensures we know the required buffer dimensions before allocating.
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        let format = state.frame_format.clone().ok_or_else(|| {
            HyprError::WaylandDispatch("no buffer format received from toplevel export".into())
        })?;

        // The capture protocol requires a wl_buffer backed by shared memory; the
        // compositor writes pixel data into it after we call frame.copy().
        let shm = state
            .shm
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no wl_shm available".into()))?;

        let buf_size = format.buffer_size();
        let mut tmpfile = create_shm_file(buf_size)
            .map_err(|e| HyprError::WaylandDispatch(format!("failed to create shm file: {e}")))?;

        let pool = shm.create_pool(tmpfile.as_fd(), buf_size as i32, qh, ());
        let buffer = pool.create_buffer(
            0,
            format.width as i32,
            format.height as i32,
            format.stride as i32,
            wl_shm::Format::Argb8888,
            qh,
            (),
        );

        // Initiate the pixel copy into our shared-memory buffer; the compositor will
        // signal "ready" or "failed" asynchronously.
        let frame = state
            .frame_handle
            .as_ref()
            .ok_or_else(|| HyprError::WaylandDispatch("no frame handle available".into()))?;
        frame.copy(&buffer, 0);

        // Wait for the compositor to finish copying pixels and send the ready/failed event.
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.frame_failed {
            return Err(HyprError::WaylandDispatch(
                "toplevel export frame capture failed".into(),
            ));
        }

        // The compositor wrote pixel data into the shared-memory fd; read it back into
        // a Vec so the caller owns a standalone copy.
        let data =
            if state.frame_ready {
                Some(read_shm_file(&mut tmpfile, buf_size).map_err(|e| {
                    HyprError::WaylandDispatch(format!("failed to read shm data: {e}"))
                })?)
            } else {
                None
            };

        // Destroy the one-shot buffer and pool; they are no longer needed
        // once we've read the pixel data.
        buffer.destroy();
        pool.destroy();
        state.frame_handle = None;

        Ok(CapturedFrame {
            format,
            flags: state.frame_flags.unwrap_or_default(),
            data,
        })
    }
}

impl fmt::Debug for ToplevelExportClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToplevelExportClient").finish()
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────
// Shared-memory file helpers. The Wayland shm protocol requires passing pixel
// data via an fd; we create a temp file and immediately unlink it so the fd
// outlives the filesystem path.

fn create_shm_file(size: usize) -> std::io::Result<std::fs::File> {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let path = format!("{dir}/hypr-sdk-toplevel-export-XXXXXX");
    let file = std::fs::File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?;
    // The shm pool requires the fd to be at least this large; pre-allocate so
    // mmap by the compositor succeeds.
    file.set_len(size as u64)?;
    // Unlink immediately so no stale file remains; the open fd stays valid.
    let _ = std::fs::remove_file(&path);
    Ok(file)
}

fn read_shm_file(file: &mut std::fs::File, size: usize) -> std::io::Result<Vec<u8>> {
    file.seek(std::io::SeekFrom::Start(0))?;
    let mut buf = vec![0u8; size];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

// ── Internal state ──────────────────────────────────────────────────────────
// Tracks the export manager, shm, and per-capture frame state that accumulates
// format/flags/ready events across roundtrips.

struct ToplevelExportState {
    manager: Option<hyprland_toplevel_export_manager_v1::HyprlandToplevelExportManagerV1>,
    shm: Option<wl_shm::WlShm>,
    frame_handle: Option<hyprland_toplevel_export_frame_v1::HyprlandToplevelExportFrameV1>,
    frame_format: Option<ToplevelFrameFormat>,
    frame_flags: Option<ToplevelFrameFlags>,
    frame_ready: bool,
    frame_failed: bool,
}

impl ToplevelExportState {
    fn new() -> Self {
        Self {
            manager: None,
            shm: None,
            frame_handle: None,
            frame_format: None,
            frame_flags: None,
            frame_ready: false,
            frame_failed: false,
        }
    }
}

// ── Dispatch implementations ────────────────────────────────────────────────
// wayland-client requires a Dispatch impl for every object type on the
// event queue.

impl Dispatch<wl_registry::WlRegistry, ()> for ToplevelExportState {
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
                "hyprland_toplevel_export_manager_v1" if state.manager.is_none() => {
                    let mgr = registry
                        .bind::<hyprland_toplevel_export_manager_v1::HyprlandToplevelExportManagerV1, (), Self>(
                            name,
                            version.min(2),
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
                _ => {}
            }
        }
    }
}

impl Dispatch<hyprland_toplevel_export_manager_v1::HyprlandToplevelExportManagerV1, ()>
    for ToplevelExportState
{
    fn event(
        _state: &mut Self,
        _proxy: &hyprland_toplevel_export_manager_v1::HyprlandToplevelExportManagerV1,
        _event: hyprland_toplevel_export_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client; this interface is request-only.
    }
}

impl Dispatch<hyprland_toplevel_export_frame_v1::HyprlandToplevelExportFrameV1, ()>
    for ToplevelExportState
{
    fn event(
        state: &mut Self,
        proxy: &hyprland_toplevel_export_frame_v1::HyprlandToplevelExportFrameV1,
        event: hyprland_toplevel_export_frame_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            hyprland_toplevel_export_frame_v1::Event::Buffer {
                format,
                width,
                height,
                stride,
            } => {
                // The compositor may offer multiple formats; take the first one since we
                // only need a single supported format for the capture.
                if state.frame_format.is_none()
                    && let WEnum::Value(fmt) = format
                {
                    state.frame_format = Some(ToplevelFrameFormat {
                        format: fmt as u32,
                        width,
                        height,
                        stride,
                    });
                }
                // Save the frame proxy so we can call copy() on it after allocating the buffer.
                if state.frame_handle.is_none() {
                    state.frame_handle = Some(proxy.clone());
                }
            }
            hyprland_toplevel_export_frame_v1::Event::Flags { flags } => {
                state.frame_flags = Some(ToplevelFrameFlags(flags.into()));
            }
            hyprland_toplevel_export_frame_v1::Event::Ready { .. } => {
                state.frame_ready = true;
            }
            hyprland_toplevel_export_frame_v1::Event::Failed => {
                state.frame_failed = true;
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_shm::WlShm, ()> for ToplevelExportState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm::WlShm,
        _event: wl_shm::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // We don't inspect shm format advertisements; we pick a format from the frame events.
    }
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for ToplevelExportState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Dispatch impl required by wayland-client; shm pools are request-only.
    }
}

impl Dispatch<wl_buffer::WlBuffer, ()> for ToplevelExportState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_buffer::WlBuffer,
        _event: wl_buffer::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Buffer release events are for reuse scenarios; we destroy the buffer after
        // each one-shot capture so release is irrelevant.
    }
}
