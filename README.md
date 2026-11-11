# PromptKey

A small menu-bar / system-tray app that runs your AI prompts on selected text. Built with Tauri 2, Rust, Svelte 5 and TypeScript for macOS and Windows. Bring your own OpenRouter key.

Select text in an editable field, press your shortcut, and continue writing. PromptKey copies the selection, applies your instructions through OpenRouter and pastes the result back. If focus or the selection changes while the request is running, the result opens for review instead.

## First Run

1. Open PromptKey and go to **Settings**.
2. Enter your [OpenRouter API key](https://openrouter.ai/settings/keys). It is stored in macOS Keychain or Windows Credential Manager, separately from settings and exports.
3. On macOS, allow PromptKey under **System Settings → Privacy & Security → Accessibility**. Use **Check Access** in PromptKey to request/check permission. You may need to restart the app after granting it.
4. Choose a default model. The starter model is `google/gemini-2.5-flash-lite`; any compatible OpenRouter text model ID can be entered. A prompt can override the default.
5. Assign shortcuts and **Save Changes**. An unavailable shortcut is shown with an error; release it in another app or choose a different one.
6. Select a short piece of text in an editor and use a prompt shortcut. Start with **Show Result for Review** if you want to inspect responses before inserting them.

Closing the settings window keeps PromptKey running in the menu bar / system tray. Use its tray menu to reopen settings or quit. Launch at login is optional and disabled initially.

## Starter Prompts

| Prompt | Default Shortcut | What It Does |
| --- | --- | --- |
| Fix Writing | Hyper + E | Correct spelling, punctuation and grammar; keep the original language and tone |
| Translate to English | Hyper + T | Translate to natural US English |
| Make It Shorter | Hyper + K | Shorten while preserving meaning and language |

**Hyper is a combination, not a special key:** Control + Option + Shift + Command on macOS; Control + Alt + Shift + Windows on Windows. PromptKey recognizes the combination but does not remap Caps Lock itself. Use an existing remapper such as Hyperkey/Karabiner on macOS or a Windows remapping tool that can emit all four modifiers. Ordinary shortcuts such as Ctrl + Alt + E also work. Some combinations are reserved by the OS or other apps.

If Hyper + E currently launches a Raycast command, remove that assignment in Raycast before using it in PromptKey. Hammerspoon and Raycast are not required to run PromptKey.

## Privacy and Behavior

- Only text explicitly selected for a shortcut, or entered in **Test Prompt**, is sent to OpenRouter and its model provider. Their data policies apply. No request is sent just by opening or saving a prompt.
- OpenRouter usage is billed to your own account. PromptKey does not include a subscription, proxy service or shared key.
- The API key is accessible only to the Rust backend except while you enter it in the settings field. It is never returned to the interface, written to settings, exported or logged.
- Settings are local JSON at the OS application-config location for `com.ronnyschlidt.promptkey`. Exports contain prompts, shortcuts, model IDs and preferences, never the API key. Imports preserve your own launch-at-login preference.
- Only one shortcut operation runs at a time. Requests time out after 45 seconds. Empty, truncated and failed responses do not replace text.
- The target app/window/control is checked during the request. Before pasting, PromptKey copies the selection again and compares it with the original. It never intentionally restores focus to another app to force a paste.
- Plain text, HTML, RTF, common images and file lists are backed up and restored around copy/paste. A clipboard change made during an API request is preserved. Application-private formats and arbitrary multi-item clipboards are not guaranteed to round-trip; use plain text when exact clipboard preservation matters.
- Inserted output is plain text. Rich formatting in the selected text may change. Use the destination app's Undo command to revert.
- Results remain in memory for the current session only and are not saved as a history. **Copy Result** deliberately replaces the clipboard.

## Platform Limits

PromptKey depends on the destination app supporting normal copy/paste. Secure/password fields, elevated Windows apps, remote desktops, terminal selections and applications with unusual clipboard handling may not work. On Windows, run the destination app at the same privilege level as PromptKey. Do not use this on a selection where a failed paste would be hard to undo.

The initial Windows implementation identifies the foreground window, focused control and caret position. macOS additionally checks the accessibility selection range when the destination exposes one. Identical text in controls that do not expose distinct accessibility identities is a remaining edge case; use review mode in those applications. There is no universal OS transaction that makes selection checking and a simulated paste atomic.

macOS development bundles use a local ad-hoc signature; Windows installers are unsigned. Downloaded builds may show macOS Gatekeeper or Windows SmartScreen prompts. Distribution signing/notarization is not configured. Build the source yourself or use artifacts from a trusted run in this private repository.

## Development

Install [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/): a recent Rust stable toolchain, Node.js 24+, and the OS build tools. macOS needs Xcode Command Line Tools. Windows needs Visual Studio C++ Build Tools and WebView2.

```sh
npm ci
npm run tauri dev
```

For interface-only work, `npm run dev` opens a browser preview. The preview does not register shortcuts, accept API keys, call OpenRouter or persist settings.

```sh
npm run check
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
```

Build a distributable on the target OS:

```sh
# macOS: creates src-tauri/target/release/bundle/macos/PromptKey.app
npm run tauri build -- --bundles app
codesign --force --deep --sign - src-tauri/target/release/bundle/macos/PromptKey.app
codesign --verify --deep --strict src-tauri/target/release/bundle/macos/PromptKey.app

# Windows: creates an installer under src-tauri/target/release/bundle/nsis/
npm run tauri build -- --bundles nsis
```

GitHub Actions runs the frontend checks and Rust tests on macOS and Windows, then builds downloadable artifacts. A successful Windows build does not replace interactive Windows testing; see [manual checks](docs/testing.md).

## Architecture

The Rust process owns shortcuts, credentials, network calls and text replacement. The settings WebView only calls specific local commands. OpenRouter's URL is fixed; imports cannot redirect credentials elsewhere. The frontend has no filesystem, shell, HTTP or global-shortcut plugin permissions. The native plugins are called from Rust.

The settings window is created once and hidden when closed; it remains a WebView process while hidden. Actual memory use depends on the OS/WebView and has not been benchmarked against other apps.

See [architecture decisions](docs/architecture.md) and [project instructions](AGENTS.md).

## License

MIT. Repository access is private; each collaborator uses their own API key.
