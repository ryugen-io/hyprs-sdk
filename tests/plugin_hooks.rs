use hyprs_sdk::plugin::HookEvent;

#[test]
fn hook_event_count() {
    assert_eq!(HookEvent::COUNT, 50);
    assert_eq!(HookEvent::ALL.len(), 50);
}

#[test]
fn hook_event_names_roundtrip() {
    for event in &HookEvent::ALL {
        let name = event.event_name();
        let parsed = HookEvent::from_name(name);
        assert_eq!(parsed, Some(*event), "roundtrip failed for {name}");
    }
}

#[test]
fn hook_event_unknown_name() {
    assert_eq!(HookEvent::from_name("nonExistent"), None);
}

#[test]
fn cancellable_events() {
    let cancellable: Vec<_> = HookEvent::ALL
        .iter()
        .filter(|e| e.is_cancellable())
        .collect();
    assert_eq!(cancellable.len(), 14);
}

#[test]
fn non_cancellable_events() {
    let non_cancellable: Vec<_> = HookEvent::ALL
        .iter()
        .filter(|e| !e.is_cancellable())
        .collect();
    assert_eq!(non_cancellable.len(), 36);
}

#[test]
fn specific_cancellable_events() {
    assert!(HookEvent::KeyPress.is_cancellable());
    assert!(HookEvent::MouseMove.is_cancellable());
    assert!(HookEvent::TouchDown.is_cancellable());
    assert!(HookEvent::PinchEnd.is_cancellable());
}

#[test]
fn specific_non_cancellable_events() {
    assert!(!HookEvent::Ready.is_cancellable());
    assert!(!HookEvent::Workspace.is_cancellable());
    assert!(!HookEvent::OpenWindow.is_cancellable());
    assert!(!HookEvent::Render.is_cancellable());
}

#[test]
fn hook_event_display() {
    assert_eq!(HookEvent::Ready.to_string(), "ready");
    assert_eq!(HookEvent::ActiveWindow.to_string(), "activeWindow");
    assert_eq!(HookEvent::SwipeBegin.to_string(), "swipeBegin");
}

#[test]
fn specific_event_names() {
    assert_eq!(HookEvent::PreMonitorAdded.event_name(), "preMonitorAdded");
    assert_eq!(HookEvent::MoveWorkspace.event_name(), "moveWorkspace");
    assert_eq!(
        HookEvent::ChangeFloatingMode.event_name(),
        "changeFloatingMode"
    );
    assert_eq!(
        HookEvent::WindowUpdateRules.event_name(),
        "windowUpdateRules"
    );
    assert_eq!(HookEvent::Screencast.event_name(), "screencast");
}
