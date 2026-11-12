# Validation

Automated checks cover shortcut capture, semantic duplicate detection, config validation, API result validation and the final selection-comparison rule. CI compiles/tests native code and builds installers on both target OSes. These checks cannot establish that another app accepted an injected paste.

## Current scope

Ronny, 2026-11-12: focus on macOS and skip Windows builds, automated tests and interactive checks until explicitly requested. Preserve the Windows implementation and existing CI matrix; this pause does not disable configured jobs when GitHub Actions runs. Do not trigger CI just to obtain Windows evidence during this scope. Report Windows validation as skipped. Historical Windows evidence below remains historical.

## Automated checks

For runtime code, tests, dependency or build/configuration changes, run the required suite on the current macOS host (`npm run verify` runs the same commands via `scripts/check.sh`; keep both in sync):

```sh
npm run check
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

The tests use local fixtures; no real OpenRouter key or desktop selection is needed. `--all-targets` checks Rust target kinds for the host, not every OS. Keep both lockfiles tracked; update dependencies intentionally rather than rerunning installs for every edit.

For **documentation/instruction/skill-only changes**, verify affected local Markdown links, skill frontmatter and shared link targets, then run `git diff --check`. Review untracked files as well. Skip the application suite unless the change also affects executable/build behavior.

For **UI changes**, also inspect the affected screen using `npm run dev` (loopback port 1420). The preview uses in-memory state and provides no evidence about native shortcuts, credentials, network requests or insertion.

For **native interaction changes**, perform applicable macOS manual scenarios below. Build/check success cannot replace desktop evidence. Reuse successful checks only while their inputs remain unchanged; after fixing a failure, rerun affected checks. Record failures/blockers and skipped checks honestly.

For **packaging changes or a requested distributable**, use the target-OS build/sign/verify commands in the [README](../README.md#development). On macOS, `npm run mac:install` (`scripts/install-macos.sh`) runs them and also replaces and restarts `/Applications/KeyDock.app`; use it when an installed-app check is needed, never alongside another Cargo build. It builds with the Cargo profile `local` (no LTO, 16 codegen units, incremental) into `src-tauri/target/local/`: about 5 s after a code change and about 1 min for the first build, versus about 1:20 min with the LTO `release` profile (2026-11-12; `npm run verify` about 12 s). Run it with a 10-minute timeout. Do not package every docs/UI task by default.

## Manual Desktop Checks

Use synthetic text and a disposable editor document. Never send a customer's or another person's text as test data without authorization.

1. Start the app without a key. Confirm that Settings can save a key and never displays it again. Export settings and check the key is absent.
2. Assign an unused combination to Fix Writing, save, select misspelled text in TextEdit/Notepad and trigger it. Confirm the toast at the bottom of the main screen shows a spinner, then the result, without taking focus (the text is replaced, not opened for review); trigger an error (e.g. no selection) and confirm it appears in the toast and settings stay closed. Confirm the menu-bar icon pulses and briefly becomes a checkmark, in light and dark appearance. Confirm only the selection changes and Undo works.
3. Confirm the original clipboard's text/HTML/RTF or supported image is restored. Copy another item during the API request and confirm it is not overwritten when the result arrives.
4. Move the caret, select different text, click another field, or switch apps while the request is running. Confirm the result opens for review and no destination text changes.
5. Trigger with no selection, in a password field, with no network, an invalid key and an unsupported model. Confirm no clipboard contents are accidentally sent as the source and no empty/partial result replaces text.
6. Record a shortcut already assigned to another prompt. Confirm save rejects it. Release an external shortcut conflict and save again.
7. Close the window while recording a shortcut. Confirm the saved shortcuts are restored and, on macOS, the Dock icon disappears while the menu-bar icon stays. Reopen from the tray (Dock icon returns), quit from the tray and confirm the process exits.
8. When cross-platform validation resumes: export and import a collection on the other OS. Confirm API keys are independent and platform shortcut labels match. This scenario is currently deferred.
9. Enable launch at login, restart the session and check the app starts quietly. Disable it again.

## Current Evidence

Record completed checks here when they are actually performed. The initial implementation does not imply interactive Windows validation; Windows CI and real keyboard testing are different evidence.

2026-11-12, menu-bar-only lifecycle (macOS development machine):

- Required suite passed (Rust: eight tests, Clippy; frontend: eleven tests, check, build). Release bundle built, ad-hoc signed, strict verification passed, installed to `/Applications`.
- `lsappinfo` showed `ApplicationType` `Foreground` (Dock icon) with the window open, `UIElement` (no Dock icon) after a `--background` launch, and `Foreground` again after a second launch reopened the window through the single-instance handler.
- Not yet observed natively: clicking the window's close button (the agent shell lacks Accessibility permission for scripted clicks); it calls the same accessory-policy switch as the background launch.

2026-11-12, shortcut recorder feedback (macOS development machine):

- Frontend tests: eleven passed (recorder rejects ⌘-only and ⇧-only combinations and explains ignored key presses); Svelte/TypeScript check and production build passed.
- Browser preview: synthetic key events showed the Caps Lock hint, the missing-modifier hint, live held modifiers and a recorded ⌃⌥⇧⌘W. Not yet checked in the native app with a real Hyper key.

2026-11-12, macOS development machine (light/dark appearance):

- Svelte/TypeScript check: zero errors and warnings. Frontend tests: eight passed (including theme preference parsing/resolution); production build passed. Rust tests: seven passed; Clippy with warnings denied passed (also validates the added `core:window:allow-set-theme` capability).
- Browser preview: visually checked prompt editor and Settings in Dark and Light; the choice survived a reload and System followed the OS appearance. Not yet checked in the native app: title-bar theme via `setTheme` and switching back to System while the window is forced to the other appearance.

2026-11-11, macOS development machine:

- Svelte/TypeScript check: zero errors and warnings.
- Frontend tests: six passed, including rejecting duplicate IDs and malformed records during import; production frontend build passed.
- Rust tests: seven passed; Clippy with warnings denied passed.
- Browser preview: visually checked prompt editor and settings; created/renamed a prompt, recorded a shortcut and saved the preview state. This is UI evidence, not native copy/paste or API evidence.
- No real OpenRouter call has been made during development; the user enters their own key in the installed app.
- Native macOS release bundle built successfully (about 6.1 MiB), locally signed, verified with `codesign --verify --deep --strict`, launched, and visually checked in the prompt editor and Settings. The initial key field is empty and the Accessibility requirement is correctly shown.
- GitHub Actions [run 42990135686](https://github.com/ronny-schlidt/keydock/actions/runs/42990135686) passed frontend checks, Rust tests/Clippy and release packaging on macOS and Windows. Windows installer creation is confirmed; interactive Windows copy/paste remains untested.

Packaging note (Tauri CLI 2.x, macOS, verified 2026-11-11): the generated `.app` without explicit bundle signing failed strict signature verification with “code has no resources but signature indicates they must be present.” Signing the finished local bundle with `codesign --force --deep --sign -` and verifying it fixed startup integrity. The workflow performs these steps before archiving. This is an ad-hoc development signature, not Developer ID signing or notarization.

Dependency audit on the same date: after updating Vite, its Svelte plugin and Vitest, npm reports only the upstream low-severity `deepmerge` development dependency advisory [GHSA-gcqx-w48m-w2h4](https://github.com/advisories/GHSA-gcqx-w48m-w2h4), propagated through the Svelte Vite plugin. There is no upstream fix available. This dependency is used to merge trusted build configuration and is not shipped in the application runtime. Recheck when updating the build toolchain; do not apply an unreviewed forced override.

## KeyDock Rename Validation

2026-11-12: the existing six frontend tests and seven Rust tests passed after renaming the app and packages. Svelte/TypeScript reported zero errors and warnings, the frontend production build passed, and Rust formatting/Clippy passed. The KeyDock title, K icon and sidebar branding were checked in the local browser preview. The app identifier, credential service and prompt schema were unchanged. This rename does not add evidence of interactive Windows testing.

## Shared Assistant Setup Validation

2026-11-12, documentation/instructions/skills only: checked 11 Markdown files and 63 local links including section fragments, the `CLAUDE.md` import, both relative Claude skill links and both skill manifests. Both skills passed the local skill-creator format validator; `git diff --check` passed. The shared instruction file has 39 lines. No application code, dependencies or CI configuration changed. Application/native tests were not rerun; Windows validation was skipped as requested. Skill discovery in a new Claude/Codex session remains to be observed; filesystem checks establish the shared files and valid targets, not live assistant loading.

2026-11-12, macOS progress toast:

- Required suite passed (Rust: eight tests, Clippy; frontend: eleven tests, check, build). Release bundle built, signed and installed.
- Browser preview via headless Chrome: working, success and error states checked in light and dark appearance.
- Not yet observed natively: the toast over other apps, that it does not take focus, and fullscreen spaces (the agent shell has no Accessibility or Screen Recording permission).
