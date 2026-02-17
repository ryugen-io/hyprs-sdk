// Miscellaneous bridge functions: notifications, config reload, function
// hooks, version info.
#include "common.h"

// ── Notifications ────────────────────────────────────────────────────

extern "C" bool hyprland_api_add_notification(
    void* handle,
    const char* text_ptr, size_t text_len,
    double r, double g, double b, double a,
    float time_ms
) {
    std::string text(text_ptr, text_len);
    CHyprColor color(
        static_cast<float>(r),
        static_cast<float>(g),
        static_cast<float>(b),
        static_cast<float>(a)
    );
    return HyprlandAPI::addNotification(handle, text, color, time_ms);
}

extern "C" bool hyprland_api_add_notification_v2(
    void* handle,
    const char* text_ptr, size_t text_len,
    uint64_t time_ms,
    double r, double g, double b, double a,
    uint8_t icon
) {
    std::string text(text_ptr, text_len);
    CHyprColor color(
        static_cast<float>(r),
        static_cast<float>(g),
        static_cast<float>(b),
        static_cast<float>(a)
    );

    std::unordered_map<std::string, std::any> data;
    data["text"] = text;
    data["time"] = time_ms;
    data["color"] = color;
    data["icon"] = static_cast<eIcons>(icon);

    return HyprlandAPI::addNotificationV2(handle, data);
}

// ── Config Reload ────────────────────────────────────────────────────

extern "C" bool hyprland_api_reload_config() {
    return HyprlandAPI::reloadConfig();
}

// ── Function Hooking ─────────────────────────────────────────────────

extern "C" void* hyprland_api_create_function_hook(
    void* handle,
    const void* source,
    const void* destination
) {
    CFunctionHook* hook = HyprlandAPI::createFunctionHook(
        handle,
        const_cast<void*>(source),
        const_cast<void*>(destination)
    );
    return static_cast<void*>(hook);
}

extern "C" bool hyprland_api_remove_function_hook(void* handle, void* hook_ptr) {
    auto* hook = static_cast<CFunctionHook*>(hook_ptr);
    return HyprlandAPI::removeFunctionHook(handle, hook);
}

extern "C" bool hyprland_api_find_functions_by_name(
    void* handle,
    const char* name_ptr, size_t name_len,
    const void** out_addresses, size_t* out_count
) {
    std::string name(name_ptr, name_len);
    std::vector<SFunctionMatch> matches = HyprlandAPI::findFunctionsByName(handle, name);

    *out_count = matches.size();
    if (matches.empty()) {
        *out_addresses = nullptr;
        return true;
    }

    // Allocate array of void* addresses (caller frees via free_bridge_array).
    auto* addrs = static_cast<const void**>(std::malloc(matches.size() * sizeof(void*)));
    if (!addrs) {
        *out_count = 0;
        *out_addresses = nullptr;
        return false;
    }
    for (size_t i = 0; i < matches.size(); i++) {
        addrs[i] = matches[i].address;
    }
    *out_addresses = reinterpret_cast<const void*>(addrs);
    return true;
}

// ── Version ──────────────────────────────────────────────────────────

// Static buffers for version info strings (valid until next call).
static thread_local std::string s_version_hash;
static thread_local std::string s_version_tag;
static thread_local std::string s_version_branch;

extern "C" bool hyprland_api_get_version(
    void* handle,
    const char** out_hash, size_t* out_hash_len,
    const char** out_tag, size_t* out_tag_len,
    bool* out_dirty,
    const char** out_branch, size_t* out_branch_len
) {
    SVersionInfo info = HyprlandAPI::getHyprlandVersion(handle);

    s_version_hash = std::move(info.hash);
    s_version_tag = std::move(info.tag);
    s_version_branch = std::move(info.branch);

    *out_hash = s_version_hash.c_str();
    *out_hash_len = s_version_hash.size();
    *out_tag = s_version_tag.c_str();
    *out_tag_len = s_version_tag.size();
    *out_dirty = info.dirty;
    *out_branch = s_version_branch.c_str();
    *out_branch_len = s_version_branch.size();
    return true;
}
