#![forbid(unsafe_code)]

pub mod error;
pub mod types;

pub use error::{HyprError, HyprResult};
pub use types::common::{MonitorId, WindowAddress, WorkspaceId};
