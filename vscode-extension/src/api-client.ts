/**
 * Operator REST API client
 *
 * Provides methods to communicate with the Operator REST API
 * for launching tickets and checking health status.
 */

import * as vscode from 'vscode';
import * as fs from 'fs/promises';
import * as path from 'path';

// Import generated types from Rust bindings (source of truth)
import type {
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
  ModelServerKindEntry,
  ModelServerModelsResponse,
  ModelServerResponse,
  CreateModelServerRequest,
  DelegatorsResponse,
  DelegatorResponse,
  CreateDelegatorRequest,
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

export interface ApiError {
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
}

/**
 * Discover Operator API URL from session file or configuration
 *
 * Checks in order:
 * 1. .tickets/operator/api-session.json (written by running Operator)
 * 2. VSCode configuration operator.apiUrl
 */
export async function discoverApiUrl(
  ticketsDir: string | undefined
): Promise<string> {
  // Try to read api-session.json from tickets directory
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

  // Fall back to configured URL
  const config = vscode.workspace.getConfiguration('operator');
  return config.get('apiUrl', 'http://localhost:7008');
}

/**
 * Client for the Operator REST API
 */
export class OperatorApiClient {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    const config = vscode.workspace.getConfiguration('operator');
    this.baseUrl = baseUrl || config.get('apiUrl', 'http://localhost:7008');
  }

  /**
   * Check if the Operator API is available
   */
  async health(): Promise<HealthResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/health`);
    if (!response.ok) {
      throw new Error('Operator API not available');
    }
    return (await response.json()) as HealthResponse;
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
    const response = await fetch(
      `${this.baseUrl}/api/v1/tickets/${encodeURIComponent(ticketId)}/launch`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          delegator: options.delegator ?? null,
          provider: options.provider,
          model: options.model,
          yolo_mode: options.yolo_mode ?? false,
          wrapper: options.wrapper,
        }),
      }
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as LaunchTicketResponse;
  }

  /**
   * Export a ticket (rendered against its issue type) to a Claude dynamic
   * workflow (.js). Goes through the same shared code path as the CLI and TUI.
   */
  async exportWorkflow(ticketId: string): Promise<WorkflowExportResponse> {
    const response = await fetch(
      `${this.baseUrl}/api/v1/tickets/${encodeURIComponent(ticketId)}/workflow-export`,
      { method: 'POST' }
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as WorkflowExportResponse;
  }

  /**
   * Pause queue processing
   *
   * Stops automatic ticket assignment and agent launches.
   */
  async pauseQueue(): Promise<QueueControlResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/queue/pause`, {
      method: 'POST',
    });

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as QueueControlResponse;
  }

  /**
   * Resume queue processing
   *
   * Resumes automatic ticket assignment and agent launches.
   */
  async resumeQueue(): Promise<QueueControlResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/queue/resume`, {
      method: 'POST',
    });

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as QueueControlResponse;
  }

  /**
   * Sync kanban collections
   *
   * Fetches issues from configured external kanban providers and creates
   * local tickets in the queue.
   */
  async syncKanban(): Promise<KanbanSyncResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/queue/sync`, {
      method: 'POST',
    });

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as KanbanSyncResponse;
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
    const response = await fetch(
      `${this.baseUrl}/api/v1/queue/sync/${encodeURIComponent(provider)}/${encodeURIComponent(projectKey)}`,
      { method: 'POST' }
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as KanbanSyncResponse;
  }

  /**
   * Approve an agent's pending review
   *
   * Clears the review state and signals the agent to continue.
   */
  async approveReview(agentId: string): Promise<ReviewResponse> {
    const response = await fetch(
      `${this.baseUrl}/api/v1/agents/${encodeURIComponent(agentId)}/approve`,
      {
        method: 'POST',
      }
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as ReviewResponse;
  }

  /**
   * Reject an agent's pending review
   *
   * Signals the agent that the review was rejected with feedback.
   */
  async rejectReview(agentId: string, reason: string): Promise<ReviewResponse> {
    const response = await fetch(
      `${this.baseUrl}/api/v1/agents/${encodeURIComponent(agentId)}/reject`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ reason }),
      }
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as ReviewResponse;
  }

  /**
   * List all configured projects with analysis data
   */
  async getProjects(): Promise<ProjectSummary[]> {
    const response = await fetch(`${this.baseUrl}/api/v1/projects`);

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as ProjectSummary[];
  }

  /**
   * Create an ASSESS ticket for a project
   */
  async assessProject(name: string): Promise<AssessTicketResponse> {
    const response = await fetch(
      `${this.baseUrl}/api/v1/projects/${encodeURIComponent(name)}/assess`,
      { method: 'POST' }
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as AssessTicketResponse;
  }

  /**
   * List all issue types from the registry
   */
  async listIssueTypes(): Promise<IssueTypeSummary[]> {
    const response = await fetch(`${this.baseUrl}/api/v1/issuetypes`);

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as IssueTypeSummary[];
  }

  /**
   * Get a single issue type by key
   */
  async getIssueType(key: string): Promise<IssueTypeResponse> {
    const response = await fetch(
      `${this.baseUrl}/api/v1/issuetypes/${encodeURIComponent(key)}`
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as IssueTypeResponse;
  }

  /**
   * Create a new issue type
   */
  async createIssueType(request: CreateIssueTypeRequest): Promise<IssueTypeResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/issuetypes`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as IssueTypeResponse;
  }

  /**
   * Update an existing issue type
   */
  async updateIssueType(key: string, request: UpdateIssueTypeRequest): Promise<IssueTypeResponse> {
    const response = await fetch(
      `${this.baseUrl}/api/v1/issuetypes/${encodeURIComponent(key)}`,
      {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(request),
      }
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as IssueTypeResponse;
  }

  /**
   * Delete an issue type by key
   */
  async deleteIssueType(key: string): Promise<void> {
    const response = await fetch(
      `${this.baseUrl}/api/v1/issuetypes/${encodeURIComponent(key)}`,
      { method: 'DELETE' }
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }
  }

  /**
   * List all collections
   */
  async listCollections(): Promise<CollectionResponse[]> {
    const response = await fetch(`${this.baseUrl}/api/v1/collections`);

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as CollectionResponse[];
  }

  /**
   * Activate a collection by name
   */
  async activateCollection(name: string): Promise<void> {
    const response = await fetch(
      `${this.baseUrl}/api/v1/collections/${encodeURIComponent(name)}/activate`,
      { method: 'PUT' }
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }
  }

  /**
   * Get the catalog of supported kanban providers (Jira, Linear, GitHub),
   * each flagged with whether it is already configured. Single source of
   * truth shared with the TUI / web `/#/kanban` list view.
   */
  async listKanbanProviderCatalog(): Promise<KanbanProviderCatalogEntry[]> {
    const response = await fetch(`${this.baseUrl}/api/v1/kanban/providers`);

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as KanbanProviderCatalogEntry[];
  }

  /**
   * Get external issue types from a kanban provider for a project
   */
  async getExternalIssueTypes(
    provider: string,
    projectKey: string
  ): Promise<ExternalIssueTypeSummary[]> {
    const response = await fetch(
      `${this.baseUrl}/api/v1/kanban/${encodeURIComponent(provider)}/${encodeURIComponent(projectKey)}/issuetypes`
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as ExternalIssueTypeSummary[];
  }

  /**
   * Sync kanban issue types from a provider for a project.
   * Triggers a fresh fetch from the external provider and persists to the local catalog.
   */
  async syncKanbanIssueTypes(
    provider: string,
    projectKey: string
  ): Promise<SyncKanbanIssueTypesResponse> {
    const response = await fetch(
      `${this.baseUrl}/api/v1/kanban/${encodeURIComponent(provider)}/${encodeURIComponent(projectKey)}/issuetypes/sync`,
      { method: 'POST' }
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as SyncKanbanIssueTypesResponse;
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
    const response = await fetch(`${this.baseUrl}/api/v1/kanban/validate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as ValidateKanbanCredentialsResponse;
  }

  /**
   * List available projects/teams from a kanban provider using ephemeral
   * credentials. No persistence side effects.
   */
  async listKanbanProjects(
    req: ListKanbanProjectsRequest
  ): Promise<KanbanProjectInfo[]> {
    const response = await fetch(`${this.baseUrl}/api/v1/kanban/projects`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    const body = (await response.json()) as ListKanbanProjectsResponse;
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
    const response = await fetch(`${this.baseUrl}/api/v1/kanban/config`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as WriteKanbanConfigResponse;
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
    const response = await fetch(
      `${this.baseUrl}/api/v1/kanban/session-env`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(req),
      }
    );

    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }

    return (await response.json()) as SetKanbanSessionEnvResponse;
  }

  // --- Model providers ---

  private async getJson<T>(path: string): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`);
    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }
    return (await response.json()) as T;
  }

  private async postJson<T>(path: string, body: unknown): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
    if (!response.ok) {
      const error = (await response.json().catch(() => ({
        error: 'unknown',
        message: `HTTP ${response.status}: ${response.statusText}`,
      }))) as ApiError;
      throw new Error(error.message);
    }
    return (await response.json()) as T;
  }

  /** The catalog of supported model providers (kinds). */
  async listProviderKinds(): Promise<ModelServerKindEntry[]> {
    return this.getJson('/api/v1/model-servers/kinds');
  }

  /** Live models for a provider kind (declared instance or kind defaults). */
  async providerModels(slug: string): Promise<ModelServerModelsResponse> {
    return this.getJson(`/api/v1/model-servers/kinds/${encodeURIComponent(slug)}/models`);
  }

  /** Connect a gateway provider by declaring an instance. */
  async createModelServer(req: CreateModelServerRequest): Promise<ModelServerResponse> {
    return this.postJson('/api/v1/model-servers', req);
  }

  async listDelegators(): Promise<DelegatorsResponse> {
    return this.getJson('/api/v1/delegators');
  }

  async createDelegator(req: CreateDelegatorRequest): Promise<DelegatorResponse> {
    return this.postJson('/api/v1/delegators', req);
  }
}
