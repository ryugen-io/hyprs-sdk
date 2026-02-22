//! Screencopy client implementation.

use std::fmt;
use std::os::fd::AsFd;

use wayland_client::{EventQueue, QueueHandle};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

use super::dispatch::ScreencopyState;
use super::types::*;

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

        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Output name events arrive asynchronously after binding; a second roundtrip
        // collects them so capture_output can identify outputs by index.
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

// ── Helpers ──────────────────────────────────────────────────────────
// Separated from the client impl to keep the public API surface clean;
// these functions encapsulate the multi-roundtrip capture lifecycle and
// shared-memory buffer management.

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
    let zeros = vec![0u8; size];
    file.write_all(&zeros)
        .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
    let _ = std::fs::remove_file(&path);
    Ok(file)
}

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
