import type { Config } from '../../src/generated/Config';
import type { IssueTypeSummary } from '../../src/generated/IssueTypeSummary';
import type { IssueTypeResponse } from '../../src/generated/IssueTypeResponse';
import type { CollectionResponse } from '../../src/generated/CollectionResponse';
import type { ExternalIssueTypeSummary } from '../../src/generated/ExternalIssueTypeSummary';

// Re-export generated types for consumers
export type { IssueTypeSummary, IssueTypeResponse, CollectionResponse, ExternalIssueTypeSummary };

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
  | { type: 'openProjectFolder'; projectPath: string }
  | { type: 'getIssueTypes' }
  | { type: 'getIssueType'; key: string }
  | { type: 'getCollections' }
  | { type: 'activateCollection'; name: string }
  | { type: 'getExternalIssueTypes'; provider: string; domain: string; projectKey: string }
  | { type: 'createIssueType'; request: import('../../src/generated/CreateIssueTypeRequest').CreateIssueTypeRequest }
  | { type: 'updateIssueType'; key: string; request: import('../../src/generated/UpdateIssueTypeRequest').UpdateIssueTypeRequest }
  | { type: 'deleteIssueType'; key: string };

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
  | { type: 'assessTicketError'; error: string; projectName: string }
  | { type: 'issueTypesLoaded'; issueTypes: IssueTypeSummary[] }
  | { type: 'issueTypeLoaded'; issueType: IssueTypeResponse }
  | { type: 'issueTypeError'; error: string }
  | { type: 'collectionsLoaded'; collections: CollectionResponse[] }
  | { type: 'collectionActivated'; name: string }
  | { type: 'collectionsError'; error: string }
  | { type: 'externalIssueTypesLoaded'; provider: string; projectKey: string; types: ExternalIssueTypeSummary[] }
  | { type: 'externalIssueTypesError'; provider: string; projectKey: string; error: string }
  | { type: 'issueTypeCreated'; issueType: IssueTypeResponse }
  | { type: 'issueTypeUpdated'; issueType: IssueTypeResponse }
  | { type: 'issueTypeDeleted'; key: string };

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
