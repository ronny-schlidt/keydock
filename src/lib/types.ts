export interface Prompt {
  id: string;
  name: string;
  instructions: string;
  shortcut: string;
  model: string;
  enabled: boolean;
  output: 'replace' | 'review';
}

export interface Config {
  version: number;
  defaultModel: string;
  launchAtLogin: boolean;
  prompts: Prompt[];
}

export interface Status {
  phase: 'idle' | 'working' | 'success' | 'review' | 'error';
  message: string;
  promptName?: string;
  result?: string;
}

export interface KeyCheck {
  limitRemaining: number | null;
  usage: number;
  isFreeTier: boolean;
  modelFound: boolean | null;
}

export interface Bootstrap {
  config: Config;
  hasKey: boolean;
  platform: 'macos' | 'windows';
  accessibility: boolean;
  status: Status;
  shortcutErrors: string[];
}
