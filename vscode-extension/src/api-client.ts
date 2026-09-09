/**
 * Operator REST API client
 *
 * Every daemon request goes through one authenticated path (`send`), which
 * attaches the credential from `auth/credentials`, retries once after a refresh on 401, and normalizes errors into `ApiError`.
 */

import * as vscode from 'vscode';
import * as fs from 'fs/promises';
import * as path from 'path';

import { credentialProvider } from './auth/credentials';
import { ApiError, AuthRequiredError } from './auth/errors';

// Import generated types from Rust bindings (source of truth)
import type {
  ActiveAgentsResponse,
  CurrentSessionResponse,
  KanbanBoardResponse,
  LaunchTicketRequest,
  LaunchTicketResponse,
  HealthResponse,
  IssueTypeSummary,
  IssueTypeResponse,
  CollectionResponse,
  ExternalIssueTypeSummary,
  CreateIssueTypeRequest,
  UpdateIssueTypeRequest,
  SyncKanbanIssueTypesResponse,
  ValidateKanbanCredentialsRequest,
  ValidateKanbanCredentialsResponse,
  ListKanbanProjectsRequest,
  ListKanbanProjectsResponse,
  KanbanProjectInfo,
  KanbanProviderCatalogEntry,
  WriteKanbanConfigRequest,
  WriteKanbanConfigResponse,
  SetKanbanSessionEnvRequest,
  SetKanbanSessionEnvResponse,
  WorkflowExportResponse,
  WorkflowFormatDto,
  ModelServerKindEntry,
  ModelServerModelsResponse,
  ModelServerResponse,
  ModelServersResponse,
  CreateModelServerRequest,
  DelegatorsResponse,
  DelegatorResponse,
  CreateDelegatorRequest,
  DefaultLlmResponse,
  SetDefaultLlmRequest,
  LlmToolsResponse,
  ExecutionTargetsResponse,
  McpDescriptorResponse,
} from './generated';

// Re-export generated types for consumers
export type {
  LaunchTicketResponse,
  HealthResponse,
  IssueTypeSummary,
  IssueTypeResponse,
  CollectionResponse,
  ExternalIssueTypeSummary,
  CreateIssueTypeRequest,
  UpdateIssueTypeRequest,
  SyncKanbanIssueTypesResponse,
  ValidateKanbanCredentialsRequest,
  ValidateKanbanCredentialsResponse,
  ListKanbanProjectsRequest,
  ListKanbanProjectsResponse,
  KanbanProjectInfo,
  KanbanProviderCatalogEntry,
  WriteKanbanConfigRequest,
  WriteKanbanConfigResponse,
  SetKanbanSessionEnvRequest,
  SetKanbanSessionEnvResponse,
  ModelServerKindEntry,
  ModelServerModelsResponse,
  ModelServerResponse,
  CreateModelServerRequest,
  DelegatorsResponse,
  DelegatorResponse,
  CreateDelegatorRequest,
};
export { ApiError, AuthRequiredError };

/**
 * Summary of a project from the Operator REST API
 */
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

/**
 * Response from creating an ASSESS ticket
 */
export interface AssessTicketResponse {
  ticket_id: string;
  ticket_path: string;
  project_name: string;
}

/** Error body the daemon returns on non-2xx responses. */
export interface ApiErrorBody {
  error: string;
  message: string;
}

/**
 * Response from queue pause/resume operations
 */
export interface QueueControlResponse {
  paused: boolean;
  message: string;
}

/**
 * Response from kanban sync operations
 */
export interface KanbanSyncResponse {
  created: string[];
  skipped: string[];
  errors: string[];
  total_processed: number;
}

/**
 * Response from agent review operations
 */
export interface ReviewResponse {
  agent_id: string;
  status: string;
  message: string;
}

/**
 * Request to reject an agent's review
 */
export interface RejectReviewRequest {
  reason: string;
}

/**
 * API session info written by Operator when running in API mode
 */
