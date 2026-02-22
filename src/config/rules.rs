//! Window and layer rule effect types.
//!
//! Maps to `eWindowRuleEffect` and `eLayerRuleEffect` from
//! `src/desktop/rule/*/` in Hyprland.

/// Window rule effect type.
///
/// Each variant represents a rule that can be applied to windows
/// matching certain criteria. Maps to `eWindowRuleEffect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WindowRuleEffect {
    // Static effects are applied once when the window is first mapped and never
    // re-evaluated, so they must capture the initial desired state.
    Float,
    Tile,
    Fullscreen,
    Maximize,
    FullscreenState,
    Move,
    Size,
    Center,
    Pseudo,
    Monitor,
    Workspace,
    NoInitialFocus,
    Pin,
    Group,
    SuppressEvent,
    Content,
    NoCloseFor,

    // Dynamic effects are re-evaluated whenever window properties change (e.g. title,
    // class, focus state), allowing rules to respond to runtime state transitions.
    Rounding,
    RoundingPower,
    PersistentSize,
    Animation,
    BorderColor,
    IdleInhibit,
    Opacity,
    Tag,
    MaxSize,
    MinSize,
    BorderSize,
    AllowsInput,
    DimAround,
    Decorate,
    FocusOnActivate,
    KeepAspectRatio,
    NearestNeighbor,
    NoAnim,
    NoBlur,
    NoDim,
    NoFocus,
    NoFollowMouse,
    NoMaxSize,
    NoShadow,
    NoShortcutsInhibit,
    Opaque,
    ForceRgbx,
    SyncFullscreen,
    Immediate,
    Xray,
    RenderUnfocused,
    NoScreenShare,
    NoVrr,
    ScrollMouse,
    ScrollTouchpad,
    StayFocused,
}

/// Layer rule effect type.
///
/// Each variant represents a rule that can be applied to layer surfaces
/// matching certain criteria. Maps to `eLayerRuleEffect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LayerRuleEffect {
    NoAnim,
    Blur,
    BlurPopups,
    IgnoreAlpha,
    DimAround,
    Xray,
    Animation,
    Order,
    AboveLock,
    NoScreenShare,
}

/// A window rule with its effect and associated value string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowRule {
    /// The effect this rule applies.
    pub effect: WindowRuleEffect,
    /// The value/argument string (e.g. `"0.9 0.8"` for opacity).
    pub value: String,
}

/// A layer rule with its effect and associated value string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerRule {
    /// The effect this rule applies.
    pub effect: LayerRuleEffect,
    /// The value/argument string.
    pub value: String,
}
