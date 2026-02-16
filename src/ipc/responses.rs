//! Serde types for JSON responses from Hyprland IPC commands.
//!
//! Each type maps to the JSON format returned by a specific `hyprctl`
//! command when the `j` (JSON) flag is set.

use serde::Deserialize;

// ── version ─────────────────────────────────────────────────────────

/// Response from the `version` command.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct VersionInfo {
    /// Git branch.
    pub branch: String,
    /// Git commit hash.
    pub commit: String,
    /// Version string (e.g., "0.53.0").
    pub version: String,
    /// Whether the build has uncommitted changes.
    pub dirty: bool,
    /// Git commit message.
    pub commit_message: String,
    /// Git commit date.
    pub commit_date: String,
    /// Git tag.
    pub tag: String,
    /// Number of commits since tag.
    pub commits: String,
    /// Build-time Aquamarine version.
    #[serde(rename = "buildAquamarine")]
    pub build_aquamarine: String,
    /// Build-time Hyprlang version.
    #[serde(rename = "buildHyprlang")]
    pub build_hyprlang: String,
    /// Build-time Hyprutils version.
    #[serde(rename = "buildHyprutils")]
    pub build_hyprutils: String,
    /// Build-time Hyprcursor version.
    #[serde(rename = "buildHyprcursor")]
    pub build_hyprcursor: String,
    /// Build-time Hyprgraphics version.
    #[serde(rename = "buildHyprgraphics")]
    pub build_hyprgraphics: String,
    /// System Aquamarine version.
    #[serde(rename = "systemAquamarine")]
    pub system_aquamarine: String,
    /// System Hyprlang version.
    #[serde(rename = "systemHyprlang")]
    pub system_hyprlang: String,
    /// System Hyprutils version.
    #[serde(rename = "systemHyprutils")]
    pub system_hyprutils: String,
    /// System Hyprcursor version.
    #[serde(rename = "systemHyprcursor")]
    pub system_hyprcursor: String,
    /// System Hyprgraphics version.
    #[serde(rename = "systemHyprgraphics")]
    pub system_hyprgraphics: String,
    /// ABI hash string.
    #[serde(rename = "abiHash")]
    pub abi_hash: String,
    /// Build flags (e.g., "debug", "no xwayland").
    pub flags: Vec<String>,
}

// ── devices ─────────────────────────────────────────────────────────

/// Response from the `devices` command.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DevicesResponse {
    /// Mouse/pointer devices.
    pub mice: Vec<Mouse>,
    /// Keyboard devices.
    pub keyboards: Vec<Keyboard>,
    /// Tablet devices (pads, tools, and drawing tablets).
    pub tablets: Vec<Tablet>,
    /// Touch input devices.
    pub touch: Vec<TouchDevice>,
    /// Switch devices (e.g., lid switch).
    pub switches: Vec<SwitchDevice>,
}

/// A mouse/pointer device.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Mouse {
    /// Device address.
    pub address: String,
    /// Device name.
    pub name: String,
    /// Default pointer speed.
    #[serde(rename = "defaultSpeed")]
    pub default_speed: f64,
    /// Scroll factor.
    #[serde(rename = "scrollFactor")]
    pub scroll_factor: f64,
}

/// A keyboard device.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Keyboard {
    /// Device address.
    pub address: String,
    /// Device name.
    pub name: String,
    /// XKB rules.
    pub rules: String,
    /// XKB model.
    pub model: String,
    /// XKB layout.
    pub layout: String,
    /// XKB variant.
    pub variant: String,
    /// XKB options.
    pub options: String,
    /// Active keymap name.
    pub active_keymap: String,
    /// Caps Lock state.
    #[serde(rename = "capsLock")]
    pub caps_lock: bool,
    /// Num Lock state.
    #[serde(rename = "numLock")]
    pub num_lock: bool,
    /// Whether this is the main keyboard.
    pub main: bool,
}

