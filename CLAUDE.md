# hlrsgw -- Hyprland Rust SDK / Gateway

Rust SDK wrapping the full Hyprland API: IPC, Wayland protocols, plugin FFI, config, desktop types, hooks. Reference source: `.sources/Hyprland/` (read-only, v0.53.0). Always verify against the C++ source -- do not guess.

## What to build

### 1. IPC Client (Socket1 + Socket2)

Wrap both Unix domain sockets for external process communication with a running Hyprland instance.

**Socket1** (`.socket.sock`) -- request/response commands and queries. Reference: `.sources/Hyprland/src/debug/HyprCtl.cpp` for all command handlers and JSON response formats, `.sources/Hyprland/hyprctl/src/main.cpp` for the client-side wire protocol.

- Connect, write command, read response, close. One connection per request.
- Flag chars before `/` separator: `j` = JSON, `r` = reload, `a` = all, `c` = config. Space before `/` stops flag parsing.
- Batch: `[[BATCH]]cmd1;cmd2;cmd3`.
- Cover every registered command -- grep `registerCommand` in HyprCtl.cpp for the full list.
- Cover every dispatcher -- grep `m_dispatchers["` in `src/managers/KeybindManager.cpp`.
- Typed builders for commands, serde deserialization for JSON responses. No raw string APIs exposed to users.

**Socket2** (`.socket2.sock`) -- persistent event stream. Reference: `.sources/Hyprland/src/managers/EventManager.cpp`.

- Events arrive as `EVENTNAME>>DATA\n`. Data truncated to 1024 bytes, embedded newlines replaced with spaces.
- Server drops clients that queue >64 undelivered events.
- Cover every event -- grep all `postEvent(SHyprIPCEvent{` calls across the source.
- Strongly-typed event enum with parsed fields. Include an `Unknown` catch-all variant.

**Instance discovery**: scan `$XDG_RUNTIME_DIR/hypr/` for directories with `hyprland.lock` (PID on line 1, Wayland socket on line 2). Validate PID is alive. Support multiple instances.

### 2. Wayland Protocol Client Bindings

Hyprland implements 58 protocol handlers. Reference: `.sources/Hyprland/src/protocols/` for server-side implementations, `.sources/Hyprland/protocols/*.xml` for protocol XML definitions.

Build client-side Rust bindings for the protocols that are usable from external Wayland clients. Use `wayland-client` and `wayland-protocols` crates where standard protocol bindings exist. For Hyprland-specific protocols, generate bindings from the XML files.

Key protocols to prioritize (these are what external tools actually use):

- **wlr-layer-shell** -- panels, taskbars, overlays (LayerShell.cpp)
- **wlr-foreign-toplevel-management** -- taskbar window lists and control (ForeignToplevelWlr.cpp)
- **ext-foreign-toplevel-list** -- modern toplevel list (ForeignToplevel.cpp)
- **hyprland-toplevel-export** -- window content capture (ToplevelExport.cpp)
- **wlr-screencopy** -- screenshot capture (Screencopy.cpp)
- **hyprland-global-shortcuts** -- global keybind registration (GlobalShortcuts.cpp)
- **wlr-output-management** -- monitor configuration (OutputManagement.cpp)
- **wlr-output-power-management** -- DPMS control (OutputPower.cpp)
- **wlr-gamma-control** -- brightness/gamma adjustment (GammaControl.cpp)
- **ext-idle-notify** -- idle detection (IdleNotify.cpp)
- **idle-inhibit** -- prevent idle (IdleInhibit.cpp)
- **ext-session-lock** -- lock screen (SessionLock.cpp)
- **wlr-data-control** -- clipboard access (DataDeviceWlr.cpp)
- **wlr-virtual-pointer** -- synthetic mouse input (VirtualPointer.cpp)
- **virtual-keyboard** -- synthetic keyboard input (VirtualKeyboard.cpp)
- **hyprland-ctm-control** -- color transform matrix (CTMControl.cpp)
- **hyprland-surface** -- Hyprland surface extensions (HyprlandSurface.cpp)
- **ext-workspace** -- workspace management (ExtWorkspace.cpp)
- **hyprland-focus-grab** -- focus control (FocusGrab.cpp)
- **hyprland-pointer-warp** -- cursor warping (PointerWarp.cpp)

