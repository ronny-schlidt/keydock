# Desktop Architecture

Decision date: 2026-11-11.

KeyDock needs a small settings UI, global per-prompt shortcuts, selected-text transformation through OpenRouter, and native integration on macOS and Windows. A Hammerspoon script would cover macOS only, so the implementation is a standalone desktop application.

## Decisions

- **Tauri 2 + Rust:** share the application and network logic while isolating platform-specific focus checks. Use the system WebView for settings. Do not promise a specific memory budget without measuring actual builds.
- **Svelte 5 + TypeScript:** a compact frontend for a prompt list, editor, shortcuts and settings. This is a UI preference, not a requirement for native performance. React would be a reasonable alternative for a team that already uses it.
- **Tray lifecycle:** closing the window hides it; on macOS the app switches to the accessory activation policy, so it has no Dock icon while hidden and shows one (with its app menu and Cmd+Tab entry) while settings are open; quitting from the tray terminates the app. The backend continues handling shortcuts while settings are hidden. On macOS the tray icon pulses (opacity 100–30 %) during a shortcut operation and is replaced by a checkmark for 1.5 s after a replacement; the menu-bar width never changes. A toast at the bottom of the main screen shows the prompt name with a spinner, then the result or the error (errors no longer open settings). It is a separate transparent window that cannot become key, ignores the mouse and is ordered in front with `orderFrontRegardless` instead of `show()` (which calls `makeKeyAndOrderFront`), so the focus/target check is not disturbed. Windows only updates the tooltip and still opens settings on setup errors. A single-instance plugin prevents duplicate shortcut registration.
- **Explicit configuration:** edit drafts and save to activate them. Global shortcuts are temporarily unregistered while recording a replacement shortcut, then restored from saved settings. Invalid/duplicate combinations are rejected; registration conflicts are shown rather than silently ignored.
- **Local JSON + OS credentials:** no database or hosted service. Keyring 3.6 is selected for its established native platform-store API; a future 4.x migration should evaluate the newer credential-store split deliberately.
- **Fixed OpenRouter API:** Rust sends a system instruction and selected source text to `https://openrouter.ai/api/v1/chat/completions`. No tools, agent loop or prompt-controlled URLs. The key check reads the fixed `/api/v1/key` and `/api/v1/models` endpoints and returns only credit/usage fields and whether the model is listed, never the key or its masked label. An API failure does not trigger a silent model fallback.
- **Conservative insertion:** wait for modifier release, read selection via copy, restore clipboard, call OpenRouter, watch focus/selection during the request, copy and compare the selection again, then paste. If a check fails, show the result for review. The original app is never force-focused.

## Application Identity

On 2026-11-12, PromptKey was renamed to KeyDock in the UI, packages, icons, project folder and GitHub repository. The application identifier and keyring service remain `com.ronnyschlidt.promptkey` to preserve existing local settings, credentials and installer identity. No application features or prompt schemas changed.

## Boundaries to Revisit

Native key remapping is intentionally left to existing tools. Global shortcuts recognize Hyper, but KeyDock does not install keyboard hooks to turn Caps Lock into Hyper. Built-in remapping would require its own configuration, accessibility/input-monitoring evaluation and platform testing.

Clipboard-rs covers common clipboard content, not arbitrary application-owned representations or multi-item transactions. Improve fidelity with dedicated platform snapshot implementations if users need exact rich clipboard restoration.

On Windows, foreground/control/caret identity and final copied-text comparison are the initial checks. UI Automation selection-range support would improve ambiguous same-text cases. On both OSes, a paste can fail even after its key events are sent; success presently means the paste shortcut was delivered, not that every destination acknowledged insertion. Keep review mode available.

Signing, notarization and trusted automatic updates are separate distribution work. Do not add an unsigned updater as a shortcut.

## References

- [Tauri architecture](https://v2.tauri.app/concept/architecture/)
- [Tauri global shortcuts](https://v2.tauri.app/plugin/global-shortcut/)
- [Tauri system tray](https://v2.tauri.app/learn/system-tray/)
- [OpenRouter chat completions](https://openrouter.ai/docs/api/api-reference/chat/create-a-chat-completion)
