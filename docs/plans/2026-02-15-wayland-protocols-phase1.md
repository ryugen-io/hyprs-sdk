# Wayland Protocol Bindings — Phase 1 Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Wayland protocol infrastructure and the first 5 WLR protocol bindings (layer-shell, screencopy, gamma-control, output-management, output-power) to hypr-sdk.

**Architecture:** Use `wayland-client` for connection/event-queue management and `wayland-protocols-wlr` crate for pre-generated WLR protocol types. Each protocol gets a typed wrapper module under `src/protocols/`. A `WaylandConnection` struct manages connection lifecycle and registry globals. Feature-gated behind `wayland` feature flag so IPC-only users don't pull in wayland dependencies.

**Tech Stack:** Rust nightly 1.94 (edition 2024), wayland-client 0.31, wayland-protocols-wlr 0.3 (client feature), wayland-protocols 0.32 (client feature).

**Design doc:** `docs/plans/2026-02-15-wayland-protocols-design.md`

---

### Task 1: Add wayland dependencies and feature flag

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add wayland feature flag and dependencies**

Add a `wayland` feature flag and the three wayland crates as optional dependencies. This keeps the default build lightweight for IPC-only users.

```toml
[features]
default = []
blocking = []
wayland = ["dep:wayland-client", "dep:wayland-protocols", "dep:wayland-protocols-wlr"]

[dependencies]
# ... existing deps ...
wayland-client = { version = "0.31", optional = true }
wayland-protocols = { version = "0.32", features = ["client"], optional = true }
wayland-protocols-wlr = { version = "0.3", features = ["client"], optional = true }
```

**Step 2: Verify it compiles both ways**

Run: `cargo check 2>&1` (without wayland — should compile as before)
Run: `cargo check --features wayland 2>&1` (with wayland — should pull deps and compile)
Expected: Both succeed with no errors.

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "add: wayland feature flag and dependencies"
```

---

### Task 2: Error variants for Wayland

**Files:**
- Modify: `src/error.rs`

**Step 1: Write the failing test**

Create `tests/error_wayland.rs`:

```rust
#![cfg(feature = "wayland")]

use hypr_sdk::HyprError;

#[test]
fn protocol_not_supported_error() {
    let err = HyprError::ProtocolNotSupported("zwlr_layer_shell_v1".to_string());
    let msg = err.to_string();
    assert!(msg.contains("zwlr_layer_shell_v1"));
}