Don't try to bind all 58 at once. Provide the infrastructure (protocol generation from XML, connection management) and implement the most-used ones first. The rest can be added incrementally.

### 3. Plugin FFI (Writing Hyprland Plugins in Rust)

Provide safe Rust bindings for writing Hyprland plugins that get loaded as shared libraries into the compositor. Reference: `.sources/Hyprland/src/plugins/PluginAPI.hpp`, `HookSystem.hpp`, `PluginSystem.hpp`.

The C++ plugin API uses `extern "C"` functions in the `HyprlandAPI` namespace. Model:

**Plugin lifecycle**: `pluginAPIVersion()` -> `pluginInit(HANDLE)` -> `pluginExit()`

**Core API functions to wrap**:
- Config: `addConfigValue`, `getConfigValue`, `addConfigKeyword`
- Events: `registerCallbackDynamic` (subscribe to 48 hook events -- full list in `src/managers/HookSystemManager.hpp`, grep `EMIT_HOOK_EVENT`)
- Commands: `invokeHyprctlCommand`, `registerHyprCtlCommand`, `unregisterHyprCtlCommand`
- Dispatchers: `addDispatcherV2`, `removeDispatcher`
- Layouts: `addLayout`, `removeLayout`
- Decorations: `addWindowDecoration`, `removeWindowDecoration`
- Notifications: `addNotification`, `addNotificationV2`
- Function hooks: `createFunctionHook`, `removeFunctionHook`, `findFunctionsByName` (advanced/unstable)
- Metadata: `getHyprlandVersion`

Provide a `#[hyprland_plugin]` proc-macro or a safe registration macro that generates the extern "C" boilerplate. Users should write safe Rust and the SDK handles the ABI boundary.

### 4. Desktop Object Types

Model Hyprland's internal desktop objects as Rust types. Reference: `.sources/Hyprland/src/desktop/`.

- **Window** (CWindow / PHLWINDOW) -- position, size, class, title, workspace, monitor, fullscreen state, floating, pinned, grouped, tags, decorations. Read `src/desktop/view/Window.hpp`.
- **Workspace** (CWorkspace / PHLWORKSPACE) -- id, name, monitor, fullscreen mode, special workspace flag, visibility. Read `src/desktop/Workspace.hpp`.
- **Monitor** (CMonitor / PHLMONITOR) -- id, name, position, size, scale, refresh rate, transform, DPMS, VRR, 10-bit, color management. Read `src/helpers/Monitor.hpp`.
- **Layer Surface** (CLayerSurface / PHLLS) -- layer shell surfaces for panels/overlays. Read `src/desktop/view/LayerSurface.hpp`.
- **Popup**, **Subsurface**, **WLSurface** -- nested surface types.

These types serve dual purpose: deserialization targets for IPC JSON responses AND type definitions for the plugin API.

### 5. Config System Types

Model Hyprland's configuration types. Reference: `.sources/Hyprland/src/config/ConfigManager.hpp`, `ConfigValue.hpp`.

- Config option types: `Bool`, `Int`, `Float`, `StringShort`, `StringLong`, `Color`, `Choice`, `Gradient`, `Vector`
- Rule types: `SMonitorRule`, `SWorkspaceRule`, window rules, layer rules
- Plugin config lives under `plugin:` namespace

This is primarily for the plugin FFI (plugins register and read config values) but also useful for config file parsing tools.

### 6. Hook Event Types

Model all 48 hook events that plugins can subscribe to. Reference: `.sources/Hyprland/src/managers/HookSystemManager.hpp`, grep `EMIT_HOOK_EVENT` across the entire source.

Categories:
- Workspace/monitor: `workspace`, `createWorkspace`, `destroyWorkspace`, `moveWorkspace`, `focusedMon`, `monitorAdded`, `monitorRemoved`, `monitorLayoutChanged`, etc.
- Window: `openWindow`, `closeWindow`, `destroyWindow`, `moveWindow`, `fullscreen`, `changeFloatingMode`, `pin`, `windowTitle`, `activeWindow`, etc.
- Input: `keyPress` (cancellable), `mouseMove`, `mouseAxis`, `mouseButton`, gestures (swipe/pinch begin/update/end), `tabletTip`
- Rendering: `render` (with eRenderStage substages), `preRender`
- Lifecycle: `ready`, `tick`, `configReloaded`, `preConfigReload` (cancellable)
- Layer: `openLayer`, `closeLayer`

