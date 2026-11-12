#!/usr/bin/env bash
# Builds KeyDock.app with the fast `local` Cargo profile, signs it, replaces /Applications/KeyDock.app and starts it.
# KEYDOCK_SIGN_IDENTITY selects a signing identity. Without it, a local "KeyDock Local Signing"
# certificate is used when present, so the Accessibility approval survives rebuilds; otherwise "-" (ad-hoc).
set -euo pipefail
cd "$(dirname "$0")/.."

[[ "$(uname)" == Darwin ]] || { echo "This script only runs on macOS." >&2; exit 1; }

bundle=src-tauri/target/local/bundle/macos/KeyDock.app
target=/Applications/KeyDock.app
binary="$target/Contents/MacOS/keydock"
local_identity="$(security find-identity -p codesigning | awk '/"KeyDock Local Signing"/ {print $2; exit}')"
identity="${KEYDOCK_SIGN_IDENTITY:-${local_identity:--}}"
echo "Signing with ${identity}."

npm run tauri build -- --bundles app -- --profile local
codesign --force --deep --sign "$identity" "$bundle"
codesign --verify --deep --strict "$bundle"

# Quit only the installed copy, not a `tauri dev` process.
if pgrep -f "^$binary" >/dev/null; then
  osascript -e 'quit app "KeyDock"' >/dev/null 2>&1 || true
  for _ in {1..20}; do
    pgrep -f "^$binary" >/dev/null || break
    sleep 0.25
  done
  pkill -f "^$binary" || true
fi

rm -rf "$target"
ditto "$bundle" "$target"
open "$target"
echo "Installed and started $target."
