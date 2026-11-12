<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, isTauri } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import type { Bootstrap, Config, Status } from './lib/types';
  import { prettyShortcut, recordShortcut } from './lib/shortcuts';
  import { parseImport } from './lib/import';

  const desktop = isTauri();
  const defaultConfig: Config = {
    version: 1, defaultModel: 'google/gemini-2.5-flash-lite', launchAtLogin: false,
    prompts: [
      { id: 'correct', name: 'Fix Writing', instructions: 'Correct spelling, punctuation and grammar. Preserve the original language, tone, meaning and paragraph structure. Make minimal changes. Return only the corrected text.', shortcut: 'Ctrl+Alt+Shift+Super+E', model: '', enabled: true, output: 'replace' },
      { id: 'translate', name: 'Translate to English', instructions: 'Translate the text into natural US English. Preserve meaning, tone and paragraph structure. Return only the translation.', shortcut: 'Ctrl+Alt+Shift+Super+T', model: '', enabled: true, output: 'replace' },
      { id: 'shorten', name: 'Make It Shorter', instructions: 'Make the text more concise without losing essential information. Preserve its language, meaning and tone. Return only the revised text.', shortcut: 'Ctrl+Alt+Shift+Super+K', model: '', enabled: true, output: 'replace' },
    ],
  };

  let config = $state<Config>(structuredClone(defaultConfig));
  let saved = $state(JSON.stringify(defaultConfig));
  let selectedId = $state('correct');
  let section = $state<'prompts' | 'settings'>('prompts');
  let hasKey = $state(false);
  let apiKey = $state('');
  let mac = $state(true);
  let accessibility = $state(true);
  let loading = $state(desktop);
  let saving = $state(false);
  let error = $state('');
  let notice = $state('');
  let shortcutErrors = $state<string[]>([]);
  let recording = $state(false);
  let deleteArmed = $state(false);
  let sample = $state('hallo, ich wolte fragen ob wir uns morgen trefen könen.');
  let sampleResult = $state('');
  let testing = $state(false);
  let status = $state<Status>({ phase: 'idle', message: desktop ? 'Ready when you are' : 'Browser preview · run the desktop app to use shortcuts' });
  const prompt = $derived(config.prompts.find(p => p.id === selectedId));
  const dirty = $derived(JSON.stringify(config) !== saved);
  const busy = $derived(status.phase === 'working');

  function applyBootstrap(data: Bootstrap) {
    config = data.config;
    saved = JSON.stringify(config);
    selectedId = config.prompts[0]?.id ?? '';
    hasKey = data.hasKey;
    mac = data.platform === 'macos';
    accessibility = data.accessibility;
    status = data.status;
    shortcutErrors = data.shortcutErrors;
  }

  onMount(() => {
    if (!desktop) return;
    let disposed = false;
    let unlisten: (() => void) | undefined;
    listen<Status>('status', event => { status = event.payload; }).then(fn => { if (disposed) fn(); else unlisten = fn; }).catch(e => { error = String(e); });
    invoke<Bootstrap>('bootstrap').then(applyBootstrap).catch(e => { error = String(e); }).finally(() => { loading = false; });
    return () => { disposed = true; unlisten?.(); };
  });

  async function save() {
    saving = true; error = ''; notice = '';
    try {
      if (desktop) shortcutErrors = await invoke<string[]>('save_config', { config });
      saved = JSON.stringify(config);
      notice = desktop ? (shortcutErrors.length ? 'Settings saved. Resolve the shortcut conflicts below.' : 'Changes saved. Your shortcuts are ready.') : 'Preview saved in memory only.';
    } catch (e) { error = String(e); }
    finally { saving = false; }
  }

  async function saveKey() {
    error = ''; notice = '';
    if (!desktop) { error = 'API keys are only entered in the installed desktop app.'; return; }
    try {
      await invoke('set_api_key', { key: apiKey });
      apiKey = ''; hasKey = true; notice = 'API key saved in your system credential store.';
    } catch (e) { error = String(e); }
  }

  async function removeKey() {
    try { await invoke('delete_api_key'); hasKey = false; notice = 'API key removed.'; }
    catch (e) { error = String(e); }
  }

  async function startRecording() {
    error = '';
    try {
      if (desktop) await invoke('set_recording', { active: true });
      recording = true;
    } catch (e) { error = String(e); }
  }

  async function stopRecording() {
    if (!recording) return;
    recording = false;
    try { if (desktop) shortcutErrors = await invoke<string[]>('set_recording', { active: false }); }
    catch (e) { error = String(e); }
  }

  function keydown(event: KeyboardEvent) {
    if (!recording) return;
    event.preventDefault(); event.stopPropagation();
    if (event.key === 'Escape') { void stopRecording(); return; }
    const shortcut = recordShortcut(event);
    if (shortcut && prompt) { prompt.shortcut = shortcut; void stopRecording(); }
  }

  function select(id: string) {
    void stopRecording(); selectedId = id; section = 'prompts'; deleteArmed = false; sampleResult = '';
  }

  function add() {
    const id = crypto.randomUUID();
    config.prompts.push({ id, name: 'New Prompt', instructions: 'Rewrite the selected text. Return only the revised text.', shortcut: '', model: '', enabled: true, output: 'replace' });
    select(id);
  }

  function removePrompt() {
    if (!deleteArmed) { deleteArmed = true; return; }
    config.prompts = config.prompts.filter(p => p.id !== selectedId);
    selectedId = config.prompts[0]?.id ?? ''; deleteArmed = false;
  }

  async function test() {
    if (!prompt) return;
    const testedId = prompt.id;
    testing = true; error = ''; sampleResult = '';
    try {
      const result = await invoke<string>('test_prompt', { prompt, text: sample });
      if (selectedId === testedId) sampleResult = result;
    }
    catch (e) { error = String(e); }
    finally { testing = false; }
  }

  function exportPrompts() {
    const blob = new Blob([JSON.stringify(config, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a'); link.href = url; link.download = 'keydock-prompts.json'; link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  async function importPrompts(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0]; if (!file) return;
    try {
      if (file.size > 4_000_000) throw new Error('Choose a settings file smaller than 4 MB.');
      config = parseImport(await file.text(), config.launchAtLogin);
      selectedId = config.prompts[0]?.id ?? '';
      notice = 'Imported. Review your prompts and save to activate them.';
    } catch (e) { error = String(e); }
    input.value = '';
  }

  async function checkPermission() {
    await invoke('request_accessibility');
    const data = await invoke<Bootstrap>('bootstrap'); accessibility = data.accessibility;
    if (!accessibility) notice = 'Enable KeyDock in Accessibility, then click Check Access again.';
  }

  async function copyResult() {
    try { await invoke('copy_result'); notice = 'Result copied.'; }
    catch (e) { error = String(e); }
  }
</script>

<svelte:window onkeydown={keydown} onblur={() => { void stopRecording(); }} />

<div class="app-shell">
  <aside class="sidebar">
    <div class="brand"><span class="brand-mark">K<span>↗</span></span><div>KeyDock<small>YOUR SHORTCUTS, ONE PLACE</small></div></div>
    <div class="sidebar-caption"><span>YOUR PROMPTS</span><span>{config.prompts.length.toString().padStart(2, '0')}</span></div>
    <nav aria-label="Prompts">
      {#each config.prompts as item, index (item.id)}
        <button class:active={section === 'prompts' && selectedId === item.id} class="prompt-link" onclick={() => select(item.id)}>
          <span class="prompt-icon">{['Aa', '文', '↙'][index % 3]}</span>
          <span class="prompt-link-name">{item.name}<small>{item.enabled ? (item.shortcut ? (item.output === 'replace' ? 'Replace selected text' : 'Review result') : 'Add a shortcut') : 'Paused'}</small></span>
          {#if item.shortcut}<kbd>{item.shortcut.split('+').at(-1)}</kbd>{/if}
        </button>
      {/each}
    </nav>
    <button class="add-prompt" onclick={add}><span>＋</span> New Prompt</button>
    <div class="sidebar-bottom">
      <button class:active={section === 'settings'} class="settings-link" onclick={() => { void stopRecording(); section = 'settings'; }}>⚙ <span>Settings</span><span class:connected={hasKey} class="connection-dot"></span></button>
      <p>Runs on your device.<br />Your own key. Your own prompts.</p>
      <span class="version">KEYDOCK <span>0.1.0</span></span>
    </div>
  </aside>

  <main>
    <header class="topbar"><span>{section === 'settings' ? 'Workspace / Settings' : 'Workspace / Prompts'}</span><div>{#if dirty}<span class="unsaved">Unsaved changes</span>{/if}<button class="primary small" disabled={loading || saving || busy || recording} onclick={save}>{saving ? 'Saving…' : 'Save Changes'} <span>↗</span></button></div></header>
    <div class="content">
      {#if loading}<p class="muted">Loading your workspace…</p>{:else}
        {#if error}<div class="banner error" role="alert">{error}<button aria-label="Dismiss error" onclick={() => error = ''}>×</button></div>{/if}
        {#if notice}<div class="banner notice" role="status">{notice}<button aria-label="Dismiss notice" onclick={() => notice = ''}>×</button></div>{/if}
        {#if shortcutErrors.length}<div class="banner error" role="alert">{#each shortcutErrors as warning}<p>{warning}</p>{/each}</div>{/if}
        {#if !desktop}<div class="banner notice">Browser preview. Global shortcuts and OpenRouter run in the desktop app.</div>{/if}

        {#if section === 'settings'}
          <div class="eyebrow">MAKE IT YOURS</div><h1>Settings</h1><p class="intro">Connect your model, bring your shortcuts, keep control.</p>
          <section class="card">
            <div class="card-heading"><h2>OpenRouter Connection</h2><span class:ready={hasKey} class="pill">{hasKey ? 'Key stored' : 'Not connected'}</span></div>
            <p class="muted">Your key is stored in {mac ? 'macOS Keychain' : 'Windows Credential Manager'}. It is never included in prompt exports.</p>
            <label for="api-key">API Key</label><div class="input-action"><input id="api-key" type="password" autocomplete="off" spellcheck="false" bind:value={apiKey} placeholder={hasKey ? 'Enter a new key to replace it' : 'sk-or-…'} disabled={!desktop} /><button class="primary" onclick={saveKey} disabled={!apiKey || !desktop}>Save Key</button></div>
            {#if hasKey}<button class="text-button danger" onclick={removeKey}>Remove stored key</button>{/if}
            <label for="default-model">Default Model</label><input id="default-model" bind:value={config.defaultModel} spellcheck="false" placeholder="provider/model-id" /><p class="hint">Use an OpenRouter model ID. Each prompt can override this.</p>
          </section>
          <section class="card"><h2>Shortcuts & Startup</h2>
            <p class="muted">Hyper means {mac ? 'Control + Option + Shift + Command' : 'Control + Alt + Shift + Windows'}. Map a key such as Caps Lock using your existing keyboard tool. Ordinary shortcuts work too.</p>
            <label class="toggle-row"><span>Launch KeyDock at login<small>Start quietly in the {mac ? 'menu bar' : 'system tray'}.</small></span><input type="checkbox" bind:checked={config.launchAtLogin} /></label>
            {#if mac}<div class="permission-row"><div><strong>Accessibility Access</strong><p class="hint">{accessibility ? 'Access granted.' : 'Required to read and replace text in other apps.'}</p></div><button class="secondary" onclick={checkPermission} disabled={!desktop}>Check Access</button></div>{/if}
          </section>
          <section class="card"><h2>Take Your Prompts With You</h2><p class="muted">Export your settings or import a shared collection. API keys stay on each person's device.</p><div class="button-row"><button class="secondary" onclick={exportPrompts}>Export Prompts ↓</button><label class="secondary file-label">Import Prompts ↑<input type="file" accept="application/json,.json" onchange={importPrompts} /></label></div></section>
        {:else if prompt}
          <div class="editor-heading"><div><div class="eyebrow">YOUR PROMPTS</div><h1>{prompt.name}</h1><p class="intro">Select some text. Press your shortcut. Keep going.</p></div><label class="enabled-toggle"><input type="checkbox" bind:checked={prompt.enabled} /><span>{prompt.enabled ? 'Enabled' : 'Paused'}</span></label></div>
          {#if !hasKey}<button class="setup-callout" onclick={() => section = 'settings'}><span class="callout-icon">↗</span><span><strong>One key. All your prompts.</strong><small>Add your OpenRouter API key to start using KeyDock.</small></span><span>Set Up →</span></button>{/if}
          <section class="card editor-card">
            <div class="section-number">01 <span>THE PROMPT</span></div>
            <label for="prompt-name">Name</label><input id="prompt-name" bind:value={prompt.name} maxlength="120" placeholder="Give this prompt a name" />
            <div class="label-line"><label for="instructions">Instructions</label><span>Applies to your selected text</span></div><textarea id="instructions" class="instructions" bind:value={prompt.instructions} spellcheck="false" placeholder="What should happen to the selected text?"></textarea>
            <div class="two-column"><div><label for="prompt-model">Model</label><input id="prompt-model" bind:value={prompt.model} placeholder="Use workspace default" spellcheck="false" /><p class="hint">{prompt.model || config.defaultModel}</p></div><div><label for="output">After the Response</label><select id="output" bind:value={prompt.output}><option value="replace">Replace Selection</option><option value="review">Show Result for Review</option></select><p class="hint">If the selection changes, the result opens for review.</p></div></div>
          </section>
          <section class="card shortcut-card"><div><div class="section-number">02 <span>THE SHORTCUT</span></div><h2>Your Shortcut</h2><p class="muted">Works while you're in another app.</p></div><div class="shortcut-controls"><button class:recording class="shortcut-recorder" aria-label="Record shortcut" onclick={() => recording ? stopRecording() : startRecording()} disabled={busy}>{#if recording}<span>Press your keys…</span>{:else}{#each prettyShortcut(prompt.shortcut, mac) as key}<kbd>{key}</kbd>{/each}{/if}</button><div class="recorder-actions">{#if recording}<button class="text-button" onclick={stopRecording}>Cancel · Esc</button>{:else}<span>Click to record</span>{#if prompt.shortcut}<button class="text-button" onclick={() => prompt.shortcut = ''}>Clear</button>{/if}{/if}</div></div></section>
          <section class="card"><div class="card-heading"><div><div class="section-number">03 <span>TRY IT OUT</span></div><h2>Test This Prompt</h2></div><button class="secondary" onclick={test} disabled={!desktop || !hasKey || testing || !sample.trim()}>{testing ? 'Working…' : 'Test Prompt ↗'}</button></div><label for="sample" class="sr-only">Sample text</label><textarea id="sample" class="sample" bind:value={sample}></textarea><p class="hint">Sends only this sample to OpenRouter. No other app is changed.</p>{#if sampleResult}<div class="test-result"><div class="eyebrow">RESULT</div><p>{sampleResult}</p></div>{/if}</section>
          <div class="editor-footer"><span>Changes take effect when you save.</span><button class="text-button danger" onclick={removePrompt} onblur={() => deleteArmed = false}>{deleteArmed ? 'Confirm Delete' : 'Delete Prompt'}</button></div>
        {:else}
          <div class="empty-state"><div class="eyebrow">YOUR NEXT SHORTCUT STARTS HERE</div><h1>One Less Repeated Task.</h1><p class="intro">Add a prompt for something you write every day.</p><button class="primary" onclick={add}>＋ Create Your First Prompt</button></div>
        {/if}

        {#if status.result}<section class="card latest-result"><div class="card-heading"><div><div class="eyebrow">LATEST RESULT · {status.promptName ?? 'PROMPT'}</div><h2>{status.phase === 'review' ? 'Ready for You to Use.' : 'Done.'}</h2></div><button class="secondary" onclick={copyResult} disabled={!desktop || busy}>Copy Result</button></div><p class="result-text">{status.result}</p></section>{/if}
      {/if}
    </div>
    <footer class="statusbar"><span class:working={busy} class:failed={status.phase === 'error'} class="status-dot"></span><span>{status.message}</span><span class="statusbar-end">{desktop ? (mac ? 'macOS' : 'Windows') : 'PREVIEW'} · OPENROUTER</span></footer>
  </main>
</div>