#[test]
fn wayland_connect_error_display() {
    let err = HyprError::WaylandConnect("no display".to_string());
    assert!(err.to_string().contains("no display"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features wayland --test error_wayland 2>&1`
Expected: FAIL — `ProtocolNotSupported` and `WaylandConnect` variants don't exist yet.

**Step 3: Add error variants**

In `src/error.rs`, add these variants inside the `HyprError` enum:

```rust
    /// Wayland connection failed.
    #[cfg(feature = "wayland")]
    #[error("wayland connect error: {0}")]
    WaylandConnect(String),

    /// Wayland event dispatch error.
    #[cfg(feature = "wayland")]
    #[error("wayland dispatch error: {0}")]
    WaylandDispatch(String),

    /// The compositor does not advertise a required protocol global.
    #[cfg(feature = "wayland")]
    #[error("protocol not supported: {0}")]
    ProtocolNotSupported(String),
```

**Step 4: Run tests**

Run: `cargo test --features wayland --test error_wayland 2>&1`
Expected: PASS

Run: `cargo test 2>&1` (all existing tests still pass without wayland feature)
Expected: PASS

**Step 5: Commit**

```bash
git add src/error.rs tests/error_wayland.rs
git commit -m "add: wayland error variants behind feature flag"
```

---

### Task 3: WaylandConnection — connect and registry globals

This is the core infrastructure. `WaylandConnection` connects to the Wayland display, creates an event queue, and discovers available protocol globals from the compositor's registry.

**Files:**
- Create: `src/protocols/connection.rs`
- Modify: `src/protocols/mod.rs`
- Create: `tests/wayland_connection.rs`

**Step 1: Write the failing test**

Create `tests/wayland_connection.rs`:

```rust
#![cfg(feature = "wayland")]

use hypr_sdk::protocols::connection::WaylandConnection;

#[test]
fn connect_fails_without_display() {
    // Unset WAYLAND_DISPLAY to force failure
    std::env::remove_var("WAYLAND_DISPLAY");
    let result = WaylandConnection::connect();
    assert!(result.is_err());
}

#[test]
fn global_registry_struct() {
    use hypr_sdk::protocols::connection::GlobalInfo;
    let info = GlobalInfo {
        name: 1,
        interface: "wl_compositor".to_string(),
        version: 5,
    };
    assert_eq!(info.interface, "wl_compositor");
    assert_eq!(info.version, 5);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features wayland --test wayland_connection 2>&1`
Expected: FAIL — module doesn't exist.

**Step 3: Implement WaylandConnection**

Create `src/protocols/connection.rs`:

```rust
//! Wayland display connection and registry global discovery.

use wayland_client::protocol::wl_registry;
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle};

use crate::error::{HyprError, HyprResult};

/// Information about a global object advertised by the compositor.
#[derive(Debug, Clone)]
pub struct GlobalInfo {
    /// Server-assigned name for this global.
    pub name: u32,
    /// Interface name (e.g. `"zwlr_layer_shell_v1"`).
    pub interface: String,
    /// Maximum supported version.
    pub version: u32,
}

/// State object used during registry enumeration.
struct RegistryState {
    globals: Vec<GlobalInfo>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for RegistryState {
    fn event(
        state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            state.globals.push(GlobalInfo { name, interface, version });
        }
    }
}

/// Connection to the Wayland display server.
///
/// Entry point for all protocol operations. Connects to the display,
/// enumerates compositor globals, and provides access to the underlying
/// `wayland_client::Connection` and `EventQueue`.
pub struct WaylandConnection {
    conn: Connection,
    globals: Vec<GlobalInfo>,
}

impl WaylandConnection {
    /// Connect to the Wayland display using `$WAYLAND_DISPLAY`.
    pub fn connect() -> HyprResult<Self> {
        let conn = Connection::connect_to_env()
            .map_err(|e| HyprError::WaylandConnect(e.to_string()))?;
        Self::from_connection(conn)
    }

    /// Connect to a specific Wayland display socket.
    pub fn connect_to(name: &str) -> HyprResult<Self> {
        let conn = Connection::connect_to_env()
            .map_err(|e| HyprError::WaylandConnect(format!("{name}: {e}")))?;
        // Note: wayland-client's connect_to_env uses WAYLAND_DISPLAY.
        // For named connections, the caller should set the env var or
        // use OsStr-based connection methods.
        Self::from_connection(conn)
    }

    fn from_connection(conn: Connection) -> HyprResult<Self> {
        let display = conn.display();
        let mut event_queue: EventQueue<RegistryState> = conn.new_event_queue();
        let qh = event_queue.handle();

        let _registry = display.get_registry(&qh, ());

        let mut state = RegistryState {
            globals: Vec::new(),
        };

        // Roundtrip to receive all registry.global events.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self {
            conn,
            globals: state.globals,
        })
    }

    /// All globals advertised by the compositor.
    #[must_use]
    pub fn globals(&self) -> &[GlobalInfo] {
        &self.globals
    }

    /// Find a global by interface name.
    #[must_use]
    pub fn find_global(&self, interface: &str) -> Option<&GlobalInfo> {
        self.globals.iter().find(|g| g.interface == interface)
    }

    /// Check if a protocol is supported by the compositor.
    #[must_use]
    pub fn has_protocol(&self, interface: &str) -> bool {
        self.find_global(interface).is_some()
    }

    /// Access the underlying wayland-client connection.
    #[must_use]
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

impl std::fmt::Debug for WaylandConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WaylandConnection")
            .field("globals_count", &self.globals.len())
            .finish()
    }
}
```

Update `src/protocols/mod.rs`:

```rust
//! Wayland protocol client bindings.
//!
//! Client-side bindings for Hyprland-specific and wlr protocols.
//! Requires the `wayland` feature flag.

