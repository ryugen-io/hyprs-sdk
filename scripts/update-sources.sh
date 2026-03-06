#!/usr/bin/env bash
set -euo pipefail

# Update or initialize the Hyprland reference source checkout.
# Usage:
#   ./scripts/update-sources.sh              # update to latest tag
#   ./scripts/update-sources.sh v0.54.0      # update to specific version
#   ./scripts/update-sources.sh --check      # just show what would change
#   ./scripts/update-sources.sh --diff       # show full diff of SDK-relevant files
#   ./scripts/update-sources.sh --audit      # hard-fail source-sync audit only

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

# Validate that SDK registries/surfaces match the checked-out Hyprland source.
extract_source_commands() {
    perl -ne 'while(/registerCommand\(SHyprCtlCommand\{[^\n]*?"([^"]+)"/g){print "$1\n"}' \
        "$TARGET_DIR/src/debug/HyprCtl.cpp" \
        | sort -u
}

extract_source_props() {
    perl -ne 'while(/PROP == "([^"]+)"/g){print "$1\n"}' \
        "$TARGET_DIR/src/managers/KeybindManager.cpp" \
        | sort -u
}

extract_sdk_commands() {
    perl -ne '
        while(/flagged\(flags,\s*"([^"]+)"/g){print "$1\n";}
        while(/"([^"]+)"\.into\(\)/g){print "$1\n";}
        while(/format!\("([^"]+)"/g){
            my $s = $1;
            if ($s =~ /^\[\[BATCH\]\]/) {
                print "[[BATCH]]\n";
            } else {
                my ($cmd) = split(/ /, $s, 2);
                print "$cmd\n";
            }
        }
    ' src/ipc/commands.rs \
        | awk '/^[a-z][a-z0-9]*$/ || /^\[\[BATCH\]\]$/' \
        | sort -u
}

extract_sdk_props() {
    perl -ne 'while(/=> "([^"]+)"/g){print "$1\n"}' \
        src/ipc/window_property.rs \
        | sort -u
}

