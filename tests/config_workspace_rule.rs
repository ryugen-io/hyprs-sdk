use hypr_sdk::config::*;
use hypr_sdk::types::common::WorkspaceId;

#[test]
fn workspace_rule_defaults() {
    let rule = WorkspaceRule::default();
    assert!(rule.monitor.is_empty());
    assert_eq!(rule.workspace_id, WorkspaceId(-1));
    assert!(!rule.is_default);
    assert!(!rule.is_persistent);
    assert!(rule.gaps_in.is_none());
    assert!(rule.border_size.is_none());
    assert!(rule.layout_opts.is_empty());
}

#[test]
fn workspace_rule_with_gaps() {
    let rule = WorkspaceRule {
        workspace_name: "dev".to_string(),
        workspace_id: WorkspaceId(1),
        gaps_in: Some(CssGapData::uniform(5)),
        gaps_out: Some(CssGapData::symmetric(10, 20)),
        ..Default::default()
    };
    assert_eq!(rule.gaps_in.unwrap().top, 5);
    assert_eq!(rule.gaps_out.unwrap().right, 20);
}

#[test]
fn workspace_rule_with_options() {
    let mut rule = WorkspaceRule {
        workspace_name: "main".to_string(),
        workspace_id: WorkspaceId(1),
        is_default: true,
        is_persistent: true,
        decorate: Some(true),
        no_rounding: Some(false),
        ..Default::default()
    };
    rule.layout_opts
        .insert("orientation".to_string(), "left".to_string());
    assert_eq!(rule.layout_opts["orientation"], "left");
    assert!(rule.is_persistent);
}

#[test]
fn workspace_rule_on_created_empty() {
    let rule = WorkspaceRule {
        workspace_name: "browser".to_string(),
        workspace_id: WorkspaceId(2),
        on_created_empty_run_cmd: Some("firefox".to_string()),
        ..Default::default()
    };
    assert_eq!(rule.on_created_empty_run_cmd.as_deref(), Some("firefox"));
}