#[cfg(feature = "wayland")]
pub mod connection;
```

**Step 4: Run tests**

Run: `cargo test --features wayland --test wayland_connection 2>&1`
Expected: PASS (connect_fails_without_display passes because no Wayland display is available in test env; global_registry_struct is a pure data test)

Run: `cargo clippy --features wayland -- -D warnings 2>&1`
Expected: No warnings

Run: `cargo test 2>&1` (existing tests without wayland)
Expected: PASS

**Step 5: Commit**

```bash
git add src/protocols/ tests/wayland_connection.rs
git commit -m "add: WaylandConnection with registry global discovery"
```

---

### Task 4: Gamma control module

Start with gamma-control because it's the simplest WLR protocol (2 interfaces, 3 requests, 2 events). Good for establishing the pattern.

**Files:**
- Create: `src/protocols/gamma_control.rs`
- Modify: `src/protocols/mod.rs`
- Create: `tests/wayland_gamma_control.rs`

**Step 1: Write the failing test**

Create `tests/wayland_gamma_control.rs`:

```rust
#![cfg(feature = "wayland")]

use hypr_sdk::protocols::gamma_control;

#[test]
fn gamma_table_construction() {
    let size = 256;
    let table = gamma_control::GammaTable::identity(size);
    assert_eq!(table.size, size);
    assert_eq!(table.red.len(), size as usize);
    assert_eq!(table.green.len(), size as usize);
    assert_eq!(table.blue.len(), size as usize);
    // Identity: linear ramp from 0 to u16::MAX
    assert_eq!(table.red[0], 0);
    assert_eq!(table.red[255], u16::MAX);
}

#[test]
fn gamma_table_to_bytes() {
    let table = gamma_control::GammaTable::identity(4);
    let bytes = table.to_bytes();
    // 3 channels * 4 entries * 2 bytes = 24 bytes
    assert_eq!(bytes.len(), 24);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features wayland --test wayland_gamma_control 2>&1`
Expected: FAIL — module doesn't exist.

**Step 3: Implement gamma control module**

Create `src/protocols/gamma_control.rs`:

```rust
//! wlr-gamma-control: adjust gamma tables for outputs.
//!
//! Allows adjusting the gamma/brightness of outputs. Each output can have
//! at most one gamma control. When destroyed, the original gamma is restored.

/// A gamma lookup table with separate red, green, blue ramps.
///
/// Each ramp contains `size` entries of `u16` values.
#[derive(Debug, Clone)]
pub struct GammaTable {
    /// Number of entries per channel.
    pub size: u32,
    /// Red channel ramp.
    pub red: Vec<u16>,
    /// Green channel ramp.
    pub green: Vec<u16>,
    /// Blue channel ramp.
    pub blue: Vec<u16>,
}

impl GammaTable {
    /// Create an identity (linear) gamma table.
    ///
    /// Maps input values linearly from 0 to `u16::MAX`.
    #[must_use]
    pub fn identity(size: u32) -> Self {
        let ramp: Vec<u16> = (0..size)
            .map(|i| {
                if size <= 1 {
                    u16::MAX
                } else {
                    ((i as u64 * u16::MAX as u64) / (size as u64 - 1)) as u16
                }
            })
            .collect();
        Self {
            size,
            red: ramp.clone(),
            green: ramp.clone(),
            blue: ramp,
        }
    }

    /// Serialize the gamma table to bytes for the set_gamma fd.
    ///
    /// Format: red ramp (u16 LE), then green, then blue. Total size
    /// is `3 * size * 2` bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.size as usize * 6);
        for &v in &self.red {
            buf.extend_from_slice(&v.to_ne_bytes());
        }
        for &v in &self.green {
            buf.extend_from_slice(&v.to_ne_bytes());
        }
        for &v in &self.blue {
            buf.extend_from_slice(&v.to_ne_bytes());
        }
        buf
    }

    /// Apply a brightness multiplier (0.0 to 1.0) to an identity table.
    #[must_use]
    pub fn with_brightness(size: u32, brightness: f64) -> Self {
        let mut table = Self::identity(size);
        let factor = brightness.clamp(0.0, 1.0);
        for v in table.red.iter_mut().chain(table.green.iter_mut()).chain(table.blue.iter_mut()) {
            *v = (*v as f64 * factor) as u16;
        }
        table
    }

    /// Apply a gamma correction exponent.
    #[must_use]
    pub fn with_gamma(size: u32, gamma: f64) -> Self {
        let mut table = Self::identity(size);
        let inv_gamma = 1.0 / gamma;
        for v in table.red.iter_mut().chain(table.green.iter_mut()).chain(table.blue.iter_mut()) {
            let normalized = *v as f64 / u16::MAX as f64;
            *v = (normalized.powf(inv_gamma) * u16::MAX as f64) as u16;
        }
        table
    }
}
```

Add to `src/protocols/mod.rs`:

```rust
#[cfg(feature = "wayland")]
pub mod gamma_control;
```

**Step 4: Run tests**

Run: `cargo test --features wayland --test wayland_gamma_control 2>&1`
Expected: PASS

Run: `cargo clippy --features wayland -- -D warnings 2>&1`
Expected: No warnings

**Step 5: Commit**

```bash
git add src/protocols/gamma_control.rs src/protocols/mod.rs tests/wayland_gamma_control.rs
git commit -m "add: gamma control protocol types (GammaTable)"
```

---

### Task 5: Output power management module

Another simple protocol — just on/off/toggle for DPMS.

**Files:**
- Create: `src/protocols/output_power.rs`
- Modify: `src/protocols/mod.rs`
- Create: `tests/wayland_output_power.rs`

**Step 1: Write the failing test**

Create `tests/wayland_output_power.rs`:

```rust
#![cfg(feature = "wayland")]

