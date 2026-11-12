export interface ModelSuggestion {
  id: string;
  name: string;
  note: string;
  price: string;
}

// Prices are OpenRouter list prices per million input/output tokens, checked 2026-11-12.
export const suggestedModels: ModelSuggestion[] = [
  { id: 'google/gemini-2.5-flash-lite', name: 'Gemini 2.5 Flash-Lite', note: 'Cheapest. Quick for short edits.', price: '$0.10 / $0.40' },
  { id: 'anthropic/claude-haiku-5.5', name: 'Claude Haiku 5.5', note: 'Best writing quality at this price.', price: '$0.10 / $0.50' },
  { id: 'mistralai/mistral-small-2603', name: 'Mistral Small 4', note: 'Runs at Mistral (EU). Strong in German and French.', price: '$0.15 / $0.60' },
  { id: 'google/gemini-3.1-flash-lite', name: 'Gemini 3.1 Flash-Lite', note: 'Newer Google generation for longer texts.', price: '$0.25 / $1.50' },
];

export function modelName(id: string): string {
  return suggestedModels.find(model => model.id === id)?.name ?? id;
}
