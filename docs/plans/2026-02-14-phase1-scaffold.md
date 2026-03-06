# Phase 1: Project Scaffold Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Set up the hyprs-sdk project skeleton so it compiles, has all module stubs, and is ready for feature implementation.

**Architecture:** Cargo library crate with edition 2024, async-first (tokio), module tree matching the design doc. All modules start as stubs with doc comments explaining their purpose. Tests live in `/tests` directory (integration test style).

**Tech Stack:** Rust nightly 1.94, tokio, thiserror, serde, serde_json

---

### Task 1: Initialize Git + Cargo Project

**Files:**
- Create: `Cargo.toml`
- Create: `.gitignore`
- Create: `rust-toolchain.toml`

**Step 1: Initialize git repo**

Run: `cd /code/git/ryugen-io/projects/rust/hyprland-rs/hyprs-sdk && git init`

**Step 2: Create rust-toolchain.toml**

```toml
[toolchain]
channel = "nightly"
```

**Step 3: Create .gitignore**

```gitignore
/target
.sources/Hyprland/
.sources/ratatui/
.sources/sysrat-rs.bak/
.sources/tachyonfx/
```

**Step 4: Create Cargo.toml**

```toml
[package]
name = "hyprs-sdk"
version = "0.1.0"
edition = "2024"
description = "Comprehensive Rust SDK for the Hyprland compositor"
license = "MIT"
repository = "https://github.com/ryugen-io/hyprs-sdk"

[dependencies]
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["net", "io-util", "rt", "macros"] }

[features]
default = []
blocking = []
```

**Step 5: Create empty src/lib.rs so cargo check works**

```rust
// placeholder
```

**Step 6: Verify**

Run: `cargo check`
Expected: compiles with no errors

**Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock .gitignore rust-toolchain.toml src/lib.rs CLAUDE.md docs/
git commit -m "init: hyprs-sdk project scaffold"
```

---

### Task 2: Create update-sources.sh Script

**Files:**
- Create: `scripts/update-sources.sh`
- Create: `.sources/.version`

**Step 1: Create the script**

```bash
#!/usr/bin/env bash
set -euo pipefail

REPO="https://github.com/hyprwm/Hyprland.git"
TARGET_DIR=".sources/Hyprland"
VERSION="${1:-v0.53.0}"

if [ -d "$TARGET_DIR/.git" ]; then
    echo "Updating Hyprland source to $VERSION..."
    cd "$TARGET_DIR"
    git fetch --tags
    git checkout "$VERSION"
    cd - > /dev/null
else
    echo "Cloning Hyprland source at $VERSION..."
    git clone --depth 1 --branch "$VERSION" "$REPO" "$TARGET_DIR"
fi

echo "$VERSION" > .sources/.version
echo "Hyprland source ready at $VERSION"
```

**Step 2: Make executable**

Run: `chmod +x scripts/update-sources.sh`

**Step 3: Create .sources/.version**

Contents: `v0.53.0`

**Step 4: Commit**

```bash
git add scripts/update-sources.sh .sources/.version
git commit -m "add: update-sources.sh for Hyprland source tracking"
```

---

### Task 3: Create src/error.rs

**Files:**
- Create: `src/error.rs`
- Create: `tests/error.rs`

**Step 1: Write the failing tests**

Create `tests/error.rs`:

```rust
use hyprs_sdk::error::HyprError;

#[test]
fn error_display_io() {
    let err = HyprError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "socket missing"));
    assert!(err.to_string().contains("socket missing"));
}

#[test]
fn error_display_parse() {
    let err = HyprError::Parse("bad json".into());
    assert!(err.to_string().contains("bad json"));
}

#[test]
fn error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken");
    let err: HyprError = io_err.into();
    assert!(matches!(err, HyprError::Io(_)));
}

