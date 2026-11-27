#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

echo "Building Solace for macOS..."
npm ci
npm run tauri:build -- --bundles app

APP_BUNDLE="$ROOT_DIR/src-tauri/target/release/bundle/macos/Solace.app"
if [[ ! -d "$APP_BUNDLE" ]]; then
  echo "Error: Built app bundle not found at $APP_BUNDLE" >&2
  exit 1
fi

echo "Installing to /Applications/Solace.app..."
osascript -e 'tell application "Solace" to quit' 2>/dev/null || true
rm -rf "/Applications/Solace.app"
cp -R "$APP_BUNDLE" "/Applications/Solace.app"

echo "Launching Solace..."
open "/Applications/Solace.app"
echo "Done! Solace is now installed in /Applications/Solace.app and running in your menu bar."
