# Repository map and analysis

Source inspection: 2026-11-12. This is a navigation and maintenance assessment, not proof of runtime safety. See [Testing](testing.md) for executed checks and [Architecture](architecture.md) for decisions.

## File ownership

| Area | Source | Responsibility |
| --- | --- | --- |
| Settings interface | [`src/App.svelte`](../src/App.svelte), [`src/style.css`](../src/style.css) | Prompt drafts, explicit save, key entry, shortcut recording, import/export, test input, status/results; colors are CSS variables with light and `data-theme=dark` sets |
| Appearance | [`src/lib/theme.ts`](../src/lib/theme.ts) | System/Light/Dark preference in `localStorage`, applied before mount; sets `data-theme` and the native window theme |
| Frontend contract | [`src/lib/types.ts`](../src/lib/types.ts) | `Prompt`, `Config`, `Bootstrap`, `Status` IPC shapes |
| Import normalization | [`src/lib/import.ts`](../src/lib/import.ts) | Select supported fields, reject malformed/duplicate records, retain this device's startup preference |
| Shortcut UI | [`src/lib/shortcuts.ts`](../src/lib/shortcuts.ts) | Physical key-code capture and OS-specific labels |
| Native orchestration | [`src-tauri/src/lib.rs`](../src-tauri/src/lib.rs) | IPC handlers, credentials, shared state, shortcut lifecycle, single operation guard, tray, autostart, request/focus coordination |
| Settings storage | [`src-tauri/src/config.rs`](../src-tauri/src/config.rs) | Rust schema/defaults/validation, local `config.json`, temporary-file replacement |
| Text interaction | [`src-tauri/src/desktop.rs`](../src-tauri/src/desktop.rs) | Clipboard guard, modifier release, native target identity, selection capture/comparison, simulated copy/paste; platform `native` modules |
| Model requests | [`src-tauri/src/openrouter.rs`](../src-tauri/src/openrouter.rs) | Fixed URL, model override, request body, HTTP errors, complete nonempty output validation; key check (credit fields, model listed) |
| Model suggestions | [`src/lib/models.ts`](../src/lib/models.ts) | Curated default/prompt model choices with dated OpenRouter list prices; update prices and IDs together |
| Tauri boundary | [`src-tauri/tauri.conf.json`](../src-tauri/tauri.conf.json), [`capabilities/default.json`](../src-tauri/capabilities/default.json) | App identity/window/CSP/bundling (`macOSPrivateApi` for the transparent toast); core WebView permissions plus `core:window:allow-set-theme` |
| Startup | [`src/main.ts`](../src/main.ts), [`src-tauri/src/main.rs`](../src-tauri/src/main.rs) | Mount the UI and start the Rust library |
| Progress toast (macOS) | [`toast.html`](../toast.html), [`src/toast.ts`](../src/toast.ts), [`capabilities/toast.json`](../src-tauri/capabilities/toast.json), `show_toast` in `lib.rs` | Click-through, non-focusable window at the bottom of the main screen; renders `status` events and hides itself |
| Toolchain | [`package.json`](../package.json), [`Cargo.toml`](../src-tauri/Cargo.toml), [`vite.config.ts`](../vite.config.ts) | Scripts/dependencies; Vite loopback port 1420 with strict port selection; pages `index.html` and `toast.html` |
| Local scripts | [`scripts/check.sh`](../scripts/check.sh), [`scripts/install-macos.sh`](../scripts/install-macos.sh) | `npm run verify` (check suite from testing.md); `npm run mac:install` (build, sign, replace and restart `/Applications/KeyDock.app`) |
| CI | [`.github/workflows/check.yml`](../.github/workflows/check.yml) | macOS/Windows check/build matrix, macOS ad-hoc signature, artifacts |
| Release | [`.github/workflows/release.yml`](../.github/workflows/release.yml), [`.github/release-notes.md`](../.github/release-notes.md) | On a `v*` tag: macOS build, ad-hoc signature, `KeyDock-macos.zip` attached to a GitHub release with install notes |
| Visual assets | [`app-icon.svg`](../app-icon.svg), [`tray-icon.svg`](../tray-icon.svg), [`tray-done.svg`](../tray-done.svg), `src-tauri/icons/` | Editable SVG sources and generated desktop icons |

