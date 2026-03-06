use hyprs_sdk::HyprError;
use hyprs_sdk::hyprpm::{HyprPm, parse_list_output};

#[test]
fn missing_binary_returns_io_error() {
    let pm = HyprPm::with_binary("/definitely/not/a/real/hyprpm");
    let err = pm.list().expect_err("expected missing binary error");
    assert!(matches!(err, HyprError::Io(_)));
}

#[test]
fn parses_list_output() {
    let raw = "\
\u{1b}[0m→\u{1b}[0m Repository hyprland-plugins (by ):
  │ Plugin hyprexpo
  └─ enabled: \u{1b}[32mtrue

  │ Plugin hyprbars
  └─ enabled: \u{1b}[31mfalse
";
    let parsed = parse_list_output(raw);
    assert_eq!(parsed.repositories.len(), 1);
    assert_eq!(parsed.repositories[0].name, "hyprland-plugins");
    assert_eq!(parsed.repositories[0].plugins.len(), 2);
    assert_eq!(parsed.repositories[0].plugins[0].name, "hyprexpo");
    assert!(parsed.repositories[0].plugins[0].enabled);
    assert_eq!(parsed.repositories[0].plugins[1].name, "hyprbars");
    assert!(!parsed.repositories[0].plugins[1].enabled);
}
