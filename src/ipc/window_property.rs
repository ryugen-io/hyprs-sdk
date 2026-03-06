//! Typed window properties supported by Hyprland `getprop` / `setprop`.
//!
//! Property names are sourced from Hyprland's `CKeybindManager::setProp`
//! implementation (local source tree).

/// Window property name accepted by Hyprland `getprop` / `setprop`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum WindowProperty {
    MaxSize,
    MinSize,
    ActiveBorderColor,
    InactiveBorderColor,
    Opacity,
    OpacityInactive,
    OpacityFullscreen,
    OpacityOverride,
    OpacityInactiveOverride,
    OpacityFullscreenOverride,
    AllowsInput,
    Decorate,
    FocusOnActivate,
    KeepAspectRatio,
    NearestNeighbor,
    NoAnim,
    NoBlur,
    NoDim,
    NoFocus,
    NoMaxSize,
    NoShadow,
    NoShortcutsInhibit,
    DimAround,
    Opaque,
    ForceRgbx,
    SyncFullscreen,
    Immediate,
    Xray,
    RenderUnfocused,
    NoFollowMouse,
    NoScreenShare,
    NoVrr,
    PersistentSize,
    StayFocused,
    IdleInhibit,
    BorderSize,
    Rounding,
    RoundingPower,
    ScrollMouse,
    ScrollTouchpad,
    Animation,
}

impl WindowProperty {
    /// All known properties from Hyprland's `setProp` implementation.
    pub const ALL: [Self; 41] = [
        Self::MaxSize,
        Self::MinSize,
        Self::ActiveBorderColor,
        Self::InactiveBorderColor,
        Self::Opacity,
        Self::OpacityInactive,
        Self::OpacityFullscreen,
        Self::OpacityOverride,
        Self::OpacityInactiveOverride,
        Self::OpacityFullscreenOverride,
        Self::AllowsInput,
        Self::Decorate,
        Self::FocusOnActivate,
        Self::KeepAspectRatio,
        Self::NearestNeighbor,
        Self::NoAnim,
        Self::NoBlur,
        Self::NoDim,
        Self::NoFocus,
        Self::NoMaxSize,
        Self::NoShadow,
        Self::NoShortcutsInhibit,
        Self::DimAround,
        Self::Opaque,
        Self::ForceRgbx,
        Self::SyncFullscreen,
        Self::Immediate,
        Self::Xray,
        Self::RenderUnfocused,
        Self::NoFollowMouse,
        Self::NoScreenShare,
        Self::NoVrr,
        Self::PersistentSize,
        Self::StayFocused,
        Self::IdleInhibit,
        Self::BorderSize,
        Self::Rounding,
        Self::RoundingPower,
        Self::ScrollMouse,
        Self::ScrollTouchpad,
        Self::Animation,
    ];

    /// Convert a property to its Hyprland wire name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MaxSize => "max_size",
            Self::MinSize => "min_size",
            Self::ActiveBorderColor => "active_border_color",
            Self::InactiveBorderColor => "inactive_border_color",
            Self::Opacity => "opacity",
            Self::OpacityInactive => "opacity_inactive",
            Self::OpacityFullscreen => "opacity_fullscreen",
            Self::OpacityOverride => "opacity_override",
            Self::OpacityInactiveOverride => "opacity_inactive_override",
            Self::OpacityFullscreenOverride => "opacity_fullscreen_override",
            Self::AllowsInput => "allows_input",
            Self::Decorate => "decorate",
            Self::FocusOnActivate => "focus_on_activate",
            Self::KeepAspectRatio => "keep_aspect_ratio",
            Self::NearestNeighbor => "nearest_neighbor",
            Self::NoAnim => "no_anim",
            Self::NoBlur => "no_blur",
            Self::NoDim => "no_dim",
            Self::NoFocus => "no_focus",
            Self::NoMaxSize => "no_max_size",
            Self::NoShadow => "no_shadow",
            Self::NoShortcutsInhibit => "no_shortcuts_inhibit",
            Self::DimAround => "dim_around",
            Self::Opaque => "opaque",
            Self::ForceRgbx => "force_rgbx",
            Self::SyncFullscreen => "sync_fullscreen",
            Self::Immediate => "immediate",
            Self::Xray => "xray",
            Self::RenderUnfocused => "render_unfocused",
            Self::NoFollowMouse => "no_follow_mouse",
            Self::NoScreenShare => "no_screen_share",
            Self::NoVrr => "no_vrr",
            Self::PersistentSize => "persistent_size",
            Self::StayFocused => "stay_focused",
            Self::IdleInhibit => "idle_inhibit",
            Self::BorderSize => "border_size",
            Self::Rounding => "rounding",
            Self::RoundingPower => "rounding_power",
            Self::ScrollMouse => "scroll_mouse",
            Self::ScrollTouchpad => "scroll_touchpad",
            Self::Animation => "animation",
        }
    }

    /// Parse a Hyprland property name to a typed value.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "max_size" => Some(Self::MaxSize),
            "min_size" => Some(Self::MinSize),
            "active_border_color" => Some(Self::ActiveBorderColor),
            "inactive_border_color" => Some(Self::InactiveBorderColor),
            "opacity" => Some(Self::Opacity),
            "opacity_inactive" => Some(Self::OpacityInactive),
            "opacity_fullscreen" => Some(Self::OpacityFullscreen),
            "opacity_override" => Some(Self::OpacityOverride),
            "opacity_inactive_override" => Some(Self::OpacityInactiveOverride),
            "opacity_fullscreen_override" => Some(Self::OpacityFullscreenOverride),
            "allows_input" => Some(Self::AllowsInput),
            "decorate" => Some(Self::Decorate),
            "focus_on_activate" => Some(Self::FocusOnActivate),
            "keep_aspect_ratio" => Some(Self::KeepAspectRatio),
            "nearest_neighbor" => Some(Self::NearestNeighbor),
            "no_anim" => Some(Self::NoAnim),
            "no_blur" => Some(Self::NoBlur),
            "no_dim" => Some(Self::NoDim),
            "no_focus" => Some(Self::NoFocus),
            "no_max_size" => Some(Self::NoMaxSize),
            "no_shadow" => Some(Self::NoShadow),
            "no_shortcuts_inhibit" => Some(Self::NoShortcutsInhibit),
            "dim_around" => Some(Self::DimAround),
            "opaque" => Some(Self::Opaque),
            "force_rgbx" => Some(Self::ForceRgbx),
            "sync_fullscreen" => Some(Self::SyncFullscreen),
            "immediate" => Some(Self::Immediate),
            "xray" => Some(Self::Xray),
            "render_unfocused" => Some(Self::RenderUnfocused),
            "no_follow_mouse" => Some(Self::NoFollowMouse),
            "no_screen_share" => Some(Self::NoScreenShare),
            "no_vrr" => Some(Self::NoVrr),
            "persistent_size" => Some(Self::PersistentSize),
            "stay_focused" => Some(Self::StayFocused),
            "idle_inhibit" => Some(Self::IdleInhibit),
            "border_size" => Some(Self::BorderSize),
            "rounding" => Some(Self::Rounding),
            "rounding_power" => Some(Self::RoundingPower),
            "scroll_mouse" => Some(Self::ScrollMouse),
            "scroll_touchpad" => Some(Self::ScrollTouchpad),
            "animation" => Some(Self::Animation),
            _ => None,
        }
    }
}

impl std::fmt::Display for WindowProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for WindowProperty {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::str::FromStr for WindowProperty {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}