#[test]
fn error_from_serde() {
    let json_err = serde_json::from_str::<String>("not json").unwrap_err();
    let err: HyprError = json_err.into();
    assert!(matches!(err, HyprError::Json(_)));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test error`
Expected: FAIL — `src/error.rs` doesn't exist yet

**Step 3: Write the implementation**

Create `src/error.rs`:

```rust
/// Errors returned by hyprs-sdk operations.
#[derive(Debug, thiserror::Error)]
pub enum HyprError {
    /// I/O error (socket connection, read, write).
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON deserialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Failed to parse a response or event from Hyprland.
    #[error("parse error: {0}")]
    Parse(String),

    /// Command rejected by Hyprland.
    #[error("command failed: {0}")]
    Command(String),

    /// No running Hyprland instance found.
    #[error("no hyprland instance found")]
    NoInstance,

    /// Instance with given signature not found.
    #[error("instance not found: {0}")]
    InstanceNotFound(String),
}

/// Convenience result type for hyprs-sdk.
pub type HyprResult<T> = std::result::Result<T, HyprError>;
```

Update `src/lib.rs` to export the module:

```rust
#![forbid(unsafe_code)]

pub mod error;

pub use error::{HyprError, HyprResult};
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --test error`
Expected: 4 tests PASS

**Step 5: Commit**

```bash
git add src/error.rs src/lib.rs tests/error.rs
git commit -m "add: HyprError type with thiserror"
```

---

### Task 4: Create src/types/common.rs with Newtypes

**Files:**
- Create: `src/types/mod.rs`
- Create: `src/types/common.rs`
- Create: `tests/types_common.rs`

**Step 1: Write failing tests**

Create `tests/types_common.rs`:

```rust
use hyprs_sdk::types::common::{MonitorId, WindowAddress, WorkspaceId};

#[test]
fn window_address_from_hex_string() {
    let addr: WindowAddress = "0x55a3f2c0".parse().unwrap();
    assert_eq!(addr.0, 0x55a3f2c0);
}

#[test]
fn window_address_display_hex() {
    let addr = WindowAddress(0x55a3f2c0);
    assert_eq!(addr.to_string(), "0x55a3f2c0");
}

#[test]
fn window_address_serde_roundtrip() {
    let addr = WindowAddress(0xdead);
    let json = serde_json::to_string(&addr).unwrap();
    assert_eq!(json, "\"0xdead\"");
    let back: WindowAddress = serde_json::from_str(&json).unwrap();
    assert_eq!(back, addr);
}

#[test]
fn workspace_id_serde() {
    let id = WorkspaceId(3);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "3");
    let back: WorkspaceId = serde_json::from_str(&json).unwrap();
    assert_eq!(back, id);
}

#[test]
fn monitor_id_serde() {
    let id = MonitorId(0);
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "0");
}

