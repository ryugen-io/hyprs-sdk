# hyprs-sdk Skeleton Design

## Crate

- **Name**: `hyprs-sdk` (`use hyprs_sdk::...`)
- **Edition**: 2024, Rust nightly 1.94
- **Type**: Library only, no binary

## Dependencies

| Crate | Purpose |
|-------|---------|
| `thiserror` 2 | Error types |
| `serde` 1 + `serde_json` 1 | IPC JSON deserialization |
| `tokio` 1 (net, io-util, rt, macros) | Async runtime |

Additional deps added later per module (wayland-client for protocols, proc-macro2/syn/quote for plugin macros).

## Module Layout

```
src/
  lib.rs               -- #![forbid(unsafe_code)], module re-exports
  error.rs             -- HyprSdkError enum
  ipc/
    mod.rs
    socket.rs          -- low-level socket connect/send/recv
    client.rs          -- HyprlandClient with typed methods
    commands.rs        -- command builders
    responses.rs       -- serde types for JSON responses
    events.rs          -- Socket2 event stream + typed event enum
    batch.rs           -- batch command builder
    instance.rs        -- instance discovery
  types/
    mod.rs
    common.rs          -- WindowAddress(u64), WorkspaceId(i64), MonitorId(i64), shared enums
    window.rs
    workspace.rs
    monitor.rs
    layer.rs
  config/
    mod.rs             -- config option types, rule types
  dispatch/
    mod.rs             -- typed dispatcher command builders
  plugin/
    mod.rs             -- safe plugin API wrapper
    ffi.rs             -- #[allow(unsafe_code)] raw extern "C" bindings
    hooks.rs           -- hook event types and registration
    lifecycle.rs       -- plugin init/exit macros
    config.rs          -- plugin config registration
    dispatcher.rs      -- custom dispatcher registration
    layout.rs          -- custom layout registration
    decoration.rs      -- custom window decoration
  protocols/
    mod.rs             -- protocol infrastructure + per-protocol modules
```

## Key Design Decisions

1. `#![forbid(unsafe_code)]` at crate root; only `plugin/ffi.rs` gets `#[allow(unsafe_code)]`
2. Newtypes for IDs: `WindowAddress(u64)`, `WorkspaceId(i64)`, `MonitorId(i64)`
3. Forward-compat: `#[serde(default)]`, no `deny_unknown_fields`, `Unknown` catch-all variants
4. Async-first (tokio), `blocking` feature flag for sync wrappers
5. `pub const HYPRLAND_TARGET_VERSION: &str = "0.53.0";`

## Hyprland Source Tracking

- Script-based: `scripts/update-sources.sh` clones/updates to a specific tag
- `.sources/Hyprland/` gitignored, `.sources/.version` tracked
- Only latest version supported, forward-compatible types

## Build Order

1. Project scaffold + `error.rs` + `types/common.rs`
2. `types/` complete (Window, Workspace, Monitor, Layer)
3. `ipc/` (Socket1+2, Events, Instance Discovery)
4. `dispatch/` (Typed Dispatcher Commands)
5. `config/` + Hook Event Types
6. `plugin/` (FFI, Macros, safe Wrapper)
7. `protocols/` (Wayland Protocol Bindings)
