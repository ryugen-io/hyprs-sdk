#!/usr/bin/env bash
set -euo pipefail

# Update or initialize the Hyprland reference source checkout.
# Usage:
#   ./scripts/update-sources.sh              # update to latest tag
#   ./scripts/update-sources.sh v0.54.0      # update to specific version
#   ./scripts/update-sources.sh --check      # just show what would change
#   ./scripts/update-sources.sh --diff       # show full diff of SDK-relevant files

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

REPO="https://github.com/hyprwm/Hyprland.git"
TARGET_DIR=".sources/Hyprland"
VERSION_FILE=".sources/.version"

# SDK-relevant paths — these are the files we actually read from.
SDK_PATHS=(
    "src/debug/HyprCtl.cpp"
    "src/managers/EventManager.cpp"
    "src/managers/KeybindManager.cpp"
    "src/managers/HookSystemManager.hpp"
    "src/plugins/PluginAPI.hpp"
    "src/plugins/HookSystem.hpp"
    "src/plugins/PluginSystem.hpp"
    "src/config/ConfigManager.hpp"
    "src/config/ConfigValue.hpp"
    "src/desktop/view/Window.hpp"
    "src/desktop/Workspace.hpp"
    "src/helpers/Monitor.hpp"
    "src/desktop/view/LayerSurface.hpp"
    "protocols/"
    "src/protocols/"
)

# ── Helpers ──────────────────────────────────────────────────────────

current_version() {
    if [ -f "$VERSION_FILE" ]; then
        cat "$VERSION_FILE"
    else
        echo "none"
    fi
}

latest_tag() {
    git -C "$TARGET_DIR" tag --list 'v*' \
        | sort -V \
        | tail -1
}

ensure_repo() {
    if [ ! -d "$TARGET_DIR/.git" ]; then
        echo "Cloning Hyprland source..."
        git clone --no-checkout "$REPO" "$TARGET_DIR"
    fi
}

fetch_tags() {
    echo "Fetching tags..."
    git -C "$TARGET_DIR" fetch --tags --quiet
}

validate_version() {
    local ver="$1"
    if ! git -C "$TARGET_DIR" rev-parse --verify "refs/tags/$ver" >/dev/null 2>&1; then
        echo "Error: tag '$ver' does not exist."
        echo ""
        echo "Available versions:"
        git -C "$TARGET_DIR" tag --list 'v*' | sort -V | tail -10
        exit 1
    fi
}