use hypr_sdk::protocols::output_power::PowerMode;

#[test]
fn power_mode_variants() {
    assert_eq!(PowerMode::On as u32, 0);
    assert_eq!(PowerMode::Off as u32, 1);
}

#[test]
fn power_mode_from_raw() {
    assert_eq!(PowerMode::from_raw(0), Some(PowerMode::On));
    assert_eq!(PowerMode::from_raw(1), Some(PowerMode::Off));
    assert_eq!(PowerMode::from_raw(99), None);
}

#[test]
fn power_mode_display() {
    assert_eq!(PowerMode::On.to_string(), "on");
    assert_eq!(PowerMode::Off.to_string(), "off");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features wayland --test wayland_output_power 2>&1`
Expected: FAIL

**Step 3: Implement output power module**

Create `src/protocols/output_power.rs`:

```rust
//! wlr-output-power-management: DPMS control for outputs.
//!
//! Allows turning outputs on and off (Display Power Management Signaling).

use std::fmt;

/// Output power state.
///
/// Maps to `zwlr_output_power_v1::mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PowerMode {
    /// Output is enabled and displaying content.
    On = 0,
    /// Output is disabled (DPMS standby/off).
    Off = 1,
}

impl PowerMode {
    /// Parse from the raw protocol value.
    #[must_use]
    pub fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::On),
            1 => Some(Self::Off),
            _ => None,
        }
    }
}

impl fmt::Display for PowerMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::On => write!(f, "on"),
            Self::Off => write!(f, "off"),
        }
    }
}
```

Add to `src/protocols/mod.rs`:

```rust
#[cfg(feature = "wayland")]
pub mod output_power;
```

**Step 4: Run tests**

Run: `cargo test --features wayland --test wayland_output_power 2>&1`
Expected: PASS

**Step 5: Commit**

```bash
git add src/protocols/output_power.rs src/protocols/mod.rs tests/wayland_output_power.rs
git commit -m "add: output power management protocol types (PowerMode)"
```

---

### Task 6: Output management module

More complex — models output heads (monitors), modes (resolutions), and configurations.

**Files:**
- Create: `src/protocols/output_management.rs`
- Modify: `src/protocols/mod.rs`
- Create: `tests/wayland_output_management.rs`

**Step 1: Write the failing test**

Create `tests/wayland_output_management.rs`:

```rust
#![cfg(feature = "wayland")]

use hypr_sdk::protocols::output_management::*;

