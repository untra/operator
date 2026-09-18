import type { Host } from "./host";
import type { TargetDef } from "@operator/bindings/TargetDef";
import type { ProfileSummary } from "@operator/bindings/ProfileSummary";
import type { LicenseResponse } from "@operator/bindings/LicenseResponse";
import type { TargetResponse } from "@operator/bindings/TargetResponse";
import type { TargetsResponse } from "@operator/bindings/TargetsResponse";
import type { TargetProbeResponse } from "@operator/bindings/TargetProbeResponse";
import type { HealthResponse } from "@operator/bindings/HealthResponse";
import type { StatusResponse } from "@operator/bindings/StatusResponse";
import type { SectionDto } from "@operator/bindings/SectionDto";
import type { SectionRowDto } from "@operator/bindings/SectionRowDto";
import type { QueueStatusResponse } from "@operator/bindings/QueueStatusResponse";
import type { KanbanBoardResponse } from "@operator/bindings/KanbanBoardResponse";
import type { KanbanTicketCard } from "@operator/bindings/KanbanTicketCard";
import type { ActiveAgentsResponse } from "@operator/bindings/ActiveAgentsResponse";
import type { CreateTicketRequest } from "@operator/bindings/CreateTicketRequest";
import type { CreateTicketResponse } from "@operator/bindings/CreateTicketResponse";
import type { IssueTypeSummary } from "@operator/bindings/IssueTypeSummary";
import type { IssueTypeResponse } from "@operator/bindings/IssueTypeResponse";
import type { CollectionResponse } from "@operator/bindings/CollectionResponse";
import type { ProjectSummary } from "@operator/bindings/ProjectSummary";
import type { CompletedTicket } from "@operator/bindings/CompletedTicket";
import type { CreateIssueTypeRequest } from "@operator/bindings/CreateIssueTypeRequest";
import type { UpdateIssueTypeRequest } from "@operator/bindings/UpdateIssueTypeRequest";
import type { LaunchTicketRequest } from "@operator/bindings/LaunchTicketRequest";
import type { LaunchTicketResponse } from "@operator/bindings/LaunchTicketResponse";
import type { QueueControlResponse } from "@operator/bindings/QueueControlResponse";
import type { ConfigurationResponse } from "@operator/bindings/ConfigurationResponse";
import type { UpdateConfigurationRequest } from "@operator/bindings/UpdateConfigurationRequest";
import type { ExecutionTargetsResponse } from "@operator/bindings/ExecutionTargetsResponse";
import type { LlmToolsResponse } from "@operator/bindings/LlmToolsResponse";
import type { AgentDetailResponse } from "@operator/bindings/AgentDetailResponse";
import type { WorkflowExportResponse } from "@operator/bindings/WorkflowExportResponse";
import type { WorkflowPreviewResponse } from "@operator/bindings/WorkflowPreviewResponse";
import type { IssueType } from "@operator/bindings/IssueType";
import type { ModelServerKindEntry } from "@operator/bindings/ModelServerKindEntry";
import type { ModelServerModelsResponse } from "@operator/bindings/ModelServerModelsResponse";
import type { ModelServersResponse } from "@operator/bindings/ModelServersResponse";
import type { ModelServerResponse } from "@operator/bindings/ModelServerResponse";
import type { CreateModelServerRequest } from "@operator/bindings/CreateModelServerRequest";
import type { DelegatorsResponse } from "@operator/bindings/DelegatorsResponse";
import type { DelegatorResponse } from "@operator/bindings/DelegatorResponse";
import type { CreateDelegatorRequest } from "@operator/bindings/CreateDelegatorRequest";
import type { IntegrationCatalogEntryDto } from "@operator/bindings/IntegrationCatalogEntryDto";
import type { SetupStatusResponse } from "@operator/bindings/SetupStatusResponse";
import type { SetupStepResponse } from "@operator/bindings/SetupStepResponse";
import type { SetupCollectionResponse } from "@operator/bindings/SetupCollectionResponse";
import type { SetupInitializeRequest } from "@operator/bindings/SetupInitializeRequest";
import type { SetupInitializeResponse } from "@operator/bindings/SetupInitializeResponse";
import type { GitProviderOnboardingResponse } from "@operator/bindings/GitProviderOnboardingResponse";
import type { ValidateGitTokenRequest } from "@operator/bindings/ValidateGitTokenRequest";
import type { ValidateGitTokenResponse } from "@operator/bindings/ValidateGitTokenResponse";
import type { WriteGitConfigRequest } from "@operator/bindings/WriteGitConfigRequest";
import type { WriteGitConfigResponse } from "@operator/bindings/WriteGitConfigResponse";
import type { SetGitSessionEnvRequest } from "@operator/bindings/SetGitSessionEnvRequest";
import type { SetGitSessionEnvResponse } from "@operator/bindings/SetGitSessionEnvResponse";
import type { KanbanProviderCatalogEntry } from "@operator/bindings/KanbanProviderCatalogEntry";
import type { ValidateKanbanCredentialsRequest } from "@operator/bindings/ValidateKanbanCredentialsRequest";
import type { ValidateKanbanCredentialsResponse } from "@operator/bindings/ValidateKanbanCredentialsResponse";
import type { ListKanbanProjectsRequest } from "@operator/bindings/ListKanbanProjectsRequest";
import type { ListKanbanProjectsResponse } from "@operator/bindings/ListKanbanProjectsResponse";
import type { ListKanbanStatusesRequest } from "@operator/bindings/ListKanbanStatusesRequest";
import type { ListKanbanStatusesResponse } from "@operator/bindings/ListKanbanStatusesResponse";
import type { WriteKanbanConfigRequest } from "@operator/bindings/WriteKanbanConfigRequest";
import type { WriteKanbanConfigResponse } from "@operator/bindings/WriteKanbanConfigResponse";
import type { SetKanbanSessionEnvRequest } from "@operator/bindings/SetKanbanSessionEnvRequest";
import type { SetKanbanSessionEnvResponse } from "@operator/bindings/SetKanbanSessionEnvResponse";

