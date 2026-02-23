#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${HYPRLAND_INSTANCE_SIGNATURE:-}" ]]; then
  echo "HYPRLAND_INSTANCE_SIGNATURE is not set" >&2
  exit 1
fi

runtime_base="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/hypr/${HYPRLAND_INSTANCE_SIGNATURE}"
if [[ ! -S "${runtime_base}/.socket.sock" ]]; then
  echo "missing Hyprland socket: ${runtime_base}/.socket.sock" >&2
  exit 1
fi

plugin_name="hypr-sdk-smoke-plugin"
plugin_src="tests/fixtures/plugin_smoke_cpp.cpp"

if ! command -v g++ >/dev/null 2>&1; then
  echo "g++ not found (required for live plugin E2E fixture build)" >&2
  exit 1
fi

if [[ ! -f "${plugin_src}" ]]; then
  echo "missing plugin fixture source: ${plugin_src}" >&2
  exit 1
fi

hyprland_cflags="$(pkg-config --cflags hyprland 2>/dev/null || true)"
if [[ -z "${hyprland_cflags}" ]]; then
  echo "pkg-config could not provide hyprland headers (required for plugin fixture)" >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT
plugin_so="${tmp_dir}/hypr-sdk-smoke-plugin.so"

g++ -std=c++2b -shared -fPIC ${hyprland_cflags} "${plugin_src}" -o "${plugin_so}"

if [[ ! -f "${plugin_so}" ]]; then
  echo "expected plugin shared object not found: ${plugin_so}" >&2
  exit 1
fi

export HYPR_SDK_TEST_PLUGIN_SO="$(realpath "${plugin_so}")"
export HYPR_SDK_TEST_PLUGIN_NAME="${plugin_name}"

cargo test --test live_plugin_e2e -- --ignored --nocapture
