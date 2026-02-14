//! Low-level Unix socket communication with Hyprland.
//!
//! Socket1 (`.socket.sock`): one connection per request/response.
//! Socket2 (`.socket2.sock`): persistent event stream.

use std::path::Path;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::error::{HyprError, HyprResult};

/// Send a raw command over Socket1 and return the response.
///
/// Opens a new connection, writes the command, reads the full response,
/// and closes the socket. Matches Hyprland's one-connection-per-request model.
pub async fn request(socket_path: &Path, command: &str) -> HyprResult<String> {
    let mut stream = UnixStream::connect(socket_path)
        .await
        .map_err(HyprError::Io)?;

    stream
        .write_all(command.as_bytes())
        .await
        .map_err(HyprError::Io)?;
    stream.shutdown().await.map_err(HyprError::Io)?;

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .await
        .map_err(HyprError::Io)?;

    String::from_utf8(response).map_err(|e| HyprError::Parse(e.to_string()))
}

/// Connect to Socket2 (event stream) and return the raw stream.
///
/// The stream emits events as `EVENT>>DATA\n` lines.
/// Use [`crate::ipc::events`] for parsed event types.
pub async fn connect_event_stream(socket_path: &Path) -> HyprResult<UnixStream> {
    UnixStream::connect(socket_path)
        .await
        .map_err(HyprError::Io)
}

/// Blocking variants (requires `blocking` feature).
#[cfg(feature = "blocking")]
pub mod blocking {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    use std::path::Path;
    use std::time::Duration;

    use crate::error::{HyprError, HyprResult};

    /// Send a raw command over Socket1 and return the response (blocking).
    pub fn request(socket_path: &Path, command: &str) -> HyprResult<String> {
        let mut stream = UnixStream::connect(socket_path).map_err(HyprError::Io)?;
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(HyprError::Io)?;

        stream
            .write_all(command.as_bytes())
            .map_err(HyprError::Io)?;
        stream
            .shutdown(std::net::Shutdown::Write)
            .map_err(HyprError::Io)?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response).map_err(HyprError::Io)?;

        String::from_utf8(response).map_err(|e| HyprError::Parse(e.to_string()))
    }
}
