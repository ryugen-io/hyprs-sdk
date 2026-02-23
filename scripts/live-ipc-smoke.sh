#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${HYPRLAND_INSTANCE_SIGNATURE:-}" ]]; then
  echo "HYPRLAND_INSTANCE_SIGNATURE is not set" >&2
  exit 1
fi

runtime_base="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/hypr/${HYPRLAND_INSTANCE_SIGNATURE}"
for sock in .socket.sock .socket2.sock; do
  if [[ ! -S "${runtime_base}/${sock}" ]]; then
    echo "missing Hyprland socket: ${runtime_base}/${sock}" >&2
    exit 1
  fi
done

cargo test --test live_ipc_smoke -- --ignored --nocapture