/// A tablet device.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Tablet {
    /// Device address.
    pub address: String,
    /// Device name (if available).
    #[serde(default)]
    pub name: String,
    /// Tablet type ("tabletPad", "tabletTool", or absent for tablet).
    #[serde(rename = "type")]
    pub tablet_type: String,
    /// Parent device (for tablet pads).
    #[serde(rename = "belongsTo")]
    pub belongs_to: Option<TabletParent>,
}

/// Parent device reference for tablet pads.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TabletParent {
    /// Parent device address.
    pub address: String,
    /// Parent device name.
    pub name: String,
}

/// A touch input device.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TouchDevice {
    /// Device address.
    pub address: String,
    /// Device name.
    pub name: String,
}

/// A switch device (e.g., lid switch).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct SwitchDevice {
    /// Device address.
    pub address: String,
    /// Device name.
    pub name: String,
}

// ── binds ───────────────────────────────────────────────────────────

/// A keybinding entry from the `binds` command.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Bind {
    /// Whether the bind works when locked.
    pub locked: bool,
    /// Whether this is a mouse binding.
    pub mouse: bool,
    /// Whether this triggers on key release.
    pub release: bool,
    /// Whether this repeats when held.
    pub repeat: bool,
    /// Whether this is a long-press binding.
    #[serde(rename = "longPress")]
    pub long_press: bool,
    /// Whether the bind is non-consuming (input passes through).
    pub non_consuming: bool,
    /// Whether the bind has an explicit description.
    pub has_description: bool,
    /// Modifier bitmask (Shift=1, Caps=2, Ctrl=4, Alt=8, Mod2=16, Mod3=32, Super=64, Mod5=128).
    pub modmask: u32,
    /// Submap this bind belongs to.
    pub submap: String,
    /// Universal submap for this bind.
    pub submap_universal: String,
    /// Key name.
    pub key: String,
    /// Key code (0 if not specified).
    pub keycode: i32,
    /// Whether this is a catch-all binding.
    pub catch_all: bool,
    /// Human-readable description.
    pub description: String,
    /// Dispatcher to invoke.
    pub dispatcher: String,
    /// Argument to the dispatcher.
    pub arg: String,
}

// ── cursorpos ───────────────────────────────────────────────────────

/// Response from the `cursorpos` command.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct CursorPosition {
    /// X coordinate.
    pub x: i32,
    /// Y coordinate.
    pub y: i32,
}

// ── animations ──────────────────────────────────────────────────────

/// Response from the `animations` command.
///
/// The JSON is a two-element array: `[[animations], [beziers]]`.
/// Use [`AnimationsResponse::from_json`] to parse.
#[derive(Debug, Clone, Default)]
pub struct AnimationsResponse {
    /// Animation configurations.
    pub animations: Vec<Animation>,
    /// Bezier curve definitions.
    pub beziers: Vec<BezierCurve>,
}

impl AnimationsResponse {
    /// Parse from the raw JSON (a two-element array).
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is malformed.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let raw: (Vec<Animation>, Vec<BezierCurve>) = serde_json::from_str(json)?;
        Ok(Self {
            animations: raw.0,
            beziers: raw.1,
        })
    }
}

/// An animation configuration entry.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Animation {
    /// Animation name (e.g., "windowsIn", "fade").
    pub name: String,
    /// Whether this animation is overridden by user config.
    pub overridden: bool,
    /// Bezier curve name used for this animation.
    pub bezier: String,
    /// Whether the animation is enabled.
    pub enabled: bool,
    /// Animation speed.
    pub speed: f64,
    /// Animation style (e.g., "slide", "popin").
    pub style: String,
}

/// A bezier curve definition.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct BezierCurve {
    /// Curve name.
    pub name: String,
    /// First control point X.
    #[serde(rename = "X0")]
    pub x0: f64,
    /// First control point Y.
    #[serde(rename = "Y0")]
    pub y0: f64,
    /// Second control point X.
    #[serde(rename = "X1")]
    pub x1: f64,
    /// Second control point Y.
    #[serde(rename = "Y1")]
    pub y1: f64,
}

