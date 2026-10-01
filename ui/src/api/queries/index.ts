export const QUERY_KEYS = {
  profiles: "profiles",
  currentSession: "current-session",
  bootstrapStatus: "bootstrap-status",
  setupStatus: "setup-status",
  setupSteps: "setup-steps",
  setupCollections: "setup-collections",
  integrations: "integrations",
  health: "health",
  status: "status",
  sections: "sections",
  queueStatus: "queue-status",
  kanban: "kanban",
  activeAgents: "active-agents",
  agent: "agent",
  configuration: "configuration",
  executionTargets: "execution-targets",
  llmTools: "llm-tools",
  issueTypes: "issue-types",
  issueType: "issue-type",
  issueTypeDocument: "issue-type-document",
  collections: "collections",
  projects: "projects",
  license: "license",
  targets: "targets",
  sessions: "sessions",
  accessKeys: "access-keys",
  providerKinds: "provider-kinds",
  providerModels: "provider-models",
  modelServers: "model-servers",
  delegators: "delegators",
  gitProviders: "git-providers",
  kanbanProviders: "kanban-providers",
} as const;

export type QueryKey = (typeof QUERY_KEYS)[keyof typeof QUERY_KEYS];

export const STATUS_POLL_MS = 3000;
export const AGENT_POLL_MS = 5000;

export const TICKET_QUERY_KEYS = [
  QUERY_KEYS.kanban,
  QUERY_KEYS.queueStatus,
  QUERY_KEYS.activeAgents,
  QUERY_KEYS.agent,
  QUERY_KEYS.sections,
] as const;

export const PROFILE_QUERY_KEYS = [
  QUERY_KEYS.profiles,
  QUERY_KEYS.setupStatus,
  QUERY_KEYS.configuration,
  QUERY_KEYS.status,
  QUERY_KEYS.sections,
] as const;

export const LICENSE_QUERY_KEYS = [
  QUERY_KEYS.license,
  QUERY_KEYS.targets,
  QUERY_KEYS.executionTargets,
] as const;

export const ISSUE_TYPE_QUERY_KEYS = [
  QUERY_KEYS.issueTypes,
  QUERY_KEYS.issueType,
  QUERY_KEYS.issueTypeDocument,
  QUERY_KEYS.collections,
  QUERY_KEYS.status,
] as const;

export const SESSION_QUERY_KEYS = [
  QUERY_KEYS.currentSession,
  QUERY_KEYS.sessions,
  QUERY_KEYS.accessKeys,
] as const;

export const MODEL_QUERY_KEYS = [
  QUERY_KEYS.providerModels,
  QUERY_KEYS.modelServers,
  QUERY_KEYS.delegators,
  QUERY_KEYS.configuration,
] as const;

export const ALL_QUERY_KEYS = Object.values(QUERY_KEYS);
