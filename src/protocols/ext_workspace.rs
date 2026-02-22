//! ext-workspace: workspace management protocol.
//!
//! Provides [`ExtWorkspaceClient`] for listing and controlling workspaces
//! via the `ext_workspace_manager_v1` protocol.

use std::fmt;
use std::ops::BitOr;

use wayland_client::protocol::{wl_output, wl_registry};
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, event_created_child};
use wayland_protocols::ext::workspace::v1::client::{
    ext_workspace_group_handle_v1, ext_workspace_handle_v1, ext_workspace_manager_v1,
};

use crate::error::{HyprError, HyprResult};
use crate::protocols::connection::WaylandConnection;

/// Workspace state flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WorkspaceState(u32);

impl WorkspaceState {
    /// The workspace is currently active/focused.
    pub const ACTIVE: Self = Self(1);
    /// The workspace has an urgent notification.
    pub const URGENT: Self = Self(2);
    /// The workspace is hidden.
    pub const HIDDEN: Self = Self(4);

    /// Returns `true` if no state flags are set.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all flags in `other` are set in `self`.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Returns `true` if the workspace is active.
    #[must_use]
    pub fn is_active(self) -> bool {
        self.contains(Self::ACTIVE)
    }

    /// Returns `true` if the workspace has an urgent notification.
    #[must_use]
    pub fn is_urgent(self) -> bool {
        self.contains(Self::URGENT)
    }

    /// Returns `true` if the workspace is hidden.
    #[must_use]
    pub fn is_hidden(self) -> bool {
        self.contains(Self::HIDDEN)
    }
}

impl BitOr for WorkspaceState {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// Capabilities of a workspace group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WorkspaceGroupCapabilities(u32);

impl WorkspaceGroupCapabilities {
    /// The group supports creating new workspaces.
    pub const CREATE_WORKSPACE: Self = Self(1);

    /// Returns `true` if no capabilities are set.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all capabilities in `other` are set in `self`.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

/// Capabilities of an individual workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WorkspaceCapabilities(u32);

impl WorkspaceCapabilities {
    /// The workspace can be activated.
    pub const ACTIVATE: Self = Self(1);
    /// The workspace can be deactivated.
    pub const DEACTIVATE: Self = Self(2);
    /// The workspace can be removed.
    pub const REMOVE: Self = Self(4);

