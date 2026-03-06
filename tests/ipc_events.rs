use hyprs_sdk::ipc::events::{Event, parse_event};
use hyprs_sdk::types::common::{WindowAddress, WorkspaceId};

#[test]
fn parse_workspace_event() {
    let ev = parse_event("workspace>>main").unwrap();
    assert_eq!(
        ev,
        Event::Workspace {
            name: "main".into()
        }
    );
}

#[test]
fn parse_workspace_v2_event() {
    let ev = parse_event("workspacev2>>3,main").unwrap();
    assert_eq!(
        ev,
        Event::WorkspaceV2 {
            id: WorkspaceId(3),
            name: "main".into()
        }
    );
}

#[test]
fn parse_create_workspace() {
    let ev = parse_event("createworkspace>>dev").unwrap();
    assert_eq!(ev, Event::CreateWorkspace { name: "dev".into() });
}

#[test]
fn parse_destroy_workspace_v2() {
    let ev = parse_event("destroyworkspacev2>>5,work").unwrap();
    assert_eq!(
        ev,
        Event::DestroyWorkspaceV2 {
            id: WorkspaceId(5),
            name: "work".into()
        }
    );
}

#[test]
fn parse_move_workspace() {
    let ev = parse_event("moveworkspace>>main,DP-1").unwrap();
    assert_eq!(
        ev,
        Event::MoveWorkspace {
            name: "main".into(),
            monitor: "DP-1".into()
        }
    );
}

#[test]
fn parse_move_workspace_v2() {
    let ev = parse_event("moveworkspacev2>>2,main,DP-1").unwrap();
    assert_eq!(
        ev,
        Event::MoveWorkspaceV2 {
            id: WorkspaceId(2),
            name: "main".into(),
            monitor: "DP-1".into()
        }
    );
}

#[test]
fn parse_rename_workspace() {
    let ev = parse_event("renameworkspace>>1,dev").unwrap();
    assert_eq!(
        ev,
        Event::RenameWorkspace {
            id: WorkspaceId(1),
            new_name: "dev".into()
        }
    );
}

#[test]
fn parse_focused_mon() {
    let ev = parse_event("focusedmon>>DP-1,main").unwrap();
    assert_eq!(
        ev,
        Event::FocusedMon {
            monitor: "DP-1".into(),
            workspace: "main".into()
        }
    );
}

#[test]
fn parse_monitor_added() {
    let ev = parse_event("monitoradded>>HDMI-A-1").unwrap();
    assert_eq!(
        ev,
        Event::MonitorAdded {
            name: "HDMI-A-1".into()
        }
    );
}

#[test]
fn parse_monitor_added_v2() {
    let ev = parse_event("monitoraddedv2>>1,DP-2,Dell U2723QE").unwrap();
    assert_eq!(
        ev,
        Event::MonitorAddedV2 {
            id: "1".into(),
            name: "DP-2".into(),
            description: "Dell U2723QE".into()
        }
    );
}

#[test]
fn parse_active_window() {
    let ev = parse_event("activewindow>>kitty,~").unwrap();
    assert_eq!(
        ev,
        Event::ActiveWindow {
            class: "kitty".into(),
            title: "~".into()
        }
    );
}

#[test]
fn parse_active_window_v2() {
    let ev = parse_event("activewindowv2>>55a3f2c0").unwrap();
    assert_eq!(
        ev,
        Event::ActiveWindowV2 {
            address: WindowAddress(0x55a3f2c0)
        }
    );
}

#[test]
fn parse_open_window() {
    let ev = parse_event("openwindow>>abcdef,1,kitty,fish").unwrap();
    assert_eq!(
        ev,
        Event::OpenWindow {
            address: WindowAddress(0xabcdef),
            workspace: "1".into(),
            class: "kitty".into(),
            title: "fish".into()
        }
    );
}

#[test]
fn parse_close_window() {
    let ev = parse_event("closewindow>>abcdef").unwrap();
    assert_eq!(
        ev,
        Event::CloseWindow {
            address: WindowAddress(0xabcdef)
        }
    );
}

#[test]
fn parse_window_title_v2() {
    let ev = parse_event("windowtitlev2>>abcdef,new title").unwrap();
    assert_eq!(
        ev,
        Event::WindowTitleV2 {
            address: WindowAddress(0xabcdef),
            title: "new title".into()
        }
    );
}

