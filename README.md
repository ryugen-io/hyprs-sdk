# hypr-sdk

Comprehensive Rust SDK for the [Hyprland](https://hyprland.org/) compositor.

Covers IPC (Socket1 + Socket2), typed dispatchers, desktop object types, and more.
Verified against Hyprland **v0.53.0**.

## Quick Start

```toml
[dependencies]
hypr-sdk = "0.1.0"
```

```rust,no_run
use hypr_sdk::ipc::{HyprlandClient, Flags, Event, EventStream};
use hypr_sdk::dispatch::{self, Direction};

#[tokio::main]
async fn main() -> hypr_sdk::HyprResult<()> {
    let client = HyprlandClient::current()?;

    // Query monitors (plain text)
    let text = client.monitors(Flags::default()).await?;

    // Query monitors (JSON, deserialized into typed structs)
    let monitors = client.monitors_typed().await?;
    for m in &monitors {
        println!("{}: {}x{}", m.name, m.width, m.height);
    }

    // Dispatch a command
    client.dispatch_cmd(dispatch::workspace::switch("3")).await?;
    client.dispatch_cmd(dispatch::focus::move_focus(Direction::Left)).await?;

    // Listen to events
    let stream = client.event_stream().await?;
    let mut events = EventStream::new(stream);
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

## Features

### IPC Client

Full Socket1 (request/response) and Socket2 (event stream) support.

- **All 37 IPC commands** with configurable output flags (JSON, plain text, reload, all, config)
- **Raw and typed APIs** — use `client.monitors(Flags::default())` for text or `client.monitors_typed()` for deserialized structs
- **Action commands** — `dispatch`, `keyword`, `reload`, `kill`, `notify`, `set_cursor`, etc.
- **Batch** — send multiple commands in one request
- **Async by default** (tokio), blocking variant behind `blocking` feature flag

### Typed Dispatchers

All 72 Hyprland dispatchers with strongly-typed arguments, split by domain:

| Module | Dispatchers |
|---|---|
| `dispatch::exec` | `exec`, `execr`, `exit` |
| `dispatch::window` | `kill_active`, `close_window`, `toggle_floating`, `pin`, `fullscreen`, `set_prop`, ... |
| `dispatch::focus` | `move_focus`, `focus_window`, `focus_monitor`, `cycle_next`, ... |
| `dispatch::movement` | `move_window`, `resize_active`, `move_to_workspace`, `move_cursor`, ... |
| `dispatch::workspace` | `switch`, `rename`, `toggle_special`, `move_to_monitor`, `swap_active_workspaces`, ... |
| `dispatch::group` | `toggle_group`, `lock_groups`, `move_into_group`, `move_window_or_group`, ... |
| `dispatch::layout` | `pseudo`, `toggle_split`, `split_ratio`, `layout_msg` |
| `dispatch::input` | `submap`, `dpms`, `send_shortcut`, `pass`, `global`, `mouse` |
| `dispatch::misc` | `force_renderer_reload`, `event`, `force_idle` |

### Event Stream

All 43 Socket2 events parsed into a strongly-typed `Event` enum, including v2 variants. Unknown events are captured in `Event::Unknown` for forward compatibility.

### Desktop Types

Full desktop object types derived from the Hyprland C++ source, not just IPC JSON:

- **Window** — 33 fields (identity, geometry, state, fullscreen, groups, tags, XDG metadata, plugin-only)
- **Workspace** — 15 fields (IPC + plugin)
- **Monitor** — 30 fields (resolution, position, workspaces, display settings, color management, plugin-only)
- **LayerSurface** — 9 fields (position, size, namespace, plugin-only)
- Newtypes: `WindowAddress`, `WorkspaceId`, `MonitorId`
- Enums: `FullscreenMode`, `OutputTransform`, `Layer`, `ContentType`

### Instance Discovery

Scan `$XDG_RUNTIME_DIR/hypr/` for running Hyprland instances, validate PIDs, resolve socket paths.

## Planned

- **Config types** — config option types, monitor/workspace/window rules
- **Plugin FFI** — safe Rust bindings for writing Hyprland plugins
- **Wayland protocol bindings** — client-side bindings for Hyprland-specific and wlr protocols

## Requirements

- Rust nightly (edition 2024)
- A running Hyprland instance (for IPC)

## License

MIT