#[test]
fn workspace_id_special() {
    let special = WorkspaceId::SPECIAL;
    assert!(special.is_special());
    assert!(!WorkspaceId(1).is_special());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --test types_common`
Expected: FAIL — module doesn't exist yet

**Step 3: Write implementation**

Create `src/types/common.rs`:

```rust
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Unique address of a Hyprland window (hex pointer value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowAddress(pub u64);

impl fmt::Display for WindowAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

impl FromStr for WindowAddress {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.strip_prefix("0x").unwrap_or(s);
        u64::from_str_radix(hex, 16).map(WindowAddress)
    }
}

impl Serialize for WindowAddress {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for WindowAddress {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Workspace identifier. Negative values are special workspaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct WorkspaceId(pub i64);

impl WorkspaceId {
    /// The special (scratchpad) workspace base ID.
    pub const SPECIAL: Self = Self(-99);

    /// Returns true if this is a special workspace (negative ID).
    #[must_use]
    pub fn is_special(self) -> bool {
        self.0 < 0
    }
}

impl fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Monitor identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub struct MonitorId(pub i64);

impl fmt::Display for MonitorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

Create `src/types/mod.rs`:

```rust
pub mod common;

pub use common::{MonitorId, WindowAddress, WorkspaceId};
```

Update `src/lib.rs`:

```rust
#![forbid(unsafe_code)]

pub mod error;
pub mod types;

pub use error::{HyprError, HyprResult};
pub use types::common::{MonitorId, WindowAddress, WorkspaceId};
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --test types_common`
Expected: 6 tests PASS

**Step 5: Commit**

```bash
git add src/types/ src/lib.rs tests/types_common.rs
git commit -m "add: common newtypes (WindowAddress, WorkspaceId, MonitorId)"
```

---

### Task 5: Create lib.rs Final Form + README

**Files:**
- Modify: `src/lib.rs`
- Create: `README.md`

**Step 1: Create a minimal README.md**

```markdown
# hyprs-sdk

Comprehensive Rust SDK for the Hyprland compositor.

Covers IPC (Socket1 + Socket2), Wayland protocol bindings, plugin FFI, desktop types, config types, and hook events.
```

**Step 2: Update lib.rs**

```rust
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

/// Target Hyprland version this SDK was verified against.
pub const HYPRLAND_TARGET_VERSION: &str = "0.53.0";

pub mod error;
pub mod types;

// Stubs — uncommented as implementation proceeds
// pub mod config;
// pub mod dispatch;
// pub mod ipc;
// pub mod plugin;
// pub mod protocols;

pub use error::{HyprError, HyprResult};
pub use types::common::{MonitorId, WindowAddress, WorkspaceId};
```

**Step 3: Verify**

Run: `cargo test && cargo clippy -- -D warnings && cargo doc --no-deps`
Expected: all pass, no warnings

**Step 4: Commit**

```bash
git add src/lib.rs README.md
git commit -m "add: lib.rs with doc include and version constant"
```

---

### Task 6: Create Stub Modules for Future Phases

**Files:**
- Create: `src/ipc/mod.rs`
- Create: `src/config/mod.rs`
- Create: `src/dispatch/mod.rs`
- Create: `src/plugin/mod.rs`
- Create: `src/protocols/mod.rs`
- Create: `src/types/window.rs`, `workspace.rs`, `monitor.rs`, `layer.rs`

**Step 1: Create all stub files**

Each stub module gets a single doc comment explaining its purpose.

`src/ipc/mod.rs`:
```rust
//! IPC client for communicating with a running Hyprland instance.
//!
//! Covers Socket1 (request/response) and Socket2 (event stream).
```

`src/config/mod.rs`:
```rust
//! Hyprland configuration types.
//!
//! Config option types, monitor rules, workspace rules, window rules.
```

`src/dispatch/mod.rs`:
```rust
//! Typed dispatcher command builders.
//!
//! One method per Hyprland dispatcher, with strongly-typed arguments.
```

`src/plugin/mod.rs`:
```rust
//! Plugin FFI for writing Hyprland plugins in Rust.
//!
//! This module contains unsafe code at the FFI boundary.
```

`src/protocols/mod.rs`:
```rust
//! Wayland protocol client bindings.
//!
//! Client-side bindings for Hyprland-specific and wlr protocols.
```

`src/types/window.rs`:
```rust
//! Window type — desktop window representation.
//!
//! Deserialization target for IPC JSON and type definition for plugin API.
```

`src/types/workspace.rs`:
```rust
//! Workspace type — virtual desktop representation.
```

`src/types/monitor.rs`:
```rust
//! Monitor type — physical output representation.
```

`src/types/layer.rs`:
```rust
//! Layer surface type — panels, overlays, backgrounds.
```

**Step 2: Update lib.rs to declare all modules**

```rust
#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

/// Target Hyprland version this SDK was verified against.
pub const HYPRLAND_TARGET_VERSION: &str = "0.53.0";

pub mod config;
pub mod dispatch;
pub mod error;
pub mod ipc;
pub mod plugin;
pub mod protocols;
pub mod types;

pub use error::{HyprError, HyprResult};
pub use types::common::{MonitorId, WindowAddress, WorkspaceId};
```

Update `src/types/mod.rs`:

```rust
pub mod common;
pub mod layer;
pub mod monitor;
pub mod window;
pub mod workspace;

pub use common::{MonitorId, WindowAddress, WorkspaceId};
```

**Step 3: Verify**

Run: `cargo test && cargo clippy -- -D warnings && cargo doc --no-deps`
Expected: all pass, no warnings

**Step 4: Commit**

```bash
git add src/
git commit -m "add: stub modules for all planned components"
```

---

### Task 7: Final Verification + Tag

**Step 1: Run full quality gates**

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test && cargo doc --no-deps && cargo test --doc
```

Expected: all green

**Step 2: Verify project structure**

Run: `find src -name '*.rs' | sort`
Expected output:
```
src/config/mod.rs
src/dispatch/mod.rs
src/error.rs
src/ipc/mod.rs
src/lib.rs
src/plugin/mod.rs
src/protocols/mod.rs
src/types/common.rs
src/types/layer.rs
src/types/mod.rs
src/types/monitor.rs
src/types/window.rs
src/types/workspace.rs
```

Also verify test structure:
Run: `find tests -name '*.rs' | sort`
Expected:
```
tests/error.rs
tests/types_common.rs
```

**Step 3: Tag**

```bash
git tag phase1-scaffold
```

---

## Summary

After Phase 1, the project has:
- Compiling Cargo project with all dependencies
- Error type with 6 variants, tested in `tests/error.rs`
- 3 ID newtypes with serde + Display + FromStr, tested in `tests/types_common.rs`
- All module stubs in place, ready for Phase 2 (types/ implementation)
- Source tracking script for Hyprland updates
- Clean git history with atomic commits
- All tests in `/tests` directory (integration test style)