#[test]
fn parse_fullscreen() {
    assert_eq!(
        parse_event("fullscreen>>1").unwrap(),
        Event::Fullscreen { enabled: true }
    );
    assert_eq!(
        parse_event("fullscreen>>0").unwrap(),
        Event::Fullscreen { enabled: false }
    );
}

#[test]
fn parse_change_floating_mode() {
    let ev = parse_event("changefloatingmode>>abcdef,1").unwrap();
    assert_eq!(
        ev,
        Event::ChangeFloatingMode {
            address: WindowAddress(0xabcdef),
            is_tiled: true
        }
    );
}

#[test]
fn parse_move_window_v2() {
    let ev = parse_event("movewindowv2>>abcdef,2,work").unwrap();
    assert_eq!(
        ev,
        Event::MoveWindowV2 {
            address: WindowAddress(0xabcdef),
            workspace_id: WorkspaceId(2),
            workspace_name: "work".into()
        }
    );
}

#[test]
fn parse_pin() {
    let ev = parse_event("pin>>abcdef,1").unwrap();
    assert_eq!(
        ev,
        Event::Pin {
            address: WindowAddress(0xabcdef),
            pinned: true
        }
    );
}

#[test]
fn parse_minimized() {
    let ev = parse_event("minimized>>abcdef,0").unwrap();
    assert_eq!(
        ev,
        Event::Minimized {
            address: WindowAddress(0xabcdef),
            minimized: false
        }
    );
}

#[test]
fn parse_toggle_group() {
    let ev = parse_event("togglegroup>>1,abcdef").unwrap();
    assert_eq!(
        ev,
        Event::ToggleGroup {
            state: true,
            addresses: vec![WindowAddress(0xabcdef)]
        }
    );
}

#[test]
fn parse_lock_groups() {
    assert_eq!(
        parse_event("lockgroups>>1").unwrap(),
        Event::LockGroups { locked: true }
    );
}

#[test]
fn parse_open_layer() {
    let ev = parse_event("openlayer>>waybar").unwrap();
    assert_eq!(
        ev,
        Event::OpenLayer {
            namespace: "waybar".into()
        }
    );
}

#[test]
fn parse_active_layout() {
    let ev = parse_event("activelayout>>at-keyboard-1,English (US)").unwrap();
    assert_eq!(
        ev,
        Event::ActiveLayout {
            keyboard: "at-keyboard-1".into(),
            layout: "English (US)".into()
        }
    );
}

#[test]
fn parse_submap() {
    let ev = parse_event("submap>>resize").unwrap();
    assert_eq!(
        ev,
        Event::Submap {
            name: "resize".into()
        }
    );
}

#[test]
fn parse_screencast() {
    let ev = parse_event("screencast>>1,42").unwrap();
    assert_eq!(
        ev,
        Event::Screencast {
            active: true,
            owner: "42".into()
        }
    );
}

#[test]
fn parse_config_reloaded() {
    let ev = parse_event("configreloaded>>").unwrap();
    assert_eq!(ev, Event::ConfigReloaded);
}

#[test]
fn parse_custom_event() {
    let ev = parse_event("custom>>my data here").unwrap();
    assert_eq!(
        ev,
        Event::Custom {
            data: "my data here".into()
        }
    );
}

#[test]
fn parse_unknown_event() {
    let ev = parse_event("futurevent>>some data").unwrap();
    assert_eq!(
        ev,
        Event::Unknown {
            name: "futurevent".into(),
            data: "some data".into()
        }
    );
}

#[test]
fn parse_no_separator_returns_none() {
    assert!(parse_event("garbage").is_none());
}

#[test]
fn parse_empty_active_window() {
    let ev = parse_event("activewindow>>,").unwrap();
    assert_eq!(
        ev,
        Event::ActiveWindow {
            class: "".into(),
            title: "".into()
        }
    );
}

#[test]
fn parse_create_workspace_v2() {
    let ev = parse_event("createworkspacev2>>7,scratch").unwrap();
    assert_eq!(
        ev,
        Event::CreateWorkspaceV2 {
            id: WorkspaceId(7),
            name: "scratch".into()
        }
    );
}

#[test]
fn parse_destroy_workspace() {
    let ev = parse_event("destroyworkspace>>old").unwrap();
    assert_eq!(ev, Event::DestroyWorkspace { name: "old".into() });
}

#[test]
fn parse_focused_mon_v2() {
    let ev = parse_event("focusedmonv2>>DP-3,9").unwrap();
    assert_eq!(
        ev,
        Event::FocusedMonV2 {
            monitor: "DP-3".into(),
            workspace_id: WorkspaceId(9)
        }
    );
}

