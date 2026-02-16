//! wlr-screencopy: screen content capturing via shared memory buffers.
//!
//! Provides [`ScreencopyClient`] for capturing output frames (screenshots)
//! via the `zwlr_screencopy_manager_v1` protocol.

use std::fmt;
use std::os::fd::AsFd;

use wayland_client::protocol::{wl_buffer, wl_output, wl_registry, wl_shm, wl_shm_pool};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, WEnum};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1, zwlr_screencopy_manager_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// Pixel format for captured frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PixelFormat {
    /// 32-bit ARGB with alpha channel.
    Argb8888 = 0,
    /// 32-bit XRGB without alpha channel (alpha ignored).
    Xrgb8888 = 1,
}

impl PixelFormat {
    /// Convert a raw protocol value to a `PixelFormat`.
    ///
    /// Returns `None` for unrecognized values.
    #[must_use]
    pub fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Argb8888),
            1 => Some(Self::Xrgb8888),
            _ => None,
        }
    }

    /// Returns the number of bytes per pixel for this format.
    #[must_use]
    pub fn bytes_per_pixel(self) -> u32 {
        4
    }

    fn from_wl_format(format: WEnum<wl_shm::Format>) -> Option<Self> {
        match format {
            WEnum::Value(wl_shm::Format::Argb8888) => Some(Self::Argb8888),
            WEnum::Value(wl_shm::Format::Xrgb8888) => Some(Self::Xrgb8888),
            _ => None,
        }
    }

    fn to_wl_format(self) -> wl_shm::Format {
        match self {
            Self::Argb8888 => wl_shm::Format::Argb8888,
            Self::Xrgb8888 => wl_shm::Format::Xrgb8888,
        }
    }
}

/// Describes the format and dimensions of a captured frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameFormat {
    /// Pixel format of the frame.
    pub pixel_format: PixelFormat,
    /// Width of the frame in pixels.
    pub width: u32,
    /// Height of the frame in pixels.
    pub height: u32,
    /// Number of bytes per row.
    pub stride: u32,
}

impl FrameFormat {
    /// Calculate the total buffer size in bytes needed for this frame.
    #[must_use]
    pub fn buffer_size(&self) -> usize {
        self.stride as usize * self.height as usize
    }
}

/// A rectangular region for partial screen capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureRegion {
    /// X offset of the capture region.
    pub x: i32,
    /// Y offset of the capture region.
    pub y: i32,
    /// Width of the capture region.
    pub width: i32,
    /// Height of the capture region.
    pub height: i32,
}

/// Bitflags describing properties of a captured frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FrameFlags(u32);

impl FrameFlags {
    /// The frame is vertically inverted (Y axis flipped).
    pub const Y_INVERT: Self = Self(1);

    /// Create an empty set of flags.
    #[must_use]
    pub fn empty() -> Self {
        Self(0)
    }

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

/// Result of a successful frame capture.
#[derive(Debug, Clone)]
pub struct CapturedFrame {
    /// Frame format and dimensions.
    pub format: FrameFormat,
    /// Frame flags (e.g. Y_INVERT).
    pub flags: FrameFlags,
    /// Raw pixel data.
    pub data: Vec<u8>,
}

/// Client for the `zwlr_screencopy_manager_v1` protocol.
///
/// Captures screenshots of compositor outputs using shared memory buffers.
///
/// # Example
///
/// ```no_run
/// use hypr_sdk::protocols::connection::WaylandConnection;
/// use hypr_sdk::protocols::screencopy::ScreencopyClient;
///
/// let wl = WaylandConnection::connect().unwrap();
/// let mut client = ScreencopyClient::connect(&wl).unwrap();
///
/// // List available outputs
/// for (i, name) in client.output_names().iter().enumerate() {
///     println!("{i}: {name}");
/// }
///
/// // Capture first output
/// let frame = client.capture_output(0, true).unwrap();
/// println!("Captured {}x{} frame", frame.format.width, frame.format.height);
/// ```
pub struct ScreencopyClient {
    state: ScreencopyState,
    event_queue: EventQueue<ScreencopyState>,
    qh: QueueHandle<ScreencopyState>,
}

impl ScreencopyClient {
    /// Connect to the screencopy manager.
    ///
    /// Binds `zwlr_screencopy_manager_v1` and `wl_shm`, discovers outputs.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `zwlr_screencopy_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("zwlr_screencopy_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "zwlr_screencopy_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<ScreencopyState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = ScreencopyState::new();