    /// Returns `true` if no capabilities are set.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if all capabilities in `other` are set in `self`.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

/// Workspace coordinate pair for multi-dimensional layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WorkspaceCoordinates {
    /// Horizontal coordinate.
    pub x: i32,
    /// Vertical coordinate.
    pub y: i32,
}

/// Information about a workspace group.
#[derive(Debug, Clone)]
pub struct WorkspaceGroupInfo {
    /// Workspace names in this group.
    pub workspaces: Vec<String>,
    /// Capabilities of this group.
    pub capabilities: WorkspaceGroupCapabilities,
}

/// Information about a workspace.
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    /// Workspace name.
    pub name: String,
    /// Current state flags.
    pub state: WorkspaceState,
    /// Workspace capabilities.
    pub capabilities: WorkspaceCapabilities,
    /// Workspace coordinates (if set).
    pub coordinates: Option<WorkspaceCoordinates>,
}

/// Client for the `ext_workspace_manager_v1` protocol.
///
/// Lists and controls workspaces and workspace groups.
///
/// # Example
///
/// ```no_run
/// use hypr_sdk::protocols::connection::WaylandConnection;
/// use hypr_sdk::protocols::ext_workspace::ExtWorkspaceClient;
///
/// let wl = WaylandConnection::connect().unwrap();
/// let mut client = ExtWorkspaceClient::connect(&wl).unwrap();
///
/// for ws in client.workspaces() {
///     println!("{}: active={}", ws.name, ws.state.is_active());
/// }
/// ```
pub struct ExtWorkspaceClient {
    state: ExtWorkspaceState,
    event_queue: EventQueue<ExtWorkspaceState>,
}

impl ExtWorkspaceClient {
    /// Connect to the workspace manager.
    ///
    /// # Errors
    ///
    /// Returns [`HyprError::ProtocolNotSupported`] if the compositor
    /// doesn't advertise `ext_workspace_manager_v1`.
    pub fn connect(wl: &WaylandConnection) -> HyprResult<Self> {
        if !wl.has_protocol("ext_workspace_manager_v1") {
            return Err(HyprError::ProtocolNotSupported(
                "ext_workspace_manager_v1".into(),
            ));
        }

        let conn = wl.connection();
        let mut event_queue: EventQueue<ExtWorkspaceState> = conn.new_event_queue();
        let qh = event_queue.handle();
        let display = conn.display();

        let mut state = ExtWorkspaceState::new();

        let _registry = display.get_registry(&qh, ());
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        if state.manager.is_none() {
            return Err(HyprError::ProtocolNotSupported(
                "ext_workspace_manager_v1".into(),
            ));
        }

        // The manager sends workspace_group and workspace events asynchronously after
        // binding; a roundtrip collects the child objects before querying properties.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        // Property events (name, state, done) arrive on the workspace handles from the
        // previous roundtrip; an extra roundtrip collects them all.
        event_queue
            .roundtrip(&mut state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(Self { state, event_queue })
    }

    /// Get all workspaces.
    #[must_use]
    pub fn workspaces(&self) -> Vec<WorkspaceInfo> {
        self.state
            .workspaces
            .iter()
            .filter(|w| !w.removed)
            .map(|w| WorkspaceInfo {
                name: w.name.clone().unwrap_or_default(),
                state: w.state,
                capabilities: w.capabilities,
                coordinates: w.coordinates,
            })
            .collect()
    }

    /// Activate a workspace by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the workspace is not found or dispatch fails.
    pub fn activate(&mut self, name: &str) -> HyprResult<()> {
        let Self { state, event_queue } = self;

        let ws = state
            .workspaces
            .iter()
            .find(|w| w.name.as_deref() == Some(name) && !w.removed)
            .ok_or_else(|| HyprError::WaylandDispatch(format!("workspace not found: {name}")))?;

        ws.handle.activate();

        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Deactivate a workspace by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the workspace is not found or dispatch fails.
    pub fn deactivate(&mut self, name: &str) -> HyprResult<()> {
        let Self { state, event_queue } = self;

        let ws = state
            .workspaces
            .iter()
            .find(|w| w.name.as_deref() == Some(name) && !w.removed)
            .ok_or_else(|| HyprError::WaylandDispatch(format!("workspace not found: {name}")))?;

        ws.handle.deactivate();

        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;

        Ok(())
    }

    /// Re-dispatch events to update workspace state.
    ///
    /// # Errors
    ///
    /// Returns an error if event dispatch fails.
    pub fn refresh(&mut self) -> HyprResult<()> {
        let Self { state, event_queue } = self;
        event_queue
            .roundtrip(state)
            .map_err(|e| HyprError::WaylandDispatch(e.to_string()))?;
        Ok(())
    }
}

impl fmt::Debug for ExtWorkspaceClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExtWorkspaceClient")
            .field("workspaces", &self.state.workspaces.len())
            .finish()
    }
}

// ── Internal state ──────────────────────────────────────────────────────────
// Tracks workspace groups and individual workspaces with their properties.

struct ExtWorkspaceState {
    manager: Option<ext_workspace_manager_v1::ExtWorkspaceManagerV1>,
    groups: Vec<GroupEntry>,
    workspaces: Vec<WorkspaceEntry>,
}

struct GroupEntry {
    handle: ext_workspace_group_handle_v1::ExtWorkspaceGroupHandleV1,
    capabilities: WorkspaceGroupCapabilities,
    removed: bool,
}

struct WorkspaceEntry {
    handle: ext_workspace_handle_v1::ExtWorkspaceHandleV1,
    name: Option<String>,
    state: WorkspaceState,
    capabilities: WorkspaceCapabilities,
    coordinates: Option<WorkspaceCoordinates>,
    removed: bool,
}

impl ExtWorkspaceState {
    fn new() -> Self {
        Self {
            manager: None,
            groups: Vec::new(),
            workspaces: Vec::new(),
        }
    }
}

// ── Dispatch implementations ────────────────────────────────────────────────
// wayland-client requires a Dispatch impl for every object type on the
// event queue.

impl Dispatch<wl_registry::WlRegistry, ()> for ExtWorkspaceState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
            && interface == "ext_workspace_manager_v1"
            && state.manager.is_none()
        {
            let mgr = registry.bind::<ext_workspace_manager_v1::ExtWorkspaceManagerV1, (), Self>(
                name,
                version.min(1),
                qh,
                (),
            );
            state.manager = Some(mgr);
        }
    }
}

