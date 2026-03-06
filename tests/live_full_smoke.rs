use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hyprs_sdk::dispatch;
use hyprs_sdk::ipc::{Event, EventStream, Flags, HyprlandClient, WindowProperty};
use hyprs_sdk::{HyprError, HyprResult};

fn unique_payload(tag: &str) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_nanos();
    format!("hyprs-sdk-live-{tag}-{ts}")
}

fn env_enabled(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => matches!(v.as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => false,
    }
}

async fn strict_step<T, F>(name: &str, fut: F) -> HyprResult<T>
where
    F: std::future::Future<Output = HyprResult<T>>,
{
    println!("[live-full-smoke] {name}: running");
    fut.await
        .map_err(|e| HyprError::Parse(format!("{name} failed: {e}")))
}

async fn maybe_step<T, F>(name: &str, best_effort: bool, fut: F) -> HyprResult<Option<T>>
where
    F: std::future::Future<Output = HyprResult<T>>,
{
    println!("[live-full-smoke] {name}: running");
    match fut.await {
        Ok(value) => {
            println!("[live-full-smoke] {name}: ok");
            Ok(Some(value))
        }
        Err(err) if best_effort => {
            println!("[live-full-smoke] {name}: skipped ({err})");
            Ok(None)
        }
        Err(err) => Err(HyprError::Parse(format!("{name} failed: {err}"))),
    }
}

async fn wait_for_custom_event(
    events: &mut EventStream,
    expected_payload: &str,
    timeout_total: Duration,
) -> HyprResult<()> {
    let started = tokio::time::Instant::now();
    while started.elapsed() < timeout_total {
        let next = tokio::time::timeout(Duration::from_millis(250), events.next_event()).await;
        match next {
            Ok(Ok(Some(Event::Custom { data }))) if data == expected_payload => return Ok(()),
            Ok(Ok(Some(_))) => {}
            Ok(Ok(None)) => break,
            Ok(Err(e)) => return Err(e),
            Err(_) => {}
        }
    }

    Err(HyprError::Parse(format!(
        "timed out waiting for custom event payload: {expected_payload}"
    )))
}

fn parse_int_option(name: &str, value: hyprs_sdk::ipc::responses::OptionValue) -> HyprResult<i64> {
    value.int.ok_or_else(|| {
        HyprError::Parse(format!(
            "option '{name}' did not expose an integer value in get_option_typed"
        ))
    })
}