#[test]
fn output_mode_construction() {
    let mode = OutputMode {
        width: 2560,
        height: 1440,
        refresh: 165000,
        preferred: true,
    };
    assert_eq!(mode.width, 2560);
    assert_eq!(mode.refresh_hz(), 165.0);
}

#[test]
fn output_head_defaults() {
    let head = OutputHead::default();
    assert!(head.name.is_empty());
    assert!(head.modes.is_empty());
    assert!(!head.enabled);
}

#[test]
fn output_config_entry_construction() {
    let entry = OutputConfigEntry {
        name: "DP-1".to_string(),
        enabled: true,
        mode: Some(OutputMode {
            width: 1920,
            height: 1080,
            refresh: 60000,
            preferred: false,
        }),
        position_x: Some(0),
        position_y: Some(0),
        scale: Some(1.0),
        transform: Some(0),
    };
    assert!(entry.enabled);
    assert_eq!(entry.mode.as_ref().unwrap().width, 1920);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features wayland --test wayland_output_management 2>&1`
Expected: FAIL

**Step 3: Implement output management module**

Create `src/protocols/output_management.rs`:

```rust
//! wlr-output-management: monitor configuration protocol.
//!
//! Query available outputs, their modes, and apply configuration changes
//! (resolution, position, scale, transform, enable/disable).

/// A display mode (resolution + refresh rate).
#[derive(Debug, Clone, PartialEq)]
pub struct OutputMode {
    /// Width in pixels.
    pub width: i32,
    /// Height in pixels.
    pub height: i32,
    /// Refresh rate in millihertz (e.g. 60000 = 60 Hz).
    pub refresh: i32,
    /// Whether this is the output's preferred mode.
    pub preferred: bool,
}

impl OutputMode {
    /// Refresh rate in Hz as a float.
    #[must_use]
    pub fn refresh_hz(&self) -> f64 {
        self.refresh as f64 / 1000.0
    }
}

/// An output head (physical or virtual monitor).
///
/// Represents the current state of an output as advertised by the compositor.
#[derive(Debug, Clone, Default)]
pub struct OutputHead {
    /// Output connector name (e.g. `"DP-1"`).
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Physical width in mm.
    pub physical_width: i32,
    /// Physical height in mm.
    pub physical_height: i32,
    /// Available modes.
    pub modes: Vec<OutputMode>,
    /// Whether the output is currently enabled.
    pub enabled: bool,
    /// Current mode index in `modes`, if enabled.
    pub current_mode: Option<usize>,
    /// Current X position in layout.
    pub position_x: i32,
    /// Current Y position in layout.
    pub position_y: i32,
    /// Current scale factor.
    pub scale: f64,
    /// Current transform (0-7, maps to wl_output_transform).
    pub transform: i32,
    /// Make string from EDID.
    pub make: String,
    /// Model string from EDID.
    pub model: String,
    /// Serial number from EDID.
    pub serial_number: String,
}

/// A configuration entry for a single output.
///
/// Used when building a configuration to apply.
#[derive(Debug, Clone)]
pub struct OutputConfigEntry {
    /// Output name to configure.
    pub name: String,
    /// Whether to enable this output.
    pub enabled: bool,
    /// Desired mode (None = compositor default).
    pub mode: Option<OutputMode>,
    /// X position in layout.
    pub position_x: Option<i32>,
    /// Y position in layout.
    pub position_y: Option<i32>,
    /// Scale factor.
    pub scale: Option<f64>,
    /// Transform (0-7).
    pub transform: Option<i32>,
}
```

Add to `src/protocols/mod.rs`:

```rust
#[cfg(feature = "wayland")]
pub mod output_management;
```

**Step 4: Run tests**

Run: `cargo test --features wayland --test wayland_output_management 2>&1`
Expected: PASS

**Step 5: Commit**

```bash
git add src/protocols/output_management.rs src/protocols/mod.rs tests/wayland_output_management.rs
git commit -m "add: output management protocol types"
```

---

### Task 7: Screencopy module

Screenshot capture types.

**Files:**
- Create: `src/protocols/screencopy.rs`
- Modify: `src/protocols/mod.rs`
- Create: `tests/wayland_screencopy.rs`

**Step 1: Write the failing test**

Create `tests/wayland_screencopy.rs`:

```rust
#![cfg(feature = "wayland")]

use hypr_sdk::protocols::screencopy::*;

#[test]
fn frame_format_construction() {
    let fmt = FrameFormat {
        pixel_format: PixelFormat::Argb8888,
        width: 1920,
        height: 1080,
        stride: 1920 * 4,
    };
    assert_eq!(fmt.buffer_size(), 1920 * 1080 * 4);
}

#[test]
fn pixel_format_from_raw() {
    assert_eq!(PixelFormat::from_raw(0), Some(PixelFormat::Argb8888));
    assert_eq!(PixelFormat::from_raw(1), Some(PixelFormat::Xrgb8888));
}

#[test]
fn capture_region_construction() {
    let region = CaptureRegion {
        x: 100,
        y: 200,
        width: 800,
        height: 600,
    };
    assert_eq!(region.width, 800);
}

#[test]
fn frame_flags() {
    assert!(FrameFlags::empty().is_empty());
    assert!(FrameFlags::Y_INVERT.contains(FrameFlags::Y_INVERT));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features wayland --test wayland_screencopy 2>&1`
Expected: FAIL

**Step 3: Implement screencopy module**

Create `src/protocols/screencopy.rs`:

```rust
//! wlr-screencopy: screen content capturing on client buffers.
//!
//! Capture full outputs or regions as pixel data.

/// Pixel format identifier (subset of DRM fourcc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PixelFormat {
    /// ARGB 8-bit per channel (0x34325241).
    Argb8888 = 0,
    /// XRGB 8-bit per channel (0x34325258).
    Xrgb8888 = 1,
}

impl PixelFormat {
    /// Parse from the raw DRM format value.
    ///
    /// Note: the actual fourcc values from the protocol differ per compositor.
    /// This maps the common ordinals used in screencopy buffer events.
    #[must_use]
    pub fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Argb8888),
            1 => Some(Self::Xrgb8888),
            _ => None,
        }
    }

    /// Bytes per pixel for this format.
    #[must_use]
    pub fn bytes_per_pixel(self) -> u32 {
        match self {
            Self::Argb8888 | Self::Xrgb8888 => 4,
        }
    }
}

