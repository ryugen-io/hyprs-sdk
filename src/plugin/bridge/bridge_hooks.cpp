// Hook event callback bridge functions.
#include "common.h"

// Bridge stores the Rust callback + user_data and wraps it in HOOK_CALLBACK_FN.
struct HookBridgeData {
    void (*callback)(void* user_data, void* callback_info, void* event_data);
    void* user_data;
    SP<HOOK_CALLBACK_FN> sp; // prevent deallocation
};

extern "C" void* hyprland_api_register_callback(
    void* handle,
    const char* event_ptr, size_t event_len,
    void (*callback)(void* user_data, void* callback_info, void* event_data),
    void* user_data
) {
    std::string event(event_ptr, event_len);

    auto rust_cb = callback;
    auto rust_ud = user_data;

    HOOK_CALLBACK_FN fn = [rust_cb, rust_ud](void* /* owner */, SCallbackInfo& info, std::any data) {
        // Pass the SCallbackInfo as a void* so Rust can set cancelled.
        // Pass the std::any pointer (Rust can inspect via the bridge).
        void* data_ptr = nullptr;
        try {
            if (data.type() != typeid(void))
                data_ptr = &data;
        } catch (...) {}
        if (rust_cb)
            rust_cb(rust_ud, static_cast<void*>(&info), data_ptr);
    };

    SP<HOOK_CALLBACK_FN> sp = HyprlandAPI::registerCallbackDynamic(handle, event, fn);
    if (!sp)
        return nullptr;

    // Allocate bridge data to keep the SP alive and allow cleanup.
    auto* bridge = new HookBridgeData{rust_cb, rust_ud, sp};
    return static_cast<void*>(bridge);
}

extern "C" bool hyprland_api_unregister_callback(void* handle, void* callback_ptr) {
    (void)handle;
    if (!callback_ptr) return false;
    auto* bridge = static_cast<HookBridgeData*>(callback_ptr);
    // Reset the shared_ptr to unregister.
    bridge->sp.reset();
    delete bridge;
    return true;
}
