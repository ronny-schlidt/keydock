# Verified learnings

Read only entries relevant to the current task. Put topic-specific facts in their owning document; this file holds reusable findings that would otherwise need rediscovery. Update entries instead of appending duplicates. Maintenance rules: [AI workflow](ai-workflow.md#durable-learning-before-handoff).

## Avoid repeated repository discovery — 2026-11-12

**Observed:** native orchestration lives in `src-tauri/src/lib.rs`; OS clipboard/focus code is in `desktop.rs`. The settings interface is one `App.svelte`, with pure import/shortcut helpers. There were no repository skills before the shared assistant setup.

**Use:** begin with the [repository map](repository-map.md), then search the relevant symbol. Do not rescan every source file for each task. Exact token/time savings have not been measured.

## Hidden paths are easy to miss — 2026-11-12

**Observed:** plain `rg --files` omits hidden `.github`, `.agents` and `.claude` paths. These contain CI and shared skill entry points.

**Use:** pass explicit paths or use `rg --files --hidden` with generated/dependency directories excluded when inspecting assistant or CI setup. Do not include `.git` internals.

## Keep browser defaults paired with native defaults — 2026-11-12

**Observed:** `App.svelte` manually defines preview defaults; Rust `Config::default` independently defines desktop defaults. `parseImport` explicitly selects fields. TypeScript/Rust contracts are not generated from one schema.

**Use:** follow the [change-coupling checklist](repository-map.md#change-coupling) when changing prompt fields/defaults. This avoids a second debugging pass caused by inspecting only one side.

## CSS has very long lines — 2026-11-12

**Observed:** `src/style.css` contains the interface styling in only a few very long lines. A line-range read can return most of the stylesheet.

**Use:** search a specific selector and inspect a bounded excerpt when changing styling. Avoid repeatedly dumping the stylesheet or reformatting it as an unrelated change.

## "Hyper key doesn't record" usually means no Hyper key is active — 2026-11-12

**Observed:** the recorder only sees what macOS delivers. Without a running Hyper key tool (Hyperkey app, Raycast Hyper Key or a Karabiner `caps_lock` → ⌃⌥⇧⌘ rule), Caps Lock arrives as `CapsLock` and the letter arrives without modifiers. On Ronny's Mac, Karabiner only had a Caps Lock + S rule, Raycast had no Hyper Key setting and no Hyperkey process was running. Combinations grabbed by another app's global hotkey never reach the webview at all. A Carbon `RegisterEventHotKey` probe does not detect other processes' registrations, so it can't prove a combination is free.

**Use:** check for an active Hyper key mapping first (`pgrep -il hyperkey`, `~/.config/karabiner/karabiner.json`). The recorder now shows held modifiers and a hint for ignored keys; use that before debugging Rust shortcut registration.

## Keep one procedure per topic — 2026-11-12

**Observed:** README and agent instructions previously repeated the check commands; assistant-specific copies would create another place to drift.

**Use:** [Testing](testing.md) owns verification commands, [Architecture](architecture.md) owns native trade-offs and the two project skills route to those owners. `CLAUDE.md` imports shared rules and Claude's skill folders link to canonical skills. Update the target once; do not maintain separate Claude/Codex procedures.

For the verified macOS ad-hoc signature workaround and historical dependency-audit result, use [Testing: current evidence](testing.md#current-evidence). Recheck dependency advisories during dependency work; a historical audit is not a current result.

## Researching OpenRouter model prices and speed — 2026-11-12

**Observed:** the public `https://openrouter.ai/api/v1/models` list (no key needed, ~850 KB) gives current IDs, prices and context sizes. The per-model `/api/v1/models/{id}/endpoints` response has `throughput_last_30m` and `latency_last_30m` fields, but they were `null` for every model checked without a key. Speed figures from third-party benchmark sites varied widely between sources.

**Use:** take prices and IDs from the models list (`jq` over `.data[].pricing`, values are per token) when updating `src/lib/models.ts`; don't spend calls on the endpoints API for speed. Treat speed claims as rough and name the source.

## Native textarea resizing is not visible in the desktop window — 2026-11-12

**Observed:** `textarea{resize:vertical}` was set, but a macOS desktop screenshot showed no resize grabber on the instructions field, so it could not be resized. Chrome preview does show the native grabber, which hides the problem.

**Use:** give a resizable field an explicit handle (see `.resize-handle` in `App.svelte`) instead of relying on CSS `resize`. Native WebView behavior was inferred from the screenshot, not tested separately.

## Native lifecycle checks from the agent shell — 2026-11-12

**Observed:** a running `/Applications/KeyDock.app` makes a dev or release instance exit through the single-instance plugin (it only reopens the installed window). The agent shell has no Accessibility permission, so scripted clicks via System Events fail with `-1719`. `lsappinfo info -only ApplicationType -app com.ronnyschlidt.promptkey` reports `Foreground` (Dock icon) or `UIElement` (menu bar only) without UI scripting.

**Use:** quit or replace the installed app (with Ronny's consent) before native testing. Check Dock/activation state with `lsappinfo`, and use `--background` or a second launch to exercise hide/show paths instead of clicking.

## Accessibility permission after an ad-hoc rebuild — 2026-11-12

**Observed:** after `npm run mac:install`, KeyDock still appeared as enabled under Accessibility, but shortcuts failed with "Allow KeyDock in System Settings → Privacy & Security → Accessibility". Toggling the entry off and on did not help. The bundle is ad-hoc signed (`codesign -dv` shows `Signature=adhoc`), and macOS ties the approval to that signature, so each rebuild invalidates it. No codesigning identity is installed (`security find-identity -v -p codesigning` finds 0).

**Use:** `scripts/install-macos.sh` now signs with a local self-signed "KeyDock Local Signing" certificate from the login keychain when one exists. The designated requirement then pins the certificate (`certificate leaf = H"…"`) instead of the build hash, which should keep the approval across rebuilds; this has not yet been confirmed across a second rebuild. Signing and `codesign --verify --strict` work even though the certificate is untrusted (`CSSMERR_TP_NOT_TRUSTED`). This applies only to Ronny's Mac; public releases need Developer ID signing and notarization. With an ad-hoc build, run `tccutil reset Accessibility com.ronnyschlidt.promptkey`, restart KeyDock, use **Check Access** and enable it again.

## Render tray icons with resvg — 2026-11-12

**Observed:** ImageMagick's built-in SVG renderer (`magick x.svg`, no `rsvg-convert` installed) silently drops stroked paths: the checkmark rendered fully transparent and the arrow of `tray-icon.svg` was missing. resvg reproduces the committed `tray.png` pixel-identically. Tauri's `Image::from_bytes` expects RGBA PNGs.

**Use:** `uv run --with resvg-py python -I -c 'import sys,resvg_py; open(sys.argv[2],"wb").write(bytes(resvg_py.svg_to_bytes(svg_path=sys.argv[1],width=44,height=44)))' tray-done.svg src-tauri/icons/tray-done.png`, then check with `file` that the output is RGBA.

## macOS tray icon swaps lose the template flag — 2026-11-12

**Observed:** in tray-icon 0.25.1, `TrayIcon::set_icon` on macOS always passes `is_template = false`, so a swapped icon stays black on a dark menu bar.

**Use:** swap tray icons with `set_icon_with_as_template(Some(image), true)` (see `set_tray_image` in `lib.rs`).

## enigo key events must run on the macOS main thread — 2026-11-12

**Observed:** on macOS 26, every prompt shortcut crashed the app (`EXC_BREAKPOINT`, `_dispatch_assert_queue_fail` in `TSMGetInputSourceProperty` on a `tokio-rt-worker` thread). enigo 0.6.1 looks up the keyboard layout to map `Key::Unicode('c')`, and HIToolbox now asserts that this happens on the main queue. Build checks cannot catch it; read the crash report in `~/Library/Logs/DiagnosticReports/keydock-*.ips`.

**Use:** send enigo chords through `dispatch2::DispatchQueue::main().exec_sync` (see `chord` in `desktop.rs`). Never call it from the main thread itself, or `exec_sync` deadlocks; the selection capture and replacement run in `spawn_blocking`.

## Local installs use a fast Cargo profile — 2026-11-12

**Observed:** with `lto = true` and `codegen-units = 1`, every release rebuild took about 76 s even after a one-line change; 16 codegen units with LTO still took 65 s. Without LTO (`lto = "off"`, 16 units, incremental) a rebuild took 1.4 s and the binary grew from 6.4 MB to 7.6 MB. A full `npm run mac:install` now takes about 5 s after a change.

**Use:** `npm run mac:install` builds with `--profile local` (output in `src-tauri/target/local/`). Keep `[profile.release]` with LTO for distributables.