#[test]
fn parse_monitor_removed() {
    let ev = parse_event("monitorremoved>>HDMI-A-2").unwrap();
    assert_eq!(
        ev,
        Event::MonitorRemoved {
            name: "HDMI-A-2".into()
        }
    );
}

#[test]
fn parse_monitor_removed_v2() {
    let ev = parse_event("monitorremovedv2>>3,DP-3,LG ULTRAGEAR").unwrap();
    assert_eq!(
        ev,
        Event::MonitorRemovedV2 {
            id: "3".into(),
            name: "DP-3".into(),
            description: "LG ULTRAGEAR".into()
        }
    );
}

#[test]
fn parse_active_special() {
    let ev = parse_event("activespecial>>scratch,DP-1").unwrap();
    assert_eq!(
        ev,
        Event::ActiveSpecial {
            name: "scratch".into(),
            monitor: "DP-1".into()
        }
    );
}

#[test]
fn parse_active_special_v2() {
    let ev = parse_event("activespecialv2>>-99,scratch,DP-1").unwrap();
    assert_eq!(
        ev,
        Event::ActiveSpecialV2 {
            id: "-99".into(),
            name: "scratch".into(),
            monitor: "DP-1".into()
        }
    );
}

#[test]
fn parse_window_title_v1() {
    let ev = parse_event("windowtitle>>55a3f2c0").unwrap();
    assert_eq!(
        ev,
        Event::WindowTitle {
            address: WindowAddress(0x55a3f2c0)
        }
    );
}

#[test]
fn parse_move_window_v1() {
    let ev = parse_event("movewindow>>55a3f2c0,4").unwrap();
    assert_eq!(
        ev,
        Event::MoveWindow {
            address: WindowAddress(0x55a3f2c0),
            workspace: "4".into()
        }
    );
}

#[test]
fn parse_urgent() {
    let ev = parse_event("urgent>>55a3f2c0").unwrap();
    assert_eq!(
        ev,
        Event::Urgent {
            address: WindowAddress(0x55a3f2c0)
        }
    );
}

#[test]
fn parse_move_into_group() {
    let ev = parse_event("moveintogroup>>55a3f2c0").unwrap();
    assert_eq!(
        ev,
        Event::MoveIntoGroup {
            address: WindowAddress(0x55a3f2c0)
        }
    );
}

#[test]
fn parse_move_out_of_group() {
    let ev = parse_event("moveoutofgroup>>55a3f2c0").unwrap();
    assert_eq!(
        ev,
        Event::MoveOutOfGroup {
            address: WindowAddress(0x55a3f2c0)
        }
    );
}

#[test]
fn parse_ignore_group_lock() {
    let ev = parse_event("ignoregrouplock>>1").unwrap();
    assert_eq!(ev, Event::IgnoreGroupLock { enabled: true });
}

#[test]
fn parse_close_layer() {
    let ev = parse_event("closelayer>>waybar").unwrap();
    assert_eq!(
        ev,
        Event::CloseLayer {
            namespace: "waybar".into()
        }
    );
}

#[test]
fn parse_bell() {
    let ev = parse_event("bell>>0x55a3f2c0").unwrap();
    assert_eq!(
        ev,
        Event::Bell {
            address: "0x55a3f2c0".into()
        }
    );
}

#[test]
fn wire_helpers_open_window() {
    let ev = parse_event("openwindow>>abcdef,2,kitty,Terminal").unwrap();
    assert_eq!(ev.wire_name(), "openwindow");
    assert_eq!(ev.wire_data(), "abcdef,2,kitty,Terminal");
    assert_eq!(ev.to_wire_line(), "openwindow>>abcdef,2,kitty,Terminal");
}

#[test]
fn wire_helpers_boolean_and_unknown_events() {
    let fullscreen = parse_event("fullscreen>>1").unwrap();
    assert_eq!(fullscreen.wire_name(), "fullscreen");
    assert_eq!(fullscreen.wire_data(), "1");
    assert_eq!(fullscreen.to_wire_line(), "fullscreen>>1");

    let unknown = parse_event("futurevent>>some data").unwrap();
    assert_eq!(unknown.wire_name(), "futurevent");
    assert_eq!(unknown.wire_data(), "some data");
    assert_eq!(unknown.to_wire_line(), "futurevent>>some data");
}
