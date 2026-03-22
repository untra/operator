import * as vscode from 'vscode';
import { StatusItem } from '../status-item';
import type { DetectedToolResult } from '../walkthrough';

/** Shared context provided by the orchestrator to all sections */
export interface SectionContext {
  extensionContext: vscode.ExtensionContext;
  ticketsDir: string | undefined;
  readConfigToml: () => Promise<Record<string, unknown>>;
  /** True when working dir is set AND config.toml exists. Set after checks complete. */
  configReady: boolean;
  /** True when API or webhook is connected. Set after checks complete. */
  connectionsReady: boolean;
  /** True when any kanban provider is configured. Set after checks complete. */
  kanbanConfigured: boolean;
  /** True when any LLM tool is detected. Set after checks complete. */
  llmConfigured: boolean;
  /** True when git section is configured. Set after checks complete. */
  gitConfigured: boolean;
  /** Live webhook server state (provided by extension.ts) */
  webhookServer?: {
    isRunning: () => boolean;
    getPort: () => number;
  };
}

/** Every status tree section implements this interface */
export interface StatusSection {
  readonly sectionId: string;
  check(ctx: SectionContext): Promise<void>;
  getTopLevelItem(ctx: SectionContext): StatusItem;
  getChildren(ctx: SectionContext, element?: StatusItem): StatusItem[];
}

/**
 * Webhook server connection status
 */
export interface WebhookStatus {
  running: boolean;
  version?: string;
  port?: number;
  workspace?: string;
  sessionFile?: string;
}

/**
 * Operator REST API connection status
 */
export interface ApiStatus {
  connected: boolean;
  version?: string;
  port?: number;
  url?: string;
}

/** Internal state for the Configuration section */
export interface ConfigState {
  workingDirSet: boolean;
  workingDir: string;
  configExists: boolean;
  configPath: string;
}

/** Config-driven state for a single kanban provider */
export interface KanbanProviderState {
  provider: 'jira' | 'linear';
  key: string;
  enabled: boolean;
  displayName: string;
  url: string;
  projects: Array<{
    key: string;
    collectionName: string;
    url: string;
  }>;
}

/** Internal state for the Kanban section */
export interface KanbanState {
  configured: boolean;
  providers: KanbanProviderState[];
}

/** Per-tool info with model aliases */
export interface LlmToolInfo {
  name: string;
  version?: string;
  models: string[];
}

/** Internal state for the LLM Tools section */
export interface LlmState {
  detected: boolean;
  tools: DetectedToolResult[];
  configDetected: Array<{ name: string; version?: string }>;
  toolDetails: LlmToolInfo[];
}

/** Internal state for the Git section */
export interface GitState {
  configured: boolean;
  provider?: string;
  githubEnabled?: boolean;
  tokenSet?: boolean;
  branchFormat?: string;
  useWorktrees?: boolean;
}