impl Dispatch<ext_workspace_manager_v1::ExtWorkspaceManagerV1, ()> for ExtWorkspaceState {
    fn event(
        state: &mut Self,
        _proxy: &ext_workspace_manager_v1::ExtWorkspaceManagerV1,
        event: ext_workspace_manager_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            ext_workspace_manager_v1::Event::WorkspaceGroup { workspace_group } => {
                state.groups.push(GroupEntry {
                    handle: workspace_group,
                    capabilities: WorkspaceGroupCapabilities::default(),
                    removed: false,
                });
            }
            ext_workspace_manager_v1::Event::Workspace { workspace } => {
                state.workspaces.push(WorkspaceEntry {
                    handle: workspace,
                    name: None,
                    state: WorkspaceState::default(),
                    capabilities: WorkspaceCapabilities::default(),
                    coordinates: None,
                    removed: false,
                });
            }
            ext_workspace_manager_v1::Event::Done => {
                // The protocol batches updates and signals completion with "done";
                // no action needed here since we read state after roundtrip.
            }
            _ => {}
        }
    }

    event_created_child!(ExtWorkspaceState, ext_workspace_manager_v1::ExtWorkspaceManagerV1, [
        // wayland-client dispatches child-object creation by opcode, not name;
        // opcode 0 is the workspace_group event that spawns a group handle.
        0 => (ext_workspace_group_handle_v1::ExtWorkspaceGroupHandleV1, ()),
        // Opcode 1 is the workspace event that spawns a workspace handle.
        1 => (ext_workspace_handle_v1::ExtWorkspaceHandleV1, ()),
    ]);
}

impl Dispatch<ext_workspace_group_handle_v1::ExtWorkspaceGroupHandleV1, ()> for ExtWorkspaceState {
    fn event(
        state: &mut Self,
        proxy: &ext_workspace_group_handle_v1::ExtWorkspaceGroupHandleV1,
        event: ext_workspace_group_handle_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let Some(group) = state.groups.iter_mut().find(|g| g.handle == *proxy) {
            match event {
                ext_workspace_group_handle_v1::Event::Capabilities { capabilities } => {
                    group.capabilities = WorkspaceGroupCapabilities(capabilities.into());
                }
                ext_workspace_group_handle_v1::Event::Removed => {
                    group.removed = true;
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<ext_workspace_handle_v1::ExtWorkspaceHandleV1, ()> for ExtWorkspaceState {
    fn event(
        state: &mut Self,
        proxy: &ext_workspace_handle_v1::ExtWorkspaceHandleV1,
        event: ext_workspace_handle_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let Some(ws) = state.workspaces.iter_mut().find(|w| w.handle == *proxy) {
            match event {
                ext_workspace_handle_v1::Event::Name { name } => {
                    ws.name = Some(name);
                }
                ext_workspace_handle_v1::Event::State { state: state_data } => {
                    ws.state = WorkspaceState(state_data.into());
                }
                ext_workspace_handle_v1::Event::Capabilities { capabilities } => {
                    ws.capabilities = WorkspaceCapabilities(capabilities.into());
                }
                ext_workspace_handle_v1::Event::Coordinates { coordinates } => {
                    if coordinates.len() >= 8 {
                        let x = i32::from_ne_bytes([
                            coordinates[0],
                            coordinates[1],
                            coordinates[2],
                            coordinates[3],
                        ]);
                        let y = i32::from_ne_bytes([
                            coordinates[4],
                            coordinates[5],
                            coordinates[6],
                            coordinates[7],
                        ]);
                        ws.coordinates = Some(WorkspaceCoordinates { x, y });
                    }
                }
                ext_workspace_handle_v1::Event::Removed => {
                    ws.removed = true;
                }
                _ => {}
            }
        }
    }
}

// Dispatch needed because workspace group events can reference wl_output objects
// (output_enter/output_leave); wayland-client would panic without this impl.
impl Dispatch<wl_output::WlOutput, ()> for ExtWorkspaceState {
    fn event(
        _state: &mut Self,
        _proxy: &wl_output::WlOutput,
        _event: wl_output::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // We don't track output properties; we only need the Dispatch impl to avoid panics.
    }
}
