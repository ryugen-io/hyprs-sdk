use std::collections::BTreeSet;
use std::process::Command;

use hyprs_sdk::ipc::commands::{self, Flags};
use hyprs_sdk::ipc::{HyprlandClient, WindowProperty};
use hyprs_sdk::{HyprError, HyprResult};
use serde_json::Value;

#[derive(Debug, Clone)]
struct ParityCase {
    name: String,
    sdk_command: String,
    hyprctl_args: Vec<String>,
}

fn run_hyprctl(args: &[String]) -> HyprResult<String> {
    let output = Command::new("hyprctl")
        .args(args)
        .output()
        .map_err(HyprError::Io)?;

    if !output.status.success() {
        return Err(HyprError::Command(format!(
            "hyprctl {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| HyprError::Parse(format!("hyprctl stdout was not utf8: {e}")))
}

fn parse_json(label: &str, raw: &str) -> HyprResult<Value> {
    serde_json::from_str(raw)
        .map_err(|e| HyprError::Parse(format!("{label} json parse failed: {e}")))
}

fn shape(value: &Value) -> Value {
    match value {
        Value::Null => Value::String("null".into()),
        Value::Bool(_) => Value::String("bool".into()),
        Value::Number(_) => Value::String("number".into()),
        Value::String(_) => Value::String("string".into()),
        Value::Array(items) => {
            let mut uniq = BTreeSet::new();
            for item in items.iter().take(64) {
                let s =
                    serde_json::to_string(&shape(item)).unwrap_or_else(|_| "\"invalid\"".into());
                uniq.insert(s);
            }
            let normalized = uniq
                .into_iter()
                .map(|s| serde_json::from_str(&s).unwrap_or(Value::String(s)))
                .collect::<Vec<_>>();
            Value::Array(normalized)
        }
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            let mut keys = map.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                if let Some(v) = map.get(&key) {
                    out.insert(key, shape(v));
                }
            }
            Value::Object(out)
        }
    }
}

#[tokio::test]
#[ignore = "requires running Hyprland + hyprctl installed"]
async fn live_cli_parity_for_json_queries() -> HyprResult<()> {
    let _ = Command::new("hyprctl")
        .arg("--help")
        .output()
        .map_err(|e| HyprError::Parse(format!("hyprctl not available: {e}")))?;

    let client = HyprlandClient::current()?;
    let mut cases = vec![
        ParityCase {
            name: "monitors".into(),
            sdk_command: commands::monitors(Flags::json()),
            hyprctl_args: vec!["-j".into(), "monitors".into()],
        },
        ParityCase {
            name: "workspaces".into(),
            sdk_command: commands::workspaces(Flags::json()),
            hyprctl_args: vec!["-j".into(), "workspaces".into()],
        },
        ParityCase {
            name: "workspacerules".into(),
            sdk_command: commands::workspace_rules(Flags::json()),
            hyprctl_args: vec!["-j".into(), "workspacerules".into()],
        },
        ParityCase {
            name: "clients".into(),
            sdk_command: commands::clients(Flags::json()),
            hyprctl_args: vec!["-j".into(), "clients".into()],
        },
        ParityCase {
            name: "layers".into(),
            sdk_command: commands::layers(Flags::json()),
            hyprctl_args: vec!["-j".into(), "layers".into()],
        },
        ParityCase {
            name: "devices".into(),
            sdk_command: commands::devices(Flags::json()),
            hyprctl_args: vec!["-j".into(), "devices".into()],
        },
        ParityCase {
            name: "binds".into(),
            sdk_command: commands::binds(Flags::json()),
            hyprctl_args: vec!["-j".into(), "binds".into()],
        },
        ParityCase {
            name: "animations".into(),
            sdk_command: commands::animations(Flags::json()),
            hyprctl_args: vec!["-j".into(), "animations".into()],
        },
        ParityCase {
            name: "globalshortcuts".into(),
            sdk_command: commands::global_shortcuts(Flags::json()),
            hyprctl_args: vec!["-j".into(), "globalshortcuts".into()],
        },
        ParityCase {
            name: "layers".into(),
            sdk_command: commands::layers(Flags::json()),
            hyprctl_args: vec!["-j".into(), "layers".into()],
        },
        ParityCase {
            name: "configerrors".into(),
            sdk_command: commands::config_errors(Flags::json()),
            hyprctl_args: vec!["-j".into(), "configerrors".into()],
        },
        ParityCase {
            name: "locked".into(),
            sdk_command: commands::locked(Flags::json()),
            hyprctl_args: vec!["-j".into(), "locked".into()],
        },
        ParityCase {
            name: "descriptions".into(),
            sdk_command: commands::descriptions(Flags::json()),
            hyprctl_args: vec!["-j".into(), "descriptions".into()],
        },
        ParityCase {
            name: "version".into(),
            sdk_command: commands::version(Flags::json()),
            hyprctl_args: vec!["-j".into(), "version".into()],
        },
        ParityCase {
            name: "getoption-general:border_size".into(),
            sdk_command: commands::get_option("general:border_size", Flags::json()),
            hyprctl_args: vec![
                "-j".into(),
                "getoption".into(),
                "general:border_size".into(),
            ],
        },
        ParityCase {
            name: "plugin-list".into(),
            sdk_command: "j/plugin list".into(),
            hyprctl_args: vec!["-j".into(), "plugin".into(), "list".into()],
        },
    ];

    let clients = client.clients_typed().await?;
    if let Some(window) = clients.first() {
        let addr = window.address.to_string();
        let selector = format!("address:{addr}");
        let prop = WindowProperty::Decorate;
        cases.push(ParityCase {
            name: "decorations".into(),
            sdk_command: commands::decorations(&selector, Flags::json()),
            hyprctl_args: vec!["-j".into(), "decorations".into(), selector.clone()],
        });
        cases.push(ParityCase {
            name: format!("getprop-{}", prop.as_str()),
            sdk_command: commands::get_prop(&selector, prop.as_str(), Flags::json()),
            hyprctl_args: vec![
                "-j".into(),
                "getprop".into(),
                selector,
                prop.as_str().to_string(),
            ],
        });
    }

    for case in cases {
        let sdk_raw = client.request(&case.sdk_command).await?;
        let hyprctl_raw = run_hyprctl(&case.hyprctl_args)?;

        let sdk_json = parse_json(&format!("sdk {}", case.name), &sdk_raw)?;
        let hyprctl_json = parse_json(&format!("hyprctl {}", case.name), &hyprctl_raw)?;

        let sdk_shape = shape(&sdk_json);
        let hyprctl_shape = shape(&hyprctl_json);
        if sdk_shape != hyprctl_shape {
            return Err(HyprError::Parse(format!(
                "CLI parity shape mismatch for '{}'\nSDK shape: {}\nhyprctl shape: {}",
                case.name, sdk_shape, hyprctl_shape
            )));
        }
    }

    Ok(())
}
