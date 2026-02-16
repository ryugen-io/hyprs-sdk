use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hypr_sdk::ipc::responses::*;
use hypr_sdk::types::monitor::Monitor;
use hypr_sdk::types::window::Window;
use hypr_sdk::types::workspace::Workspace;

// -- Realistic JSON payloads -----------------------------------------------

const VERSION_JSON: &str = r#"{
    "branch": "main",
    "commit": "abc123def456",
    "version": "0.53.3",
    "dirty": false,
    "commit_message": "fix: trailing comma in devices output",
    "commit_date": "2025-06-15",
    "tag": "v0.53.3",
    "commits": "42",
    "buildAquamarine": "0.4.6",
    "buildHyprlang": "0.5.3",
    "buildHyprutils": "0.2.8",
    "buildHyprcursor": "0.1.10",
    "buildHyprgraphics": "0.1.2",
    "systemAquamarine": "0.4.6",
    "systemHyprlang": "0.5.3",
    "systemHyprutils": "0.2.8",
    "systemHyprcursor": "0.1.10",
    "systemHyprgraphics": "0.1.2",
    "abiHash": "deadbeefcafe1234",
    "flags": ["debug", "no xwayland", "systemd"]
}"#;

const WINDOW_JSON: &str = r#"{
    "address": "0x55a3f2c0dead",
    "pid": 12345,
    "class": "kitty",
    "title": "Terminal - zsh",
    "initialClass": "kitty",
    "initialTitle": "kitty",
    "at": [100, 50],
    "size": [1200, 800],
    "workspace": {"id": 3, "name": "3"},
    "monitor": 0,
    "mapped": true,
    "hidden": false,
    "floating": false,
    "pseudo": false,
    "pinned": false,
    "xwayland": false,
    "fullscreen": 0,
    "fullscreenClient": 0,
    "overFullscreen": false,
    "grouped": [],
    "tags": ["dev", "terminal"],
    "swallowing": "0x0",
    "focusHistoryID": 0,
    "inhibitingIdle": false,
    "xdgTag": "",
    "xdgDescription": "",
    "contentType": "none"
}"#;

const MONITOR_JSON: &str = r#"{
    "id": 0,
    "name": "DP-1",
    "description": "Samsung Electric Company 27\" (DP-1)",
    "make": "Samsung Electric Company",
    "model": "C27JG5x",
    "serial": "H4ZR500123",
    "width": 2560,
    "height": 1440,
    "physicalWidth": 597,
    "physicalHeight": 336,
    "refreshRate": 144.001007,
    "x": 0,
    "y": 0,
    "activeWorkspace": {"id": 1, "name": "1"},
    "specialWorkspace": {"id": 0, "name": ""},
    "reserved": [0, 38, 0, 0],
    "scale": 1.0,
    "transform": 0,
    "focused": true,
    "dpmsStatus": true,
    "vrr": false,
    "disabled": false,
    "solitary": "0x0",
    "solitaryBlockedBy": 0,
    "activelyTearing": false,
    "tearingBlockedBy": 0,
    "directScanoutTo": "0x0",
    "directScanoutBlockedBy": 0,
    "currentFormat": "DRM_FORMAT_XRGB8888",
    "mirrorOf": "",
    "availableModes": ["2560x1440@144.00Hz", "1920x1080@60.00Hz"],
    "colorManagementPreset": "sRGB",
    "sdrBrightness": 1.0,
    "sdrSaturation": 1.0,
    "sdrMinLuminance": 0.0,
    "sdrMaxLuminance": 200
}"#;

const WORKSPACE_JSON: &str = r#"{
    "id": 1,
    "name": "1",
    "monitor": "DP-1",
    "monitorID": 0,
    "windows": 4,
    "hasfullscreen": false,
    "lastwindow": "0x55a3f2c0dead",
    "lastwindowtitle": "Terminal - zsh",
    "ispersistent": false
}"#;

const DEVICES_JSON: &str = r#"{
    "mice": [
        {"address": "0x1234", "name": "Logitech G Pro", "defaultSpeed": 0.0, "scrollFactor": 1.0},
        {"address": "0x5678", "name": "SteelSeries Rival 600", "defaultSpeed": -0.5, "scrollFactor": 1.0}
    ],
    "keyboards": [
        {"address": "0xaaaa", "name": "AT Translated Set 2 keyboard", "rules": "evdev",
         "model": "pc105", "layout": "us", "variant": "", "options": "",
         "active_keymap": "English (US)", "capsLock": false, "numLock": true, "main": true}
    ],
    "tablets": [],
    "touch": [],
    "switches": [{"address": "0xbbbb", "name": "Lid Switch"}]
}"#;

