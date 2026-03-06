// Floating algorithm bridge: C++ IFloatingAlgorithm subclass delegating to Rust.
//
// v0.54+ API: HyprlandAPI::addFloatingAlgo
#include "common.h"
#include <hyprland/src/layout/algorithm/FloatingAlgorithm.hpp>
#include <hyprland/src/layout/algorithm/Algorithm.hpp>

struct ModeAlgoVtable {
    void (*new_target)(void*, void*);
    void (*moved_target)(void*, void*, bool, double, double);
    void (*remove_target)(void*, void*);
    void (*resize_target)(void*, double, double, void*, uint8_t);
    void (*recalculate)(void*);
    void (*swap_targets)(void*, void*, void*);
    void (*move_target_in_direction)(void*, void*, int8_t, bool);
    bool (*layout_msg)(void*, const char*, size_t, char**, size_t*);
    bool (*predict_size)(void*, double*, double*);
    void (*drop_fn)(void*);
};

struct FloatingAlgoVtable {
    ModeAlgoVtable base;
    void (*move_target_delta)(void*, double, double, void*);
    void (*set_target_geom)(void*, double, double, double, double, void*);
    void* (*factory_fn)(void*);
};

class RustFloatingAlgoBridge : public Layout::IFloatingAlgorithm {
    void*              m_rust;
    FloatingAlgoVtable m_vt;

public:
    RustFloatingAlgoBridge(void* rust, const FloatingAlgoVtable& vt)
        : m_rust(rust), m_vt(vt) {}

    ~RustFloatingAlgoBridge() override {
        if (m_vt.base.drop_fn && m_rust)
            m_vt.base.drop_fn(m_rust);
    }

    void newTarget(SP<Layout::ITarget> t) override {
        m_vt.base.new_target(m_rust, t ? static_cast<void*>(t.get()) : nullptr);
    }

    void movedTarget(SP<Layout::ITarget> t, std::optional<Vector2D> focal) override {
        bool has_focal = focal.has_value();
        double fx = has_focal ? focal->x : 0.0;
        double fy = has_focal ? focal->y : 0.0;
        m_vt.base.moved_target(m_rust, t ? static_cast<void*>(t.get()) : nullptr,
                                has_focal, fx, fy);
    }

    void removeTarget(SP<Layout::ITarget> t) override {
        m_vt.base.remove_target(m_rust, t ? static_cast<void*>(t.get()) : nullptr);
    }

    void resizeTarget(const Vector2D& delta, SP<Layout::ITarget> t,
                      Layout::eRectCorner corner) override {
        m_vt.base.resize_target(m_rust, delta.x, delta.y,
                                 t ? static_cast<void*>(t.get()) : nullptr,
                                 static_cast<uint8_t>(corner));
    }

    void recalculate() override {
        m_vt.base.recalculate(m_rust);
    }

    void swapTargets(SP<Layout::ITarget> a, SP<Layout::ITarget> b) override {
        m_vt.base.swap_targets(m_rust,
                                a ? static_cast<void*>(a.get()) : nullptr,
                                b ? static_cast<void*>(b.get()) : nullptr);
    }

    void moveTargetInDirection(SP<Layout::ITarget> t, Math::eDirection dir,
                               bool silent) override {
        m_vt.base.move_target_in_direction(m_rust,
                                            t ? static_cast<void*>(t.get()) : nullptr,
                                            static_cast<int8_t>(dir), silent);
    }

    std::expected<void, std::string> layoutMsg(const std::string_view& sv) override {
        char* out_ptr = nullptr;
        size_t out_len = 0;
        bool ok = m_vt.base.layout_msg(m_rust, sv.data(), sv.size(),
                                         &out_ptr, &out_len);
        if (ok)
            return {};
        if (out_ptr && out_len > 0) {
            std::string err(out_ptr, out_len);
            std::free(out_ptr);
            return std::unexpected(err);
        }
        return std::unexpected(std::string("layout message failed"));
    }

    std::optional<Vector2D> predictSizeForNewTarget() override {
        double x = 0.0, y = 0.0;
        if (m_vt.base.predict_size && m_vt.base.predict_size(m_rust, &x, &y))
            return Vector2D{x, y};
        return std::nullopt;
    }

    void moveTarget(const Vector2D& delta, SP<Layout::ITarget> t) override {
        m_vt.move_target_delta(m_rust, delta.x, delta.y,
                                t ? static_cast<void*>(t.get()) : nullptr);
    }

    void setTargetGeom(const CBox& geom, SP<Layout::ITarget> t) override {
        m_vt.set_target_geom(m_rust, geom.x, geom.y, geom.w, geom.h,
                              t ? static_cast<void*>(t.get()) : nullptr);
    }
};

extern "C" bool hyprland_api_add_floating_algo(
    void* handle,
    const char* name_ptr, size_t name_len,
    void* rust_factory_data,
    const FloatingAlgoVtable* vtable
) {
    if (!vtable || !rust_factory_data) return false;
    std::string name(name_ptr, name_len);

    FloatingAlgoVtable vt_copy = *vtable;
    auto factory = [rust_factory_data, vt_copy]() -> UP<Layout::IFloatingAlgorithm> {
        void* rust_algo = vt_copy.factory_fn(rust_factory_data);
        if (!rust_algo) return nullptr;
        return UP<Layout::IFloatingAlgorithm>(new RustFloatingAlgoBridge(rust_algo, vt_copy));
    };

    return HyprlandAPI::addFloatingAlgo(handle, name,
                                         &typeid(RustFloatingAlgoBridge),
                                         std::move(factory));
}
