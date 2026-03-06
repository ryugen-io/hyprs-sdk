use hyprs_sdk::plugin::*;

#[test]
fn plugin_handle_null() {
    let handle = PluginHandle::NULL;
    assert!(handle.is_null());
}

#[test]
fn plugin_handle_non_null() {
    let mut dummy: u8 = 42;
    let handle = PluginHandle(std::ptr::addr_of_mut!(dummy).cast());
    assert!(!handle.is_null());
}

#[test]
fn plugin_description_default() {
    let desc = PluginDescription::default();
    assert!(desc.name.is_empty());
    assert!(desc.description.is_empty());
    assert!(desc.author.is_empty());
    assert!(desc.version.is_empty());
}

#[test]
fn dispatch_result_ok() {
    let r = DispatchResult::ok();
    assert!(r.success);
    assert!(!r.pass_event);
    assert!(r.error.is_empty());
}

#[test]
fn dispatch_result_err() {
    let r = DispatchResult::err("something failed");
    assert!(!r.success);
    assert_eq!(r.error, "something failed");
}

#[test]
fn dispatch_result_pass() {
    let r = DispatchResult::pass();
    assert!(r.success);
    assert!(r.pass_event);
}

#[test]
fn callback_info_default() {
    let info = CallbackInfo::default();
    assert!(!info.cancelled);
}

#[test]
fn notification_icon_from_raw() {
    assert_eq!(
        NotificationIcon::from_raw(0),
        Some(NotificationIcon::Warning)
    );
    assert_eq!(NotificationIcon::from_raw(3), Some(NotificationIcon::Error));
    assert_eq!(NotificationIcon::from_raw(6), Some(NotificationIcon::None));
    assert_eq!(NotificationIcon::from_raw(7), Option::None);
}

#[test]
fn notification_icon_display() {
    assert_eq!(NotificationIcon::Warning.to_string(), "warning");
    assert_eq!(NotificationIcon::Ok.to_string(), "ok");
}

#[test]
fn notification_icon_default() {
    let icon = NotificationIcon::default();
    assert_eq!(icon, NotificationIcon::None);
}

#[test]
fn render_stage_from_raw() {
    assert_eq!(RenderStage::from_raw(0), Some(RenderStage::Pre));
    assert_eq!(RenderStage::from_raw(5), Some(RenderStage::LastMoment));
    assert_eq!(RenderStage::from_raw(9), Some(RenderStage::PostWindow));
    assert_eq!(RenderStage::from_raw(10), None);
}

#[test]
fn render_stage_display() {
    assert_eq!(RenderStage::Pre.to_string(), "RENDER_PRE");
    assert_eq!(RenderStage::PostMirror.to_string(), "RENDER_POST_MIRROR");
}

#[test]
fn hyprctl_output_format_default() {
    let fmt = HyprCtlOutputFormat::default();
    assert_eq!(fmt, HyprCtlOutputFormat::Normal);
}

#[test]
fn input_type_from_raw() {
    assert_eq!(InputType::from_raw(0), Some(InputType::Axis));
    assert_eq!(InputType::from_raw(4), Some(InputType::Motion));
    assert_eq!(InputType::from_raw(5), None);
}

#[test]
fn api_version_constant() {
    assert_eq!(HYPRLAND_API_VERSION, "0.1");
    assert_eq!(HYPRLAND_API_VERSION_CSTR, b"0.1\0");
}

#[test]
fn version_info_default() {
    let v = VersionInfo::default();
    assert!(v.hash.is_empty());
    assert!(!v.dirty);
}
