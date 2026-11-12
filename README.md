# KeyDock

A small menu-bar / system-tray app that runs your AI prompts on selected text. Built with Tauri 2, Rust, Svelte 5 and TypeScript for macOS and Windows. Bring your own OpenRouter key.

Select text in an editable field, press your shortcut, and continue writing. KeyDock copies the selection, applies your instructions through OpenRouter and pastes the result back. If focus or the selection changes while the request is running, the result opens for review instead.

## Install

On a Mac with Apple Silicon, download `KeyDock-macos.zip` from the [latest release](https://github.com/ronny-schlidt/keydock/releases/latest), unzip it and move `KeyDock.app` to **Applications**. The app is not notarized, so macOS blocks the first start: open **System Settings → Privacy & Security** and click **Open Anyway**. On other platforms, [build from source](#development).

## First Run

1. Open KeyDock and go to **Settings**.
2. Enter your [OpenRouter API key](https://openrouter.ai/settings/keys). It is stored in macOS Keychain or Windows Credential Manager, separately from settings and exports.
3. On macOS, allow KeyDock under **System Settings → Privacy & Security → Accessibility**. Use **Check Access** in KeyDock to request/check permission. You may need to restart the app after granting it.
4. Choose a default model from the suggestions (fast, low-cost models with their OpenRouter prices) or enter any compatible OpenRouter text model ID. The starter model is `google/gemini-2.5-flash-lite`. A prompt can override the default. **Check Key** confirms the stored key works, shows its remaining credit limit and whether OpenRouter lists the default model.
5. Assign shortcuts and **Save Changes**. An unavailable shortcut is shown with an error; release it in another app or choose a different one.
6. Select a short piece of text in an editor and use a prompt shortcut. Start with **Show Result for Review** if you want to inspect responses before inserting them.

Closing the settings window keeps KeyDock running in the menu bar / system tray. On macOS the Dock icon appears only while the settings window is open, and while a prompt runs a small toast at the bottom of the main screen shows its progress, then the result or the error; errors no longer open the window. The menu-bar icon also pulses and briefly shows a checkmark. Use its tray menu to reopen settings or quit. Launch at login is optional and disabled initially.

## Renaming from PromptKey

The app and repository are now named KeyDock. The application identifier and OS credential namespace intentionally remain `com.ronnyschlidt.promptkey`, so existing settings and saved API keys continue to use the same locations. Existing prompt exports still import normally. This rename adds no features.

## Starter Prompts

| Prompt | Default Shortcut | What It Does |
| --- | --- | --- |
| Fix Writing | Hyper + W | Correct spelling, punctuation and grammar; keep the original language and tone |
| Translate to English | Hyper + T | Translate to natural US English |
| Make It Shorter | Hyper + K | Shorten while preserving meaning and language |

**Hyper is a combination, not a special key:** Control + Option + Shift + Command on macOS; Control + Alt + Shift + Windows on Windows. KeyDock recognizes the combination but does not remap Caps Lock itself. Use an existing remapper such as Hyperkey/Karabiner on macOS or a Windows remapping tool that can emit all four modifiers. Ordinary shortcuts such as Ctrl + Alt + E also work. Some combinations are reserved by the OS or other apps.

If Hyper + W currently launches a Raycast command, remove that assignment in Raycast before using it in KeyDock. Hammerspoon and Raycast are not required to run KeyDock.

## Privacy and Behavior

- Only text explicitly selected for a shortcut, or entered in **Test Prompt**, is sent to OpenRouter and its model provider. Their data policies apply. No request is sent just by opening or saving a prompt. **Check Key** sends only the stored key to OpenRouter's key endpoint and fetches the public model list; no text is sent.
- OpenRouter usage is billed to your own account. KeyDock does not include a subscription, proxy service or shared key.
- The API key is accessible only to the Rust backend except while you enter it in the settings field. It is never returned to the interface, written to settings, exported or logged.
- Settings are local JSON at the OS application-config location for `com.ronnyschlidt.promptkey`. Exports contain prompts, shortcuts, model IDs and preferences, never the API key. Imports preserve your own launch-at-login preference. The appearance (System, Light or Dark under **Settings → Appearance**) is a per-device choice kept in the settings window's WebView storage, not in `config.json` or exports.
- Only one shortcut operation runs at a time. Requests time out after 45 seconds. Empty, truncated and failed responses do not replace text.
- The target app/window/control is checked during the request. Before pasting, KeyDock copies the selection again and compares it with the original. It never intentionally restores focus to another app to force a paste.
- Plain text, HTML, RTF, common images and file lists are backed up and restored around copy/paste. A clipboard change made during an API request is preserved. Application-private formats and arbitrary multi-item clipboards are not guaranteed to round-trip; use plain text when exact clipboard preservation matters.
- Inserted output is plain text. Rich formatting in the selected text may change. Use the destination app's Undo command to revert.
- Results remain in memory for the current session only and are not saved as a history. **Copy Result** deliberately replaces the clipboard.

## Platform Limits

KeyDock depends on the destination app supporting normal copy/paste. Secure/password fields, elevated Windows apps, remote desktops, terminal selections and applications with unusual clipboard handling may not work. On Windows, run the destination app at the same privilege level as KeyDock. Do not use this on a selection where a failed paste would be hard to undo.

The initial Windows implementation identifies the foreground window, focused control and caret position. macOS additionally checks the accessibility selection range when the destination exposes one. Identical text in controls that do not expose distinct accessibility identities is a remaining edge case; use review mode in those applications. There is no universal OS transaction that makes selection checking and a simulated paste atomic.

macOS development bundles use a local ad-hoc signature; `npm run mac:install` signs with a local "KeyDock Local Signing" certificate from the login keychain when one exists, so the Accessibility approval is not lost on each rebuild. Windows installers are unsigned. Downloaded builds may show macOS Gatekeeper or Windows SmartScreen prompts. Distribution signing/notarization is not configured. Download builds only from this repository's releases, or build the source yourself.

## Development

Install [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/): a recent Rust stable toolchain, Node.js 24+, and the OS build tools. macOS needs Xcode Command Line Tools. Windows needs Visual Studio C++ Build Tools and WebView2.

```sh
npm ci
npm run tauri dev
```

For interface-only work, `npm run dev` opens a browser preview. The preview does not register shortcuts, accept API keys, call OpenRouter or persist settings.

See [validation commands and current scope](docs/testing.md#automated-checks) for the required checks. Current development focuses on macOS; Windows validation is deferred until requested. The existing two-platform CI workflow remains configured.

For work with Claude Code or ChatGPT/Codex, use the [shared project instructions](AGENTS.md), [assistant setup and skills](docs/ai-workflow.md), and [documentation index](docs/README.md).

Build a distributable on the target OS:

On macOS, `npm run mac:install` builds a local test copy with the fast Cargo profile `local` (no LTO, so it is about 18 % larger), signs it, replaces `/Applications/KeyDock.app`, and restarts it. Distributables use the commands below with the LTO `release` profile. `npm run verify` runs the full check suite.

```sh
# macOS: creates src-tauri/target/release/bundle/macos/KeyDock.app
npm run tauri build -- --bundles app
codesign --force --deep --sign - src-tauri/target/release/bundle/macos/KeyDock.app
codesign --verify --deep --strict src-tauri/target/release/bundle/macos/KeyDock.app

# Windows: creates an installer under src-tauri/target/release/bundle/nsis/
npm run tauri build -- --bundles nsis
```

GitHub Actions runs the frontend checks and Rust tests on macOS and Windows, then builds downloadable artifacts. Pushing a version tag such as `v0.1.0` builds the macOS app and publishes it as a GitHub release ([release workflow](.github/workflows/release.yml)); keep the version in `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` in sync with the tag. A successful Windows build does not replace interactive Windows testing; see [manual checks](docs/testing.md).

## Architecture

The Rust process owns shortcuts, credentials, network calls and text replacement. The settings WebView only calls specific local commands. OpenRouter's URL is fixed; imports cannot redirect credentials elsewhere. The frontend has no filesystem, shell, HTTP or global-shortcut plugin permissions. The native plugins are called from Rust.

The settings window is created once and hidden when closed; it remains a WebView process while hidden. Actual memory use depends on the OS/WebView and has not been benchmarked against other apps.

See [architecture decisions](docs/architecture.md), [repository map and analysis](docs/repository-map.md), and the [documentation index](docs/README.md).

## License

MIT. Each user brings their own OpenRouter API key.