        // Registry roundtrip: bind manager + shm + outputs.
        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Second roundtrip: receive output name events.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "zwlr_screencopy_manager_v1".into(),
            ));
        }
        if state.shm.is_none() {
            return Err(HyprError::WaylandDispatch("wl_shm not available".into()));
        }

        Ok(Self {
            state,
            event_queue,
            qh,
        })
    }

    /// Get the names of all available outputs.
    #[must_use]
    pub fn output_names(&self) -> Vec<String> {
        self.state
            .outputs
            .iter()
            .map(|o| o.name.clone().unwrap_or_default())
            .collect()
    }

    /// Capture a full output by index.
    ///
    /// `overlay_cursor` controls whether the hardware cursor is included
    /// in the captured frame.
    ///
    /// # Errors
    ///
    /// Returns an error if the output index is invalid, buffer allocation
    /// fails, or the compositor reports a capture failure.
    pub fn capture_output(
        &mut self,
        output_index: usize,
        overlay_cursor: bool,
    ) -> HyprResult<CapturedFrame> {
        let Self {
            state,
            event_queue,
            qh,
        } = self;

        let output = state.outputs.get(output_index).ok_or_else(|| {
            HyprError::WaylandDispatch(format!("output index {output_index} out of range"))
        })?;

        let manager = state
            .manager
            .as_ref()
            .ok_or_else(|| HyprError::ProtocolNotSupported("zwlr_screencopy_manager_v1".into()))?;

        let cursor = if overlay_cursor { 1 } else { 0 };
        let _frame = manager.capture_output(cursor, &output.output, qh, ());

        complete_capture(state, event_queue, qh)
    }

    /// Capture a region of an output by index.
    ///
    /// # Errors
    ///
    /// Returns an error if the output index is invalid, buffer allocation
    /// fails, or the compositor reports a capture failure.
    pub fn capture_output_region(
        &mut self,
        output_index: usize,
        region: CaptureRegion,
        overlay_cursor: bool,
    ) -> HyprResult<CapturedFrame> {
        let Self {
            state,
            event_queue,
            qh,
        } = self;

        let output = state.outputs.get(output_index).ok_or_else(|| {
            HyprError::WaylandDispatch(format!("output index {output_index} out of range"))
        })?;

        let manager = state
            .manager
            .as_ref()
            .ok_or_else(|| HyprError::ProtocolNotSupported("zwlr_screencopy_manager_v1".into()))?;

        let cursor = if overlay_cursor { 1 } else { 0 };
        let _frame = manager.capture_output_region(
            cursor,
            &output.output,
            region.x,
            region.y,
            region.width,
            region.height,
            qh,
            (),
        );

        complete_capture(state, event_queue, qh)
    }
}

impl fmt::Debug for ScreencopyClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScreencopyClient")
            .field("outputs", &self.state.outputs.len())
            .finish()
    }
}

// ── Shared memory helpers ───────────────────────────────────────────

/// Create a shared memory file of the given size.
fn create_shm_file(size: usize) -> HyprResult<std::fs::File> {
    use std::io::Write;

    let path = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let path =
        std::path::PathBuf::from(path).join(format!("hypr-sdk-screencopy-{}", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
    // Pre-allocate the file to the required size.
    let zeros = vec![0u8; size];
    file.write_all(&zeros)
        .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
    // Delete file immediately — fd stays valid.
    let _ = std::fs::remove_file(&path);
    Ok(file)
}

/// Read back data from a shared memory file.
fn read_shm_file(file: &std::fs::File, size: usize) -> HyprResult<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};

    let mut file = file;
    file.seek(SeekFrom::Start(0))
        .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
    let mut data = vec![0u8; size];
    file.read_exact(&mut data)
        .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
    Ok(data)
}

/// Shared capture logic: wait for buffer info, allocate shm, copy frame, read pixels.
fn complete_capture(
    state: &mut ScreencopyState,
    event_queue: &mut EventQueue<ScreencopyState>,
    qh: &QueueHandle<ScreencopyState>,
) -> HyprResult<CapturedFrame> {
    state.frame_buffer_info = None;
    state.frame_flags = None;
    state.frame_ready = false;
    state.frame_failed = false;

    event_queue
        .roundtrip(state)
        .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

    let buf_info = state.frame_buffer_info.as_ref().ok_or_else(|| {
        HyprError::WaylandDispatch("no buffer info received from screencopy".into())
    })?;

    let format = buf_info.format;
    let width = buf_info.width;
    let height = buf_info.height;
    let stride = buf_info.stride;
    let size = (stride * height) as usize;

    let shm = state
        .shm
        .as_ref()
        .ok_or_else(|| HyprError::WaylandDispatch("wl_shm not available".into()))?;

    let file = create_shm_file(size)?;
    let pool = shm.create_pool(file.as_fd(), size as i32, qh, ());
    let buffer = pool.create_buffer(
        0,
        width as i32,
        height as i32,
        stride as i32,
        format.to_wl_format(),
        qh,
        (),
    );

    if let Some(ref frame_proxy) = state.frame_proxy {
        frame_proxy.copy(&buffer);
    }

    event_queue
        .roundtrip(state)
        .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

    if !state.frame_ready && !state.frame_failed {
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
    }

    if state.frame_failed {
        return Err(HyprError::WaylandDispatch("screencopy frame failed".into()));
    }

    if !state.frame_ready {
        return Err(HyprError::WaylandDispatch(
            "screencopy frame not ready after roundtrips".into(),
        ));
    }

    let data = read_shm_file(&file, size)?;

    buffer.destroy();
    pool.destroy();

    Ok(CapturedFrame {
        format: FrameFormat {
            pixel_format: format,
            width,
            height,
            stride,
        },
        flags: FrameFlags(state.frame_flags.unwrap_or(0)),
        data,
    })
}

// ── Internal state ───────────────────────────────────────────────────

struct ScreencopyState {
    manager: Option<zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1>,
    shm: Option<wl_shm::WlShm>,
    outputs: Vec<OutputEntry>,
    // Frame capture state.
    frame_proxy: Option<zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1>,
    frame_buffer_info: Option<BufferInfo>,
    frame_flags: Option<u32>,
    frame_ready: bool,
    frame_failed: bool,
}

struct OutputEntry {
    global_name: u32,
    output: wl_output::WlOutput,
    name: Option<String>,
}

#[derive(Clone, Copy)]
struct BufferInfo {
    format: PixelFormat,
    width: u32,
    height: u32,
    stride: u32,
}

impl ScreencopyState {
    fn new() -> Self {
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
        // Format events not needed — we know the formats from screencopy.
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
        // Pool has no events.
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
        // Buffer release events not needed for screencopy.
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
        // Manager has no events.
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
