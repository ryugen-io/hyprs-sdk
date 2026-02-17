// Shared includes and helpers for the Hyprland bridge modules.
//
// Each bridge_*.cpp includes this header for access to HyprlandAPI,
// standard types, and the alloc_cstr helper.

#pragma once

#include <hyprland/src/plugins/PluginAPI.hpp>
#include <hyprland/src/SharedDefs.hpp>
#include <hyprland/src/helpers/Color.hpp>

#include <cstdlib>
#include <cstring>
#include <any>
#include <mutex>
#include <string>
#include <unordered_map>
#include <vector>

// Allocate a C string copy of a std::string (caller frees with std::free).
inline char* alloc_cstr(const std::string& s, size_t* out_len) {
    *out_len = s.size();
    if (s.empty()) return nullptr;
    char* buf = static_cast<char*>(std::malloc(s.size()));
    if (buf) std::memcpy(buf, s.data(), s.size());
    return buf;
}
