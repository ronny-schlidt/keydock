import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

export type ThemePreference = 'system' | 'light' | 'dark';

// Appearance is a per-device view preference, so it stays out of config.json and prompt exports.
const storageKey = 'keydock.theme';

export function parseTheme(value: string | null): ThemePreference {
  return value === 'light' || value === 'dark' ? value : 'system';
}

export function resolveTheme(preference: ThemePreference, systemDark: boolean): 'light' | 'dark' {
  return preference === 'system' ? (systemDark ? 'dark' : 'light') : preference;
}

export function loadTheme(): ThemePreference {
  try { return parseTheme(localStorage.getItem(storageKey)); } catch { return 'system'; }
}

let current: ThemePreference = 'system';
let media: MediaQueryList | undefined;

export function applyTheme(preference: ThemePreference) {
  current = preference;
  if (!media) {
    media = matchMedia('(prefers-color-scheme: dark)');
    // The native window theme also drives this query, so it changes after setTheme(null) and with the system.
    media.addEventListener('change', () => { if (current === 'system') applyTheme('system'); });
  }
  document.documentElement.dataset.theme = resolveTheme(preference, media.matches);
  // Keep the native title bar in step with the page.
  if (isTauri()) getCurrentWindow().setTheme(preference === 'system' ? null : preference).catch(() => {});
}

export function saveTheme(preference: ThemePreference) {
  try { localStorage.setItem(storageKey, preference); } catch { /* Unavailable storage keeps the choice for this session. */ }
  applyTheme(preference);
}
