# Wayland Protocol Client Bindings — Design

## Goal

Add client-side Wayland protocol bindings to hypr-sdk so Rust programs can interact with Hyprland beyond IPC: capture screenshots, read/write clipboard, create layer-shell surfaces, register global shortcuts, build lock screens, send synthetic input, and more.

## Architecture

Three layers:

```
User Code
  ↓
protocols::<name>::high_level_api()          ← Typed Rust API (we build)
  ↓
wayland-client + generated protocol types     ← Generated from XML at compile time
  ↓
Wayland Unix Socket                           ← Managed by wayland-client
```

**WaylandConnection** is the entry point. Holds the `wayland_client::Connection`, `EventQueue`, and caches registry globals. All protocol modules receive `&WaylandConnection` to bind their interfaces.

No rendering built-in — the SDK provides protocol plumbing and surface management. Users bring their own rendering (wgpu, vulkan, cairo, software).

## Dependencies

| Crate | Purpose | Type |
|---|---|---|
| `wayland-client` | Connection, registry, event queue, object lifecycle | runtime |
| `wayland-protocols` | Bindings for standard ext- protocols (idle-notify, session-lock, etc.) | runtime |
| `wayland-protocols-wlr` | Bindings for wlr- protocols (layer-shell, screencopy, etc.) | runtime |
| `wayland-scanner` | Generate Rust code from XML for Hyprland-specific protocols | build |

## Module Structure

```
src/protocols/
  mod.rs                 — WaylandConnection, globals, re-exports
  layer_shell.rs         — wlr-layer-shell: panels, overlays, taskbars
  screencopy.rs          — wlr-screencopy: screenshot capture
  gamma_control.rs       — wlr-gamma-control: brightness/gamma
  output_management.rs   — wlr-output-management: monitor configuration
  output_power.rs        — wlr-output-power-management: DPMS control
  foreign_toplevel.rs    — wlr-foreign-toplevel-management: window list + control
  data_control.rs        — wlr-data-control: clipboard access
  virtual_pointer.rs     — wlr-virtual-pointer: synthetic mouse
  virtual_keyboard.rs    — virtual-keyboard: synthetic keyboard
  idle.rs                — ext-idle-notify + idle-inhibit
  session_lock.rs        — ext-session-lock: lock screen
  global_shortcuts.rs    — hyprland-global-shortcuts: keybind registration
  toplevel_export.rs     — hyprland-toplevel-export: window content capture
  ctm_control.rs         — hyprland-ctm-control: color transform matrix
  hyprland_surface.rs    — hyprland-surface: surface extensions
  focus_grab.rs          — hyprland-focus-grab: focus control
  pointer_warp.rs        — hyprland-pointer-warp: cursor warping
  ext_workspace.rs       — ext-workspace: workspace management
  ext_foreign_toplevel.rs — ext-foreign-toplevel-list: modern toplevel list
```

## XML Sources

**Available in Hyprland repo (12 files):** wlr-layer-shell, wlr-screencopy, wlr-gamma-control, wlr-output-management, wlr-output-power-management, wlr-foreign-toplevel-management, wlr-data-control, wlr-virtual-pointer, virtual-keyboard, wayland-drm, kde-server-decoration, input-method-v2.

**From wayland-protocols/wayland-protocols-wlr crates:** ext-idle-notify, idle-inhibit, ext-session-lock, ext-foreign-toplevel-list, ext-workspace. These crates already provide generated Rust bindings — no XML needed.

**Extract from C++ headers (6 Hyprland-specific):** hyprland-toplevel-export, hyprland-global-shortcuts, hyprland-ctm-control, hyprland-surface, hyprland-focus-grab, hyprland-pointer-warp. Derive minimal XML definitions from the generated C++ protocol headers in `src/protocols/`.

## Phasing

### Phase 1 — Infrastructure + WLR protocols (XML available)

WaylandConnection, registry globals, build.rs scanner setup. Then: layer-shell, screencopy, gamma-control, output-management, output-power.

### Phase 2 — Standard + WLR protocols (from crates)

foreign-toplevel, data-control, virtual-pointer, virtual-keyboard, idle (notify + inhibit), session-lock.

### Phase 3 — Hyprland-specific protocols (extract XMLs)

global-shortcuts, toplevel-export, ctm-control, hyprland-surface, focus-grab, pointer-warp, ext-workspace, ext-foreign-toplevel.

## Error Handling

New variants in `HyprError`:

```rust
Wayland(wayland_client::ConnectError),
WaylandDispatch(wayland_client::DispatchError),
ProtocolNotSupported(String),
```

## Testing

Protocol modules are hard to unit-test without a running compositor. Strategy:

- **Type/construction tests** — verify API types and builders compile and construct correctly
- **Integration tests** — `#[ignore]` tests that require a running Hyprland, run manually or in CI with a headless Hyprland instance
- **Mock tests** where feasible — mock the Wayland socket for connection/registry logic

## Non-Goals

- No rendering engine. Users bring their own.
- No widget toolkit or application framework.
- No reimplementation of wayland-client wire protocol.
