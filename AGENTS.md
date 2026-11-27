# AGENTS.md — `solace-rs`

Cross-platform (macOS, Windows, Linux) Tauri v2 + Rust + React 19 clone of Solace (automatic Light/Dark mode, paired wallpapers, solar scheduling, evening warmth, weather-aware switching).

## Architecture

- **Frontend (`src/`)**: React 19 + TypeScript + Tailwind CSS v4 + Zustand (`src/store/useSolaceStore.ts`). Renders a compact 376×680 floating popover panel anchored to the menu bar / system tray.
- **Backend (`src-tauri/src/`)**:
  - `config.rs`: Persistent JSON config at `$CONFIG_DIR/solace-rs/config.json`, manual override state, and 10-day weather decision log.
  - `solar.rs`: Pure-Rust NOAA solar sunrise/sunset calculator (100% local, unit-tested).
  - `appearance.rs`: Cross-platform OS dark/light mode getter/setter (`osascript`/`defaults` on macOS, Registry + `WM_SETTINGCHANGE` on Windows, `gsettings`/`plasma-apply-colorscheme` on Linux).
  - `wallpaper.rs`: Cross-platform multi-display wallpaper getter/setter.
  - `warmth.rs`: Tanner Helland Kelvin→RGB curve (`1900K–6500K`), 15-minute gradual fade, and native display gamma LUT (`CGSetDisplayTransferByTable` on macOS, `SetDeviceGammaRamp` on Windows, `xrandr` on Linux).
  - `weather.rs`: Open-Meteo forecast & geocoding client (zero API key) + IP geolocation fallback.
  - `scheduler.rs`: 15-second background tick syncing external OS changes, solar transitions, weather decisions, and warmth re-assertion.
  - `lib.rs`: Tauri v2 app setup, dynamic Sun/Moon tray icon, popover positioning, global shortcut registration, and IPC commands.

## Build & Test Commands

```bash
npm ci
npm run build
cd src-tauri && cargo test
npm run tauri:build
```

## Release Workflow

Pushing a tag `v*` triggers `.github/workflows/release.yml`, building signed/bundled installers for macOS (`aarch64-apple-darwin`, `x86_64-apple-darwin`), Windows, and Ubuntu via `tauri-apps/tauri-action@v0`.
