// Compatibility symbols for Hyprland header-declared types.
//
// Some constructors used by PluginAPI headers are only provided by the
// Hyprland executable at runtime (not by shared libs available during tests).
// We provide weak definitions so test binaries can link outside Hyprland.
#include "common.h"

__attribute__((weak)) CHyprColor::CHyprColor()
    : r(0.0), g(0.0), b(0.0), a(0.0), m_okLab(Hyprgraphics::CColor::SOkLab{}) {}

__attribute__((weak)) CHyprColor::CHyprColor(float red, float green, float blue, float alpha)
    : r(static_cast<double>(red)),
      g(static_cast<double>(green)),
      b(static_cast<double>(blue)),
      a(static_cast<double>(alpha)),
      m_okLab(Hyprgraphics::CColor(Hyprgraphics::CColor::SSRGB{
          static_cast<double>(red),
          static_cast<double>(green),
          static_cast<double>(blue),
      }).asOkLab()) {}

__attribute__((weak)) CHyprColor::CHyprColor(const Hyprgraphics::CColor& col, float alpha)
    : a(static_cast<double>(alpha)), m_okLab(col.asOkLab()) {
    const auto rgb = col.asRgb();
    r = rgb.r;
    g = rgb.g;
    b = rgb.b;
}

__attribute__((weak)) CHyprColor::CHyprColor(uint64_t argb)
    : r(static_cast<double>((argb >> 16) & 0xFFu) / 255.0),
      g(static_cast<double>((argb >> 8) & 0xFFu) / 255.0),
      b(static_cast<double>(argb & 0xFFu) / 255.0),
      a(static_cast<double>((argb >> 24) & 0xFFu) / 255.0),
      m_okLab(Hyprgraphics::CColor(Hyprgraphics::CColor::SSRGB{
          static_cast<double>((argb >> 16) & 0xFFu) / 255.0,
          static_cast<double>((argb >> 8) & 0xFFu) / 255.0,
          static_cast<double>(argb & 0xFFu) / 255.0,
      }).asOkLab()) {}

namespace Log {
__attribute__((weak)) CLogger::CLogger() = default;
}

// Layout virtual method stubs — these live in the Hyprland binary but are
// needed at link time for the bridge's vtables when building test binaries.
#include <hyprland/src/layout/algorithm/ModeAlgorithm.hpp>
#include <hyprland/src/layout/algorithm/TiledAlgorithm.hpp>
#include <hyprland/src/layout/algorithm/FloatingAlgorithm.hpp>

namespace Layout {
__attribute__((weak)) std::expected<void, std::string> IModeAlgorithm::layoutMsg(const std::string_view&) {
    return std::unexpected(std::string("not implemented"));
}
__attribute__((weak)) std::optional<Vector2D> IModeAlgorithm::predictSizeForNewTarget() {
    return std::nullopt;
}
__attribute__((weak)) std::optional<Vector2D> IModeAlgorithm::focalPointForDir(SP<ITarget>, Math::eDirection) {
    return std::nullopt;
}
__attribute__((weak)) void IFloatingAlgorithm::recenter(SP<ITarget>) {}
__attribute__((weak)) void IFloatingAlgorithm::recalculate() {}
}

// HyprlandAPI stubs — only the running compositor provides the real implementations.
__attribute__((weak)) bool HyprlandAPI::addTiledAlgo(HANDLE, const std::string&, const std::type_info*, std::function<UP<Layout::ITiledAlgorithm>()>&&) {
    return false;
}
__attribute__((weak)) bool HyprlandAPI::addFloatingAlgo(HANDLE, const std::string&, const std::type_info*, std::function<UP<Layout::IFloatingAlgorithm>()>&&) {
    return false;
}
__attribute__((weak)) bool HyprlandAPI::removeAlgo(HANDLE, const std::string&) {
    return false;
}
