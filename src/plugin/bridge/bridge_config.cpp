// Config value and keyword handler bridge functions.
#include "common.h"

// ── Config values ────────────────────────────────────────────────────

extern "C" bool hyprland_api_add_config_value(
    void* handle,
    const char* name_ptr, size_t name_len,
    uint8_t value_type,
    int64_t int_val,
    double float_val,
    double float_val2,
    const char* str_ptr, size_t str_len
) {
    std::string name(name_ptr, name_len);

    switch (value_type) {
        case 0: // Bool (as Hyprlang::INT)
            return HyprlandAPI::addConfigValue(handle, name,
                Hyprlang::CConfigValue(static_cast<Hyprlang::INT>(int_val ? 1 : 0)));
        case 1: // Int
            return HyprlandAPI::addConfigValue(handle, name,
                Hyprlang::CConfigValue(static_cast<Hyprlang::INT>(int_val)));
        case 2: // Float
            return HyprlandAPI::addConfigValue(handle, name,
                Hyprlang::CConfigValue(static_cast<Hyprlang::FLOAT>(float_val)));
        case 3: { // String
            std::string str(str_ptr, str_len);
            return HyprlandAPI::addConfigValue(handle, name,
                Hyprlang::CConfigValue(str.c_str()));
        }
        case 4: // Color (as Hyprlang::INT, RGBA u32)
            return HyprlandAPI::addConfigValue(handle, name,
                Hyprlang::CConfigValue(static_cast<Hyprlang::INT>(int_val)));
        case 5: // Vec2
            return HyprlandAPI::addConfigValue(handle, name,
                Hyprlang::CConfigValue(Hyprlang::SVector2D{
                    static_cast<float>(float_val),
                    static_cast<float>(float_val2)
                }));
        default:
            return false;
    }
}

extern "C" void* hyprland_api_get_config_value(
    void* handle,
    const char* name_ptr, size_t name_len
) {
    std::string name(name_ptr, name_len);
    return static_cast<void*>(HyprlandAPI::getConfigValue(handle, name));
}

// ── Config keyword handler ───────────────────────────────────────────
//
// Hyprlang::PCONFIGHANDLERFUNC is a plain C function pointer:
//   CParseResult(*)(const char* COMMAND, const char* VALUE)
// No user_data parameter, so we use a global registry keyed by keyword name.

struct KeywordCallbackEntry {
    bool (*callback)(void* user_data, const char* value_ptr, size_t value_len);
    void* user_data;
};

static std::mutex s_keyword_mutex;
static std::unordered_map<std::string, KeywordCallbackEntry> s_keyword_handlers;

// Global trampoline that looks up the handler by keyword name.
static Hyprlang::CParseResult keyword_trampoline(const char* command, const char* value) {
    Hyprlang::CParseResult result;

    std::string cmd_name;
    if (command) cmd_name = command;

    KeywordCallbackEntry entry{};
    {
        std::lock_guard<std::mutex> lock(s_keyword_mutex);
        auto it = s_keyword_handlers.find(cmd_name);
        if (it != s_keyword_handlers.end())
            entry = it->second;
    }

    if (entry.callback) {
        size_t len = value ? std::strlen(value) : 0;
        bool ok = entry.callback(entry.user_data, value, len);
        if (!ok) {
            result.setError("Rust keyword handler returned error");
        }
    }

    return result;
}

extern "C" bool hyprland_api_add_config_keyword(
    void* handle,
    const char* name_ptr, size_t name_len,
    bool (*callback)(void* user_data, const char* value_ptr, size_t value_len),
    void* user_data,
    bool allow_flags,
    bool /* allow_default -- not supported by Hyprlang::SHandlerOptions */
) {
    std::string name(name_ptr, name_len);

    // Register in our global map before calling Hyprland.
    {
        std::lock_guard<std::mutex> lock(s_keyword_mutex);
        s_keyword_handlers[name] = KeywordCallbackEntry{callback, user_data};
    }

    Hyprlang::SHandlerOptions opts;
    opts.allowFlags = allow_flags;

    return HyprlandAPI::addConfigKeyword(handle, name, keyword_trampoline, opts);
}
