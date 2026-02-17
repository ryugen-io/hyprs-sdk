// Layout bridge: C++ IHyprLayout subclass delegating to Rust Layout trait.
#include "common.h"
#include <hyprland/src/layout/IHyprLayout.hpp>

// Vtable of Rust trampoline function pointers for Layout trait methods.
// Must match the Rust repr(C) struct LayoutVtable in ffi.rs exactly.
struct LayoutVtable {
    void (*on_enable)(void*);
    void (*on_disable)(void*);
    void (*get_layout_name)(void*, const char**, size_t*);
    void (*on_window_created_tiling)(void*, void*, int8_t);
    void (*on_window_removed_tiling)(void*, void*);
    bool (*is_window_tiled)(void*, void*);
    void (*recalculate_monitor)(void*, int64_t);
    void (*recalculate_window)(void*, void*);
    void (*resize_active_window)(void*, double, double, uint8_t, void*);
    void (*fullscreen_request)(void*, void*, int8_t, int8_t);
    bool (*layout_message)(void*, void*, const char*, size_t, char**, size_t*);
    void (*switch_windows)(void*, void*, void*);
    void (*move_window_to)(void*, void*, const char*, size_t, bool);
    void (*alter_split_ratio)(void*, void*, float, bool);
    void (*replace_window_data)(void*, void*, void*);
    void (*predict_size)(void*, double*, double*);
    void (*drop_fn)(void*);
};

// C++ IHyprLayout subclass that delegates all virtual calls to Rust.
class RustLayoutBridge : public IHyprLayout {
    void*        m_rustLayout;
    LayoutVtable m_vtable;

public:
    RustLayoutBridge(void* rustLayout, const LayoutVtable& vt)
        : m_rustLayout(rustLayout), m_vtable(vt) {}

    ~RustLayoutBridge() override {
        if (m_vtable.drop_fn && m_rustLayout)
            m_vtable.drop_fn(m_rustLayout);
    }

    void onEnable() override {
        m_vtable.on_enable(m_rustLayout);
    }

    void onDisable() override {
        m_vtable.on_disable(m_rustLayout);
    }

    std::string getLayoutName() override {
        const char* ptr = nullptr;
        size_t len = 0;
        m_vtable.get_layout_name(m_rustLayout, &ptr, &len);
        return (ptr && len > 0) ? std::string(ptr, len) : std::string();
    }

    void onWindowCreatedTiling(PHLWINDOW w, eDirection d = DIRECTION_DEFAULT) override {
        m_vtable.on_window_created_tiling(
            m_rustLayout, w ? static_cast<void*>(w.get()) : nullptr,
            static_cast<int8_t>(d));
    }

    void onWindowRemovedTiling(PHLWINDOW w) override {
        m_vtable.on_window_removed_tiling(
            m_rustLayout, w ? static_cast<void*>(w.get()) : nullptr);
    }

    bool isWindowTiled(PHLWINDOW w) override {
        return m_vtable.is_window_tiled(
            m_rustLayout, w ? static_cast<void*>(w.get()) : nullptr);
    }

    void recalculateMonitor(const MONITORID& id) override {
        m_vtable.recalculate_monitor(m_rustLayout, id);
    }

    void recalculateWindow(PHLWINDOW w) override {
        m_vtable.recalculate_window(
            m_rustLayout, w ? static_cast<void*>(w.get()) : nullptr);
    }

    void resizeActiveWindow(const Vector2D& delta, eRectCorner corner = CORNER_NONE,
                            PHLWINDOW w = nullptr) override {
        m_vtable.resize_active_window(
            m_rustLayout, delta.x, delta.y,
            static_cast<uint8_t>(corner),
            w ? static_cast<void*>(w.get()) : nullptr);
    }

    void fullscreenRequestForWindow(PHLWINDOW w, const eFullscreenMode cur,
                                     const eFullscreenMode eff) override {
        m_vtable.fullscreen_request(
            m_rustLayout, w ? static_cast<void*>(w.get()) : nullptr,
            static_cast<int8_t>(cur), static_cast<int8_t>(eff));
    }

    std::any layoutMessage(SLayoutMessageHeader header, std::string msg) override {
        void* win = header.pWindow ? static_cast<void*>(header.pWindow.get()) : nullptr;
        char* out_ptr = nullptr;
        size_t out_len = 0;
        bool has_result = m_vtable.layout_message(
            m_rustLayout, win, msg.data(), msg.size(), &out_ptr, &out_len);
        if (has_result && out_ptr && out_len > 0) {
            std::string result(out_ptr, out_len);
            std::free(out_ptr);
            return std::any(result);
        }
        return {};
    }

    SWindowRenderLayoutHints requestRenderHints(PHLWINDOW) override {
        return {}; // Default: no custom render hints
    }

    void switchWindows(PHLWINDOW a, PHLWINDOW b) override {
        m_vtable.switch_windows(
            m_rustLayout,
            a ? static_cast<void*>(a.get()) : nullptr,
            b ? static_cast<void*>(b.get()) : nullptr);
    }

    void moveWindowTo(PHLWINDOW w, const std::string& dir, bool silent = false) override {
        m_vtable.move_window_to(
            m_rustLayout,
            w ? static_cast<void*>(w.get()) : nullptr,
            dir.data(), dir.size(), silent);
    }

    void alterSplitRatio(PHLWINDOW w, float ratio, bool exact = false) override {
        m_vtable.alter_split_ratio(
            m_rustLayout,
            w ? static_cast<void*>(w.get()) : nullptr,
            ratio, exact);
    }

    void replaceWindowDataWith(PHLWINDOW from, PHLWINDOW to) override {
        m_vtable.replace_window_data(
            m_rustLayout,
            from ? static_cast<void*>(from.get()) : nullptr,
            to ? static_cast<void*>(to.get()) : nullptr);
    }

    Vector2D predictSizeForNewWindowTiled() override {
        double x = 0.0, y = 0.0;
        m_vtable.predict_size(m_rustLayout, &x, &y);
        return {x, y};
    }
};

extern "C" void* hyprland_api_add_layout(
    void* handle,
    const char* name_ptr, size_t name_len,
    void* rust_layout,
    const LayoutVtable* vtable
) {
    // Early validation — clean up Rust data if we can't proceed.
    if (!vtable || !rust_layout) {
        if (vtable && vtable->drop_fn && rust_layout)
            vtable->drop_fn(rust_layout);
        return nullptr;
    }
    std::string name(name_ptr, name_len);

    auto* bridge = new RustLayoutBridge(rust_layout, *vtable);
    bool ok = HyprlandAPI::addLayout(handle, name, bridge);
    if (!ok) {
        // delete calls ~RustLayoutBridge which calls vtable.drop_fn(rust_layout).
        delete bridge;
        return nullptr;
    }
    return static_cast<void*>(bridge);
}

extern "C" bool hyprland_api_remove_layout(void* handle, void* layout_bridge_ptr) {
    if (!layout_bridge_ptr) return false;
    auto* bridge = static_cast<IHyprLayout*>(layout_bridge_ptr);
    bool ok = HyprlandAPI::removeLayout(handle, bridge);
    if (ok) {
        delete bridge; // ~RustLayoutBridge calls vtable.drop(rust_layout)
    }
    return ok;
}
