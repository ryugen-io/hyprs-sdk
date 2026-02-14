use hypr_sdk::ipc::instance::{runtime_dir, socket1_path, socket2_path};

#[test]
fn runtime_dir_uses_xdg() {
    // This test just verifies the function returns a path ending in /hypr
    let dir = runtime_dir();
    assert!(
        dir.ends_with("/hypr"),
        "expected path ending in /hypr, got: {dir}"
    );
}

#[test]
fn socket_paths_from_signature() {
    let sig = "abc_123_456";
    let s1 = socket1_path(sig);
    let s2 = socket2_path(sig);
    assert!(s1.ends_with("abc_123_456/.socket.sock"));
    assert!(s2.ends_with("abc_123_456/.socket2.sock"));
}
