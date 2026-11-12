type KeyInput = Pick<KeyboardEvent, 'key' | 'code' | 'ctrlKey' | 'altKey' | 'shiftKey' | 'metaKey'>;

const modifierKeys = ['Control', 'Alt', 'Shift', 'Meta', 'CapsLock', 'AltGraph', 'Dead'];

export function heldModifiers(event: Omit<KeyInput, 'key' | 'code'>): string {
  return [event.ctrlKey && 'Ctrl', event.altKey && 'Alt', event.shiftKey && 'Shift', event.metaKey && 'Super'].filter(Boolean).join('+');
}

function keyName(code: string): string | null {
  const key = /^Key[A-Z]$/.test(code) ? code.slice(3) : /^Digit[0-9]$/.test(code) ? code.slice(5) : code;
  return /^(?:[A-Z0-9]|F(?:[1-9]|1[0-9]|2[0-4])|Space|Enter|Tab|Backspace|Arrow(?:Up|Down|Left|Right))$/.test(key) ? key : null;
}

// Shift alone would swallow typing; Command/Win alone collides with every app's menu shortcuts.
function weakModifiers(modifiers: string): boolean {
  return modifiers === 'Shift' || modifiers === 'Super';
}

export function recordShortcut(event: KeyInput): string | null {
  if (modifierKeys.includes(event.key)) return null;
  const modifiers = heldModifiers(event);
  if (!modifiers || weakModifiers(modifiers)) return null;
  const key = keyName(event.code);
  return key ? `${modifiers}+${key}` : null;
}

export function recordingHint(event: KeyInput, mac: boolean): string {
  if (event.key === 'CapsLock') return 'Caps Lock arrived as Caps Lock, so no Hyper key is active. Start your Hyper key tool or hold the modifiers yourself.';
  if (modifierKeys.includes(event.key)) return '';
  const modifiers = heldModifiers(event);
  if (!modifiers) return `Hold ${mac ? '⌃, ⌥ or ⌘' : 'Ctrl or Alt'} together with the key.`;
  if (weakModifiers(modifiers)) return `${prettyShortcut(modifiers, mac).join('')} alone is taken by other apps. Add ${mac ? '⌃ or ⌥' : 'Ctrl or Alt'}.`;
  if (!keyName(event.code)) return 'Use a letter, digit, F-key, Space, Enter, Tab, Backspace or arrow key.';
  return '';
}

export function prettyShortcut(shortcut: string, mac: boolean): string[] {
  if (!shortcut) return ['No shortcut'];
  return shortcut.split('+').map(part => ({
    Ctrl: mac ? '⌃' : 'Ctrl', Alt: mac ? '⌥' : 'Alt', Shift: mac ? '⇧' : 'Shift',
    Super: mac ? '⌘' : 'Win', Meta: mac ? '⌘' : 'Win',
  }[part] ?? part));
}
