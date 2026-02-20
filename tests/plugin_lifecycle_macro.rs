use std::ffi::{c_char, c_void};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use hypr_sdk::plugin::{PluginDescription, PluginHandle};

static TEST_LOCK: Mutex<()> = Mutex::new(());
static INIT_CALLS: AtomicUsize = AtomicUsize::new(0);
static EXIT_CALLS: AtomicUsize = AtomicUsize::new(0);
static SHOULD_FAIL_INIT: AtomicBool = AtomicBool::new(false);

fn test_init(_handle: PluginHandle) -> Result<PluginDescription, String> {
    INIT_CALLS.fetch_add(1, Ordering::SeqCst);
    if SHOULD_FAIL_INIT.load(Ordering::SeqCst) {
        return Err("init\0failed".into());
    }

    Ok(PluginDescription {
        name: "hypr-sdk-test".into(),
        description: "lifecycle macro smoke".into(),
        author: "qa".into(),
        version: "0.1.0".into(),
    })
}

fn test_exit() {
    EXIT_CALLS.fetch_add(1, Ordering::SeqCst);
}

hypr_sdk::hyprland_plugin! {
    init: test_init,
    exit: test_exit,
}

fn read_bytes(ptr: *const c_char, len: usize) -> String {
    let bytes = unsafe { std::slice::from_raw_parts(ptr.cast::<u8>(), len) };
    String::from_utf8(bytes.to_vec()).expect("plugin metadata should be valid UTF-8")
}

#[test]
fn lifecycle_success_flow_exposes_description_and_clears_on_exit() {
    let _guard = TEST_LOCK.lock().expect("test lock poisoned");
    SHOULD_FAIL_INIT.store(false, Ordering::SeqCst);

    unsafe {
        assert!(hyprland_rs_plugin_init(std::ptr::null_mut::<c_void>()));

        let mut name_ptr: *const c_char = std::ptr::null();
        let mut name_len = 0usize;
        let mut desc_ptr: *const c_char = std::ptr::null();
        let mut desc_len = 0usize;
        let mut author_ptr: *const c_char = std::ptr::null();
        let mut author_len = 0usize;
        let mut version_ptr: *const c_char = std::ptr::null();
        let mut version_len = 0usize;

        assert!(hyprland_rs_plugin_get_description(
            &mut name_ptr,
            &mut name_len,
            &mut desc_ptr,
            &mut desc_len,
            &mut author_ptr,
            &mut author_len,
            &mut version_ptr,
            &mut version_len,
        ));

        assert_eq!(read_bytes(name_ptr, name_len), "hypr-sdk-test");
        assert_eq!(read_bytes(desc_ptr, desc_len), "lifecycle macro smoke");
        assert_eq!(read_bytes(author_ptr, author_len), "qa");
        assert_eq!(read_bytes(version_ptr, version_len), "0.1.0");

        let mut err_ptr: *const c_char = std::ptr::null();
        let mut err_len = 123usize;
        assert!(!hyprland_rs_plugin_get_error(&mut err_ptr, &mut err_len));
        assert!(err_ptr.is_null());
        assert_eq!(err_len, 0);

        hyprland_rs_plugin_exit();
        assert!(EXIT_CALLS.load(Ordering::SeqCst) >= 1);

        assert!(!hyprland_rs_plugin_get_description(
            &mut name_ptr,
            &mut name_len,
            &mut desc_ptr,
            &mut desc_len,
            &mut author_ptr,
            &mut author_len,
            &mut version_ptr,
            &mut version_len,
        ));
        assert!(name_ptr.is_null());
        assert_eq!(name_len, 0);
    }
}

#[test]
fn lifecycle_error_flow_exposes_error_and_sanitizes_nul_bytes() {
    let _guard = TEST_LOCK.lock().expect("test lock poisoned");
    SHOULD_FAIL_INIT.store(true, Ordering::SeqCst);

    unsafe {
        assert!(!hyprland_rs_plugin_init(std::ptr::null_mut::<c_void>()));

        let mut err_ptr: *const c_char = std::ptr::null();
        let mut err_len = 0usize;
        assert!(hyprland_rs_plugin_get_error(&mut err_ptr, &mut err_len));
        let err = read_bytes(err_ptr, err_len);
        assert_eq!(err, "init failed");

        let mut name_ptr: *const c_char = std::ptr::null();
        let mut name_len = 99usize;
        assert!(!hyprland_rs_plugin_get_description(
            &mut name_ptr,
            &mut name_len,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ));
        assert!(name_ptr.is_null());
        assert_eq!(name_len, 0);

        hyprland_rs_plugin_exit();
        assert!(!hyprland_rs_plugin_get_error(&mut err_ptr, &mut err_len));
        assert!(err_ptr.is_null());
        assert_eq!(err_len, 0);
    }

    SHOULD_FAIL_INIT.store(false, Ordering::SeqCst);
}

#[test]
fn lifecycle_exports_api_version_string() {
    let _guard = TEST_LOCK.lock().expect("test lock poisoned");

    let version = unsafe {
        std::ffi::CStr::from_ptr(hyprland_rs_plugin_api_version())
            .to_str()
            .expect("api version should be valid UTF-8")
    };
    assert_eq!(version, hypr_sdk::plugin::HYPRLAND_API_VERSION);
}
