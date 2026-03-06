use hyprs_sdk::ipc::WindowProperty;

#[test]
fn window_property_list_matches_hyprland_setprop_source() {
    let expected = [
        "max_size",
        "min_size",
        "active_border_color",
        "inactive_border_color",
        "opacity",
        "opacity_inactive",
        "opacity_fullscreen",
        "opacity_override",
        "opacity_inactive_override",
        "opacity_fullscreen_override",
        "allows_input",
        "decorate",
        "focus_on_activate",
        "keep_aspect_ratio",
        "nearest_neighbor",
        "no_anim",
        "no_blur",
        "no_dim",
        "no_focus",
        "no_max_size",
        "no_shadow",
        "no_shortcuts_inhibit",
        "dim_around",
        "opaque",
        "force_rgbx",
        "sync_fullscreen",
        "immediate",
        "xray",
        "render_unfocused",
        "no_follow_mouse",
        "no_screen_share",
        "no_vrr",
        "persistent_size",
        "stay_focused",
        "idle_inhibit",
        "border_size",
        "rounding",
        "rounding_power",
        "scroll_mouse",
        "scroll_touchpad",
        "animation",
    ];

    let actual = WindowProperty::ALL.map(WindowProperty::as_str);
    assert_eq!(actual, expected);
}

#[test]
fn window_property_roundtrip() {
    for property in WindowProperty::ALL {
        assert_eq!(WindowProperty::parse(property.as_str()), Some(property));
        assert_eq!(
            property.as_str().parse::<WindowProperty>().ok(),
            Some(property)
        );
        assert_eq!(property.to_string(), property.as_str());
    }
}

#[test]
fn window_property_rejects_unknown_names() {
    assert_eq!(WindowProperty::parse("title"), None);
    assert_eq!(WindowProperty::parse("foo_bar"), None);
    assert!("title".parse::<WindowProperty>().is_err());
}
