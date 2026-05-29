import type { Host } from './host';
import type { HealthResponse } from '@operator/bindings/HealthResponse';
import type { StatusResponse } from '@operator/bindings/StatusResponse';
import type { SectionDto } from '@operator/bindings/SectionDto';
import type { SectionRowDto } from '@operator/bindings/SectionRowDto';
import type { QueueStatusResponse } from '@operator/bindings/QueueStatusResponse';
import type { KanbanBoardResponse } from '@operator/bindings/KanbanBoardResponse';
import type { KanbanTicketCard } from '@operator/bindings/KanbanTicketCard';
import type { ActiveAgentsResponse } from '@operator/bindings/ActiveAgentsResponse';
import type { IssueTypeSummary } from '@operator/bindings/IssueTypeSummary';
import type { IssueTypeResponse } from '@operator/bindings/IssueTypeResponse';
import type { CollectionResponse } from '@operator/bindings/CollectionResponse';
import type { ProjectSummary } from '@operator/bindings/ProjectSummary';
import type { CompletedTicket } from '@operator/bindings/CompletedTicket';
import type { CreateIssueTypeRequest } from '@operator/bindings/CreateIssueTypeRequest';
import type { UpdateIssueTypeRequest } from '@operator/bindings/UpdateIssueTypeRequest';
import type { LaunchTicketRequest } from '@operator/bindings/LaunchTicketRequest';
import type { LaunchTicketResponse } from '@operator/bindings/LaunchTicketResponse';
import type { QueueControlResponse } from '@operator/bindings/QueueControlResponse';
import type { Config } from '@operator/bindings/Config';
import type { AgentDetailResponse } from '@operator/bindings/AgentDetailResponse';
import type { WorkflowExportResponse } from '@operator/bindings/WorkflowExportResponse';

export type {
  HealthResponse,
  StatusResponse,
  SectionDto,
  SectionRowDto,
  QueueStatusResponse,
  KanbanBoardResponse,
  KanbanTicketCard,
  ActiveAgentsResponse,
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
  Config,
  AgentDetailResponse,
  WorkflowExportResponse,
};

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

async function request<T>(base: string, path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${base}${path}`, init);
  if (!res.ok) {
    const body = await res.json().catch(() => ({ message: `HTTP ${res.status}` }));
    throw new ApiError(res.status, body.message ?? body.error ?? `HTTP ${res.status}`);
  }
  return res.json() as Promise<T>;
}

async function requestVoid(base: string, path: string, init?: RequestInit): Promise<void> {
  const res = await fetch(`${base}${path}`, init);
  if (!res.ok) {
    const body = await res.json().catch(() => ({ message: `HTTP ${res.status}` }));
    throw new ApiError(res.status, body.message ?? body.error ?? `HTTP ${res.status}`);
  }
}

export class OperatorApi {
  private base: string;

  constructor(host: Host) {
    this.base = host.baseUrl();
  }

  // --- Health ---

  health(): Promise<HealthResponse> {
    return request(this.base, '/api/v1/health');
  }

  status(): Promise<StatusResponse> {
    return request(this.base, '/api/v1/status');
  }

  // --- Status sections (canonical, shared with TUI / VS Code) ---

  sections(): Promise<SectionDto[]> {
    return request(this.base, '/api/v1/sections');
  }

  // --- Queue ---

  queueStatus(): Promise<QueueStatusResponse> {
    return request(this.base, '/api/v1/queue/status');
  }

  kanban(): Promise<KanbanBoardResponse> {
    return request(this.base, '/api/v1/queue/kanban');
  }

  pauseQueue(): Promise<QueueControlResponse> {
    return request(this.base, '/api/v1/queue/pause', { method: 'POST' });
  }

  resumeQueue(): Promise<QueueControlResponse> {
    return request(this.base, '/api/v1/queue/resume', { method: 'POST' });
  }

  syncKanban(): Promise<void> {
    return requestVoid(this.base, '/api/v1/queue/sync', { method: 'POST' });
  }

  // --- Agents ---

  activeAgents(): Promise<ActiveAgentsResponse> {
    return request(this.base, '/api/v1/agents/active');
  }

  getAgent(agentId: string): Promise<AgentDetailResponse> {
    return request(this.base, `/api/v1/agents/${encodeURIComponent(agentId)}`);
  }

  approveReview(agentId: string): Promise<void> {
    return requestVoid(this.base, `/api/v1/agents/${encodeURIComponent(agentId)}/approve`, {
      method: 'POST',
    });
  }

  rejectReview(agentId: string, reason: string): Promise<void> {
    return requestVoid(this.base, `/api/v1/agents/${encodeURIComponent(agentId)}/reject`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ reason }),
    });
  }

  // --- Tickets ---

  launchTicket(ticketId: string, options: LaunchTicketRequest): Promise<LaunchTicketResponse> {
    return request(this.base, `/api/v1/tickets/${encodeURIComponent(ticketId)}/launch`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(options),
    });
  }

  // --- Projects ---

  listProjects(): Promise<ProjectSummary[]> {
    return request(this.base, '/api/v1/projects');
  }

  // --- Issue Types ---

  listIssueTypes(): Promise<IssueTypeSummary[]> {
    return request(this.base, '/api/v1/issuetypes');
  }

  getIssueType(key: string): Promise<IssueTypeResponse> {
    return request(this.base, `/api/v1/issuetypes/${encodeURIComponent(key)}`);
  }

  createIssueType(req: CreateIssueTypeRequest): Promise<IssueTypeResponse> {
    return request(this.base, '/api/v1/issuetypes', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });
  }

  updateIssueType(key: string, req: UpdateIssueTypeRequest): Promise<IssueTypeResponse> {
    return request(this.base, `/api/v1/issuetypes/${encodeURIComponent(key)}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    });
  }

  deleteIssueType(key: string): Promise<void> {
    return requestVoid(this.base, `/api/v1/issuetypes/${encodeURIComponent(key)}`, {
      method: 'DELETE',
    });
  }

  // --- Collections ---

  listCollections(): Promise<CollectionResponse[]> {
    return request(this.base, '/api/v1/collections');
  }

  activateCollection(name: string): Promise<void> {
    return requestVoid(this.base, `/api/v1/collections/${encodeURIComponent(name)}/activate`, {
      method: 'PUT',
    });
  }

  // --- Configuration ---

  getConfiguration(): Promise<Config> {
    return request(this.base, '/api/v1/configuration');
  }

  updateConfiguration(config: Partial<Config>): Promise<Config> {
    return request(this.base, '/api/v1/configuration', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
  }

  // --- Workflow export ---

  /** Export a ticket (rendered against its issue type) to a Claude dynamic workflow (.js). */
  exportWorkflow(ticketId: string): Promise<WorkflowExportResponse> {
    return request(
      this.base,
      `/api/v1/tickets/${encodeURIComponent(ticketId)}/workflow-export`,
      { method: 'POST' },
    );
  }
}
