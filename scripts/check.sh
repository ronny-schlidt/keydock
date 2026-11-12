#!/usr/bin/env bash
# Runs the required macOS check suite from docs/testing.md.
set -euo pipefail
cd "$(dirname "$0")/.."

manifest=src-tauri/Cargo.toml
npm run check
npm test
npm run build
cargo fmt --manifest-path "$manifest" --check
cargo test --locked --manifest-path "$manifest"
cargo clippy --locked --manifest-path "$manifest" --all-targets -- -D warnings
echo "All checks passed."
