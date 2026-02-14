#!/usr/bin/env bash
set -euo pipefail

REPO="https://github.com/hyprwm/Hyprland.git"
TARGET_DIR=".sources/Hyprland"
VERSION="${1:-v0.53.0}"

if [ -d "$TARGET_DIR/.git" ]; then
    echo "Updating Hyprland source to $VERSION..."
    cd "$TARGET_DIR"
    git fetch --tags
    git checkout "$VERSION"
    cd - > /dev/null
else
    echo "Cloning Hyprland source at $VERSION..."
    git clone --depth 1 --branch "$VERSION" "$REPO" "$TARGET_DIR"
fi

echo "$VERSION" > .sources/.version
echo "Hyprland source ready at $VERSION"
