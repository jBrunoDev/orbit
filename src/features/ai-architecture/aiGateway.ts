import { invoke } from "@tauri-apps/api/core";

export type AiSettings = { enabled: boolean; provider: string; model: string; hasOpenaiApiKey: boolean; hasGithubToken: boolean };
export type ArchitectureGeneration = { id: string; status: string; sourceMode: string; sources: Array<{ path: string; kind: string }>; architectureModel?: { nodes: unknown[]; edges: unknown[]; assumptions: string[] }; documentationMarkdown?: string; error?: string };

export const aiGateway = {
  settings: () => invoke<AiSettings>("get_ai_settings"),
  saveSettings: (enabled: boolean, model: string) => invoke<AiSettings>("save_ai_settings", { input: { enabled, model } }),
  saveOpenAiKey: (value: string) => invoke<void>("save_openai_api_key", { input: { value } }),
  saveGithubToken: (value: string) => invoke<void>("save_github_token", { input: { value } }),
  analyze: (input: { projectId: string; repositoryUrl?: string; documentPath?: string; folderPath?: string; consentToSendSources: boolean }) => invoke<ArchitectureGeneration>("start_architecture_analysis", { input }),
};
