import type { Config, Prompt } from './types';

export function parseImport(text: string, launchAtLogin: boolean): Config {
  const value: unknown = JSON.parse(text);
  if (!value || typeof value !== 'object') throw new Error('That is not a supported PromptKey settings file.');
  const data = value as Record<string, unknown>;
  if (data.version !== 1 || !Array.isArray(data.prompts) || typeof data.defaultModel !== 'string' || data.prompts.length > 100) {
    throw new Error('That is not a supported PromptKey settings file.');
  }
  const ids = new Set<string>();
  const prompts: Prompt[] = data.prompts.map(value => {
    if (!value || typeof value !== 'object') throw new Error('A prompt in this file is invalid.');
    const p = value as Record<string, unknown>;
    if (!['id', 'name', 'instructions', 'shortcut', 'model', 'output'].every(k => typeof p[k] === 'string') || typeof p.enabled !== 'boolean' || !['replace', 'review'].includes(String(p.output))) {
      throw new Error('A prompt in this file is invalid.');
    }
    const prompt = p as unknown as Prompt;
    if (!prompt.id.trim() || ids.has(prompt.id)) throw new Error('Prompt IDs must be unique and nonempty.');
    ids.add(prompt.id);
    return { id: prompt.id, name: prompt.name, instructions: prompt.instructions, shortcut: prompt.shortcut, model: prompt.model, output: prompt.output, enabled: prompt.enabled };
  });
  return { version: 1, defaultModel: data.defaultModel, launchAtLogin, prompts };
}
