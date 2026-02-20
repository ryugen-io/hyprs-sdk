//! Socket2 event stream — strongly-typed Hyprland events.
//!
//! Wire format: `EVENT>>DATA\n` per line, data truncated to 1024 bytes.

mod parser;
mod stream;
mod types;

pub use parser::parse_event;
pub use stream::EventStream;
pub use types::Event;
