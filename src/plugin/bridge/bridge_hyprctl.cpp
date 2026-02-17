// HyprCtl command bridge functions.
#include "common.h"

extern "C" bool hyprland_api_invoke_hyprctl(
    const char* call_ptr, size_t call_len,
    const char* args_ptr, size_t args_len,
    const char* format_ptr, size_t format_len,
    char** out_ptr, size_t* out_len
) {
    std::string call(call_ptr, call_len);
    std::string args(args_ptr, args_len);
    std::string format(format_ptr, format_len);

    std::string result = HyprlandAPI::invokeHyprctlCommand(call, args, format);
    *out_ptr = alloc_cstr(result, out_len);
    return true;
}

// HyprCtl command callback trampoline state.
struct HyprCtlCommandBridgeData {
    void (*callback)(void* user_data, uint8_t format, const char* args_ptr, size_t args_len,
                     char** out_ptr, size_t* out_len);
    void* user_data;
    SP<SHyprCtlCommand> sp;
};

extern "C" void* hyprland_api_register_hyprctl_command(
    void* handle,
    const char* name_ptr, size_t name_len,
    bool exact,
    void (*callback)(void* user_data, uint8_t format, const char* args_ptr, size_t args_len,
                     char** out_ptr, size_t* out_len),
    void* user_data
) {
    std::string name(name_ptr, name_len);

    auto rust_cb = callback;
    auto rust_ud = user_data;

    SHyprCtlCommand cmd;
    cmd.name = name;
    cmd.exact = exact;
    cmd.fn = [rust_cb, rust_ud](eHyprCtlOutputFormat format, std::string args) -> std::string {
        if (!rust_cb) return "";
        char* out_ptr = nullptr;
        size_t out_len = 0;
        rust_cb(rust_ud, static_cast<uint8_t>(format),
                args.data(), args.size(), &out_ptr, &out_len);
        std::string result;
        if (out_ptr && out_len > 0) {
            result.assign(out_ptr, out_len);
            std::free(out_ptr);
        }
        return result;
    };

    SP<SHyprCtlCommand> sp = HyprlandAPI::registerHyprCtlCommand(handle, cmd);
    if (!sp)
        return nullptr;

    auto* bridge = new HyprCtlCommandBridgeData{rust_cb, rust_ud, sp};
    return static_cast<void*>(bridge);
}

extern "C" bool hyprland_api_unregister_hyprctl_command(void* handle, void* cmd_ptr) {
    if (!cmd_ptr) return false;
    auto* bridge = static_cast<HyprCtlCommandBridgeData*>(cmd_ptr);
    bool result = HyprlandAPI::unregisterHyprCtlCommand(handle, bridge->sp);
    delete bridge;
    return result;
}
