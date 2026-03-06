// Legacy layout bridge stubs.
//
// v0.54 removed IHyprLayout.hpp and forward-declares the class only. The
// deprecated addLayout/removeLayout still exist in PluginAPI.hpp but can't
// be called without the full class definition. These stubs keep the Rust
// FFI declarations linkable; the Rust side treats nullptr/false returns as
// registration failures.
#include "common.h"

struct LayoutVtable;

extern "C" void* hyprland_api_add_layout(
    void* /* handle */,
    const char* /* name_ptr */, size_t /* name_len */,
    void* rust_layout,
    const LayoutVtable* /* vtable */
) {
    (void)rust_layout;
    return nullptr;
}

extern "C" bool hyprland_api_remove_layout(void* /* handle */, void* /* layout_ptr */) {
    return false;
}