extract_source_events() {
    find "$TARGET_DIR/src" -type f \( -name '*.cpp' -o -name '*.hpp' \) -print0 \
        | xargs -0 perl -0777 -ne '
            while(/postEvent\s*\(\s*(?:SHyprIPCEvent\s*)?\{\s*(?:\.event\s*=\s*"([^"]+)"|"([^"]+)")/g){
                my $event = defined($1) ? $1 : $2;
                print "$event\n";
            }
        ' \
        | awk '/^[a-z][a-z0-9]*$/' \
        | sort -u
}

extract_sdk_events() {
    perl -ne 'while(/=>\s*"([a-z0-9]+)"/g){print "$1\n"}' \
        src/ipc/events/types.rs \
        | sort -u
}

extract_source_dispatchers() {
    perl -ne 'while(/m_dispatchers\["([^"]+)"\]/g){print "$1\n"}' \
        "$TARGET_DIR/src/managers/KeybindManager.cpp" \
        | awk '/^[a-z][a-z0-9]*$/' \
        | sort -u
}

extract_sdk_dispatchers() {
    perl -ne '
        while(/DispatchCmd::no_args\("([^"]+)"\)/g){print "$1\n";}
        while(/name:\s*"([^"]+)"/g){print "$1\n";}
    ' src/dispatch/*.rs \
        | awk '/^[a-z][a-z0-9]*$/' \
        | sort -u
}

extract_source_hooks() {
    # v0.54+ uses a typed CEventBus in src/event/EventBus.hpp instead of EMIT_HOOK_EVENT macros.
    # Parse the nested struct layout, build qualified paths, and map to legacy SDK names.
    local bus_header="$TARGET_DIR/src/event/EventBus.hpp"
    if [ -f "$bus_header" ]; then
        perl -0777 -ne '
            # Mapping from "structpath.field" to legacy SDK hook name.
            # When Hyprland adds a new event field, add its mapping here.
            my %MAP = (
                "ready"                     => "ready",
                "tick"                      => "tick",
                "window.open"               => "openWindow",
                "window.openEarly"          => "openWindowEarly",
                "window.destroy"            => "destroyWindow",
                "window.close"              => "closeWindow",
                "window.active"             => "activeWindow",
                "window.urgent"             => "urgent",
                "window.title"              => "windowTitle",
                "window.class_"             => "windowClass",
                "window.pin"                => "pin",
                "window.fullscreen"         => "fullscreen",
                "window.updateRules"        => "windowUpdateRules",
                "window.moveToWorkspace"    => "moveWindow",
                "layer.opened"              => "openLayer",
                "layer.closed"              => "closeLayer",
                "input.mouse.move"          => "mouseMove",
                "input.mouse.button"        => "mouseButton",
                "input.mouse.axis"          => "mouseAxis",
                "input.keyboard.key"        => "keyPress",
                "input.keyboard.layout"     => "activeLayout",
                "input.keyboard.focus"      => "keyboardFocus",
                "input.tablet.axis"         => "tabletAxis",
                "input.tablet.button"       => "tabletButton",
                "input.tablet.proximity"    => "tabletProximity",
                "input.tablet.tip"          => "tabletTip",
                "input.touch.cancel"        => "touchCancel",
                "input.touch.down"          => "touchDown",
                "input.touch.up"            => "touchUp",
                "input.touch.motion"        => "touchMove",
                "render.pre"                => "preRender",
                "render.stage"              => "render",
                "screenshare.state"         => "screencast",
                "gesture.swipe.begin"       => "swipeBegin",
                "gesture.swipe.end"         => "swipeEnd",
                "gesture.swipe.update"      => "swipeUpdate",
                "gesture.pinch.begin"       => "pinchBegin",
                "gesture.pinch.end"         => "pinchEnd",
                "gesture.pinch.update"      => "pinchUpdate",
                "monitor.newMon"            => "newMonitor",
                "monitor.preAdded"          => "preMonitorAdded",
                "monitor.added"             => "monitorAdded",
                "monitor.preRemoved"        => "preMonitorRemoved",
                "monitor.removed"           => "monitorRemoved",
                "monitor.preCommit"         => "preMonitorCommit",
                "monitor.focused"           => "focusedMon",
                "monitor.layoutChanged"     => "monitorLayoutChanged",
                "workspace.moveToMonitor"   => "moveWorkspace",
                "workspace.active"          => "workspace",
                "workspace.created"         => "createWorkspace",
                "workspace.removed"         => "destroyWorkspace",
                "config.preReload"          => "preConfigReload",
                "config.reloaded"           => "configReloaded",
                "keybinds.submap"           => "submap",
            );

            my @lines = split /\n/, $_;

            # Pass 1: find all named struct scopes (start_line, end_line, name).
            # C++ names anonymous structs at the closing brace: } name;
            my @scopes;
            for my $i (0..$#lines) {
                next unless $lines[$i] =~ /\}\s*([a-zA-Z_]\w*)\s*;/;
                my $name = $1;
                next if $name eq "m_events";
                # Walk backwards to find matching struct {
                my $depth = 1;
                for my $j (reverse 0..($i-1)) {
                    my $l = $lines[$j];
                    $depth++ while $l =~ /\}/g;
                    $depth-- while $l =~ /\{/g;
                    if ($depth == 0) {
                        push @scopes, [$j, $i, $name];
                        last;
                    }
                }
            }

            # Pass 2: for each Event/Cancellable field, find enclosing scopes.
            for my $i (0..$#lines) {
                next unless $lines[$i] =~ /(?:Event|Cancellable)<.*>\s+([a-zA-Z_]\w*)\s*;/;
                my $field = $1;
                # Collect enclosing named scopes (sorted by start line = outermost first)
                my @path;
                for my $s (sort { $a->[0] <=> $b->[0] } @scopes) {
                    push @path, $s->[2] if $s->[0] < $i && $i < $s->[1];
                }
                push @path, $field;
                my $full = join(".", @path);
                if (exists $MAP{$full}) {
                    print "$MAP{$full}\n";
                } else {
                    print "UNMAPPED:$full\n";
                }
            }
        ' "$bus_header" | sort -u
    else
        # Legacy: pre-0.54 used EMIT_HOOK_EVENT macros
        find "$TARGET_DIR/src" -type f \( -name '*.cpp' -o -name '*.hpp' \) -print0 \
            | xargs -0 perl -ne 'while(/EMIT_HOOK_EVENT(?:_CANCELLABLE)?\("([^"]+)"/g){print "$1\n"}' \
            | awk '/^[A-Za-z][A-Za-z0-9]*$/' \
            | sort -u
    fi
}

