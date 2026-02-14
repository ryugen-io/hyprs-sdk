//! Hyprland instance discovery.
//!
//! Scans `$XDG_RUNTIME_DIR/hypr/` for running Hyprland instances by reading
//! lock files and validating that the compositor process is still alive.

use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use crate::error::{HyprError, HyprResult};

/// A discovered running Hyprland instance.
#[derive(Debug, Clone)]
pub struct Instance {
    /// Instance signature (directory name under the runtime dir).
    pub signature: String,

    /// PID of the Hyprland compositor process.
    pub pid: u64,

    /// Wayland socket name (e.g. "wayland-1").
    pub wayland_socket: String,
}

impl Instance {
    /// Path to Socket1 (request/response) for this instance.
    #[must_use]
    pub fn socket1_path(&self) -> PathBuf {
        PathBuf::from(runtime_dir())
            .join(&self.signature)
            .join(".socket.sock")
    }

    /// Path to Socket2 (event stream) for this instance.
    #[must_use]
    pub fn socket2_path(&self) -> PathBuf {
        PathBuf::from(runtime_dir())
            .join(&self.signature)
            .join(".socket2.sock")
    }
}

/// Returns the Hyprland runtime directory.
///
/// Uses `$XDG_RUNTIME_DIR/hypr` if set, otherwise `/run/user/$UID/hypr`.
#[must_use]
pub fn runtime_dir() -> String {
    match std::env::var("XDG_RUNTIME_DIR") {
        Ok(xdg) => format!("{xdg}/hypr"),
        Err(_) => {
            let uid = fs::metadata("/proc/self").map(|m| m.uid()).unwrap_or(0);
            format!("/run/user/{uid}/hypr")
        }
    }
}

/// Convenience: Socket1 path for a given instance signature.
#[must_use]
pub fn socket1_path(signature: &str) -> PathBuf {
    PathBuf::from(runtime_dir())
        .join(signature)
        .join(".socket.sock")
}

/// Convenience: Socket2 path for a given instance signature.
#[must_use]
pub fn socket2_path(signature: &str) -> PathBuf {
    PathBuf::from(runtime_dir())
        .join(signature)
        .join(".socket2.sock")
}

/// Discover all running Hyprland instances.
///
/// Scans the runtime directory for instance directories containing
/// `hyprland.lock`. Validates that the PID in the lock file is alive.
pub fn discover_instances() -> HyprResult<Vec<Instance>> {
    let dir = runtime_dir();
    let entries = fs::read_dir(&dir).map_err(|_| HyprError::NoInstance)?;

    let mut instances = Vec::new();

    for entry in entries.flatten() {
        if !entry.file_type().is_ok_and(|ft| ft.is_dir()) {
            continue;
        }

        if let Some(instance) = parse_instance(&entry.path())
            && is_pid_alive(instance.pid)
        {
            instances.push(instance);
        }
    }

    Ok(instances)
}

/// Get the instance identified by `$HYPRLAND_INSTANCE_SIGNATURE`.
///
/// This is the standard way to find the current Hyprland session.
pub fn current_instance() -> HyprResult<Instance> {
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_| HyprError::NoInstance)?;

    let dir = PathBuf::from(runtime_dir()).join(&sig);
    parse_instance(&dir).ok_or(HyprError::InstanceNotFound(sig))
}

// -- Internal ----------------------------------------------------------------

fn parse_instance(dir: &std::path::Path) -> Option<Instance> {
    let lock_path = dir.join("hyprland.lock");
    let content = fs::read_to_string(lock_path).ok()?;
    let mut lines = content.lines();

    let pid: u64 = lines.next()?.parse().ok()?;
    let wayland_socket = lines.next()?.to_string();

    // Lock file should have exactly 2 lines.
    if lines.next().is_some_and(|l| !l.is_empty()) {
        return None;
    }

    let signature = dir.file_name()?.to_str()?.to_string();

    // Validate signature format: must contain underscores.
    if !signature.contains('_') {
        return None;
    }

    Some(Instance {
        signature,
        pid,
        wayland_socket,
    })
}

fn is_pid_alive(pid: u64) -> bool {
    // Check /proc/{pid} existence — works for any process regardless of owner.
    fs::metadata(format!("/proc/{pid}")).is_ok()
}
