# hyprs-sdk

Comprehensive Rust SDK for the [Hyprland](https://hyprland.org/) compositor.

`hyprs-sdk` combines IPC, typed dispatchers, event parsing, desktop data models, Wayland protocol clients, and plugin FFI bindings in one crate.

Version detected automatically from system via `pkg-config` at build time.

## Installation

```toml
[dependencies]
hyprs-sdk = "0.1.0"
```

Optional features:

```toml
[dependencies]
hyprs-sdk = { version = "0.1.0", features = ["blocking", "wayland", "plugin-ffi"] }
```

## Quick Start (Async IPC)

```rust,no_run
use hyprs_sdk::dispatch::{self, Direction};
use hyprs_sdk::ipc::{Event, EventStream, Flags, HyprlandClient};

#[tokio::main]
async fn main() -> hyprs_sdk::HyprResult<()> {
    let client = HyprlandClient::current()?;

    // Raw query
    let _monitors_text = client.monitors(Flags::default()).await?;

    // Typed query
    let monitors = client.monitors_typed().await?;
    for m in &monitors {
        println!("{}: {}x{}", m.name, m.width, m.height);
    }

    // Typed dispatchers
    client.dispatch_cmd(dispatch::workspace::switch("3")).await?;
    client
        .dispatch_cmd(dispatch::focus::move_focus(Direction::Left))
        .await?;

    // Event stream
    let socket2 = client.event_stream().await?;
    let mut events = EventStream::new(socket2);

    while let Some(event) = events.next_event().await? {
        match event {
            Event::Workspace { name } => println!("workspace: {name}"),
            Event::ActiveWindowV2 { address } => println!("window: {address}"),
            _ => {}
        }
    }

    Ok(())
}
```

## Feature Overview

- **IPC (`hyprctl` over Socket1 + Socket2)**
  - All IPC command builders
  - Async `HyprlandClient`
  - Typed JSON helpers (`*_typed` methods)
  - Blocking client via `blocking` feature
- **HyprPM wrapper**
  - `hyprs_sdk::hyprpm::HyprPm` for plugin repo lifecycle commands (`add/remove/enable/disable/update/list/reload/purge-cache`)
- **Typed dispatchers**
  - 72 dispatcher builders under `hyprs_sdk::dispatch::*`
- **Typed events**
  - Parsed Socket2 events via `Event` + `EventStream`
  - Unknown events preserved as `Event::Unknown`
- **Desktop types**
  - `Window`, `Workspace`, `Monitor`, `LayerSurface`
  - Shared newtypes/enums: `WindowAddress`, `WorkspaceId`, `MonitorId`, etc.
- **Config model types**
  - Config option descriptors and monitor/workspace/window/layer rule types
- **Wayland protocol clients** (`wayland` feature)
  - Connection/discovery + Hyprland/wlr protocol wrappers
  - Includes protocols like layer-shell, screencopy, session-lock, virtual keyboard/pointer, output management, and more
- **Plugin API bindings** (`plugin-ffi` feature)
  - Lifecycle macro (`hyprland_plugin!`)
  - Hooks, custom dispatchers, hyprctl commands, notifications
  - Layout and decoration registration APIs

## Feature Flags

- `blocking`: enables synchronous IPC client (`ipc::BlockingClient`)
- `wayland`: enables Wayland protocol modules under `hyprs_sdk::protocols`
- `plugin-ffi`: enables C++ bridge-backed plugin API integration

Default features are empty.

## Additional Usage

Blocking IPC client:

```rust,ignore
use hyprs_sdk::ipc::BlockingClient;

fn main() -> hyprs_sdk::HyprResult<()> {
    let client = BlockingClient::current()?;
    let version = client.version_typed()?;
    println!("{} ({})", version.tag, version.hash);
    Ok(())
}
```

HyprPM wrapper:

```rust,no_run
use hyprs_sdk::hyprpm::HyprPm;

fn main() -> hyprs_sdk::HyprResult<()> {
    let pm = HyprPm::new();
    let out = pm.list()?;
    println!("{}", out.stdout);
    Ok(())
}
```

Wayland protocol client (`wayland` feature):

```rust,ignore
use hyprs_sdk::protocols::connection::WaylandConnection;

fn main() -> hyprs_sdk::HyprResult<()> {
    let wl = WaylandConnection::connect()?;
    for g in wl.globals() {
        println!("{} v{}", g.interface, g.version);
    }
    Ok(())
}
```

Plugin lifecycle macro (`plugin-ffi` feature):

```rust,ignore
use hyprs_sdk::plugin::*;

fn init(_handle: PluginHandle) -> Result<PluginDescription, String> {
    Ok(PluginDescription {
        name: "example-plugin".into(),
        description: "example".into(),
        author: "you".into(),
        version: "0.1.0".into(),
    })
}

fn exit() {}

hyprland_plugin! {
    init: init,
    exit: exit,
}
```

## Requirements

- Rust **nightly** (`edition = 2024`)
- Hyprland runtime for IPC/Wayland operations
- For `plugin-ffi`:
  - Hyprland development headers available via `pkg-config`
  - C++ toolchain with C++2b support

## Quality Commands

```bash
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo bench --no-run
```

Live integration checks (requires a running Hyprland session):

```bash
export HYPRLAND_INSTANCE_SIGNATURE="<your-signature>"
scripts/live-ipc-smoke.sh
scripts/live-full-smoke.sh
scripts/live-plugin-e2e.sh
```

Optional live-smoke toggles:

```bash
# visible on-screen marker (notify + temporary error banner)
HYPR_SDK_SMOKE_VISUAL=1 scripts/live-full-smoke.sh

# include extra mutating checks (reload/reloadshaders/forcerendererreload)
HYPR_SDK_SMOKE_MUTATING=1 scripts/live-full-smoke.sh

# also run plugin load/unload live test
HYPR_SDK_INCLUDE_PLUGIN_SMOKE=1 scripts/live-full-smoke.sh
```

`scripts/live-full-smoke.sh` runs:
- baseline live IPC smoke (`tests/live_ipc_smoke.rs`)
- full read/set/dispatch smoke (`tests/live_full_smoke.rs`)
- SDK vs `hyprctl` JSON parity smoke (`tests/live_cli_parity.rs`)

`scripts/live-plugin-e2e.sh` builds a minimal C++ plugin fixture and requires
Hyprland headers (via `pkg-config hyprland`) plus a C++ toolchain.

Dedicated CI workflow for this exists at `.github/workflows/live-hyprland.yml`
and targets a self-hosted runner labeled `linux` + `hyprland`.

Fuzzing (optional):

```bash
cargo install cargo-fuzz
cd fuzz
cargo fuzz run fuzz_event_parse -- -max_total_time=30
cargo fuzz run fuzz_json_responses -- -max_total_time=30
cargo fuzz run fuzz_window_address -- -max_total_time=30
cargo fuzz run fuzz_command_building -- -max_total_time=30
```

## License

MIT