import type { AccessKeyListResponse } from "@operator/bindings/AccessKeyListResponse";
import type { BootstrapStatusResponse } from "@operator/bindings/BootstrapStatusResponse";
import type { BootstrapSubmitRequest } from "@operator/bindings/BootstrapSubmitRequest";
import type { BootstrapSubmitResponse } from "@operator/bindings/BootstrapSubmitResponse";
import type { CreateAccessKeyRequest } from "@operator/bindings/CreateAccessKeyRequest";
import type { CreateAccessKeyResponse } from "@operator/bindings/CreateAccessKeyResponse";
import type { CsrfTokenResponse } from "@operator/bindings/CsrfTokenResponse";
import type { CurrentSessionResponse } from "@operator/bindings/CurrentSessionResponse";
import type { DeviceApprovalRequest } from "@operator/bindings/DeviceApprovalRequest";
import type { DeviceApprovalResponse } from "@operator/bindings/DeviceApprovalResponse";
import type { LoginRequest } from "@operator/bindings/LoginRequest";
import type { LoginResponse } from "@operator/bindings/LoginResponse";
import type { LogoutResponse } from "@operator/bindings/LogoutResponse";
import type { ForgotPasswordRequest } from "@operator/bindings/ForgotPasswordRequest";
import type { ForgotPasswordResponse } from "@operator/bindings/ForgotPasswordResponse";
import type { ResetPasswordRequest } from "@operator/bindings/ResetPasswordRequest";
import type { ResetPasswordResponse } from "@operator/bindings/ResetPasswordResponse";
import type { RevokeAccessKeyResponse } from "@operator/bindings/RevokeAccessKeyResponse";
import type { SessionListResponse } from "@operator/bindings/SessionListResponse";

export type { ProfileSummary, LicenseResponse, TargetResponse, TargetsResponse };