extract_sdk_hooks() {
    perl -ne 'while(/"([A-Za-z][A-Za-z0-9]*)"\s*=>\s*Some\(Self::/g){print "$1\n"}' \
        src/plugin/hooks.rs \
        | sort -u
}

extract_source_hyprpm_commands() {
    perl -ne 'while(/command\[0\]\s*==\s*"([a-z0-9-]+)"/g){print "$1\n"}' \
        "$TARGET_DIR/hyprpm/src/main.cpp" \
        | sort -u
}

extract_sdk_hyprpm_commands() {
    perl -ne '
        while(/run_raw\(&\["([a-z0-9-]+)"/g){print "$1\n";}
        while(/vec!\["([a-z0-9-]+)"/g){print "$1\n";}
    ' src/hyprpm.rs \
        | sort -u
}

extract_source_plugin_api_methods() {
    perl -ne '
        while(/APICALL\s+([^\n;]*?)\b([A-Za-z_][A-Za-z0-9_]*)\s*\(/g){
            my $sig = $1;
            my $name = $2;
            next if $sig =~ /\[\[deprecated\]\]/;
            next if $name =~ /^__hyprland_/;
            print "$name\n";
        }
    ' "$TARGET_DIR/src/plugins/PluginAPI.hpp" \
        | sort -u
}

extract_sdk_plugin_api_methods() {
    # Skip deprecated API calls that our bridge still uses for now.
    local -a DEPRECATED=(addLayout removeLayout addDispatcher registerCallbackDynamic
                         unregisterCallback getFunctionAddressFromSignature)
    local filter
    filter=$(printf '%s\n' "${DEPRECATED[@]}" | paste -sd'|')

    perl -ne 'while(/HyprlandAPI::([A-Za-z_][A-Za-z0-9_]*)\(/g){print "$1\n"}' \
        src/plugin/bridge/*.cpp \
        | grep -Ev "^($filter)$" \
        | sort -u
}

compare_sets() {
    local label="$1" expected="$2" actual="$3"
    local missing extra

    missing=$(comm -23 "$expected" "$actual" || true)
    extra=$(comm -13 "$expected" "$actual" || true)

    if [ -n "$missing" ] || [ -n "$extra" ]; then
        echo ""
        echo "ERROR: $label drift detected."
        if [ -n "$missing" ]; then
            echo "  Missing in SDK:"
            echo "$missing" | sed 's/^/    - /'
        fi
        if [ -n "$extra" ]; then
            echo "  Extra in SDK:"
            echo "$extra" | sed 's/^/    + /'
        fi
        return 1
    fi

    return 0
}

run_source_sync_audit() {
    local target="$1"
    local tmpdir \
        source_cmds source_props source_events source_dispatchers source_hooks \
        source_hyprpm source_plugin_api \
        sdk_cmds sdk_props sdk_events sdk_dispatchers sdk_hooks \
        sdk_hyprpm sdk_plugin_api

    tmpdir=$(mktemp -d)
    source_cmds="$tmpdir/source_cmds.txt"
    source_props="$tmpdir/source_props.txt"
    source_events="$tmpdir/source_events.txt"
    source_dispatchers="$tmpdir/source_dispatchers.txt"
    source_hooks="$tmpdir/source_hooks.txt"
    source_hyprpm="$tmpdir/source_hyprpm.txt"
    source_plugin_api="$tmpdir/source_plugin_api.txt"
    sdk_cmds="$tmpdir/sdk_cmds.txt"
    sdk_props="$tmpdir/sdk_props.txt"
    sdk_events="$tmpdir/sdk_events.txt"
    sdk_dispatchers="$tmpdir/sdk_dispatchers.txt"
    sdk_hooks="$tmpdir/sdk_hooks.txt"
    sdk_hyprpm="$tmpdir/sdk_hyprpm.txt"
    sdk_plugin_api="$tmpdir/sdk_plugin_api.txt"

    extract_source_commands > "$source_cmds"
    extract_source_props > "$source_props"
    extract_source_events > "$source_events"
    extract_source_dispatchers > "$source_dispatchers"
    extract_source_hooks > "$source_hooks"
    extract_source_hyprpm_commands > "$source_hyprpm"
    extract_source_plugin_api_methods > "$source_plugin_api"
    extract_sdk_commands > "$sdk_cmds"
    extract_sdk_props > "$sdk_props"
    extract_sdk_events > "$sdk_events"
    extract_sdk_dispatchers > "$sdk_dispatchers"
    extract_sdk_hooks > "$sdk_hooks"
    extract_sdk_hyprpm_commands > "$sdk_hyprpm"
    extract_sdk_plugin_api_methods > "$sdk_plugin_api"

    echo ""
    echo "Running source-sync audit against $target..."
    compare_sets "hyprctl command registry" "$source_cmds" "$sdk_cmds"
    compare_sets "setprop property registry" "$source_props" "$sdk_props"
    compare_sets "socket2 event registry" "$source_events" "$sdk_events"
    compare_sets "dispatcher registry" "$source_dispatchers" "$sdk_dispatchers"
    compare_sets "hook event registry" "$source_hooks" "$sdk_hooks"
    compare_sets "hyprpm command registry" "$source_hyprpm" "$sdk_hyprpm"
    compare_sets "plugin API (non-deprecated methods)" "$source_plugin_api" "$sdk_plugin_api"
    echo "Source-sync audit passed."

    rm -rf "$tmpdir"
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
    --audit)
        MODE="audit"
        ;;
    --help|-h)
        echo "Usage: $0 [VERSION|--check|--diff|--audit]"
        echo ""
        echo "  (no args)   Update to latest tag"
        echo "  VERSION     Update to specific version (e.g., v0.54.0)"
        echo "  --check     Show what would change without updating"
        echo "  --diff      Show full diff of SDK-relevant files"
        echo "  --audit     Run source-sync audit and fail on drift"
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

# Audit-only mode — enforce source-sync parity and fail hard on drift.
if [ "$MODE" = "audit" ]; then
    echo "Checking out $TARGET for audit..."
    git -C "$TARGET_DIR" checkout --quiet "$TARGET"
    run_source_sync_audit "$TARGET"
    echo "Audit finished successfully."
    exit 0
fi

# Same version — nothing to do.
if [ "$CURRENT" = "$TARGET" ]; then
    echo ""
    echo "Already at $TARGET. Nothing to do."
    echo "Checking out $TARGET for audit..."
    git -C "$TARGET_DIR" checkout --quiet "$TARGET"
    run_source_sync_audit "$TARGET"
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
run_source_sync_audit "$TARGET"

if [ "$CURRENT" != "none" ] && [ "$CURRENT" != "$TARGET" ]; then
    echo ""
    echo "Review the changes above and update SDK bindings as needed."
    echo "Run 'cargo test --features wayland,blocking' to check for breakage."
fi
