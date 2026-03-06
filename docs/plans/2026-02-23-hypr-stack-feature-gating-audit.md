# Hypr Stack Feature-Gating Audit (hyprs-sdk / hyprlog / hyprs-conf)

Date: 2026-02-23  
Scope: `hyprs-sdk`, `hyprlog`, `hyprs-conf`  
Goal: maximal kontrollierbare Builds (pro User-Usecase), ohne API/CLI-Divergenz.

## 1) Ist-Zustand

### hyprs-sdk
- Features:
  - `blocking`
  - `wayland`
  - `plugin-ffi`
- Default: leer (`default = []`)
- Status:
  - IPC + Dispatch + Types sind immer verfügbar.
  - Wayland-Protokolle korrekt hinter `wayland`.
  - C++ Bridge korrekt hinter `plugin-ffi`.
  - Neu: `hyprpm` Wrapper (`hyprs_sdk::hyprpm`) für Plugin-Lifecycle-CLI.

### hyprlog
- Features:
  - `cli` (default)
  - `ffi`
  - `hyprland`
- Status:
  - Gute Trennung CLI vs Library.
  - Hyprland-Integration korrekt optional (`hyprs-sdk` + `tokio` nur unter `hyprland`).
  - `hyprs-conf` wird aktuell immer eingebunden.

### hyprs-conf
- Features (neu umgesetzt):
  - `strict-header` (default on)
  - `discovery` (default on)
- Status:
  - Default-Verhalten bleibt unverändert (strikter Header + Rekursion).
  - `--no-default-features` Build funktioniert (ohne Discovery/Strict).

## 2) API/CLI-Parity (Soll)

### hyprs-sdk
- Parität zwischen:
  - `hyprs_sdk::ipc` (Rust API)
  - `hyprctl` (CLI)
- Neu umgesetzt:
  - `tests/live_cli_parity.rs` vergleicht JSON-Shape SDK vs `hyprctl` live.
  - `tests/live_full_smoke.rs` deckt read/set/dispatch live ab.

### hyprlog
- Parität zwischen:
  - Library API (`hyprlog::*`)
  - `hyprlog` CLI
- Offene Arbeit:
  - automatisierter Parity-Snapshot je Subcommand/Output-Backend.

### hyprs-conf
- Parität zwischen:
  - Library-Resolver
  - Tool-Install/Template-Output (`# hypr metadata` + `type`)
- Status:
  - im aktuellen Stand konsistent umgesetzt.

## 3) Gate-Matrix (konkret)

### Bereits umgesetzt
1. `hyprs-conf`: `strict-header`, `discovery`  
2. `hyprs-sdk`: Live-Smoke + CLI-Parity-Suite  
3. `hyprs-sdk`: `hyprpm` Rust-Wrapper für CLI-Lifecycle
4. `hyprs-sdk`: Source-Drift-Gate (`update-sources.sh --audit` + CI-Workflow)  
   - Hard-Fail Parity für: `hyprctl` commands, `setprop`, Socket2 events, Dispatchers, Hook-Events, `hyprpm` commands, Plugin-API (non-deprecated).

### Empfohlene nächste Gates (low risk)
1. `hyprlog`: `conf-discovery` Feature (für `hyprs-conf`-Integration)  
2. `hyprlog`: `highlight` Feature (regex/highlight optional)  
3. `hyprlog`: `file-output` Feature (flate2/retention optional)

### Empfohlene nächste Gates (medium risk)
1. `hyprs-sdk`: optionales Feature `hyprpm` (derzeit always-on API)  
2. `hyprs-sdk`: optionales Feature `plugin-api` (reine Rust plugin module separierbar von Core IPC)

## 4) Akzeptanzkriterien

1. `cargo check`/`cargo test` in Default-Build grün.  
2. `--no-default-features` für `hyprs-conf` grün.  
3. Live-Tests:
   - `scripts/live-full-smoke.sh` grün in laufender Hyprland-Session.
4. Keine semantischen Diffs zwischen SDK-JSON und `hyprctl -j` JSON-Shape in Parity-Test.
