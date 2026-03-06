use crate::types::common::{WindowAddress, WorkspaceId};

/// A parsed Hyprland event from the Socket2 event stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    // Workspace events are grouped together because they share the same ID/name semantics
    // and v1/v2 pairing pattern that Hyprland uses for backward compatibility.
    /// Active workspace changed.
    Workspace {
        name: String,
    },
    WorkspaceV2 {
        id: WorkspaceId,
        name: String,
    },
    /// Workspace created.
    CreateWorkspace {
        name: String,
    },
    CreateWorkspaceV2 {
        id: WorkspaceId,
        name: String,
    },
    /// Workspace destroyed.
    DestroyWorkspace {
        name: String,
    },
    DestroyWorkspaceV2 {
        id: WorkspaceId,
        name: String,
    },
    /// Workspace moved to a different monitor.
    MoveWorkspace {
        name: String,
        monitor: String,
    },
    MoveWorkspaceV2 {
        id: WorkspaceId,
        name: String,
        monitor: String,
    },
    /// Workspace renamed.
    RenameWorkspace {
        id: WorkspaceId,
        new_name: String,
    },

    // Monitor events are separate from workspace events because they track hardware topology
    // (hotplug/unplug) rather than logical workspace state.
    /// Monitor focus changed.
    FocusedMon {
        monitor: String,
        workspace: String,
    },
    FocusedMonV2 {
        monitor: String,
        workspace_id: WorkspaceId,
    },
    /// Monitor connected.
    MonitorAdded {
        name: String,
    },
    MonitorAddedV2 {
        id: String,
        name: String,
        description: String,
    },
    /// Monitor disconnected.
    MonitorRemoved {
        name: String,
    },
    MonitorRemovedV2 {
        id: String,
        name: String,
        description: String,
    },

    // Special workspaces (scratchpads) have distinct toggle semantics: they exist per-monitor
    // and can be shown/hidden independently, unlike regular workspace switches.
    /// Special workspace toggled.
    ActiveSpecial {
        name: String,
        monitor: String,
    },
    ActiveSpecialV2 {
        id: String,
        name: String,
        monitor: String,
    },

    // Window lifecycle events (open/close/move/focus/title) are the most common IPC events.
    // They carry WindowAddress for precise identification across workspace moves.
    /// Active window changed.
    ActiveWindow {
        class: String,
        title: String,
    },
    ActiveWindowV2 {
        address: WindowAddress,
    },
    /// Window opened.
    OpenWindow {
        address: WindowAddress,
        workspace: String,
        class: String,
        title: String,
    },
    /// Window closed.
    CloseWindow {
        address: WindowAddress,
    },
    /// Window title changed.
    WindowTitle {
        address: WindowAddress,
    },
    WindowTitleV2 {
        address: WindowAddress,
        title: String,
    },
    /// Window moved to workspace.
    MoveWindow {
        address: WindowAddress,
        workspace: String,
    },
    MoveWindowV2 {
        address: WindowAddress,
        workspace_id: WorkspaceId,
        workspace_name: String,
    },

    // Window state toggles are separate from lifecycle events because they represent transient
    // property changes (fullscreen, floating, pin) on already-open windows.
    /// Fullscreen state changed.
    Fullscreen {
        enabled: bool,
    },
    /// Window urgent hint.
    Urgent {
        address: WindowAddress,
    },
    /// Window minimized/unminimized.
    Minimized {
        address: WindowAddress,
        minimized: bool,
    },
    /// Window pinned/unpinned.
    Pin {
        address: WindowAddress,
        pinned: bool,
    },

    // Group (tabbed window) events form their own category because group membership affects
    // layout and rendering independently of individual window state.
    /// Groups locked/unlocked.
    LockGroups {
        locked: bool,
    },
    /// Window moved into group.
    MoveIntoGroup {
        address: WindowAddress,
    },
    /// Window moved out of group.
    MoveOutOfGroup {
        address: WindowAddress,
    },
    /// Group lock ignore state changed.
    IgnoreGroupLock {
        enabled: bool,
    },

    // Layer shell surfaces (panels, overlays) live outside the tiling layout and use namespace
    // strings instead of window addresses for identification.
    /// Layer surface opened.
    OpenLayer {
        namespace: String,
    },
    /// Layer surface closed.
    CloseLayer {
        namespace: String,
    },

    // Input events (layout changes, submap switches) are per-device and affect keybinding
    // behavior, so they're grouped separately from window or compositor state.
    /// Keyboard layout changed.
    ActiveLayout {
        keyboard: String,
        layout: String,
    },
    /// Keybind submap changed.
    Submap {
        name: String,
    },

    // Miscellaneous events that don't fit other categories. Unknown preserves unrecognized
    // event names for forward compatibility with newer Hyprland releases.
    /// Bell notification.
    Bell {
        address: String,
    },
    /// Screencast state changed.
    Screencast {
        active: bool,
        owner: String,
    },
    /// Screencast state changed (v2 with name).
    ScreencastV2 {
        active: bool,
        owner: String,
        name: String,
    },
    /// Configuration reloaded.
    ConfigReloaded,
    /// Custom user event (from `dispatch event` command).
    Custom {
        data: String,
    },

    /// Unknown event type (forward compatibility).
    Unknown {
        name: String,
        data: String,
    },
}

