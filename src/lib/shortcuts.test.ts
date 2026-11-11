import { describe, expect, it } from 'vitest';
import { prettyShortcut, recordShortcut } from './shortcuts';

describe('shortcut recording', () => {
  it('uses physical key codes even when Option changes the character', () => {
    expect(recordShortcut({ key: '´', code: 'KeyE', ctrlKey: true, altKey: true, shiftKey: true, metaKey: true })).toBe('Ctrl+Alt+Shift+Super+E');
  });
  it('does not register plain typing or modifier-only events', () => {
    expect(recordShortcut({ key: 'e', code: 'KeyE', ctrlKey: false, altKey: false, shiftKey: false, metaKey: false })).toBeNull();
    expect(recordShortcut({ key: 'Shift', code: 'ShiftLeft', ctrlKey: true, altKey: false, shiftKey: true, metaKey: false })).toBeNull();
  });
  it('renders the same stored shortcut for either OS', () => {
    expect(prettyShortcut('Ctrl+Alt+Shift+Super+E', true)).toEqual(['⌃', '⌥', '⇧', '⌘', 'E']);
    expect(prettyShortcut('Ctrl+Alt+Shift+Super+E', false)).toEqual(['Ctrl', 'Alt', 'Shift', 'Win', 'E']);
  });
});