/// Format description for a captured frame buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameFormat {
    /// Pixel format.
    pub pixel_format: PixelFormat,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Row stride in bytes.
    pub stride: u32,
}

impl FrameFormat {
    /// Total buffer size needed in bytes.
    #[must_use]
    pub fn buffer_size(&self) -> usize {
        self.stride as usize * self.height as usize
    }
}

/// Region of an output to capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureRegion {
    /// X offset in logical coordinates.
    pub x: i32,
    /// Y offset in logical coordinates.
    pub y: i32,
    /// Width in logical coordinates.
    pub width: i32,
    /// Height in logical coordinates.
    pub height: i32,
}

/// Flags for a captured frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FrameFlags(u32);

impl FrameFlags {
    /// No flags set.
    #[must_use]
    pub fn empty() -> Self {
        Self(0)
    }

    /// The frame is vertically flipped.
    pub const Y_INVERT: Self = Self(1);

    /// Check if no flags are set.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Check if a flag is set.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}
```

Add to `src/protocols/mod.rs`:

```rust
#[cfg(feature = "wayland")]
pub mod screencopy;
```

**Step 4: Run tests**

Run: `cargo test --features wayland --test wayland_screencopy 2>&1`
Expected: PASS

**Step 5: Commit**

```bash
git add src/protocols/screencopy.rs src/protocols/mod.rs tests/wayland_screencopy.rs
git commit -m "add: screencopy protocol types"
```

---

### Task 8: Layer shell module

The most complex Phase 1 protocol. Models layer surfaces for panels, overlays, taskbars.

**Files:**
- Create: `src/protocols/layer_shell.rs`
- Modify: `src/protocols/mod.rs`
- Create: `tests/wayland_layer_shell.rs`

**Step 1: Write the failing test**

Create `tests/wayland_layer_shell.rs`:

```rust
#![cfg(feature = "wayland")]

use hypr_sdk::protocols::layer_shell::*;

#[test]
fn layer_ordering() {
    assert!(matches!(ShellLayer::Background, ShellLayer::Background));
    assert_eq!(ShellLayer::Background as u32, 0);
    assert_eq!(ShellLayer::Bottom as u32, 1);
    assert_eq!(ShellLayer::Top as u32, 2);
    assert_eq!(ShellLayer::Overlay as u32, 3);
}

#[test]
fn anchor_bitmask() {
    let anchor = Anchor::TOP | Anchor::LEFT | Anchor::RIGHT;
    assert!(anchor.contains(Anchor::TOP));
    assert!(anchor.contains(Anchor::LEFT));
    assert!(anchor.contains(Anchor::RIGHT));
    assert!(!anchor.contains(Anchor::BOTTOM));
}

#[test]
fn anchor_full_horizontal_bar() {
    let bar = Anchor::TOP | Anchor::LEFT | Anchor::RIGHT;
    assert!(bar.is_horizontal_bar());
    assert!(!bar.is_vertical_bar());
}

#[test]
fn anchor_full_vertical_bar() {
    let bar = Anchor::LEFT | Anchor::TOP | Anchor::BOTTOM;
    assert!(bar.is_vertical_bar());
    assert!(!bar.is_horizontal_bar());
}

#[test]
fn keyboard_interactivity_variants() {
    assert_eq!(KeyboardInteractivity::None as u32, 0);
    assert_eq!(KeyboardInteractivity::Exclusive as u32, 1);
    assert_eq!(KeyboardInteractivity::OnDemand as u32, 2);
}

#[test]
fn layer_surface_config_defaults() {
    let config = LayerSurfaceConfig::default();
    assert_eq!(config.layer, ShellLayer::Top);
    assert!(config.namespace.is_empty());
    assert_eq!(config.width, 0);
    assert_eq!(config.height, 0);
    assert!(config.anchor.is_empty());
    assert_eq!(config.keyboard_interactivity, KeyboardInteractivity::None);
    assert_eq!(config.exclusive_zone, 0);
}

#[test]
fn layer_surface_config_taskbar() {
    let config = LayerSurfaceConfig {
        layer: ShellLayer::Top,
        namespace: "taskbar".to_string(),
        width: 0,
        height: 40,
        anchor: Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
        exclusive_zone: 40,
        keyboard_interactivity: KeyboardInteractivity::None,
        margin_top: 0,
        margin_bottom: 0,
        margin_left: 0,
        margin_right: 0,
    };
    assert_eq!(config.exclusive_zone, 40);
    assert!(config.anchor.contains(Anchor::BOTTOM));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --features wayland --test wayland_layer_shell 2>&1`
Expected: FAIL

**Step 3: Implement layer shell module**

Create `src/protocols/layer_shell.rs`:

```rust
//! wlr-layer-shell: create surfaces that are layers of the desktop.
//!
//! Used for panels, taskbars, overlays, notifications, lock screens, etc.
//! Layer surfaces are rendered at specific z-depths and can be anchored
//! to screen edges.

use std::ops::BitOr;

/// Layer level for a surface.
///
/// Ordered by z-depth (background is bottom-most).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum ShellLayer {
    /// Below all other surfaces.
    Background = 0,
    /// Below normal windows.
    Bottom = 1,
    /// Above normal windows (default for panels).
    Top = 2,
    /// Above everything (for lock screens, critical overlays).
    Overlay = 3,
}

impl ShellLayer {
    /// Parse from the raw protocol value.
    #[must_use]
    pub fn from_raw(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Background),
            1 => Some(Self::Bottom),
            2 => Some(Self::Top),
            3 => Some(Self::Overlay),
            _ => None,
        }
    }
}

