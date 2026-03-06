// Hook event callback bridge functions.
//
// v0.54 deprecated registerCallbackDynamic (returns nullptr, HOOK_CALLBACK_FN
// is a dummy class). This bridge compiles but is non-functional until we
// migrate to the new CEventBus signal system.
#include "common.h"

struct HookBridgeData {
    void (*callback)(void* user_data, void* callback_info, void* event_data);
    void* user_data;
    SP<HOOK_CALLBACK_FN> sp;
};

extern "C" void* hyprland_api_register_callback(
    void* handle,
    const char* event_ptr, size_t event_len,
    void (*callback)(void* user_data, void* callback_info, void* event_data),
    void* user_data
) {
    // v0.54 gutted registerCallbackDynamic — it always returns nullptr.
    // Keep the bridge signature intact so Rust FFI declarations stay valid;
    // the Rust side already handles nullptr as a registration failure.
    std::string event(event_ptr, event_len);
    SP<HOOK_CALLBACK_FN> sp = HyprlandAPI::registerCallbackDynamic(handle, event, HOOK_CALLBACK_FN{});
    if (!sp)
        return nullptr;

    auto* bridge = new HookBridgeData{callback, user_data, sp};
    return static_cast<void*>(bridge);
}

extern "C" bool hyprland_api_unregister_callback(void* handle, void* callback_ptr) {
    (void)handle;
    if (!callback_ptr) return false;
    auto* bridge = static_cast<HookBridgeData*>(callback_ptr);
    bridge->sp.reset();
    delete bridge;
    return true;
}
