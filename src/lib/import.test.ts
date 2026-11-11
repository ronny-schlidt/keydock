import { describe, expect, it } from 'vitest';
import { parseImport } from './import';

const prompt = { id: 'test', name: 'Test', instructions: 'Fix this text.', shortcut: '', model: '', output: 'review', enabled: true };
const file = (prompts: unknown[]) => JSON.stringify({ version: 1, defaultModel: 'provider/model', launchAtLogin: true, apiKey: 'must-not-import', prompts });

describe('shared prompt imports', () => {
  it('rejects duplicate IDs before the keyed prompt list can crash', () => {
    expect(() => parseImport(file([prompt, prompt]), false)).toThrow('unique');
  });
  it('does not import secrets or another device’s startup preference', () => {
    const result = parseImport(file([{ ...prompt, apiKey: 'must-not-import' }]), false);
    expect(result.launchAtLogin).toBe(false);
    expect(JSON.stringify(result)).not.toContain('apiKey');
  });
  it('rejects malformed records without changing existing settings', () => {
    expect(() => parseImport(file([null]), false)).toThrow();
    expect(() => parseImport(file([{ ...prompt, instructions: null }]), false)).toThrow();
  });
});
