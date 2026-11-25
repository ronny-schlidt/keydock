## What's new in 0.1.1

- Shortcuts now work in Electron apps such as the Missive desktop app, which previously failed with "Cannot identify the active app."
- The default **Fix Writing** prompt only corrects actual errors and keeps correct informal words such as "Hi". Existing settings keep their prompt; to use the new one, paste this into **Fix Writing** in KeyDock:

  > Correct only actual errors in spelling, punctuation and grammar. Do not change correct words, even if informal, colloquial or borrowed from another language (e.g. "Hi", "okay", "cool"). Never rephrase, never replace words with synonyms and never change greetings or tone. Preserve the original language, meaning and paragraph structure. Return only the corrected text.

## Install on macOS (Apple Silicon)

1. Download `KeyDock-macos.zip` below, unzip it and move `KeyDock.app` to **Applications**.
2. Open KeyDock. macOS blocks the first start because the app is not notarized by Apple: open **System Settings → Privacy & Security** and click **Open Anyway**.
3. Allow KeyDock under **System Settings → Privacy & Security → Accessibility**.
4. In KeyDock **Settings**, enter your own [OpenRouter API key](https://openrouter.ai/settings/keys).

The build has an ad-hoc signature only. After installing an update, macOS may ask for the Accessibility permission again. Windows builds are not published yet.
