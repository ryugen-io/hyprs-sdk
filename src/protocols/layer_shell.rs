//! wlr-layer-shell: create surfaces that are layers of the desktop.
//!
//! Provides [`LayerShellClient`] for creating layer surfaces (panels,
//! taskbars, overlays, lock screens) via the `zwlr_layer_shell_v1` protocol.
//!
//! The client handles surface creation and the configure/ack lifecycle.
//! To display content, attach a buffer to the returned `wl_surface` handle
//! after the initial configure event.
//!
//! # Example
//!
//! ```no_run
//! use hyprs_sdk::protocols::connection::WaylandConnection;
//! use hyprs_sdk::protocols::layer_shell::{
//!     LayerShellClient, LayerSurfaceConfig, ShellLayer, Anchor,
//! };
//!
//! let wl = WaylandConnection::connect().unwrap();
//! let mut client = LayerShellClient::connect(&wl).unwrap();
//!
//! let config = LayerSurfaceConfig {
//!     layer: ShellLayer::Top,
//!     namespace: "my-panel".into(),
//!     width: 0,
//!     height: 48,
//!     anchor: Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
//!     exclusive_zone: 48,
//!     ..Default::default()
//! };
//!
//! let surface = client.create_surface(&config, None).unwrap();
//! println!("Configured size: {}x{}", surface.width, surface.height);
//! ```
mod client;
mod dispatch;
mod types;

pub use client::LayerShellClient;
pub use types::{
    Anchor, KeyboardInteractivity, LayerSurfaceConfig, LayerSurfaceHandle, ShellLayer,
};
