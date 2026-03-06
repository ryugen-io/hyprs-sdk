// Tiled algorithm bridge: C++ ITiledAlgorithm subclass delegating to Rust.
//
// v0.54+ API: HyprlandAPI::addTiledAlgo / removeAlgo
#include "common.h"
#include <hyprland/src/layout/algorithm/TiledAlgorithm.hpp>
#include <hyprland/src/layout/algorithm/Algorithm.hpp>
#include <unordered_map>

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

struct TiledAlgoVtable {
    ModeAlgoVtable base;
    void* (*get_next_candidate)(void*, void*);
    void* (*factory_fn)(void*);
};

class RustTiledAlgoBridge : public Layout::ITiledAlgorithm {
    void*           m_rust;
    TiledAlgoVtable m_vt;

    // getNextCandidate must return SP<ITarget> but Rust works with raw pointers.
    // Track every target we see so we can convert raw pointers back to SP.
    std::unordered_map<void*, WP<Layout::ITarget>> m_targets;

    void trackTarget(SP<Layout::ITarget> t) {
        if (t) m_targets[static_cast<void*>(t.get())] = t;
    }
    void untrackTarget(SP<Layout::ITarget> t) {
        if (t) m_targets.erase(static_cast<void*>(t.get()));
    }

public:
    RustTiledAlgoBridge(void* rust, const TiledAlgoVtable& vt)
        : m_rust(rust), m_vt(vt) {}

    ~RustTiledAlgoBridge() override {
        if (m_vt.base.drop_fn && m_rust)
            m_vt.base.drop_fn(m_rust);
    }

    void newTarget(SP<Layout::ITarget> t) override {
        trackTarget(t);
        m_vt.base.new_target(m_rust, t ? static_cast<void*>(t.get()) : nullptr);
    }

    void movedTarget(SP<Layout::ITarget> t, std::optional<Vector2D> focal) override {
        trackTarget(t);
        bool has_focal = focal.has_value();
        double fx = has_focal ? focal->x : 0.0;
        double fy = has_focal ? focal->y : 0.0;
        m_vt.base.moved_target(m_rust, t ? static_cast<void*>(t.get()) : nullptr,
                                has_focal, fx, fy);
    }

    void removeTarget(SP<Layout::ITarget> t) override {
        m_vt.base.remove_target(m_rust, t ? static_cast<void*>(t.get()) : nullptr);
        untrackTarget(t);
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

    SP<Layout::ITarget> getNextCandidate(SP<Layout::ITarget> old) override {
        void* old_raw = old ? static_cast<void*>(old.get()) : nullptr;
        void* result_raw = m_vt.get_next_candidate(m_rust, old_raw);
        if (!result_raw) return nullptr;
        auto it = m_targets.find(result_raw);
        if (it != m_targets.end())
            return it->second.lock();
        return nullptr;
    }
};

extern "C" bool hyprland_api_add_tiled_algo(
    void* handle,
    const char* name_ptr, size_t name_len,
    void* rust_factory_data,
    const TiledAlgoVtable* vtable
) {
    if (!vtable || !rust_factory_data) return false;
    std::string name(name_ptr, name_len);

    TiledAlgoVtable vt_copy = *vtable;
    auto factory = [rust_factory_data, vt_copy]() -> UP<Layout::ITiledAlgorithm> {
        void* rust_algo = vt_copy.factory_fn(rust_factory_data);
        if (!rust_algo) return nullptr;
        return UP<Layout::ITiledAlgorithm>(new RustTiledAlgoBridge(rust_algo, vt_copy));
    };

    return HyprlandAPI::addTiledAlgo(handle, name,
                                      &typeid(RustTiledAlgoBridge),
                                      std::move(factory));
}

extern "C" bool hyprland_api_remove_algo(
    void* handle,
    const char* name_ptr, size_t name_len
) {
    std::string name(name_ptr, name_len);
    return HyprlandAPI::removeAlgo(handle, name);
}
