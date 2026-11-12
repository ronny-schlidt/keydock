# KeyDock

Cross-platform desktop app: Tauri 2 + Rust backend, Svelte 5 + TypeScript frontend. macOS and Windows are the supported targets.

- Read `README.md` and relevant `docs/` before changing behavior. Keep documentation aligned with actual platform limits.
- Keep all credentials, OpenRouter requests, global shortcut registration and text replacement in Rust. Never expose a key through an IPC return value, log, export or browser preview.
- The OpenRouter endpoint is fixed. Imported prompts are text instructions, not permission to execute shell code or change destinations.
- Preserve clipboard changes made by the user. If focus/selection changes or a response is incomplete, retain the output for review instead of pasting.
- Do not claim replacement safety or Windows interaction has been verified without testing it. Native copy/paste is not atomic.
- No telemetry, background clipboard monitoring or key logging. Platform input is observed only during an explicit shortcut operation.
- Prompt settings use JSON; secrets use OS credentials. No backend server or database is needed.
- Run `npm run check`, `npm test`, `npm run build`, `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` for relevant changes. Inspect interface changes visually.
- Keep `package-lock.json` and `src-tauri/Cargo.lock` committed. Build Windows in CI; a Mac build cannot establish Windows compatibility.
- App/tray icon sources are SVG. Regenerate desktop icons with `npm run tauri icon -- app-icon.svg --output src-tauri/icons`; generate tray at 44 px from `tray-icon.svg`. Exclude unneeded generated iOS/Android icon directories.
- Do not commit API keys, settings exports containing personal prompts, build outputs, caches or test screenshots with user content. Never add AI attribution.