// ── globalshortcuts ─────────────────────────────────────────────────

/// A global shortcut entry from the `globalshortcuts` command.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct GlobalShortcutInfo {
    /// Shortcut identifier in "appid:id" format.
    pub name: String,
    /// Human-readable description.
    pub description: String,
}

// ── workspacerules ──────────────────────────────────────────────────

/// A workspace rule entry from the `workspacerules` command.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct WorkspaceRuleInfo {
    /// Workspace selector string.
    #[serde(rename = "workspaceString")]
    pub workspace_string: String,
    /// Bound monitor name.
    pub monitor: String,
    /// Whether this is the default workspace for the monitor.
    pub default: bool,
    /// Whether the workspace persists when empty.
    pub persistent: bool,
    /// Inner gaps `[top, right, bottom, left]`.
    #[serde(rename = "gapsIn")]
    pub gaps_in: Option<Vec<i64>>,
    /// Outer gaps `[top, right, bottom, left]`.
    #[serde(rename = "gapsOut")]
    pub gaps_out: Option<Vec<i64>>,
    /// Border size in pixels.
    #[serde(rename = "borderSize")]
    pub border_size: Option<i64>,
    /// Whether borders are enabled.
    pub border: Option<bool>,
    /// Whether rounding is enabled.
    pub rounding: Option<bool>,
    /// Whether decorations are enabled.
    pub decorate: Option<bool>,
    /// Whether shadows are enabled.
    pub shadow: Option<bool>,
    /// Default name for the workspace.
    #[serde(rename = "defaultName")]
    pub default_name: String,
}

// ── locked ──────────────────────────────────────────────────────────

/// Response from the `locked` command.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct LockState {
    /// Whether the session is locked.
    pub locked: bool,
}

// ── getoption ───────────────────────────────────────────────────────

/// Response from the `getoption` command.
///
/// The value field depends on the option type. Only one of
/// `int`, `float`, `str`, `vec2`, or `custom` will be present.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct OptionValue {
    /// Option name.
    pub option: String,
    /// Integer value (if option is int type).
    pub int: Option<i64>,
    /// Float value (if option is float type).
    pub float: Option<f64>,
    /// String value (if option is string type).
    pub str: Option<String>,
    /// Vec2 value `[x, y]` (if option is vec2 type).
    pub vec2: Option<[f64; 2]>,
    /// Custom type value (serialized as string).
    pub custom: Option<String>,
    /// Whether the option was explicitly set by the user.
    pub set: bool,
}

// ── decorations ─────────────────────────────────────────────────────

/// A window decoration entry from the `decorations` command.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DecorationInfo {
    /// Decoration name.
    #[serde(rename = "decorationName")]
    pub decoration_name: String,
    /// Rendering priority.
    pub priority: i32,
}

// ── descriptions ────────────────────────────────────────────────────

/// A config option description from the `descriptions` command.
///
/// The `data` field is polymorphic — its shape depends on `option_type`.
/// Use [`serde_json::Value`] accessors to extract type-specific fields
/// like `"min"`, `"max"`, `"value"`, `"current"`, `"explicit"`, etc.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ConfigDescription {
    /// Config key (e.g., "general:gaps_in").
    pub value: String,
    /// Human-readable description.
    pub description: String,
    /// Config option type (0=bool, 1=int, 2=float, 3=string_short,
    /// 4=string_long, 5=color, 6=choice, 7=gradient, 8=vector).
    #[serde(rename = "type")]
    pub option_type: u16,
    /// Option flags bitmask (1=percentage).
    pub flags: u32,
    /// Type-specific data (varies by `option_type`).
    pub data: serde_json::Value,
}

// ── plugin list ─────────────────────────────────────────────────────

/// A loaded plugin from the `plugin list` command.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PluginInfo {
    /// Plugin name.
    pub name: String,
    /// Plugin author.
    pub author: String,
    /// Plugin handle (hex address).
    pub handle: String,
    /// Plugin version string.
    pub version: String,
    /// Plugin description.
    pub description: String,
}
