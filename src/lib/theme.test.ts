import { describe, expect, it } from 'vitest';
import { parseTheme, resolveTheme } from './theme';

describe('theme preference', () => {
  it('falls back to the system appearance for missing or unknown values', () => {
    expect(parseTheme(null)).toBe('system');
    expect(parseTheme('sepia')).toBe('system');
    expect(parseTheme('dark')).toBe('dark');
  });
  it('follows the system only when no explicit theme is chosen', () => {
    expect(resolveTheme('system', true)).toBe('dark');
    expect(resolveTheme('system', false)).toBe('light');
    expect(resolveTheme('light', true)).toBe('light');
    expect(resolveTheme('dark', false)).toBe('dark');
  });
});