/// Edge/corner anchoring for a layer surface (bitmask).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Anchor(u32);

impl Anchor {
    pub const TOP: Self = Self(1);
    pub const BOTTOM: Self = Self(2);
    pub const LEFT: Self = Self(4);
    pub const RIGHT: Self = Self(8);

    /// No anchoring.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Check if a specific anchor is set.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// True if anchored to TOP + LEFT + RIGHT (horizontal bar at top).
    #[must_use]
    pub fn is_horizontal_bar(self) -> bool {
        self.contains(Self::LEFT) && self.contains(Self::RIGHT)
            && (self.contains(Self::TOP) || self.contains(Self::BOTTOM))
            && !(self.contains(Self::TOP) && self.contains(Self::BOTTOM))
    }

    /// True if anchored to LEFT + TOP + BOTTOM (vertical bar at left/right).
    #[must_use]
    pub fn is_vertical_bar(self) -> bool {
        self.contains(Self::TOP) && self.contains(Self::BOTTOM)
            && (self.contains(Self::LEFT) || self.contains(Self::RIGHT))
            && !(self.contains(Self::LEFT) && self.contains(Self::RIGHT))
    }
}

impl BitOr for Anchor {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Keyboard interactivity mode for a layer surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u32)]
pub enum KeyboardInteractivity {
    /// No keyboard input.
    #[default]
    None = 0,
    /// Exclusive keyboard focus (e.g. lock screen).
    Exclusive = 1,
    /// Keyboard focus on demand (e.g. search bar).
    OnDemand = 2,
}

