// Dispatcher bridge functions.
#include "common.h"

extern "C" bool hyprland_api_add_dispatcher(
    void* handle,
    const char* name_ptr, size_t name_len,
    void (*callback)(void* user_data, const char* args_ptr, size_t args_len,
                     bool* out_pass, bool* out_success, char** out_error_ptr, size_t* out_error_len),
    void* user_data
) {
    std::string name(name_ptr, name_len);

    auto rust_cb = callback;
    auto rust_ud = user_data;

    std::function<SDispatchResult(std::string)> handler =
        [rust_cb, rust_ud](std::string args) -> SDispatchResult {
            SDispatchResult result;
            if (!rust_cb) return result;

            bool pass = false, success = true;
            char* error_ptr = nullptr;
            size_t error_len = 0;

            rust_cb(rust_ud, args.data(), args.size(),
                    &pass, &success, &error_ptr, &error_len);

            result.passEvent = pass;
            result.success = success;
            if (error_ptr && error_len > 0) {
                result.error.assign(error_ptr, error_len);
                std::free(error_ptr);
            }
            return result;
        };

    return HyprlandAPI::addDispatcherV2(handle, name, handler);
}

extern "C" bool hyprland_api_remove_dispatcher(
    void* handle,
    const char* name_ptr, size_t name_len
) {
    std::string name(name_ptr, name_len);
    return HyprlandAPI::removeDispatcher(handle, name);
}