export type {
  AccessKeyListResponse,
  BootstrapStatusResponse,
  CreateAccessKeyResponse,
  CurrentSessionResponse,
  LoginResponse,
  SessionListResponse,
  HealthResponse,
  StatusResponse,
  SectionDto,
  SectionRowDto,
  QueueStatusResponse,
  KanbanBoardResponse,
  KanbanTicketCard,
  ActiveAgentsResponse,
  CreateTicketRequest,
  CreateTicketResponse,
  IssueTypeSummary,
  IssueTypeResponse,
  CollectionResponse,
  ProjectSummary,
  CompletedTicket,
  CreateIssueTypeRequest,
  UpdateIssueTypeRequest,
  LaunchTicketRequest,
  LaunchTicketResponse,
  QueueControlResponse,
  ConfigurationResponse,
  UpdateConfigurationRequest,
  ExecutionTargetsResponse,
  LlmToolsResponse,
  AgentDetailResponse,
  WorkflowExportResponse,
  WorkflowPreviewResponse,
  ModelServerKindEntry,
  ModelServerModelsResponse,
  ModelServersResponse,
  ModelServerResponse,
  CreateModelServerRequest,
  DelegatorsResponse,
  DelegatorResponse,
  CreateDelegatorRequest,
  IntegrationCatalogEntryDto,
  SetupStatusResponse,
  SetupStepResponse,
  SetupCollectionResponse,
  SetupInitializeRequest,
  SetupInitializeResponse,
  GitProviderOnboardingResponse,
  ValidateGitTokenResponse,
  WriteGitConfigResponse,
  SetGitSessionEnvResponse,
  KanbanProviderCatalogEntry,
  ValidateKanbanCredentialsResponse,
  ListKanbanProjectsResponse,
  ListKanbanStatusesResponse,
  WriteKanbanConfigResponse,
  SetKanbanSessionEnvResponse,
};

export class ApiError extends Error {
  status: number;
  code?: string;
  feature?: string;
  requiredTier?: string;
  constructor(
    status: number,
    message: string,
    details?: { code?: string; feature?: string; required_tier?: string },
  ) {
    super(message);
    this.status = status;
    this.code = details?.code;
    this.feature = details?.feature;
    this.requiredTier = details?.required_tier;
  }
}

/**
 * The API is authenticated, so every call goes through here.
 *
 * Three things are added centrally rather than per call site:
 *
 * - `credentials: 'same-origin'` so the session cookie is actually sent. The
 *   cookie is `HttpOnly`, so script cannot read or attach it by hand.
 * - The CSRF header on mutations. The cookie rides along automatically, so a
 *   mutation needs proof the request was intended.
 * - A `401` handler that redirects to login (or setup, on a server with no
 *   admin account yet) instead of surfacing an error the user cannot act on.
 */
const CSRF_HEADER = "x-operator-csrf";

/** In-memory only: a CSRF token in localStorage outlives the session it belongs to. */
let csrfToken: string | null = null;

export function setCsrfToken(token: string | null): void {
  csrfToken = token;
}

export function getCsrfToken(): string | null {
  return csrfToken;
}

function isMutation(method: string | undefined): boolean {
  const m = (method ?? "GET").toUpperCase();
  return m !== "GET" && m !== "HEAD" && m !== "OPTIONS";
}

/** Send the user somewhere they can actually authenticate. */
async function redirectToAuth(base: string): Promise<void> {
  let target = "#/login";
  try {
    const res = await fetch(`${base}/api/v1/auth/bootstrap`, { credentials: "same-origin" });
    if (res.ok) {
      const status = (await res.json()) as { state?: string };
      if (status.state !== "complete") {
        target = "#/setup";
      }
    }
  } catch {
    // Unreachable server: login is still the right place to land.
  }
  if (window.location.hash !== target) {
    window.location.hash = target;
  }
}

/**
 * `JSON.stringify` for request bodies that may contain `BigInt`.
 *
 * Rust `u64` fields generate as TypeScript `bigint`, and `JSON.stringify`
 * *throws* on a BigInt rather than serializing it - so a body containing one
 * fails before the request is ever sent, with no server-side trace. Numbers of
 * this kind (a day count, a seconds value) are far inside the safe-integer
 * range, so emitting them as JSON numbers is both correct and what the server
 * expects.
 */
export function toJson(value: unknown): string {
  return JSON.stringify(value, (_key, v) => (typeof v === "bigint" ? Number(v) : (v as unknown)));
}

