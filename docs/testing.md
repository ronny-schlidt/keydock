# Validation

Automated checks cover shortcut capture, semantic duplicate detection, config validation, API result validation and the final selection-comparison rule. CI compiles/tests native code and builds installers on both target OSes. These checks cannot establish that another app accepted an injected paste.

## Manual Desktop Checks

Use synthetic text and a disposable editor document. Never send a customer's or another person's text as test data without authorization.

1. Start the app without a key. Confirm that Settings can save a key and never displays it again. Export settings and check the key is absent.
2. Assign an unused combination to Fix Writing, save, select misspelled text in TextEdit/Notepad and trigger it. Confirm only the selection changes and Undo works.
3. Confirm the original clipboard's text/HTML/RTF or supported image is restored. Copy another item during the API request and confirm it is not overwritten when the result arrives.
4. Move the caret, select different text, click another field, or switch apps while the request is running. Confirm the result opens for review and no destination text changes.
5. Trigger with no selection, in a password field, with no network, an invalid key and an unsupported model. Confirm no clipboard contents are accidentally sent as the source and no empty/partial result replaces text.
6. Record a shortcut already assigned to another prompt. Confirm save rejects it. Release an external shortcut conflict and save again.
7. Close the window while recording a shortcut. Confirm the saved shortcuts are restored. Reopen from the tray, quit from the tray and confirm the process exits.
8. Export and import a collection on the other OS. Confirm API keys are independent and platform shortcut labels match.
9. Enable launch at login, restart the session and check the app starts quietly. Disable it again.

## Current Evidence

Record completed checks here when they are actually performed. The initial implementation does not imply interactive Windows validation; Windows CI and real keyboard testing are different evidence.

2026-11-11, macOS development machine:

- Svelte/TypeScript check: zero errors and warnings.
- Frontend tests: three passed; production frontend build passed.
- Rust tests: seven passed; Clippy with warnings denied passed.
- Browser preview: visually checked prompt editor and settings; created/renamed a prompt, recorded a shortcut and saved the preview state. This is UI evidence, not native copy/paste or API evidence.
- No real OpenRouter call has been made during development; the user enters their own key in the installed app.

Dependency audit on the same date: after updating Vite, its Svelte plugin and Vitest, npm reports only the upstream low-severity `deepmerge` development dependency advisory [GHSA-gcqx-w48m-w2h4](https://github.com/advisories/GHSA-gcqx-w48m-w2h4), propagated through the Svelte Vite plugin. There is no upstream fix available. This dependency is used to merge trusted build configuration and is not shipped in the application runtime. Recheck when updating the build toolchain; do not apply an unreviewed forced override.
