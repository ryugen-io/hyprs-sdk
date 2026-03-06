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

echo "[live-full-smoke] running baseline live IPC smoke"
cargo test --test live_ipc_smoke -- --ignored --nocapture

echo "[live-full-smoke] running full read/set/dispatch smoke"
cargo test --test live_full_smoke -- --ignored --nocapture

echo "[live-full-smoke] running SDK vs hyprctl parity smoke"
cargo test --test live_cli_parity -- --ignored --nocapture

if [[ "${HYPR_SDK_INCLUDE_PLUGIN_SMOKE:-0}" == "1" ]]; then
  echo "[live-full-smoke] running plugin load/unload smoke"
  scripts/live-plugin-e2e.sh
fi

echo "[live-full-smoke] all selected live checks passed"