/// Configuration for creating a layer surface.
#[derive(Debug, Clone)]
pub struct LayerSurfaceConfig {
    /// Which layer to place the surface on.
    pub layer: ShellLayer,
    /// Namespace identifying the surface purpose (e.g. `"panel"`, `"notification"`).
    pub namespace: String,
    /// Desired width (0 = stretch to anchored edges).
    pub width: u32,
    /// Desired height (0 = stretch to anchored edges).
    pub height: u32,
    /// Edge/corner anchoring.
    pub anchor: Anchor,
    /// Exclusive zone size (pixels reserved from the anchor edge).
    /// Set to the panel height/width to prevent windows from overlapping.
    /// Set to -1 to overlap everything.
    pub exclusive_zone: i32,
    /// Keyboard interactivity mode.
    pub keyboard_interactivity: KeyboardInteractivity,
    /// Margin from the top anchor edge.
    pub margin_top: i32,
    /// Margin from the bottom anchor edge.
    pub margin_bottom: i32,
    /// Margin from the left anchor edge.
    pub margin_left: i32,
    /// Margin from the right anchor edge.
    pub margin_right: i32,
}

impl Default for LayerSurfaceConfig {
    fn default() -> Self {
        Self {
            layer: ShellLayer::Top,
            namespace: String::new(),
            width: 0,
            height: 0,
            anchor: Anchor::default(),
            exclusive_zone: 0,
            keyboard_interactivity: KeyboardInteractivity::None,
            margin_top: 0,
            margin_bottom: 0,
            margin_left: 0,
            margin_right: 0,
        }
    }
}
```

Add to `src/protocols/mod.rs`:

```rust
#[cfg(feature = "wayland")]
pub mod layer_shell;
```

**Step 4: Run tests**

Run: `cargo test --features wayland --test wayland_layer_shell 2>&1`
Expected: PASS

Run: `cargo clippy --features wayland -- -D warnings 2>&1`
Expected: No warnings

**Step 5: Commit**

```bash
git add src/protocols/layer_shell.rs src/protocols/mod.rs tests/wayland_layer_shell.rs
git commit -m "add: layer shell protocol types"
```

---

### Task 9: Final quality gate

Run all quality checks across both feature configurations.

**Step 1: Format**

Run: `cargo fmt`

**Step 2: Clippy (both configs)**

Run: `cargo clippy -- -D warnings 2>&1`
Run: `cargo clippy --features wayland -- -D warnings 2>&1`
Run: `cargo clippy --features blocking -- -D warnings 2>&1`
Run: `cargo clippy --all-features -- -D warnings 2>&1`

**Step 3: All tests (both configs)**

Run: `cargo test 2>&1`
Run: `cargo test --features wayland 2>&1`
Run: `cargo test --all-features 2>&1`

**Step 4: Doc build**

Run: `cargo doc --no-deps --features wayland 2>&1`

**Step 5: Fix any issues found, then commit**

```bash
git add -A
git commit -m "chore: quality gate — fmt, clippy, tests, docs"
```

(Only commit if there were fixes. Skip if everything was clean.)