export interface ApiSessionInfo {
  port: number;
  pid: number;
  started_at: string;
  version: string;
  /** State directory holding `local-token`; absent from files written by older daemons. */
  state_dir?: string;
}

export const DEFAULT_API_URL = 'http://localhost:7008';
/** Public liveness probe: answers without a credential, unlike `/api/v1/health`. */
export const LIVEZ_PATH = '/livez';

/**
 * ts-rs maps Rust `u64` to `bigint`, and `JSON.stringify` throws on a bigint,
 * so a body containing one would fail before the request is ever sent.
 */
export function toJson(value: unknown): string {
  return JSON.stringify(value, (_key, v: unknown) =>
    typeof v === 'bigint' ? Number(v) : v
  );
}

/**
 * Discover Operator API URL.
 *
 * Checks in order:
 * 1. Explicitly-set operator.apiUrl (config.inspect distinguishes a user
 *    setting from the default value)
 * 2. .tickets/operator/api-session.json (written by a running Operator)
 * 3. The default http://localhost:7008
 */
export async function discoverApiUrl(
  ticketsDir: string | undefined
): Promise<string> {
  const config = vscode.workspace.getConfiguration('operator');
  const inspected = config.inspect<string>('apiUrl');
  const explicit =
    inspected?.workspaceFolderValue ??
    inspected?.workspaceValue ??
    inspected?.globalValue;
  if (explicit) {
    return explicit;
  }

  if (ticketsDir) {
    const sessionFile = path.join(ticketsDir, 'operator', 'api-session.json');
    try {
      const content = await fs.readFile(sessionFile, 'utf-8');
      const session = JSON.parse(content) as ApiSessionInfo;
      return `http://localhost:${session.port}`;
    } catch {
      // Session file doesn't exist or is invalid, fall through
    }
  }

  return DEFAULT_API_URL;
}

function withBearer(init: RequestInit, token: string | undefined): RequestInit {
  if (!token) {
    return init;
  }
  return {
    ...init,
    headers: { ...(init.headers as Record<string, string> | undefined), Authorization: `Bearer ${token}` },
  };
}

function jsonInit(method: string, body: unknown): RequestInit {
  return {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: toJson(body),
  };
}

/**
 * Client for the Operator REST API
 */
