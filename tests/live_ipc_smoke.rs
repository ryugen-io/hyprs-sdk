use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hyprs_sdk::dispatch;
use hyprs_sdk::ipc::{Event, EventStream, HyprlandClient};

fn unique_event_payload() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_nanos();
    format!("hyprs-sdk-live-smoke-{ts}")
}

#[tokio::test]
#[ignore = "requires a running Hyprland session"]
async fn live_ipc_smoke() -> hyprs_sdk::HyprResult<()> {
    let client = HyprlandClient::current()?;

    let version = client.version_typed().await?;
    if version.tag.is_empty() && version.commit.is_empty() && version.version.is_empty() {
        return Err(hyprs_sdk::HyprError::Parse(
            "version response is missing tag, commit, and version".into(),
        ));
    }

    let monitors = client.monitors_typed().await?;
    if monitors.iter().any(|m| m.name.is_empty()) {
        return Err(hyprs_sdk::HyprError::Parse(
            "received monitor with empty name".into(),
        ));
    }

    let socket2 = client.event_stream().await?;
    let mut events = EventStream::new(socket2);

    let payload = unique_event_payload();
    client.dispatch_cmd(dispatch::misc::event(&payload)).await?;

    let mut seen = false;
    for _ in 0..50 {
        let next = tokio::time::timeout(Duration::from_millis(200), events.next_event()).await;
        match next {
            Ok(Ok(Some(Event::Custom { data }))) if data == payload => {
                seen = true;
                break;
            }
            Ok(Ok(Some(_))) => {}
            Ok(Ok(None)) => break,
            Ok(Err(e)) => return Err(e),
            Err(_) => {}
        }
    }

    if !seen {
        return Err(hyprs_sdk::HyprError::Parse(
            "timed out waiting for custom event roundtrip".into(),
        ));
    }

    Ok(())
}
