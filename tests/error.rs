use hypr_sdk::error::HyprError;

#[test]
fn error_display_io() {
    let err = HyprError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "socket missing",
    ));
    assert!(err.to_string().contains("socket missing"));
}

#[test]
fn error_display_parse() {
    let err = HyprError::Parse("bad json".into());
    assert!(err.to_string().contains("bad json"));
}

#[test]
fn error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken");
    let err: HyprError = io_err.into();
    assert!(matches!(err, HyprError::Io(_)));
}

#[test]
fn error_from_serde() {
    let json_err = serde_json::from_str::<String>("not json").unwrap_err();
    let err: HyprError = json_err.into();
    assert!(matches!(err, HyprError::Json(_)));
}