function authInit(init?: RequestInit): RequestInit {
  const headers = new Headers(init?.headers);
  if (isMutation(init?.method) && csrfToken) {
    headers.set(CSRF_HEADER, csrfToken);
  }
  return { ...init, headers, credentials: "same-origin" };
}

type ApiConnection = { origin: string; profileId?: string; signal?: AbortSignal };

export function profileApiPath(path: string, profileId?: string): string {
  if (!profileId || /^\/api\/v1\/(auth|health|integrations|profiles)(\/|$)/.test(path)) {
    return path;
  }
  return path.replace("/api/v1/", `/api/v1/profiles/${encodeURIComponent(profileId)}/`);
}

async function send(
  connection: string | ApiConnection,
  path: string,
  init?: RequestInit,
): Promise<Response> {
  const base = typeof connection === "string" ? connection : connection.origin;
  const signal = typeof connection === "string" ? undefined : connection.signal;
  const scopedPath = profileApiPath(
    path,
    typeof connection === "string" ? undefined : connection.profileId,
  );
  signal?.throwIfAborted();
  const res = await fetch(
    `${base}${scopedPath}`,
    authInit({ ...init, signal: signal ?? init?.signal }),
  );
  signal?.throwIfAborted();
  if (res.status === 401) {
    await redirectToAuth(base);
  }
  if (!res.ok) {
    const body = await res.json().catch(() => ({ message: `HTTP ${res.status}` }));
    throw new ApiError(res.status, body.message ?? body.error ?? `HTTP ${res.status}`, body);
  }
  return res;
}

async function request<T>(
  base: string | ApiConnection,
  path: string,
  init?: RequestInit,
): Promise<T> {
  const res = await send(base, path, init);
  const result = (await res.json()) as T;
  if (typeof base !== "string") {
    base.signal?.throwIfAborted();
  }
  return result;
}

async function requestVoid(
  base: string | ApiConnection,
  path: string,
  init?: RequestInit,
): Promise<void> {
  await send(base, path, init);
}

export class OperatorApi {
  private readonly base: ApiConnection;

  constructor(host: Host) {
    this.base = { origin: host.baseUrl(), profileId: host.profileId, signal: host.signal };
  }

  profiles(): Promise<ProfileSummary[]> {
    return request(this.base, "/api/v1/profiles");
  }

  license(): Promise<LicenseResponse> {
    return request(this.base, "/api/v1/license");
  }

  installLicense(license_key: string): Promise<LicenseResponse> {
    return request(this.base, "/api/v1/license", {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: toJson({ license_key }),
    });
  }

  removeLicense(): Promise<LicenseResponse> {
    return request(this.base, "/api/v1/license", { method: "DELETE" });
  }

  targets(): Promise<TargetsResponse> {
    return request(this.base, "/api/v1/targets");
  }

  saveTarget(target: TargetDef, existingName?: string): Promise<TargetResponse> {
    return request(
      this.base,
      existingName ? `/api/v1/targets/${encodeURIComponent(existingName)}` : "/api/v1/targets",
      {
        method: existingName ? "PUT" : "POST",
        headers: { "Content-Type": "application/json" },
        body: toJson(target),
      },
    );
  }

  removeTarget(name: string): Promise<void> {
    return requestVoid(this.base, `/api/v1/targets/${encodeURIComponent(name)}`, {
      method: "DELETE",
    });
  }

  probeTarget(name: string): Promise<TargetProbeResponse> {
    return request(this.base, `/api/v1/targets/${encodeURIComponent(name)}/probe`, {
      method: "POST",
    });
  }

