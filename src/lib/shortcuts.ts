export function recordShortcut(event: Pick<KeyboardEvent, 'key' | 'code' | 'ctrlKey' | 'altKey' | 'shiftKey' | 'metaKey'>): string | null {
  if (['Control', 'Alt', 'Shift', 'Meta', 'CapsLock', 'AltGraph', 'Dead'].includes(event.key)) return null;
  const modifiers = [event.ctrlKey && 'Ctrl', event.altKey && 'Alt', event.shiftKey && 'Shift', event.metaKey && 'Super'].filter(Boolean);
  if (!modifiers.length) return null;
  const code = event.code;
  const key = /^Key[A-Z]$/.test(code) ? code.slice(3) : /^Digit[0-9]$/.test(code) ? code.slice(5) : code;
  if (!/^(?:[A-Z0-9]|F(?:[1-9]|1[0-9]|2[0-4])|Space|Enter|Tab|Backspace|Arrow(?:Up|Down|Left|Right))$/.test(key)) return null;
  return [...modifiers, key].join('+');
}

export function prettyShortcut(shortcut: string, mac: boolean): string[] {
  if (!shortcut) return ['No shortcut'];
  return shortcut.split('+').map(part => ({
    Ctrl: mac ? '⌃' : 'Ctrl', Alt: mac ? '⌥' : 'Alt', Shift: mac ? '⇧' : 'Shift',
    Super: mac ? '⌘' : 'Win', Meta: mac ? '⌘' : 'Win',
  }[part] ?? part));
}
