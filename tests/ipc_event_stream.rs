use hyprs_sdk::ipc::socket;
use hyprs_sdk::ipc::{Event, EventStream};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;

fn unique_sock_path() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("hyprs-sdk-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create test dir");
    dir.join(format!("{}.sock", uuid()))
}

fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_nanos();
    format!("{nanos:x}")
}

#[tokio::test]
async fn event_stream_skips_malformed_lines() {
    let sock = unique_sock_path();
    let listener = UnixListener::bind(&sock).expect("bind unix listener");

    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept client");
        stream
            .write_all(b"bad_line_without_separator\nworkspace>>main\n")
            .await
            .expect("write event payload");
    });

    let stream = socket::connect_event_stream(&sock)
        .await
        .expect("connect event stream");
    let mut events = EventStream::new(stream);

    let first = events.next_event().await.expect("read first event");
    assert_eq!(
        first,
        Some(Event::Workspace {
            name: "main".into()
        })
    );

    let end = events.next_event().await.expect("read stream end");
    assert_eq!(end, None);

    server.await.expect("join server task");
}