  createProfile(name: string): Promise<ProfileSummary> {
    return request(this.base, "/api/v1/profiles", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson({ name }),
    });
  }

  renameProfile(id: string, name: string): Promise<ProfileSummary> {
    return request(this.base, `/api/v1/profiles/${encodeURIComponent(id)}`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: toJson({ name }),
    });
  }

  // --- Auth ---

  bootstrapStatus(): Promise<BootstrapStatusResponse> {
    return request(this.base, "/api/v1/auth/bootstrap");
  }

  bootstrap(body: BootstrapSubmitRequest): Promise<BootstrapSubmitResponse> {
    return request(this.base, "/api/v1/auth/bootstrap", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(body),
    });
  }

  async login(username: string, password: string): Promise<LoginResponse> {
    const res = await request<LoginResponse>(this.base, "/api/v1/auth/login", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson({ username, password } satisfies LoginRequest),
    });
    // Every later mutation needs this, so capture it at the one point it is issued.
    setCsrfToken(res.csrf_token);
    return res;
  }

  forgotPassword(username: string): Promise<ForgotPasswordResponse> {
    return request(this.base, "/api/v1/auth/forgot-password", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson({ username } satisfies ForgotPasswordRequest),
    });
  }

  resetPassword(body: ResetPasswordRequest): Promise<ResetPasswordResponse> {
    return request(this.base, "/api/v1/auth/reset-password", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(body),
    });
  }

  async logout(): Promise<LogoutResponse> {
    const res = await request<LogoutResponse>(this.base, "/api/v1/auth/logout", {
      method: "POST",
    });
    setCsrfToken(null);
    return res;
  }

  currentSession(): Promise<CurrentSessionResponse> {
    return request(this.base, "/api/v1/auth/session");
  }

  /** Re-issue a CSRF token, e.g. after a page reload where the cookie survived. */
  async refreshCsrf(): Promise<string> {
    const res = await request<CsrfTokenResponse>(this.base, "/api/v1/auth/csrf");
    setCsrfToken(res.csrf_token);
    return res.csrf_token;
  }

  listSessions(): Promise<SessionListResponse> {
    return request(this.base, "/api/v1/auth/sessions");
  }

  revokeSession(id: string): Promise<LogoutResponse> {
    return request(this.base, `/api/v1/auth/sessions/${encodeURIComponent(id)}`, {
      method: "DELETE",
    });
  }

  listAccessKeys(): Promise<AccessKeyListResponse> {
    return request(this.base, "/api/v1/auth/keys");
  }

  createAccessKey(body: CreateAccessKeyRequest): Promise<CreateAccessKeyResponse> {
    return request(this.base, "/api/v1/auth/keys", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(body),
    });
  }

  revokeAccessKey(id: string): Promise<RevokeAccessKeyResponse> {
    return request(this.base, `/api/v1/auth/keys/${encodeURIComponent(id)}`, {
      method: "DELETE",
    });
  }

  approveDevice(userCode: string): Promise<DeviceApprovalResponse> {
    return request(this.base, "/api/v1/auth/device/approve", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson({ user_code: userCode } satisfies DeviceApprovalRequest),
    });
  }

  // --- Health ---

  health(): Promise<HealthResponse> {
    return request(this.base, "/api/v1/health");
  }

  status(): Promise<StatusResponse> {
    return request(this.base, "/api/v1/status");
  }

  // --- Status sections (canonical, shared with TUI / VS Code) ---

  sections(): Promise<SectionDto[]> {
    return request(this.base, "/api/v1/sections");
  }

  integrations(): Promise<IntegrationCatalogEntryDto[]> {
    return request(this.base, "/api/v1/integrations");
  }

  // --- First-run setup ---

  setupStatus(): Promise<SetupStatusResponse> {
    return request(this.base, "/api/v1/setup/status");
  }

  setupSteps(): Promise<SetupStepResponse[]> {
    return request(this.base, "/api/v1/setup/steps");
  }

  setupCollections(): Promise<SetupCollectionResponse[]> {
    return request(this.base, "/api/v1/setup/collections");
  }

  initializeSetup(body: SetupInitializeRequest): Promise<SetupInitializeResponse> {
    return request(this.base, "/api/v1/setup/initialize", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(body),
    });
  }

  // --- Git onboarding ---

  gitProviders(): Promise<GitProviderOnboardingResponse[]> {
    return request(this.base, "/api/v1/git/providers");
  }

  validateGitToken(body: ValidateGitTokenRequest): Promise<ValidateGitTokenResponse> {
    return request(this.base, "/api/v1/git/validate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(body),
    });
  }

  writeGitConfig(body: WriteGitConfigRequest): Promise<WriteGitConfigResponse> {
    return request(this.base, "/api/v1/git/config", {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: toJson(body),
    });
  }

  setGitSessionEnv(body: SetGitSessionEnvRequest): Promise<SetGitSessionEnvResponse> {
    return request(this.base, "/api/v1/git/session-env", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(body),
    });
  }

  // --- Kanban onboarding ---

  kanbanProviders(): Promise<KanbanProviderCatalogEntry[]> {
    return request(this.base, "/api/v1/kanban/providers");
  }

  validateKanbanCredentials(
    body: ValidateKanbanCredentialsRequest,
  ): Promise<ValidateKanbanCredentialsResponse> {
    return request(this.base, "/api/v1/kanban/validate", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(body),
    });
  }

  listKanbanProjects(body: ListKanbanProjectsRequest): Promise<ListKanbanProjectsResponse> {
    return request(this.base, "/api/v1/kanban/projects", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(body),
    });
  }

  listKanbanStatuses(body: ListKanbanStatusesRequest): Promise<ListKanbanStatusesResponse> {
    return request(this.base, "/api/v1/kanban/statuses", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(body),
    });
  }

  writeKanbanConfig(body: WriteKanbanConfigRequest): Promise<WriteKanbanConfigResponse> {
    return request(this.base, "/api/v1/kanban/config", {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: toJson(body),
    });
  }

  setKanbanSessionEnv(body: SetKanbanSessionEnvRequest): Promise<SetKanbanSessionEnvResponse> {
    return request(this.base, "/api/v1/kanban/session-env", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(body),
    });
  }

  // --- Queue ---

  queueStatus(): Promise<QueueStatusResponse> {
    return request(this.base, "/api/v1/queue/status");
  }

  kanban(): Promise<KanbanBoardResponse> {
    return request(this.base, "/api/v1/queue/kanban");
  }

  pauseQueue(): Promise<QueueControlResponse> {
    return request(this.base, "/api/v1/queue/pause", { method: "POST" });
  }

  resumeQueue(): Promise<QueueControlResponse> {
    return request(this.base, "/api/v1/queue/resume", { method: "POST" });
  }

  syncKanban(): Promise<void> {
    return requestVoid(this.base, "/api/v1/queue/sync", { method: "POST" });
  }

  // --- Agents ---

  activeAgents(): Promise<ActiveAgentsResponse> {
    return request(this.base, "/api/v1/agents/active");
  }

  getAgent(agentId: string): Promise<AgentDetailResponse> {
    return request(this.base, `/api/v1/agents/${encodeURIComponent(agentId)}`);
  }

  approveReview(agentId: string): Promise<void> {
    return requestVoid(this.base, `/api/v1/agents/${encodeURIComponent(agentId)}/approve`, {
      method: "POST",
    });
  }

  rejectReview(agentId: string, reason: string): Promise<void> {
    return requestVoid(this.base, `/api/v1/agents/${encodeURIComponent(agentId)}/reject`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson({ reason }),
    });
  }

  /**
   * Focus the agent's terminal session in its session wrapper. Used for cmux
   * launches: operator (running inside cmux) shells out to `cmux focus-workspace`.
   * cmux has no browser URL scheme, so the control plane is the bridge.
   */
  focusSession(agentId: string): Promise<void> {
    return requestVoid(this.base, `/api/v1/agents/${encodeURIComponent(agentId)}/focus`, {
      method: "POST",
    });
  }

  // --- Tickets ---

  launchTicket(ticketId: string, options: LaunchTicketRequest): Promise<LaunchTicketResponse> {
    return request(this.base, `/api/v1/tickets/${encodeURIComponent(ticketId)}/launch`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(options),
    });
  }

  createTicket(req: CreateTicketRequest): Promise<CreateTicketResponse> {
    return request(this.base, "/api/v1/tickets", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(req),
    });
  }

  // --- Projects ---

  listProjects(): Promise<ProjectSummary[]> {
    return request(this.base, "/api/v1/projects");
  }

  // --- Issue Types ---

  listIssueTypes(): Promise<IssueTypeSummary[]> {
    return request(this.base, "/api/v1/issuetypes");
  }

  getIssueType(key: string): Promise<IssueTypeResponse> {
    return request(this.base, `/api/v1/issuetypes/${encodeURIComponent(key)}`);
  }

  createIssueType(req: CreateIssueTypeRequest): Promise<IssueTypeResponse> {
    return request(this.base, "/api/v1/issuetypes", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(req),
    });
  }

  updateIssueType(key: string, req: UpdateIssueTypeRequest): Promise<IssueTypeResponse> {
    return request(this.base, `/api/v1/issuetypes/${encodeURIComponent(key)}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: toJson(req),
    });
  }

  deleteIssueType(key: string): Promise<void> {
    return requestVoid(this.base, `/api/v1/issuetypes/${encodeURIComponent(key)}`, {
      method: "DELETE",
    });
  }

  // --- Collections ---

  listCollections(): Promise<CollectionResponse[]> {
    return request(this.base, "/api/v1/collections");
  }

  activateCollection(name: string): Promise<void> {
    return requestVoid(this.base, `/api/v1/collections/${encodeURIComponent(name)}/activate`, {
      method: "PUT",
    });
  }

  // --- Configuration ---

  getConfiguration(): Promise<ConfigurationResponse> {
    return request(this.base, "/api/v1/configuration");
  }

  updateConfiguration(config: UpdateConfigurationRequest): Promise<ConfigurationResponse> {
    return request(this.base, "/api/v1/configuration", {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: toJson(config),
    });
  }

  executionTargets(): Promise<ExecutionTargetsResponse> {
    return request(this.base, "/api/v1/execution-targets");
  }

  listLlmTools(): Promise<LlmToolsResponse> {
    return request(this.base, "/api/v1/llm-tools");
  }

  // --- Model providers ---

  /** The catalog of supported model providers (kinds). */
  listProviderKinds(): Promise<ModelServerKindEntry[]> {
    return request(this.base, "/api/v1/model-servers/kinds");
  }

  /** Live models for a provider kind (probes declared instance or defaults). */
  providerModels(slug: string): Promise<ModelServerModelsResponse> {
    return request(this.base, `/api/v1/model-servers/kinds/${encodeURIComponent(slug)}/models`);
  }

  /** Declared servers + implicit builtins. */
  listModelServers(): Promise<ModelServersResponse> {
    return request(this.base, "/api/v1/model-servers");
  }

  /** Connect a gateway provider by declaring an instance. */
  createModelServer(req: CreateModelServerRequest): Promise<ModelServerResponse> {
    return request(this.base, "/api/v1/model-servers", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(req),
    });
  }

  // --- Delegators ---

  listDelegators(): Promise<DelegatorsResponse> {
    return request(this.base, "/api/v1/delegators");
  }

  createDelegator(req: CreateDelegatorRequest): Promise<DelegatorResponse> {
    return request(this.base, "/api/v1/delegators", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: toJson(req),
    });
  }

  updateDelegator(name: string, req: CreateDelegatorRequest): Promise<DelegatorResponse> {
    return request(this.base, `/api/v1/delegators/${encodeURIComponent(name)}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: toJson(req),
    });
  }

  // --- Operator workflows ---

  /**
   * Fetch an issue type's Operator workflow document - the native step graph,
   * in the same shape as a hosted collection's `<KEY>.json`. The export endpoints
   * below produce *other* runners' lossy formats from this same source.
   */
  getIssueTypeDocument(key: string): Promise<IssueType> {
    return request(this.base, `/api/v1/issuetypes/${encodeURIComponent(key)}/document`);
  }

  // --- Workflow export formats ---

  /** Export a ticket (rendered against its issue type) to a Claude dynamic workflow (.js). */
  exportWorkflow(ticketId: string): Promise<WorkflowExportResponse> {
    return request(this.base, `/api/v1/tickets/${encodeURIComponent(ticketId)}/workflow-export`, {
      method: "POST",
    });
  }

  /** Preview an issue type's workflow shape (.js, placeholder values) for visualization. */
  previewWorkflow(key: string): Promise<WorkflowPreviewResponse> {
    return request(this.base, `/api/v1/issuetypes/${encodeURIComponent(key)}/workflow-preview`);
  }
}
