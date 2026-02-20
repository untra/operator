import type { Config } from '../../src/generated/Config';

/** Wrapper that pairs the generated Config with extension metadata */
export interface WebviewConfig {
  config_path: string;
  working_directory: string;
  config: Config;
}

/** Summary of a project from the Operator REST API */
export interface ProjectSummary {
  project_name: string;
  project_path: string;
  exists: boolean;
  has_catalog_info: boolean;
  has_project_context: boolean;
  kind: string | null;
  kind_confidence: number | null;
  kind_tier: string | null;
  languages: string[];
  frameworks: string[];
  databases: string[];
  has_docker: boolean | null;
  has_tests: boolean | null;
  ports: number[];
  env_var_count: number;
  entry_point_count: number;
  commands: string[];
}

/** Messages from the webview to the extension host */
export type WebviewToExtensionMessage =
  | { type: 'ready' }
  | { type: 'getConfig' }
  | { type: 'updateConfig'; section: string; key: string; value: unknown }
  | { type: 'browseFile'; field: string }
  | { type: 'browseFolder'; field: string }
  | { type: 'validateJira'; domain: string; email: string; apiToken: string }
  | { type: 'validateLinear'; apiKey: string }
  | { type: 'detectLlmTools' }
  | { type: 'openExternal'; url: string }
  | { type: 'openFile'; filePath: string }
  | { type: 'checkApiHealth' }
  | { type: 'getProjects' }
  | { type: 'assessProject'; projectName: string }
  | { type: 'openProjectFolder'; projectPath: string };

/** Messages from the extension host to the webview */
export type ExtensionToWebviewMessage =
  | { type: 'configLoaded'; config: WebviewConfig }
  | { type: 'configUpdated'; config: WebviewConfig }
  | { type: 'configError'; error: string }
  | { type: 'browseResult'; field: string; path: string }
  | { type: 'jiraValidationResult'; result: JiraValidationInfo }
  | { type: 'linearValidationResult'; result: LinearValidationInfo }
  | { type: 'llmToolsDetected'; config: WebviewConfig }
  | { type: 'apiHealthResult'; reachable: boolean }
  | { type: 'projectsLoaded'; projects: ProjectSummary[] }
  | { type: 'projectsError'; error: string }
  | { type: 'assessTicketCreated'; ticketId: string; projectName: string }
  | { type: 'assessTicketError'; error: string; projectName: string };

export interface JiraValidationInfo {
  valid: boolean;
  displayName: string;
  accountId: string;
  error?: string;
  projects?: Array<{ key: string; name: string }>;
}

export interface LinearValidationInfo {
  valid: boolean;
  userName: string;
  orgName: string;
  userId: string;
  error?: string;
  teams?: Array<{ id: string; name: string; key: string }>;
}