Each hook has specific data types passed via `std::any`. Map these to strongly-typed Rust enums.

## Module layout

```
src/
  lib.rs
  error.rs              -- HlrsError enum (thiserror)
  ipc/
    mod.rs
    socket.rs            -- low-level socket connect/send/recv
    client.rs            -- HyprlandClient with typed query/command methods
    commands.rs          -- command builders
    responses.rs         -- serde types for JSON responses
    events.rs            -- Socket2 event stream + typed event enum
    batch.rs             -- batch command builder
    instance.rs          -- instance discovery
  protocols/
    mod.rs               -- protocol infrastructure
    layer_shell.rs       -- wlr-layer-shell client
    foreign_toplevel.rs  -- toplevel management
    screencopy.rs        -- screenshot capture
    global_shortcuts.rs  -- keybind registration
    output_management.rs -- monitor config
    ...                  -- one module per protocol
  plugin/
    mod.rs               -- safe plugin API wrapper
    ffi.rs               -- raw extern "C" bindings
    hooks.rs             -- hook event types and registration
    lifecycle.rs         -- plugin init/exit macros
    config.rs            -- plugin config registration
    dispatcher.rs        -- custom dispatcher registration
    layout.rs            -- custom layout registration
    decoration.rs        -- custom window decoration
  types/
    mod.rs
    window.rs
    workspace.rs
    monitor.rs
    layer.rs
    common.rs            -- shared enums, newtypes, IDs
  config/
    mod.rs               -- config option types, rule types
  dispatch/
    mod.rs               -- typed dispatcher command builders
```

## Rust conventions

- Edition 2024. `#![forbid(unsafe_code)]` at crate root, except the `plugin/ffi.rs` module which needs `unsafe` for the C ABI boundary -- isolate it and add `// SAFETY:` comments.
- `thiserror` for errors. No `anyhow` in library code. No `unwrap()` outside tests.
- `tokio` for async (net, io-util features). Default API is async. Provide `blocking` feature flag for sync variants.
- `serde` + `serde_json` for IPC response deserialization. Use `#[serde(rename_all = "camelCase")]` and `#[serde(default)]`. Derive field names from the actual format strings in `HyprCtl.cpp`.
- `wayland-client` + `wayland-protocols` for protocol bindings. Generate from XML for Hyprland-specific protocols.
- Newtypes for IDs: `WindowAddress(u64)`, `WorkspaceId(i64)`, `MonitorId(i64)`.
- Idiomatic Rust naming, not C++ names. `ActiveWindow` not `activewindowv2`.
- Every public item gets a `///` doc comment with examples for key entry points.
- `#[must_use]` on pure functions and builders.
- Minimal dependency tree. Justify every new dep.

## Quality gates -- run before every commit

1. `cargo fmt`
2. `cargo clippy -- -D warnings` -- zero warnings
3. `cargo test`
4. `cargo doc --no-deps` -- zero warnings
5. `cargo test --doc`

## Verification

Before considering any module complete:
1. Grep the Hyprland source for all items in that domain and confirm full coverage.
2. Cross-reference serde field names against the actual JSON format strings in `HyprCtl.cpp`.
3. For protocol bindings, verify against the XML definitions.
4. For plugin FFI, verify function signatures against `PluginAPI.hpp`.

## Do NOT

- Shell out to `hyprctl`. Communicate via sockets directly.
- Hardcode socket paths. Always resolve from env vars + instance discovery.
- Skip APIs. If it exists in the Hyprland source, it gets a Rust binding.
- Expose raw string command building. Typed APIs only.
- Modify anything in `.sources/`.
- Look at or reference other projects in parent directories. This is standalone.
- Add a binary/CLI. This is a library only.
- Use `deny_unknown_fields` on serde types. Hyprland adds fields between versions -- ignore unknowns for forward compatibility.