export class OperatorApiClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    const config = vscode.workspace.getConfiguration('operator');
    this.baseUrl = baseUrl || config.get('apiUrl', DEFAULT_API_URL);
  }

  /**
   * Perform an authenticated request.
   *
   * A 401 triggers exactly one refresh-and-retry. A refresh that yields the
   * same credential (or none) is not retried: the server has already rejected
   * it, and repeating the request would only repeat the rejection.
   */
  private async send(apiPath: string, init: RequestInit = {}): Promise<Response> {
    const provider = credentialProvider();
    const url = `${this.baseUrl}${apiPath}`;

    const token = await provider.bearer(this.baseUrl);
    let response = await fetch(url, withBearer(init, token));

    if (response.status === 401) {
      const refreshed = await provider.refresh(this.baseUrl);
      if (refreshed && refreshed !== token) {
        response = await fetch(url, withBearer(init, refreshed));
      }
    }

    if (response.status === 401) {
      throw new AuthRequiredError(this.baseUrl);
    }
    if (!response.ok) {
      const body = (await response.json().catch(() => ({}))) as Partial<ApiErrorBody>;
      throw new ApiError(
        response.status,
        body.message ?? body.error ?? `HTTP ${response.status}: ${response.statusText}`
      );
    }
    return response;
  }

  private async request<T>(apiPath: string, init?: RequestInit): Promise<T> {
    const response = await this.send(apiPath, init);
    return (await response.json()) as T;
  }

  private async requestVoid(apiPath: string, init?: RequestInit): Promise<void> {
    await this.send(apiPath, init);
  }

  /**
   * Whether anything is listening at the base URL. Unauthenticated: this is
   * the "is the daemon up" question, not "is it ours".
   */
  async isReachable(): Promise<boolean> {
    try {
      const response = await fetch(`${this.baseUrl}${LIVEZ_PATH}`);
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * Check if the Operator API is available
   */
  async health(): Promise<HealthResponse> {
    try {
      return await this.request<HealthResponse>('/api/v1/health');
    } catch (err) {
      if (err instanceof ApiError && !(err instanceof AuthRequiredError)) {
        throw new ApiError(err.status, 'Operator API not available');
      }
      throw err;
    }
  }

  /** The identity the daemon sees for the extension's current credential. */
  async currentSession(): Promise<CurrentSessionResponse> {
    return this.request('/api/v1/auth/session');
  }

  /**
   * Launch a ticket via the Operator API
   *
   * Claims the ticket, sets up worktree if needed, and returns
   * the command to execute in a terminal.
   */
  async launchTicket(
    ticketId: string,
    options: LaunchTicketRequest
  ): Promise<LaunchTicketResponse> {
    return this.request(
      `/api/v1/tickets/${encodeURIComponent(ticketId)}/launch`,
      jsonInit('POST', {
        ...options,
        delegator: options.delegator ?? null,
        yolo_mode: options.yolo_mode ?? false,
      })
    );
  }

  /**
   * Export a ticket (rendered against its issue type) to a Claude dynamic
   * workflow (.js). Goes through the same shared code path as the CLI and TUI.
   */
  async exportWorkflow(ticketId: string): Promise<WorkflowExportResponse> {
    return this.request(
      `/api/v1/tickets/${encodeURIComponent(ticketId)}/workflow-export`,
      { method: 'POST' }
    );
  }

  /** Agents currently running, for the review pickers. */
  async listActiveAgents(): Promise<ActiveAgentsResponse> {
    return this.request('/api/v1/agents/active');
  }

  /**
   * Pause queue processing
   *
   * Stops automatic ticket assignment and agent launches.
   */
  async pauseQueue(): Promise<QueueControlResponse> {
    return this.request('/api/v1/queue/pause', { method: 'POST' });
  }

  /**
   * Resume queue processing
   *
   * Resumes automatic ticket assignment and agent launches.
   */
  async resumeQueue(): Promise<QueueControlResponse> {
    return this.request('/api/v1/queue/resume', { method: 'POST' });
  }

  /**
   * Sync kanban collections
   *
   * Fetches issues from configured external kanban providers and creates
   * local tickets in the queue.
   */
  async syncKanban(): Promise<KanbanSyncResponse> {
    return this.request('/api/v1/queue/sync', { method: 'POST' });
  }

  /**
   * Sync a specific kanban collection
   *
   * Fetches issues from a single provider/project combination and creates
   * local tickets in the queue.
   */
  async syncKanbanCollection(
    provider: string,
    projectKey: string
  ): Promise<KanbanSyncResponse> {
    return this.request(
      `/api/v1/queue/sync/${encodeURIComponent(provider)}/${encodeURIComponent(projectKey)}`,
      { method: 'POST' }
    );
  }

  /**
   * Approve an agent's pending review
   *
   * Clears the review state and signals the agent to continue.
   */
  async approveReview(agentId: string): Promise<ReviewResponse> {
    return this.request(
      `/api/v1/agents/${encodeURIComponent(agentId)}/approve`,
      { method: 'POST' }
    );
  }

  /**
   * Reject an agent's pending review
   *
   * Signals the agent that the review was rejected with feedback.
   */
  async rejectReview(agentId: string, reason: string): Promise<ReviewResponse> {
    return this.request(
      `/api/v1/agents/${encodeURIComponent(agentId)}/reject`,
      jsonInit('POST', { reason })
    );
  }

  /**
   * List all configured projects with analysis data
   */
  async getProjects(): Promise<ProjectSummary[]> {
    return this.request('/api/v1/projects');
  }

  /**
   * Create an ASSESS ticket for a project
   */
  async assessProject(name: string): Promise<AssessTicketResponse> {
    return this.request(
      `/api/v1/projects/${encodeURIComponent(name)}/assess`,
      { method: 'POST' }
    );
  }

  /**
   * List all issue types from the registry
   */
  async listIssueTypes(): Promise<IssueTypeSummary[]> {
    return this.request('/api/v1/issuetypes');
  }

  /**
   * Get a single issue type by key
   */
  async getIssueType(key: string): Promise<IssueTypeResponse> {
    return this.request(`/api/v1/issuetypes/${encodeURIComponent(key)}`);
  }

  /**
   * Create a new issue type
   */
  async createIssueType(request: CreateIssueTypeRequest): Promise<IssueTypeResponse> {
    return this.request('/api/v1/issuetypes', jsonInit('POST', request));
  }

  /**
   * Update an existing issue type
   */
  async updateIssueType(key: string, request: UpdateIssueTypeRequest): Promise<IssueTypeResponse> {
    return this.request(
      `/api/v1/issuetypes/${encodeURIComponent(key)}`,
      jsonInit('PUT', request)
    );
  }

  /**
   * Delete an issue type by key
   */
  async deleteIssueType(key: string): Promise<void> {
    await this.requestVoid(
      `/api/v1/issuetypes/${encodeURIComponent(key)}`,
      { method: 'DELETE' }
    );
  }

  /**
   * List all collections
   */
  async listCollections(): Promise<CollectionResponse[]> {
    return this.request('/api/v1/collections');
  }

  /**
   * Activate a collection by name
   */
  async activateCollection(name: string): Promise<void> {
    await this.requestVoid(
      `/api/v1/collections/${encodeURIComponent(name)}/activate`,
      { method: 'PUT' }
    );
  }

  /**
   * Get the catalog of supported kanban providers (Jira, Linear, GitHub),
   * each flagged with whether it is already configured. Single source of
   * truth shared with the TUI / web `/#/kanban` list view.
   */
  async listKanbanProviderCatalog(): Promise<KanbanProviderCatalogEntry[]> {
    return this.request('/api/v1/kanban/providers');
  }

  /**
   * Get external issue types from a kanban provider for a project
   */
  async getExternalIssueTypes(
    provider: string,
    projectKey: string
  ): Promise<ExternalIssueTypeSummary[]> {
    return this.request(
      `/api/v1/kanban/${encodeURIComponent(provider)}/${encodeURIComponent(projectKey)}/issuetypes`
    );
  }

  /**
   * Get the external board's workflow statuses/columns for a configured
   * provider/project — populates the todo/doing/done mapping dropdowns.
   */
  async getKanbanStatuses(provider: string, projectKey: string): Promise<string[]> {
    const body = await this.request<{ statuses: string[] }>(
      `/api/v1/kanban/${encodeURIComponent(provider)}/${encodeURIComponent(projectKey)}/statuses`
    );
    return body.statuses;
  }

  /**
   * Sync kanban issue types from a provider for a project.
   * Triggers a fresh fetch from the external provider and persists to the local catalog.
   */
  async syncKanbanIssueTypes(
    provider: string,
    projectKey: string
  ): Promise<SyncKanbanIssueTypesResponse> {
    return this.request(
      `/api/v1/kanban/${encodeURIComponent(provider)}/${encodeURIComponent(projectKey)}/issuetypes/sync`,
      { method: 'POST' }
    );
  }

  // ─── Kanban Onboarding ────────────────────────────────────────────────

  /**
   * Validate kanban provider credentials against the live provider API.
   *
   * Auth failures return `valid: false` with `error` set — NOT a thrown
   * exception — so callers can display errors inline and offer retry.
   * Network / server errors throw.
   */
  async validateKanbanCredentials(
    req: ValidateKanbanCredentialsRequest
  ): Promise<ValidateKanbanCredentialsResponse> {
    return this.request('/api/v1/kanban/validate', jsonInit('POST', req));
  }

  /**
   * List available projects/teams from a kanban provider using ephemeral
   * credentials. No persistence side effects.
   */
  async listKanbanProjects(
    req: ListKanbanProjectsRequest
  ): Promise<KanbanProjectInfo[]> {
    const body = await this.request<ListKanbanProjectsResponse>(
      '/api/v1/kanban/projects',
      jsonInit('POST', req)
    );
    return body.projects;
  }

  /**
   * Write (upsert) a kanban provider + project section into config.toml.
   *
   * Does NOT receive the actual secret — only the env var name
   * (`api_key_env`). The secret is set via `setKanbanSessionEnv`.
   */
  async writeKanbanConfig(
    req: WriteKanbanConfigRequest
  ): Promise<WriteKanbanConfigResponse> {
    return this.request('/api/v1/kanban/config', jsonInit('PUT', req));
  }

  /**
   * Set kanban env vars on the server process for the current session
   * so subsequent sync calls find the API key.
   *
   * The returned `shell_export_block` uses `<your-token>` placeholders,
   * not the real secret — safe to display to the user.
   */
  async setKanbanSessionEnv(
    req: SetKanbanSessionEnvRequest
  ): Promise<SetKanbanSessionEnvResponse> {
    return this.request('/api/v1/kanban/session-env', jsonInit('POST', req));
  }

  // --- LLM tools ---

  async listLlmTools(): Promise<LlmToolsResponse> {
    return this.request('/api/v1/llm-tools');
  }

  async getDefaultLlm(): Promise<DefaultLlmResponse> {
    return this.request('/api/v1/llm-tools/default');
  }

  async setDefaultLlm(req: SetDefaultLlmRequest): Promise<void> {
    await this.requestVoid('/api/v1/llm-tools/default', jsonInit('PUT', req));
  }

  // --- Model providers ---

  /** The catalog of supported model providers (kinds). */
  async listProviderKinds(): Promise<ModelServerKindEntry[]> {
    return this.request('/api/v1/model-servers/kinds');
  }

  /** Live models for a provider kind (declared instance or kind defaults). */
  async providerModels(slug: string): Promise<ModelServerModelsResponse> {
    return this.request(`/api/v1/model-servers/kinds/${encodeURIComponent(slug)}/models`);
  }

  /** Declared model server instances plus builtins. */
  async listModelServers(): Promise<ModelServersResponse> {
    return this.request('/api/v1/model-servers');
  }

  /** Live models for one declared server. */
  async modelServerModels(name: string): Promise<ModelServerModelsResponse> {
    return this.request(`/api/v1/model-servers/${encodeURIComponent(name)}/models`);
  }

  /** Connect a gateway provider by declaring an instance. */
  async createModelServer(req: CreateModelServerRequest): Promise<ModelServerResponse> {
    return this.request('/api/v1/model-servers', jsonInit('POST', req));
  }

  /** Kanban board columns — the API-backed source for the ticket trees. */
  async getKanban(): Promise<KanbanBoardResponse> {
    return this.request('/api/v1/queue/kanban');
  }

  /** Kanban board columns — the API-backed source for the ticket trees. */
  async getKanban(): Promise<KanbanBoardResponse> {
    return this.getJson<KanbanBoardResponse>('/api/v1/queue/kanban');
  }

  async listDelegators(): Promise<DelegatorsResponse> {
    return this.request('/api/v1/delegators');
  }

  async createDelegator(req: CreateDelegatorRequest): Promise<DelegatorResponse> {
    return this.request('/api/v1/delegators', jsonInit('POST', req));
  }

  // --- Workflows, targets, MCP ---

  async listWorkflowFormats(): Promise<WorkflowFormatDto[]> {
    return this.request('/api/v1/workflow-formats');
  }

  /** Named execution targets: local, docker, `[[targets]]`, and `[[hosts]]`. */
  async listExecutionTargets(): Promise<ExecutionTargetsResponse> {
    return this.request('/api/v1/execution-targets');
  }

  async mcpDescriptor(): Promise<McpDescriptorResponse> {
    return this.request('/api/v1/mcp/descriptor');
  }
}