#[tokio::test]
#[ignore = "requires a running Hyprland session"]
async fn live_full_smoke_suite() -> HyprResult<()> {
    let visual = env_enabled("HYPR_SDK_SMOKE_VISUAL");
    let mutating = env_enabled("HYPR_SDK_SMOKE_MUTATING");
    let best_effort = env_enabled("HYPR_SDK_SMOKE_BEST_EFFORT");

    let client = HyprlandClient::current()?;
    println!(
        "[live-full-smoke] visual={}, mutating={}, best_effort={}",
        visual, mutating, best_effort
    );

    // ---- Read paths: typed -------------------------------------------------
    let version = strict_step("version_typed", client.version_typed()).await?;
    if version.tag.is_empty() && version.commit.is_empty() && version.version.is_empty() {
        return Err(HyprError::Parse(
            "version response is missing tag, commit, and version".into(),
        ));
    }

    let monitors = strict_step("monitors_typed", client.monitors_typed()).await?;
    let clients = strict_step("clients_typed", client.clients_typed()).await?;
    let workspaces = strict_step("workspaces_typed", client.workspaces_typed()).await?;
    let _ = strict_step("active_workspace_typed", client.active_workspace_typed()).await?;

    let _ = maybe_step("layers_typed", best_effort, client.layers_typed()).await?;
    let _ = maybe_step("devices_typed", best_effort, client.devices_typed()).await?;
    let _ = maybe_step("binds_typed", best_effort, client.binds_typed()).await?;
    let _ = maybe_step("cursor_pos_typed", best_effort, client.cursor_pos_typed()).await?;
    let _ = maybe_step("animations_typed", best_effort, client.animations_typed()).await?;
    let _ = maybe_step(
        "global_shortcuts_typed",
        best_effort,
        client.global_shortcuts_typed(),
    )
    .await?;
    let _ = maybe_step(
        "workspace_rules_typed",
        best_effort,
        client.workspace_rules_typed(),
    )
    .await?;
    let _ = maybe_step("layouts_typed", best_effort, client.layouts_typed()).await?;
    let _ = maybe_step(
        "config_errors_typed",
        best_effort,
        client.config_errors_typed(),
    )
    .await?;
    let _ = maybe_step("locked_typed", best_effort, client.locked_typed()).await?;
    let _ = maybe_step(
        "descriptions_typed",
        best_effort,
        client.descriptions_typed(),
    )
    .await?;
    let _ = maybe_step("plugin_list_typed", best_effort, client.plugin_list_typed()).await?;

    if monitors.iter().any(|m| m.name.is_empty()) {
        return Err(HyprError::Parse(
            "received monitor with empty name".to_string(),
        ));
    }
    if workspaces.iter().any(|w| w.name.is_empty()) {
        return Err(HyprError::Parse(
            "received workspace with empty name".to_string(),
        ));
    }

    // ---- Read paths: raw + flagged ----------------------------------------
    let _ = maybe_step("splash", best_effort, client.splash()).await?;
    let _ = maybe_step("submap", best_effort, client.submap()).await?;
    let _ = maybe_step(
        "system_info_raw",
        best_effort,
        client.system_info(Flags::default()),
    )
    .await?;
    let _ = maybe_step(
        "rolling_log_raw",
        best_effort,
        client.rolling_log(Flags::default()),
    )
    .await?;
    let _ = maybe_step(
        "version_raw_json",
        best_effort,
        client.version(Flags::json()),
    )
    .await?;
    let _ = maybe_step("locked_raw_json", best_effort, client.locked(Flags::json())).await?;
    let _ = maybe_step(
        "descriptions_raw_json",
        best_effort,
        client.descriptions(Flags::json()),
    )
    .await?;
    let _ = maybe_step(
        "monitors_raw_json",
        best_effort,
        client.monitors(Flags::json()),
    )
    .await?;
    let _ = maybe_step(
        "clients_raw_json",
        best_effort,
        client.clients(Flags::json()),
    )
    .await?;
    let _ = maybe_step(
        "workspaces_raw_json",
        best_effort,
        client.workspaces(Flags::json()),
    )
    .await?;
    let _ = maybe_step(
        "active_workspace_raw_json",
        best_effort,
        client.active_workspace(Flags::json()),
    )
    .await?;
    let _ = maybe_step("layers_raw_json", best_effort, client.layers(Flags::json())).await?;
    let _ = maybe_step(
        "request_flagged_monitors",
        best_effort,
        client.request_flagged(Flags::json(), "monitors"),
    )
    .await?;
    let _ = maybe_step(
        "request_flagged_plugin_list",
        best_effort,
        client.request_flagged(Flags::json(), "plugin list"),
    )
    .await?;
    let _ = maybe_step(
        "get_option_raw_json",
        best_effort,
        client.get_option("general:border_size", Flags::json()),
    )
    .await?;

    if let Some(window) = clients.first() {
        let addr = window.address.to_string();
        let prop = WindowProperty::Decorate;
        let _ = maybe_step(
            "get_prop_raw_json",
            best_effort,
            client.get_prop(&addr, prop.as_str(), Flags::json()),
        )
        .await?;
        let _ = maybe_step(
            "get_prop_value_json",
            best_effort,
            client.get_prop_value(&addr, prop.as_str()),
        )
        .await?;
        let _ = maybe_step(
            "decorations_raw_json",
            best_effort,
            client.decorations(&addr, Flags::json()),
        )
        .await?;
        let _ = maybe_step(
            "decorations_typed",
            best_effort,
            client.decorations_typed(&addr),
        )
        .await?;
    }

    // ---- Dispatch + event stream roundtrip --------------------------------
    let socket2 = strict_step("event_stream", client.event_stream()).await?;
    let mut events = EventStream::new(socket2);

    let payload_a = unique_payload("dispatch-cmd");
    strict_step(
        "dispatch_cmd(event)",
        client.dispatch_cmd(dispatch::misc::event(&payload_a)),
    )
    .await?;
    strict_step(
        "event_roundtrip_cmd",
        wait_for_custom_event(&mut events, &payload_a, Duration::from_secs(10)),
    )
    .await?;

    let payload_b = unique_payload("dispatch-raw");
    strict_step("dispatch_raw(event)", client.dispatch("event", &payload_b)).await?;
    strict_step(
        "event_roundtrip_raw",
        wait_for_custom_event(&mut events, &payload_b, Duration::from_secs(10)),
    )
    .await?;

    let batch_cmds = vec!["j/version".to_string(), "j/monitors".to_string()];
    let batch_response = strict_step("batch", client.batch(&batch_cmds)).await?;
    if batch_response.trim().is_empty() {
        return Err(HyprError::Parse(
            "batch response unexpectedly empty".to_string(),
        ));
    }

    // ---- Set paths (safe + restore) ---------------------------------------
    let option_name = "general:border_size";
    let original = parse_int_option(
        option_name,
        strict_step(
            "get_option_typed(original)",
            client.get_option_typed(option_name),
        )
        .await?,
    )?;
    let mutated = if original == 0 { 1 } else { original + 1 };

    let mutate_result = async {
        strict_step(
            "keyword(set mutated)",
            client.keyword(option_name, &mutated.to_string()),
        )
        .await?;

        let updated = parse_int_option(
            option_name,
            strict_step(
                "get_option_typed(updated)",
                client.get_option_typed(option_name),
            )
            .await?,
        )?;
        if updated != mutated {
            return Err(HyprError::Parse(format!(
                "keyword update mismatch for '{option_name}': expected {mutated}, got {updated}"
            )));
        }
        Ok(())
    }
    .await;

    let restore_attempt = client.keyword(option_name, &original.to_string()).await;
    if let Err(e) = mutate_result {
        let _ = restore_attempt;
        return Err(e);
    }
    restore_attempt.map_err(|e| HyprError::Parse(format!("restore keyword failed: {e}")))?;
    let restored = parse_int_option(
        option_name,
        strict_step(
            "get_option_typed(restored)",
            client.get_option_typed(option_name),
        )
        .await?,
    )?;
    if restored != original {
        return Err(HyprError::Parse(format!(
            "failed to restore '{option_name}': expected {original}, got {restored}"
        )));
    }

    if visual {
        strict_step(
            "notify(visual marker)",
            client.notify(1, 1800, "0", "hyprs-sdk live smoke: visual marker"),
        )
        .await?;
        strict_step(
            "set_error(show)",
            client.set_error("hyprs-sdk live smoke running (visual mode)"),
        )
        .await?;
        tokio::time::sleep(Duration::from_millis(550)).await;
        strict_step("set_error(clear)", client.set_error("")).await?;
    } else {
        strict_step(
            "notify(minimal)",
            client.notify(0, 1, "0", "hyprs-sdk-live-smoke"),
        )
        .await?;
    }
    strict_step("dismiss_notify", client.dismiss_notify(-1)).await?;

    if mutating {
        strict_step("reload", client.reload("")).await?;
        strict_step("reload_shaders", client.reload_shaders()).await?;
        strict_step(
            "dispatch_cmd(forcerendererreload)",
            client.dispatch_cmd(dispatch::misc::force_renderer_reload()),
        )
        .await?;
    }

    println!(
        "[live-full-smoke] ok (monitors={}, clients={}, workspaces={})",
        monitors.len(),
        clients.len(),
        workspaces.len()
    );
    Ok(())
}
