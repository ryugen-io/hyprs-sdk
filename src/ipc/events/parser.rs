use crate::types::common::{WindowAddress, WorkspaceId};

use super::Event;

/// Parse a single event line (`EVENT>>DATA`).
pub fn parse_event(line: &str) -> Option<Event> {
    let (name, data) = line.split_once(">>")?;
    Some(parse_event_inner(name, data))
}

fn parse_event_inner(name: &str, data: &str) -> Event {
    match name {
        // Workspace
        "workspace" => Event::Workspace {
            name: data.to_string(),
        },
        "workspacev2" => {
            let (id, name) = split2(data);
            Event::WorkspaceV2 {
                id: parse_ws_id(id),
                name: name.to_string(),
            }
        }
        "createworkspace" => Event::CreateWorkspace {
            name: data.to_string(),
        },
        "createworkspacev2" => {
            let (id, name) = split2(data);
            Event::CreateWorkspaceV2 {
                id: parse_ws_id(id),
                name: name.to_string(),
            }
        }
        "destroyworkspace" => Event::DestroyWorkspace {
            name: data.to_string(),
        },
        "destroyworkspacev2" => {
            let (id, name) = split2(data);
            Event::DestroyWorkspaceV2 {
                id: parse_ws_id(id),
                name: name.to_string(),
            }
        }
        "moveworkspace" => {
            let (name, monitor) = split2(data);
            Event::MoveWorkspace {
                name: name.to_string(),
                monitor: monitor.to_string(),
            }
        }
        "moveworkspacev2" => {
            let parts: Vec<&str> = data.splitn(3, ',').collect();
            Event::MoveWorkspaceV2 {
                id: parse_ws_id(parts.first().unwrap_or(&"")),
                name: parts.get(1).unwrap_or(&"").to_string(),
                monitor: parts.get(2).unwrap_or(&"").to_string(),
            }
        }
        "renameworkspace" => {
            let (id, new_name) = split2(data);
            Event::RenameWorkspace {
                id: parse_ws_id(id),
                new_name: new_name.to_string(),
            }
        }

        // Monitor
        "focusedmon" => {
            let (monitor, workspace) = split2(data);
            Event::FocusedMon {
                monitor: monitor.to_string(),
                workspace: workspace.to_string(),
            }
        }
        "focusedmonv2" => {
            let (monitor, ws_id) = split2(data);
            Event::FocusedMonV2 {
                monitor: monitor.to_string(),
                workspace_id: parse_ws_id(ws_id),
            }
        }
        "monitoradded" => Event::MonitorAdded {
            name: data.to_string(),
        },
        "monitoraddedv2" => {
            let parts: Vec<&str> = data.splitn(3, ',').collect();
            Event::MonitorAddedV2 {
                id: parts.first().unwrap_or(&"").to_string(),
                name: parts.get(1).unwrap_or(&"").to_string(),
                description: parts.get(2).unwrap_or(&"").to_string(),
            }
        }
        "monitorremoved" => Event::MonitorRemoved {
            name: data.to_string(),
        },
        "monitorremovedv2" => {
            let parts: Vec<&str> = data.splitn(3, ',').collect();
            Event::MonitorRemovedV2 {
                id: parts.first().unwrap_or(&"").to_string(),
                name: parts.get(1).unwrap_or(&"").to_string(),
                description: parts.get(2).unwrap_or(&"").to_string(),
            }
        }

        // Special workspace
        "activespecial" => {
            let (name, monitor) = split2(data);
            Event::ActiveSpecial {
                name: name.to_string(),
                monitor: monitor.to_string(),
            }
        }
        "activespecialv2" => {
            let parts: Vec<&str> = data.splitn(3, ',').collect();
            Event::ActiveSpecialV2 {
                id: parts.first().unwrap_or(&"").to_string(),
                name: parts.get(1).unwrap_or(&"").to_string(),
                monitor: parts.get(2).unwrap_or(&"").to_string(),
            }
        }

        // Window
        "activewindow" => {
            let (class, title) = split2(data);
            Event::ActiveWindow {
                class: class.to_string(),
                title: title.to_string(),
            }
        }
        "activewindowv2" => Event::ActiveWindowV2 {
            address: parse_addr(data),
        },
        "openwindow" => {
            let parts: Vec<&str> = data.splitn(4, ',').collect();
            Event::OpenWindow {
                address: parse_addr(parts.first().unwrap_or(&"")),
                workspace: parts.get(1).unwrap_or(&"").to_string(),
                class: parts.get(2).unwrap_or(&"").to_string(),
                title: parts.get(3).unwrap_or(&"").to_string(),
            }
        }
        "closewindow" => Event::CloseWindow {
            address: parse_addr(data),
        },
        "windowtitle" => Event::WindowTitle {
            address: parse_addr(data),
        },
        "windowtitlev2" => {
            let (addr, title) = split2(data);
            Event::WindowTitleV2 {
                address: parse_addr(addr),
                title: title.to_string(),
            }
        }
        "movewindow" => {
            let (addr, workspace) = split2(data);
            Event::MoveWindow {
                address: parse_addr(addr),
                workspace: workspace.to_string(),
            }
        }
        "movewindowv2" => {
            let parts: Vec<&str> = data.splitn(3, ',').collect();
            Event::MoveWindowV2 {
                address: parse_addr(parts.first().unwrap_or(&"")),
                workspace_id: parse_ws_id(parts.get(1).unwrap_or(&"")),
                workspace_name: parts.get(2).unwrap_or(&"").to_string(),
            }
        }

        // Window state
        "fullscreen" => Event::Fullscreen {
            enabled: data == "1",
        },
        "changefloatingmode" => {
            let (addr, tiled) = split2(data);
            Event::ChangeFloatingMode {
                address: parse_addr(addr),
                is_tiled: tiled == "1",
            }
        }
        "urgent" => Event::Urgent {
            address: parse_addr(data),
        },
        "minimized" => {
            let (addr, state) = split2(data);
            Event::Minimized {
                address: parse_addr(addr),
                minimized: state == "1",
            }
        }
        "pin" => {
            let (addr, state) = split2(data);
            Event::Pin {
                address: parse_addr(addr),
                pinned: state == "1",
            }
        }

        // Groups
        "togglegroup" => {
            let (state, rest) = split2(data);
            let addresses = rest
                .split(',')
                .filter(|s| !s.is_empty())
                .map(parse_addr)
                .collect();
            Event::ToggleGroup {
                state: state == "1",
                addresses,
            }
        }
        "lockgroups" => Event::LockGroups {
            locked: data == "1",
        },
        "moveintogroup" => Event::MoveIntoGroup {
            address: parse_addr(data),
        },
        "moveoutofgroup" => Event::MoveOutOfGroup {
            address: parse_addr(data),
        },
        "ignoregrouplock" => Event::IgnoreGroupLock {
            enabled: data == "1",
        },

        // Layer
        "openlayer" => Event::OpenLayer {
            namespace: data.to_string(),
        },
        "closelayer" => Event::CloseLayer {
            namespace: data.to_string(),
        },

        // Input
        "activelayout" => {
            let (keyboard, layout) = split2(data);
            Event::ActiveLayout {
                keyboard: keyboard.to_string(),
                layout: layout.to_string(),
            }
        }
        "submap" => Event::Submap {
            name: data.to_string(),
        },

        // Misc
        "bell" => Event::Bell {
            address: data.to_string(),
        },
        "screencast" => {
            let (state, owner) = split2(data);
            Event::Screencast {
                active: state == "1",
                owner: owner.to_string(),
            }
        }
        "configreloaded" => Event::ConfigReloaded,
        "custom" => Event::Custom {
            data: data.to_string(),
        },

        _ => Event::Unknown {
            name: name.to_string(),
            data: data.to_string(),
        },
    }
}

fn split2(s: &str) -> (&str, &str) {
    s.split_once(',').unwrap_or((s, ""))
}

fn parse_addr(s: &str) -> WindowAddress {
    s.parse().unwrap_or(WindowAddress(0))
}

fn parse_ws_id(s: &str) -> WorkspaceId {
    WorkspaceId(s.parse().unwrap_or(0))
}