const BINDS_JSON: &str = r#"[
    {"locked": false, "mouse": false, "release": false, "repeat": true, "longPress": false,
     "non_consuming": false, "has_description": true, "modmask": 64, "submap": "",
     "submap_universal": "", "key": "Return", "keycode": 0, "catch_all": false,
     "description": "Open terminal", "dispatcher": "exec", "arg": "kitty"},
    {"locked": false, "mouse": false, "release": false, "repeat": false, "longPress": false,
     "non_consuming": false, "has_description": false, "modmask": 64, "submap": "",
     "submap_universal": "", "key": "Q", "keycode": 0, "catch_all": false,
     "description": "", "dispatcher": "killactive", "arg": ""},
    {"locked": false, "mouse": false, "release": false, "repeat": true, "longPress": false,
     "non_consuming": false, "has_description": true, "modmask": 64, "submap": "",
     "submap_universal": "", "key": "1", "keycode": 0, "catch_all": false,
     "description": "Go to workspace 1", "dispatcher": "workspace", "arg": "1"}
]"#;

const CURSOR_JSON: &str = r#"{"x": 1280, "y": 720}"#;

const OPTION_JSON: &str = r#"{"option": "general:border_size", "int": 2, "set": true}"#;

fn bench_version(c: &mut Criterion) {
    c.bench_function("deser_version", |b| {
        b.iter(|| serde_json::from_str::<VersionInfo>(black_box(VERSION_JSON)).unwrap())
    });
}

fn bench_window(c: &mut Criterion) {
    c.bench_function("deser_window", |b| {
        b.iter(|| serde_json::from_str::<Window>(black_box(WINDOW_JSON)).unwrap())
    });
}

fn bench_monitor(c: &mut Criterion) {
    c.bench_function("deser_monitor", |b| {
        b.iter(|| serde_json::from_str::<Monitor>(black_box(MONITOR_JSON)).unwrap())
    });
}

fn bench_workspace(c: &mut Criterion) {
    c.bench_function("deser_workspace", |b| {
        b.iter(|| serde_json::from_str::<Workspace>(black_box(WORKSPACE_JSON)).unwrap())
    });
}

fn bench_devices(c: &mut Criterion) {
    c.bench_function("deser_devices", |b| {
        b.iter(|| serde_json::from_str::<DevicesResponse>(black_box(DEVICES_JSON)).unwrap())
    });
}

fn bench_binds(c: &mut Criterion) {
    c.bench_function("deser_binds_3", |b| {
        b.iter(|| serde_json::from_str::<Vec<Bind>>(black_box(BINDS_JSON)).unwrap())
    });
}

fn bench_cursor(c: &mut Criterion) {
    c.bench_function("deser_cursor", |b| {
        b.iter(|| serde_json::from_str::<CursorPosition>(black_box(CURSOR_JSON)).unwrap())
    });
}

fn bench_option_value(c: &mut Criterion) {
    c.bench_function("deser_option_value", |b| {
        b.iter(|| serde_json::from_str::<OptionValue>(black_box(OPTION_JSON)).unwrap())
    });
}

fn bench_window_address(c: &mut Criterion) {
    use hypr_sdk::types::common::WindowAddress;

    let mut group = c.benchmark_group("window_address_parse");
    group.bench_function("with_prefix", |b| {
        b.iter(|| {
            black_box("0x55a3f2c0dead")
                .parse::<WindowAddress>()
                .unwrap()
        })
    });
    group.bench_function("without_prefix", |b| {
        b.iter(|| black_box("55a3f2c0dead").parse::<WindowAddress>().unwrap())
    });
    group.finish();
}

fn bench_large_client_list(c: &mut Criterion) {
    // Simulate 20 windows (realistic desktop).
    let one_window = WINDOW_JSON;
    let many: String = format!(
        "[{}]",
        std::iter::repeat(one_window)
            .take(20)
            .collect::<Vec<_>>()
            .join(",")
    );

    c.bench_function("deser_20_windows", |b| {
        b.iter(|| serde_json::from_str::<Vec<Window>>(black_box(&many)).unwrap())
    });
}

criterion_group!(
    benches,
    bench_version,
    bench_window,
    bench_monitor,
    bench_workspace,
    bench_devices,
    bench_binds,
    bench_cursor,
    bench_option_value,
    bench_window_address,
    bench_large_client_list,
);
criterion_main!(benches);
