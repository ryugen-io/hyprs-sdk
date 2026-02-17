// Decoration bridge: C++ IHyprWindowDecoration subclass delegating to Rust.
// Also provides window handle helpers for extracting PHLWINDOW from hook data.
#include "common.h"
#include <hyprland/src/render/decorations/IHyprWindowDecoration.hpp>
#include <memory>

// Vtable of Rust trampoline function pointers for WindowDecoration trait.
// Must match the Rust repr(C) struct DecorationVtable in ffi.rs exactly.
struct DecorationVtable {
    void (*get_positioning_info)(void*, uint8_t*, uint8_t*, uint32_t*, bool*);
    void (*on_positioning_reply)(void*, bool);
    void (*draw)(void*, void*, float);
    int8_t (*get_decoration_type)(void*);
    void (*update_window)(void*, void*);
    void (*damage_entire)(void*);
    bool (*on_input)(void*, uint8_t, double, double);
    uint8_t (*get_decoration_layer)(void*);
    uint64_t (*get_decoration_flags)(void*);
    void (*get_display_name)(void*, const char**, size_t*);
    void (*drop_fn)(void*);
};

// C++ IHyprWindowDecoration subclass that delegates to Rust.
class RustDecorationBridge : public IHyprWindowDecoration {
    void*            m_rustDeco;
    DecorationVtable m_vtable;

public:
    RustDecorationBridge(PHLWINDOW pWindow, void* rustDeco, const DecorationVtable& vt)
        : IHyprWindowDecoration(pWindow), m_rustDeco(rustDeco), m_vtable(vt) {}

    ~RustDecorationBridge() override {
        if (m_vtable.drop_fn && m_rustDeco)
            m_vtable.drop_fn(m_rustDeco);
    }

    SDecorationPositioningInfo getPositioningInfo() override {
        SDecorationPositioningInfo info;
        uint8_t policy = 0, edges = 0;
        uint32_t priority = 10;
        bool reserved = false;
        m_vtable.get_positioning_info(m_rustDeco, &policy, &edges, &priority, &reserved);
        info.policy = static_cast<eDecorationPositioningPolicy>(policy);
        info.edges = static_cast<uint32_t>(edges);
        info.priority = priority;
        info.reserved = reserved;
        return info;
    }

    void onPositioningReply(const SDecorationPositioningReply& reply) override {
        m_vtable.on_positioning_reply(m_rustDeco, reply.ephemeral);
    }

    void draw(PHLMONITOR mon, float const& a) override {
        m_vtable.draw(m_rustDeco,
                       mon ? static_cast<void*>(mon.get()) : nullptr, a);
    }

    eDecorationType getDecorationType() override {
        return static_cast<eDecorationType>(m_vtable.get_decoration_type(m_rustDeco));
    }

    void updateWindow(PHLWINDOW w) override {
        m_vtable.update_window(m_rustDeco,
                                w ? static_cast<void*>(w.get()) : nullptr);
    }

    void damageEntire() override {
        m_vtable.damage_entire(m_rustDeco);
    }

    bool onInputOnDeco(const eInputType type, const Vector2D& pos, std::any = {}) override {
        return m_vtable.on_input(m_rustDeco,
                                  static_cast<uint8_t>(type), pos.x, pos.y);
    }

    eDecorationLayer getDecorationLayer() override {
        return static_cast<eDecorationLayer>(m_vtable.get_decoration_layer(m_rustDeco));
    }

    uint64_t getDecorationFlags() override {
        return m_vtable.get_decoration_flags(m_rustDeco);
    }

    std::string getDisplayName() override {
        const char* ptr = nullptr;
        size_t len = 0;
        m_vtable.get_display_name(m_rustDeco, &ptr, &len);
        return (ptr && len > 0) ? std::string(ptr, len) : std::string("custom");
    }
};

extern "C" void* hyprland_api_add_window_decoration(
    void* handle,
    void* window_handle,   // PHLWINDOW* (heap-allocated via clone_window_handle)
    void* rust_deco,
    const DecorationVtable* vtable
) {
    // Early validation — clean up Rust data if we can't proceed.
    if (!vtable || !rust_deco || !window_handle) {
        if (vtable && vtable->drop_fn && rust_deco)
            vtable->drop_fn(rust_deco);
        return nullptr;
    }

    PHLWINDOW pWindow = *static_cast<PHLWINDOW*>(window_handle);
    if (!pWindow) {
        vtable->drop_fn(rust_deco);
        return nullptr;
    }

    auto bridge = std::make_unique<RustDecorationBridge>(pWindow, rust_deco, *vtable);
    auto* raw = bridge.get();

    // addWindowDecoration takes UP<IHyprWindowDecoration> (unique ownership).
    UP<IHyprWindowDecoration> up(bridge.release());
    bool ok = HyprlandAPI::addWindowDecoration(handle, pWindow, std::move(up));
    if (!ok) {
        // UP was moved into the function. If rejected, Hyprland destroyed
        // the UP which destroyed the bridge (calling vtable.drop_fn).
        return nullptr;
    }
    return static_cast<void*>(raw);
}

extern "C" bool hyprland_api_remove_window_decoration(
    void* handle,
    void* decoration_ptr
) {
    if (!decoration_ptr) return false;
    auto* bridge = static_cast<IHyprWindowDecoration*>(decoration_ptr);
    // removeWindowDecoration tells Hyprland to release the UP, which
    // destroys the bridge (calling vtable.drop_fn on the Rust data).
    return HyprlandAPI::removeWindowDecoration(handle, bridge);
}

// ── Window Handle Helpers ────────────────────────────────────────────

extern "C" void* hyprland_api_clone_window_handle(void* any_ptr) {
    if (!any_ptr) return nullptr;
    try {
        auto& data = *static_cast<std::any*>(any_ptr);
        PHLWINDOW window = std::any_cast<PHLWINDOW>(data);
        if (!window) return nullptr;
        return static_cast<void*>(new PHLWINDOW(window));
    } catch (...) {
        return nullptr;
    }
}

extern "C" void hyprland_api_release_window_handle(void* handle) {
    if (handle)
        delete static_cast<PHLWINDOW*>(handle);
}
