// macOS progress toast. Rust positions and shows the click-through window on every status;
// this page renders it and hides the window again once the run has finished.
import { isTauri } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { loadTheme, resolveTheme } from './lib/theme';
import type { Status } from './lib/types';

// Errors stay longer so they can be read.
const visibleFor: Record<Status['phase'], number> = { idle: 0, working: 0, success: 1600, review: 2600, error: 6000 };

const toast = document.querySelector<HTMLElement>('.toast')!;
const title = document.getElementById('title')!;
const message = document.getElementById('message')!;
let hideTimer: number | undefined;

function render(status: Status) {
  // Follow the appearance chosen in settings (same origin, so same storage).
  document.documentElement.dataset.theme = resolveTheme(loadTheme(), matchMedia('(prefers-color-scheme: dark)').matches);
  document.body.dataset.phase = status.phase;
  const working = status.phase === 'working';
  title.textContent = status.promptName ?? 'KeyDock';
  message.textContent = working ? 'Working…' : status.message;
  clearTimeout(hideTimer);
  requestAnimationFrame(() => toast.classList.add('shown'));
  if (!working) hideTimer = window.setTimeout(hide, visibleFor[status.phase]);
}

function hide() {
  toast.classList.remove('shown');
  hideTimer = window.setTimeout(() => {
    if (isTauri()) getCurrentWindow().hide().catch(() => {});
  }, 220);
}

if (isTauri()) {
  listen<Status>('status', event => render(event.payload)).catch(() => {});
} else {
  // Browser preview: /toast.html?phase=error&message=… shows one state without hiding it.
  const query = new URLSearchParams(location.search);
  const phase = (query.get('phase') ?? 'working') as Status['phase'];
  render({ phase, promptName: 'Fix Writing', message: query.get('message') ?? 'Selection updated' });
  clearTimeout(hideTimer);
}
