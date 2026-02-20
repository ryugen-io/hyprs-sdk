use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::UnixStream;

use crate::error::{HyprError, HyprResult};

use super::{Event, parse_event};

/// Async event stream reader for Socket2.
///
/// Reads events line-by-line and yields parsed [`Event`] values.
pub struct EventStream {
    reader: BufReader<UnixStream>,
    buf: String,
}

impl EventStream {
    /// Wrap a connected Socket2 stream.
    #[must_use]
    pub fn new(stream: UnixStream) -> Self {
        Self {
            reader: BufReader::new(stream),
            buf: String::with_capacity(1280),
        }
    }

    /// Read the next event from the stream.
    ///
    /// Returns `None` on stream close.
    pub async fn next_event(&mut self) -> HyprResult<Option<Event>> {
        self.buf.clear();
        let n = self
            .reader
            .read_line(&mut self.buf)
            .await
            .map_err(HyprError::Io)?;
        if n == 0 {
            return Ok(None);
        }
        let line = self.buf.trim_end_matches('\n');
        Ok(parse_event(line))
    }
}