# Show stats for SDK-relevant file changes between two versions.
show_diff_summary() {
    local from="$1" to="$2"

    echo ""
    echo "=== Changes from $from -> $to ==="
    echo ""

    # Overall stats
    local total_files total_insertions total_deletions
    total_files=$(git -C "$TARGET_DIR" diff --stat "$from..$to" -- "${SDK_PATHS[@]}" | tail -1)
    echo "SDK-relevant: $total_files"
    echo ""

    # Per-category breakdown
    local ipc_changes protocol_changes plugin_changes type_changes config_changes

    ipc_changes=$(git -C "$TARGET_DIR" diff --stat "$from..$to" -- \
        "src/debug/HyprCtl.cpp" \
        "src/managers/EventManager.cpp" \
        "src/managers/KeybindManager.cpp" \
        2>/dev/null | tail -1)
    [ -n "$ipc_changes" ] && echo "  IPC:       $ipc_changes"

    protocol_changes=$(git -C "$TARGET_DIR" diff --stat "$from..$to" -- \
        "protocols/" \
        "src/protocols/" \
        2>/dev/null | tail -1)
    [ -n "$protocol_changes" ] && echo "  Protocols: $protocol_changes"

    plugin_changes=$(git -C "$TARGET_DIR" diff --stat "$from..$to" -- \
        "src/plugins/PluginAPI.hpp" \
        "src/plugins/HookSystem.hpp" \
        "src/plugins/PluginSystem.hpp" \
        "src/managers/HookSystemManager.hpp" \
        2>/dev/null | tail -1)
    [ -n "$plugin_changes" ] && echo "  Plugin:    $plugin_changes"

    type_changes=$(git -C "$TARGET_DIR" diff --stat "$from..$to" -- \
        "src/desktop/" \
        "src/helpers/Monitor.hpp" \
        2>/dev/null | tail -1)
    [ -n "$type_changes" ] && echo "  Types:     $type_changes"

    config_changes=$(git -C "$TARGET_DIR" diff --stat "$from..$to" -- \
        "src/config/ConfigManager.hpp" \
        "src/config/ConfigValue.hpp" \
        2>/dev/null | tail -1)
    [ -n "$config_changes" ] && echo "  Config:    $config_changes"

    echo ""

    # Check for new/removed IPC commands
    local new_commands removed_commands
    new_commands=$(git -C "$TARGET_DIR" diff "$from..$to" -- "src/debug/HyprCtl.cpp" \
        | grep '^+.*registerCommand' | grep -v '^+++' || true)
    removed_commands=$(git -C "$TARGET_DIR" diff "$from..$to" -- "src/debug/HyprCtl.cpp" \
        | grep '^-.*registerCommand' | grep -v '^---' || true)

    if [ -n "$new_commands" ]; then
        echo "  New IPC commands:"
        echo "$new_commands" | sed 's/^+/    +/'
        echo ""
    fi
    if [ -n "$removed_commands" ]; then
        echo "  Removed IPC commands:"
        echo "$removed_commands" | sed 's/^-/    -/'
        echo ""
    fi

    # Check for new/removed events
    local new_events removed_events
    new_events=$(git -C "$TARGET_DIR" diff "$from..$to" -- "src/managers/EventManager.cpp" \
        | grep '^+.*postEvent' | grep -v '^+++' || true)
    removed_events=$(git -C "$TARGET_DIR" diff "$from..$to" -- "src/managers/EventManager.cpp" \
        | grep '^-.*postEvent' | grep -v '^---' || true)

    if [ -n "$new_events" ]; then
        echo "  New events:"
        echo "$new_events" | sed 's/^+/    +/'
        echo ""
    fi
    if [ -n "$removed_events" ]; then
        echo "  Removed events:"
        echo "$removed_events" | sed 's/^-/    -/'
        echo ""
    fi

    # Check for new/removed hook events
    local new_hooks removed_hooks
    new_hooks=$(git -C "$TARGET_DIR" diff "$from..$to" -- "src/managers/HookSystemManager.hpp" \
        | grep -E '^\+.*(EMIT_HOOK_EVENT|HOOK_)' | grep -v '^+++' || true)
    removed_hooks=$(git -C "$TARGET_DIR" diff "$from..$to" -- "src/managers/HookSystemManager.hpp" \
        | grep -E '^-.*(EMIT_HOOK_EVENT|HOOK_)' | grep -v '^---' || true)

    if [ -n "$new_hooks" ]; then
        echo "  New hook events:"
        echo "$new_hooks" | sed 's/^+/    +/'
        echo ""
    fi
    if [ -n "$removed_hooks" ]; then
        echo "  Removed hook events:"
        echo "$removed_hooks" | sed 's/^-/    -/'
        echo ""
    fi

    # Check for protocol XML changes
    local xml_changes
    xml_changes=$(git -C "$TARGET_DIR" diff --name-only "$from..$to" -- "protocols/*.xml" || true)
    if [ -n "$xml_changes" ]; then
        echo "  Changed protocol XMLs:"
        echo "$xml_changes" | sed 's/^/    /'
        echo ""
    fi

    # Check for PluginAPI signature changes
    local api_changes
    api_changes=$(git -C "$TARGET_DIR" diff "$from..$to" -- "src/plugins/PluginAPI.hpp" \
        | grep -E '^[+-].*(inline|namespace HyprlandAPI)' | grep -Ev '^\+\+\+|^---' || true)
    if [ -n "$api_changes" ]; then
        echo "  Plugin API changes:"
        echo "$api_changes" | sed 's/^/    /'
        echo ""
    fi
}

# ── Main ─────────────────────────────────────────────────────────────

CURRENT=$(current_version)
MODE="update"
REQUESTED=""

case "${1:-}" in
    --check)
        MODE="check"
        ;;
    --diff)
        MODE="diff"
        ;;
    --help|-h)
        echo "Usage: $0 [VERSION|--check|--diff]"
        echo ""
        echo "  (no args)   Update to latest tag"
        echo "  VERSION     Update to specific version (e.g., v0.54.0)"
        echo "  --check     Show what would change without updating"
        echo "  --diff      Show full diff of SDK-relevant files"
        exit 0
        ;;
    "")
        REQUESTED=""
        ;;
    *)
        REQUESTED="$1"
        ;;
esac

ensure_repo
fetch_tags

# Resolve target version.
if [ -n "$REQUESTED" ]; then
    TARGET="$REQUESTED"
else
    TARGET=$(latest_tag)
fi

validate_version "$TARGET"

echo "Current: $CURRENT"
echo "Target:  $TARGET"

# Same version — nothing to do.
if [ "$CURRENT" = "$TARGET" ]; then
    echo ""
    echo "Already at $TARGET. Nothing to do."
    exit 0
fi

# Show diff summary.
if [ "$CURRENT" != "none" ]; then
    show_diff_summary "$CURRENT" "$TARGET"
fi

# Check-only mode — stop here.
if [ "$MODE" = "check" ]; then
    echo "Run without --check to apply the update."
    exit 0
fi

# Diff-only mode — show full diff of SDK-relevant files.
if [ "$MODE" = "diff" ] && [ "$CURRENT" != "none" ]; then
    echo "=== Full diff of SDK-relevant files ==="
    git -C "$TARGET_DIR" diff "$CURRENT..$TARGET" -- "${SDK_PATHS[@]}"
    exit 0
fi

# Apply the update.
echo "Checking out $TARGET..."
git -C "$TARGET_DIR" checkout --quiet "$TARGET"

echo "$TARGET" > "$VERSION_FILE"
echo ""
echo "Updated to $TARGET."

if [ "$CURRENT" != "none" ] && [ "$CURRENT" != "$TARGET" ]; then
    echo ""
    echo "Review the changes above and update SDK bindings as needed."
    echo "Run 'cargo test --features wayland,blocking' to check for breakage."
fi
