use std::path::PathBuf;
use std::time::Duration;

use hypr_sdk::error::{HyprError, HyprResult};
use hypr_sdk::ipc::HyprlandClient;

fn plugin_so_path() -> HyprResult<PathBuf> {
    let so = std::env::var("HYPR_SDK_TEST_PLUGIN_SO")
        .map_err(|_| HyprError::Parse("HYPR_SDK_TEST_PLUGIN_SO is not set".into()))?;
    let path = PathBuf::from(so);
    if !path.is_file() {
        return Err(HyprError::Parse(format!(
            "test plugin shared object not found: {}",
            path.display()
        )));
    }
    Ok(path)
}

fn plugin_name() -> String {
    std::env::var("HYPR_SDK_TEST_PLUGIN_NAME").unwrap_or_else(|_| "hypr-sdk-smoke-plugin".into())
}

async fn wait_for_plugin_state(
    client: &HyprlandClient,
    name: &str,
    should_exist: bool,
) -> HyprResult<()> {
    for _ in 0..40 {
        let plugins = client.plugin_list_typed().await?;
        let exists = plugins.iter().any(|p| p.name == name);
        if exists == should_exist {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    let state = if should_exist { "appear" } else { "disappear" };
    Err(HyprError::Parse(format!(
        "timed out waiting for plugin '{name}' to {state}"
    )))
}

async fn require_ok_plugin_response(client: &HyprlandClient, operation: &str) -> HyprResult<()> {
    let response = client.plugin(operation).await?;
    if response.trim() == "ok" {
        Ok(())
    } else {
        Err(HyprError::Command(response))
    }
}

#[tokio::test]
#[ignore = "requires running Hyprland + built test plugin"]
async fn live_plugin_load_unload_roundtrip() -> HyprResult<()> {
    let so_path = plugin_so_path()?;
    let so = so_path.display().to_string();
    let name = plugin_name();

    let client = HyprlandClient::current()?;

    // Best-effort cleanup if previous runs left the plugin loaded.
    let _ = client.plugin(&format!("unload {so}")).await;

    let result = async {
        require_ok_plugin_response(&client, &format!("load {so}")).await?;
        wait_for_plugin_state(&client, &name, true).await?;

        require_ok_plugin_response(&client, &format!("unload {so}")).await?;
        wait_for_plugin_state(&client, &name, false).await
    }
    .await;

    // Always try to leave Hyprland clean.
    let _ = client.plugin(&format!("unload {so}")).await;
    let _ = wait_for_plugin_state(&client, &name, false).await;

    result
}