impl Event {
    /// Canonical Socket2 event name.
    #[must_use]
    pub fn wire_name(&self) -> &str {
        match self {
            Self::Workspace { .. } => "workspace",
            Self::WorkspaceV2 { .. } => "workspacev2",
            Self::CreateWorkspace { .. } => "createworkspace",
            Self::CreateWorkspaceV2 { .. } => "createworkspacev2",
            Self::DestroyWorkspace { .. } => "destroyworkspace",
            Self::DestroyWorkspaceV2 { .. } => "destroyworkspacev2",
            Self::MoveWorkspace { .. } => "moveworkspace",
            Self::MoveWorkspaceV2 { .. } => "moveworkspacev2",
            Self::RenameWorkspace { .. } => "renameworkspace",
            Self::FocusedMon { .. } => "focusedmon",
            Self::FocusedMonV2 { .. } => "focusedmonv2",
            Self::MonitorAdded { .. } => "monitoradded",
            Self::MonitorAddedV2 { .. } => "monitoraddedv2",
            Self::MonitorRemoved { .. } => "monitorremoved",
            Self::MonitorRemovedV2 { .. } => "monitorremovedv2",
            Self::ActiveSpecial { .. } => "activespecial",
            Self::ActiveSpecialV2 { .. } => "activespecialv2",
            Self::ActiveWindow { .. } => "activewindow",
            Self::ActiveWindowV2 { .. } => "activewindowv2",
            Self::OpenWindow { .. } => "openwindow",
            Self::CloseWindow { .. } => "closewindow",
            Self::WindowTitle { .. } => "windowtitle",
            Self::WindowTitleV2 { .. } => "windowtitlev2",
            Self::MoveWindow { .. } => "movewindow",
            Self::MoveWindowV2 { .. } => "movewindowv2",
            Self::Fullscreen { .. } => "fullscreen",
            Self::Urgent { .. } => "urgent",
            Self::Minimized { .. } => "minimized",
            Self::Pin { .. } => "pin",
            Self::LockGroups { .. } => "lockgroups",
            Self::MoveIntoGroup { .. } => "moveintogroup",
            Self::MoveOutOfGroup { .. } => "moveoutofgroup",
            Self::IgnoreGroupLock { .. } => "ignoregrouplock",
            Self::OpenLayer { .. } => "openlayer",
            Self::CloseLayer { .. } => "closelayer",
            Self::ActiveLayout { .. } => "activelayout",
            Self::Submap { .. } => "submap",
            Self::Bell { .. } => "bell",
            Self::Screencast { .. } => "screencast",
            Self::ScreencastV2 { .. } => "screencastv2",
            Self::ConfigReloaded => "configreloaded",
            Self::Custom { .. } => "custom",
            Self::Unknown { name, .. } => name.as_str(),
        }
    }

