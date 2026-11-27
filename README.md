# Solace (`solace-rs`)

A cross-platform (macOS, Windows, Linux) open-source 1:1 feature clone of [Solace](https://www.theodorehq.com/solace/) built with **Rust**, **Tauri v2**, **React 19**, and **Tailwind CSS v4**.

Your display, in tune with the world around you — automatic Light/Dark mode scheduling, paired multi-display wallpapers, Evening Warmth (blue light reduction), Weather-Aware overcast switching, and instant global keyboard shortcuts in one lightweight menu bar / system tray app.

## Features (1:1 Parity)

- **Menu Bar / System Tray App**: Runs quietly in your menu bar (macOS, no Dock icon) or system tray (Windows/Linux) with a dynamic **Sun / Moon** icon that stays in sync with OS appearance changes.
- **Instant Mode Switching & First-Switch Respect**:
  - Large tactile **Light Mode / Dark Mode** card and global keyboard shortcut (`⌘⇧D` on macOS, `Ctrl+Shift+D` on Windows/Linux — customizable in-place).
  - Respectful first-switch prompt on fresh install (`Switch` / `Keep Current`).
  - Manual overrides persist across restarts until the next scheduled sunrise or sunset.
- **Paired Day & Night Wallpapers**:
  - Pick separate Light and Dark wallpapers with live previews.
  - Automatically swaps across all connected displays whenever mode changes, with "Wallpaper Not Found" detection and original wallpaper restoration when disabled.
- **Solar Scheduling (100% Local Offline NOAA Math)**:
  - Exact daily sunrise & sunset calculations on-device — zero location telemetry.
  - Interactive 24-hour solar timeline with draggable Sun & Moon markers and inline preset offset picker (`-2h` to `+2h` in 15-minute increments).
  - Built-in city search (Open-Meteo Geocoding) or optional one-click auto-detect.
- **Evening Warmth (6500K → 1900K Blue Light Reduction)**:
  - Independent schedule: **At Sunset**, **Fixed Time** (`3:00 PM` – `12:00 AM`), or **Always On**.
  - Gradual 15-minute smooth ease-in transition and automatic sunrise reset.
  - Native hardware display gamma LUT (`CoreGraphics` on macOS, `SetDeviceGammaRamp` on Windows, `xrandr` on Linux) with live drag preview, wake re-assertion, and **Compatibility Mode** toggle.
- **Weather-Aware Switching**:
  - One morning decision per day using Open-Meteo daylight cloud forecasts (no API key, no account) so your screen never flickers.
  - Three sensitivity levels: **Cloudy** (50%), **Overcast** (70%, default), and **Gloomy** (90%).
  - Live condition icon, clickable `°C / °F` temperature toggle, Cloud / Rain / UV averages, native notification on gloomy days, **Keep Light** override, and a **10-day Weather Decisions** audit log in Settings.
- **Private by Design**: No account, zero telemetry, zero analytics.

## Installation

Download the latest prebuilt installer for **macOS (Apple Silicon & Intel)**, **Windows**, or **Linux** from [GitHub Releases](https://github.com/HamidRezaAshkiyan/solace-rs/releases).

### Build & Install Locally

**macOS:**
```bash
chmod +x scripts/install.sh
./scripts/install.sh
```

**Windows (PowerShell 7):**
```powershell
pwsh ./scripts/install.ps1
```

## Development

```bash
npm install
npm run tauri:dev
```

Run tests:
```bash
npm run build
cd src-tauri && cargo test
```

## License

MIT — see [LICENSE](./LICENSE).