## Request and state flow

1. `App.svelte` detects Tauri. Desktop startup calls `bootstrap` and listens to `status`; browser preview uses in-memory defaults.
2. Editing creates frontend drafts. `save_config` validates and saves in Rust, updates autostart when changed and registers saved shortcuts. Shortcut conflicts are returned to the UI.
3. A registered shortcut finds its saved prompt. `execute` permits one shortcut operation at a time. `capture_selection` checks permissions/target, waits for modifier release, copies selected text and restores supported clipboard contents.
4. `execute_inner` calls `openrouter::complete` and checks target identity every 100 ms while waiting. The HTTP client has a 45-second timeout and refuses redirects.
5. Review mode or an observed target change retains complete output for review. Otherwise `replace_selection` re-copies and compares the selection before preparing the paste; failure falls back to review.
6. Status is held in memory and emitted as `status`. Review opens the settings window. On macOS every run, result and error appears in the toast and errors never open the window; on Windows setup errors (permissions, key, provider, clipboard) open it; selection errors that only need another attempt (`CaptureError.retry`) play the system alert sound and update the tray tooltip instead. Closing the window hides it; the tray's quit command exits.

IPC entry points are registered together in `tauri::generate_handler!` in `lib.rs`: `bootstrap`, `save_config`, `set_api_key`, `delete_api_key`, `check_api_key`, `set_recording`, `request_accessibility`, `copy_result`, `test_prompt`. The last command uses the supplied draft prompt and sample text; it does not replace a desktop selection.

## Change coupling

- **Prompt/config fields or defaults:** inspect Rust `Config`/`Prompt`, TypeScript types, browser defaults in `App.svelte`, import field selection, exports and both validators/tests. Browser defaults are manually duplicated; updating only Rust can leave the preview misleading. Stored/exported schema is version 1; don't silently break existing files.
- **Shortcuts:** UI capture uses physical `KeyboardEvent.code`; native parsing and semantic duplicate detection live in Rust. Recording temporarily unregisters global shortcuts. Blur/window-close handling must restore saved registrations.
- **Insertion behavior:** `desktop.rs` checks and `lib.rs` fallback/status handling work together. Unit tests for `can_replace` cover a predicate, not timing, clipboard fidelity or another application's acceptance of paste.
- **Network behavior:** `openrouter.rs` builds/validates requests; `lib.rs` constructs the HTTP client and coordinates focus. Preserve both boundaries when changing limits/timeouts.
- **Brand rename:** product/package names are KeyDock; the legacy app identifier/keyring service are intentional compatibility state, not leftover branding.

## Assessment and current limits

The small codebase has a clear native trust boundary and no hosted backend/database. Pure frontend import/shortcut tests and Rust config/response/comparison tests offer fast feedback without real credentials. Keep these boundaries instead of adding infrastructure for assistant convenience.

Settings UI and native orchestration are concentrated in `App.svelte` and `lib.rs`. Extract components/modules when a concrete feature benefits; no broad restructuring is needed just for this documentation setup. The main maintenance risk is manually synchronized frontend/backend schema and defaults; changes here need paired review.

Clipboard preservation supports common formats, not arbitrary private representations or multi-item transactions. Focus comparison and simulated paste remain separate OS operations. Platform code includes unsafe native API boundaries; preserve platform-specific guards and verify changed native behavior on the actual OS.

There is no checked-in end-to-end automation proving real replacement, credential-store behavior or provider requests. Existing evidence records build/UI checks separately. Distribution signing/notarization and a trusted updater are unfinished, and memory/performance claims have not been benchmarked. Windows work is currently deferred by the user; the existing CI matrix remains configured. See [Architecture: boundaries to revisit](architecture.md#boundaries-to-revisit) and [Testing](testing.md).