    /// Socket2 data payload for this event.
    #[must_use]
    pub fn wire_data(&self) -> String {
        match self {
            Self::Workspace { name }
            | Self::CreateWorkspace { name }
            | Self::DestroyWorkspace { name }
            | Self::MonitorAdded { name }
            | Self::MonitorRemoved { name }
            | Self::OpenLayer { namespace: name }
            | Self::CloseLayer { namespace: name }
            | Self::Submap { name } => name.clone(),
            Self::WorkspaceV2 { id, name }
            | Self::CreateWorkspaceV2 { id, name }
            | Self::DestroyWorkspaceV2 { id, name } => format!("{id},{name}"),
            Self::MoveWorkspace { name, monitor } => format!("{name},{monitor}"),
            Self::MoveWorkspaceV2 { id, name, monitor } => format!("{id},{name},{monitor}"),
            Self::RenameWorkspace { id, new_name } => format!("{id},{new_name}"),
            Self::FocusedMon { monitor, workspace } => format!("{monitor},{workspace}"),
            Self::FocusedMonV2 {
                monitor,
                workspace_id,
            } => format!("{monitor},{workspace_id}"),
            Self::MonitorAddedV2 {
                id,
                name,
                description,
            }
            | Self::MonitorRemovedV2 {
                id,
                name,
                description,
            } => format!("{id},{name},{description}"),
            Self::ActiveSpecial { name, monitor } => format!("{name},{monitor}"),
            Self::ActiveSpecialV2 { id, name, monitor } => format!("{id},{name},{monitor}"),
            Self::ActiveWindow { class, title } => format!("{class},{title}"),
            Self::ActiveWindowV2 { address }
            | Self::CloseWindow { address }
            | Self::WindowTitle { address }
            | Self::Urgent { address }
            | Self::MoveIntoGroup { address }
            | Self::MoveOutOfGroup { address } => format_addr(*address),
            Self::OpenWindow {
                address,
                workspace,
                class,
                title,
            } => format!("{},{workspace},{class},{title}", format_addr(*address)),
            Self::WindowTitleV2 { address, title } => {
                format!("{},{}", format_addr(*address), title)
            }
            Self::MoveWindow { address, workspace } => {
                format!("{},{}", format_addr(*address), workspace)
            }
            Self::MoveWindowV2 {
                address,
                workspace_id,
                workspace_name,
            } => format!("{},{workspace_id},{workspace_name}", format_addr(*address)),
            Self::Fullscreen { enabled } => bool_as_int(*enabled).to_string(),
            Self::Minimized { address, minimized } => {
                format!("{},{}", format_addr(*address), bool_as_int(*minimized))
            }
            Self::Pin { address, pinned } => {
                format!("{},{}", format_addr(*address), bool_as_int(*pinned))
            }
            Self::LockGroups { locked } => bool_as_int(*locked).to_string(),
            Self::IgnoreGroupLock { enabled } => bool_as_int(*enabled).to_string(),
            Self::ActiveLayout { keyboard, layout } => format!("{keyboard},{layout}"),
            Self::Bell { address } => address.clone(),
            Self::Screencast { active, owner } => format!("{},{}", bool_as_int(*active), owner),
            Self::ScreencastV2 {
                active,
                owner,
                name,
            } => format!("{},{},{}", bool_as_int(*active), owner, name),
            Self::ConfigReloaded => String::new(),
            Self::Custom { data } => data.clone(),
            Self::Unknown { data, .. } => data.clone(),
        }
    }

    /// Full Socket2 wire line (`name>>data`).
    #[must_use]
    pub fn to_wire_line(&self) -> String {
        format!("{}>>{}", self.wire_name(), self.wire_data())
    }
}

const fn bool_as_int(value: bool) -> u8 {
    if value { 1 } else { 0 }
}

fn format_addr(addr: WindowAddress) -> String {
    format!("{:x}", addr.0)
}
