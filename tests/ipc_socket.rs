use std::path::PathBuf;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;

fn test_sock_path(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("hyprs-sdk-test-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("test.sock")
}

fn cleanup(path: &std::path::Path) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
}

#[tokio::test]
async fn request_sends_command_and_receives_response() {
    let sock = test_sock_path("request");
    let listener = UnixListener::bind(&sock).unwrap();

    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut command = String::new();
        stream.read_to_string(&mut command).await.unwrap();
        assert_eq!(command, "j/monitors");
        stream.write_all(b"[{\"id\":1}]").await.unwrap();
    });

    let response = hyprs_sdk::ipc::socket::request(&sock, "j/monitors")
        .await
        .unwrap();
    assert_eq!(response, "[{\"id\":1}]");

    server.await.unwrap();
    cleanup(&sock);
}

#[tokio::test]
async fn request_handles_empty_response() {
    let sock = test_sock_path("empty");
    let listener = UnixListener::bind(&sock).unwrap();

    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        // WHY: Needed for correctness and maintainability: Server closes without writing anything.
    });

    let response = hyprs_sdk::ipc::socket::request(&sock, "version")
        .await
        .unwrap();
    assert!(response.is_empty());

    server.await.unwrap();
    cleanup(&sock);
}

#[tokio::test]
async fn request_handles_large_response() {
    let sock = test_sock_path("large");
    let listener = UnixListener::bind(&sock).unwrap();

    let expected: String = "x".repeat(32_768); // 32 KiB — larger than 8192-byte read buffer
    let expected_clone = expected.clone();

    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        stream.write_all(expected_clone.as_bytes()).await.unwrap();
    });

    let response = hyprs_sdk::ipc::socket::request(&sock, "j/clients")
        .await
        .unwrap();
    assert_eq!(response.len(), 32_768);
    assert_eq!(response, expected);

    server.await.unwrap();
    cleanup(&sock);
}

#[tokio::test]
async fn connect_event_stream_returns_readable_stream() {
    let sock = test_sock_path("events");
    let listener = UnixListener::bind(&sock).unwrap();

    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        stream
            .write_all(b"workspace>>1\nactivewindow>>kitty,kitty\n")
            .await
            .unwrap();
    });

    let mut stream = hyprs_sdk::ipc::socket::connect_event_stream(&sock)
        .await
        .unwrap();
    let mut buf = String::new();
    stream.read_to_string(&mut buf).await.unwrap();
    assert_eq!(buf, "workspace>>1\nactivewindow>>kitty,kitty\n");

    server.await.unwrap();
    cleanup(&sock);
}

#[tokio::test]
async fn request_fails_on_nonexistent_socket() {
    let result = hyprs_sdk::ipc::socket::request("/tmp/nonexistent.sock".as_ref(), "version").await;
    assert!(result.is_err());
}
