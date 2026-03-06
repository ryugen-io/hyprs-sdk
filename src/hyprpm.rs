//! Rust wrapper for the `hyprpm` CLI (Hyprland Plugin Manager).
//!
//! This complements Socket IPC APIs: `hyprpm` handles plugin repository
//! lifecycle (clone/build/install/enable), while IPC/plugin APIs handle
//! runtime compositor interaction.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{HyprError, HyprResult};

/// Result of a `hyprpm` command execution.
#[derive(Debug, Clone)]
pub struct HyprPmOutput {
    pub status_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Parsed `hyprpm list` output.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HyprPmList {
    pub repositories: Vec<HyprPmRepository>,
}

/// A plugin repository listed by `hyprpm list`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HyprPmRepository {
    pub name: String,
    pub author: String,
    pub plugins: Vec<HyprPmPlugin>,
}

/// A single plugin entry from `hyprpm list`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HyprPmPlugin {
    pub name: String,
    pub enabled: bool,
}

/// High-level wrapper around the `hyprpm` executable.
#[derive(Debug, Clone)]
pub struct HyprPm {
    binary: PathBuf,
}

impl Default for HyprPm {
    fn default() -> Self {
        Self {
            binary: PathBuf::from("hyprpm"),
        }
    }
}

impl HyprPm {
    /// Create a wrapper using the default executable name (`hyprpm`).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a wrapper with a custom executable path.
    #[must_use]
    pub fn with_binary(binary: impl AsRef<Path>) -> Self {
        Self {
            binary: binary.as_ref().to_path_buf(),
        }
    }

    /// Run an arbitrary `hyprpm` operation.
    ///
    /// # Errors
    ///
    /// Returns I/O errors when the process cannot be started, parse errors for
    /// non-UTF8 output, and command errors for non-zero exits.
    pub fn run_raw(&self, args: &[String]) -> HyprResult<HyprPmOutput> {
        let output = Command::new(&self.binary)
            .args(args)
            .output()
            .map_err(HyprError::Io)?;
        let status_code = output.status.code().unwrap_or(-1);

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| HyprError::Parse(format!("hyprpm stdout is not utf8: {e}")))?;
        let stderr = String::from_utf8(output.stderr)
            .map_err(|e| HyprError::Parse(format!("hyprpm stderr is not utf8: {e}")))?;

        if output.status.success() {
            Ok(HyprPmOutput {
                status_code,
                stdout,
                stderr,
            })
        } else {
            let cmdline = args.join(" ");
            let detail = if stderr.trim().is_empty() {
                stdout.trim().to_string()
            } else {
                stderr.trim().to_string()
            };
            Err(HyprError::Command(format!(
                "hyprpm {cmdline} failed (exit {status_code}): {detail}"
            )))
        }
    }

    /// Install a plugin repository from git.
    pub fn add(&self, url: &str, git_rev: Option<&str>) -> HyprResult<HyprPmOutput> {
        let mut args = vec!["add".to_string(), url.to_string()];
        if let Some(rev) = git_rev
            && !rev.is_empty()
        {
            args.push(rev.to_string());
        }
        self.run_raw(&args)
    }

    /// Remove an installed plugin repository.
    pub fn remove(&self, target: &str) -> HyprResult<HyprPmOutput> {
        self.run_raw(&["remove".to_string(), target.to_string()])
    }

    /// Enable a plugin by name (`name` or `author/name`).
    pub fn enable(&self, name: &str) -> HyprResult<HyprPmOutput> {
        self.run_raw(&["enable".to_string(), name.to_string()])
    }

    /// Disable a plugin by name (`name` or `author/name`).
    pub fn disable(&self, name: &str) -> HyprResult<HyprPmOutput> {
        self.run_raw(&["disable".to_string(), name.to_string()])
    }

    /// Update all plugins.
    pub fn update(&self, force: bool) -> HyprResult<HyprPmOutput> {
        let mut args = vec!["update".to_string()];
        if force {
            args.push("--force".to_string());
        }
        self.run_raw(&args)
    }

    /// Reload hyprpm state and plugin load status.
    pub fn reload(&self) -> HyprResult<HyprPmOutput> {
        self.run_raw(&["reload".to_string()])
    }

    /// List installed plugins.
    pub fn list(&self) -> HyprResult<HyprPmOutput> {
        self.run_raw(&["list".to_string()])
    }

    /// List installed plugins as a structured representation.
    pub fn list_structured(&self) -> HyprResult<HyprPmList> {
        let out = self.list()?;
        Ok(parse_list_output(&out.stdout))
    }

    /// Purge hyprpm cache/state/headers.
    pub fn purge_cache(&self) -> HyprResult<HyprPmOutput> {
        self.run_raw(&["purge-cache".to_string()])
    }
}

/// Parse raw `hyprpm list` stdout into a structured model.
#[must_use]
pub fn parse_list_output(raw: &str) -> HyprPmList {
    let mut list = HyprPmList::default();
    let mut current_repo: Option<HyprPmRepository> = None;
    let mut pending_plugin_name: Option<String> = None;

    for line in raw.lines() {
        let line = strip_ansi(line).trim().to_string();
        if line.is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("→ Repository ") {
            if let Some(repo) = current_repo.take() {
                list.repositories.push(repo);
            }

            let (name, author) = if let Some((repo_name, author_part)) = rest.split_once(" (by ") {
                let author = author_part
                    .trim_end_matches("):")
                    .trim_end_matches(')')
                    .to_string();
                (repo_name.trim().to_string(), author)
            } else {
                (rest.trim_end_matches(':').trim().to_string(), String::new())
            };

            current_repo = Some(HyprPmRepository {
                name,
                author,
                plugins: Vec::new(),
            });
            pending_plugin_name = None;
            continue;
        }

        if let Some(plugin_name) = line.strip_prefix("│ Plugin ") {
            pending_plugin_name = Some(plugin_name.trim().to_string());
            continue;
        }

        if let Some(enabled_part) = line.split_once("enabled:").map(|(_, rhs)| rhs.trim())
            && let (Some(repo), Some(plugin_name)) =
                (current_repo.as_mut(), pending_plugin_name.take())
        {
            let enabled = enabled_part.eq_ignore_ascii_case("true");
            repo.plugins.push(HyprPmPlugin {
                name: plugin_name,
                enabled,
            });
        }
    }

    if let Some(repo) = current_repo.take() {
        list.repositories.push(repo);
    }

    list
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for c in chars.by_ref() {
                if ('@'..='~').contains(&c) {
                    break;
                }
            }
            continue;
        }
        out.push(ch);
    }

    out
}
