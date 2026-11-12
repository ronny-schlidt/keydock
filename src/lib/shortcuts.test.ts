import { describe, expect, it } from 'vitest';
import { heldModifiers, prettyShortcut, recordingHint, recordShortcut } from './shortcuts';

describe('shortcut recording', () => {
  it('uses physical key codes even when Option changes the character', () => {
    expect(recordShortcut({ key: '´', code: 'KeyE', ctrlKey: true, altKey: true, shiftKey: true, metaKey: true })).toBe('Ctrl+Alt+Shift+Super+E');
  });
  it('does not register plain typing or modifier-only events', () => {
    expect(recordShortcut({ key: 'e', code: 'KeyE', ctrlKey: false, altKey: false, shiftKey: false, metaKey: false })).toBeNull();
    expect(recordShortcut({ key: 'Shift', code: 'ShiftLeft', ctrlKey: true, altKey: false, shiftKey: true, metaKey: false })).toBeNull();
  });
  it('rejects Shift-only and Command-only shortcuts that would hijack typing or menus', () => {
    expect(recordShortcut({ key: 'v', code: 'KeyV', ctrlKey: false, altKey: false, shiftKey: false, metaKey: true })).toBeNull();
    expect(recordShortcut({ key: 'A', code: 'KeyA', ctrlKey: false, altKey: false, shiftKey: true, metaKey: false })).toBeNull();
    expect(recordShortcut({ key: 'v', code: 'KeyV', ctrlKey: false, altKey: true, shiftKey: false, metaKey: true })).toBe('Alt+Super+V');
  });
  it('explains why a key press was not recorded', () => {
    const plain = { ctrlKey: false, altKey: false, shiftKey: false, metaKey: false };
    expect(recordingHint({ key: 'CapsLock', code: 'CapsLock', ...plain }, true)).toContain('no Hyper key is active');
    expect(recordingHint({ key: 'w', code: 'KeyW', ...plain }, true)).toBe('Hold ⌃, ⌥ or ⌘ together with the key.');
    expect(recordingHint({ key: 'v', code: 'KeyV', ...plain, metaKey: true }, true)).toBe('⌘ alone is taken by other apps. Add ⌃ or ⌥.');
    expect(recordingHint({ key: 'Meta', code: 'MetaLeft', ...plain, metaKey: true }, true)).toBe('');
  });
  it('lists held modifiers in storage order', () => {
    expect(heldModifiers({ ctrlKey: true, altKey: true, shiftKey: true, metaKey: true })).toBe('Ctrl+Alt+Shift+Super');
  });
  it('renders the same stored shortcut for either OS', () => {
    expect(prettyShortcut('Ctrl+Alt+Shift+Super+E', true)).toEqual(['⌃', '⌥', '⇧', '⌘', 'E']);
    expect(prettyShortcut('Ctrl+Alt+Shift+Super+E', false)).toEqual(['Ctrl', 'Alt', 'Shift', 'Win', 'E']);
  });
});
