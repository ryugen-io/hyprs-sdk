#include <hyprland/src/plugins/PluginAPI.hpp>

extern "C" std::string pluginAPIVersion() {
    return HYPRLAND_API_VERSION;
}

extern "C" PLUGIN_DESCRIPTION_INFO pluginInit(HANDLE) {
    return {
        "hypr-sdk-smoke-plugin",
        "smoke-test plugin for hypr-sdk live integration",
        "hypr-sdk",
        "0.1.0",
    };
}

extern "C" void pluginExit() {}
