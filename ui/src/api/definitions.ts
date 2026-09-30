import type { CreateAccessKeyRequest } from "@operator/bindings/CreateAccessKeyRequest";
import type { CreateDelegatorRequest } from "@operator/bindings/CreateDelegatorRequest";
import type { CreateIssueTypeRequest } from "@operator/bindings/CreateIssueTypeRequest";
import type { CreateModelServerRequest } from "@operator/bindings/CreateModelServerRequest";
import type { CreateTicketRequest } from "@operator/bindings/CreateTicketRequest";
import type { LaunchTicketRequest } from "@operator/bindings/LaunchTicketRequest";
import type { ListKanbanProjectsRequest } from "@operator/bindings/ListKanbanProjectsRequest";
import type { ListKanbanStatusesRequest } from "@operator/bindings/ListKanbanStatusesRequest";
import type { ResetPasswordRequest } from "@operator/bindings/ResetPasswordRequest";
import type { SetGitSessionEnvRequest } from "@operator/bindings/SetGitSessionEnvRequest";
import type { SetKanbanSessionEnvRequest } from "@operator/bindings/SetKanbanSessionEnvRequest";
import type { SetupInitializeRequest } from "@operator/bindings/SetupInitializeRequest";
import type { TargetDef } from "@operator/bindings/TargetDef";
import type { UpdateConfigurationRequest } from "@operator/bindings/UpdateConfigurationRequest";
import type { UpdateIssueTypeRequest } from "@operator/bindings/UpdateIssueTypeRequest";
import type { ValidateGitTokenRequest } from "@operator/bindings/ValidateGitTokenRequest";
import type { ValidateKanbanCredentialsRequest } from "@operator/bindings/ValidateKanbanCredentialsRequest";
import type { WriteGitConfigRequest } from "@operator/bindings/WriteGitConfigRequest";
import type { WriteKanbanConfigRequest } from "@operator/bindings/WriteKanbanConfigRequest";
import type { BootstrapSubmitRequest } from "@operator/bindings/BootstrapSubmitRequest";
import type { OperatorApi } from "../api-client";
import type { MutationDefinition, QueryDefinition } from "./adapter";
import {
  ISSUE_TYPE_QUERY_KEYS,
  LICENSE_QUERY_KEYS,
  MODEL_QUERY_KEYS,
  PROFILE_QUERY_KEYS,
  QUERY_KEYS,
  SESSION_QUERY_KEYS,
  TICKET_QUERY_KEYS,
} from "./queries";

type ApiResult<K extends keyof OperatorApi> = Awaited<ReturnType<OperatorApi[K]>>;

const NONE: Record<string, never> = {};

function read<Data>(key: string, fetch: QueryDefinition<Data>["fetch"]): QueryDefinition<Data> {
  return { key, params: NONE, fetch };
}

function readWith<Data, Params extends object>(
  key: string,
  params: Params,
  fetch: QueryDefinition<Data, Params>["fetch"],
): QueryDefinition<Data, Params> {
  return { key, params, fetch };
}

function staticRead<Data>(
  key: string,
  fetch: QueryDefinition<Data>["fetch"],
): () => QueryDefinition<Data> {
  const definition = read(key, fetch);
  return () => definition;
}

function memoizeDefinition<Input, Definition>(
  create: (input: Input) => Definition,
): (input: Input) => Definition {
  const definitions = new Map<Input, Definition>();
  return (input) => {
    const existing = definitions.get(input);
    if (existing) {
      return existing;
    }
    const definition = create(input);
    definitions.set(input, definition);
    return definition;
  };
}

export const profilesQuery = staticRead(QUERY_KEYS.profiles, (api) => api.profiles());

export const currentSessionQuery = staticRead(QUERY_KEYS.currentSession, (api) =>
  api.currentSession(),
);

export const bootstrapStatusQuery = staticRead(QUERY_KEYS.bootstrapStatus, (api) =>
  api.bootstrapStatus(),
);

export const setupStatusQuery = staticRead(QUERY_KEYS.setupStatus, (api) => api.setupStatus());

export const setupStepsQuery = staticRead(QUERY_KEYS.setupSteps, (api) => api.setupSteps());

export const setupCollectionsQuery = staticRead(QUERY_KEYS.setupCollections, (api) =>
  api.setupCollections(),
);

export const integrationsQuery = staticRead(QUERY_KEYS.integrations, (api) => api.integrations());

export const healthQuery = staticRead(QUERY_KEYS.health, (api) => api.health());

export const statusQuery = staticRead(QUERY_KEYS.status, (api) => api.status());

export const sectionsQuery = staticRead(QUERY_KEYS.sections, (api) => api.sections());

export const queueStatusQuery = staticRead(QUERY_KEYS.queueStatus, (api) => api.queueStatus());

export const kanbanQuery = staticRead(QUERY_KEYS.kanban, (api) => api.kanban());

export const activeAgentsQuery = staticRead(QUERY_KEYS.activeAgents, (api) => api.activeAgents());

export const agentQuery = memoizeDefinition((agentId: string) =>
  readWith(QUERY_KEYS.agent, { agentId }, (api, params) => api.getAgent(params.agentId)),
);

export const configurationQuery = staticRead(QUERY_KEYS.configuration, (api) =>
  api.getConfiguration(),
);

export const executionTargetsQuery = staticRead(QUERY_KEYS.executionTargets, (api) =>
  api.executionTargets(),
);

export const llmToolsQuery = staticRead(QUERY_KEYS.llmTools, (api) => api.listLlmTools());

export const issueTypesQuery = staticRead(QUERY_KEYS.issueTypes, (api) => api.listIssueTypes());

export const issueTypeQuery = memoizeDefinition((key: string) =>
  readWith(QUERY_KEYS.issueType, { key }, (api, params) => api.getIssueType(params.key)),
);

export const issueTypeDocumentQuery = memoizeDefinition((key: string) =>
  readWith(QUERY_KEYS.issueTypeDocument, { key }, (api, params) =>
    api.getIssueTypeDocument(params.key),
  ),
);

export const collectionsQuery = staticRead(QUERY_KEYS.collections, (api) => api.listCollections());

export const projectsQuery = staticRead(QUERY_KEYS.projects, (api) => api.listProjects());

export const licenseQuery = staticRead(QUERY_KEYS.license, (api) => api.license());

export const targetsQuery = staticRead(QUERY_KEYS.targets, (api) => api.targets());

export const sessionsQuery = staticRead(QUERY_KEYS.sessions, (api) => api.listSessions());

export const accessKeysQuery = staticRead(QUERY_KEYS.accessKeys, (api) => api.listAccessKeys());

export const providerKindsQuery = staticRead(QUERY_KEYS.providerKinds, (api) =>
  api.listProviderKinds(),
);

export const providerModelsQuery = memoizeDefinition((slug: string) =>
  readWith(QUERY_KEYS.providerModels, { slug }, (api, params) => api.providerModels(params.slug)),
);

export const modelServersQuery = staticRead(QUERY_KEYS.modelServers, (api) =>
  api.listModelServers(),
);

export const delegatorsQuery = staticRead(QUERY_KEYS.delegators, (api) => api.listDelegators());

export const gitProvidersQuery = staticRead(QUERY_KEYS.gitProviders, (api) => api.gitProviders());

export const kanbanProvidersQuery = staticRead(QUERY_KEYS.kanbanProviders, (api) =>
  api.kanbanProviders(),
);

export const loginMutation: MutationDefinition<
  ApiResult<"login">,
  { username: string; password: string }
> = {
  key: "login",
  affected: [],
  sensitive: true,
  run: (api, variables) => api.login(variables.username, variables.password),
};

export const logoutMutation: MutationDefinition<ApiResult<"logout">, Record<string, never>> = {
  key: "logout",
  affected: [],
  run: (api) => api.logout(),
};

export const bootstrapMutation: MutationDefinition<
  ApiResult<"bootstrap">,
  BootstrapSubmitRequest
> = {
  key: "bootstrap",
  affected: [QUERY_KEYS.bootstrapStatus],
  sensitive: true,
  run: (api, variables) => api.bootstrap(variables),
};

export const forgotPasswordMutation: MutationDefinition<
  ApiResult<"forgotPassword">,
  { username: string }
> = {
  key: "forgot-password",
  affected: [],
  run: (api, variables) => api.forgotPassword(variables.username),
};

export const resetPasswordMutation: MutationDefinition<
  ApiResult<"resetPassword">,
  ResetPasswordRequest
> = {
  key: "reset-password",
  affected: [],
  sensitive: true,
  run: (api, variables) => api.resetPassword(variables),
};

export const approveDeviceMutation: MutationDefinition<
  ApiResult<"approveDevice">,
  { userCode: string }
> = {
  key: "approve-device",
  affected: [...SESSION_QUERY_KEYS],
  run: (api, variables) => api.approveDevice(variables.userCode),
};

export const createAccessKeyMutation: MutationDefinition<
  ApiResult<"createAccessKey">,
  CreateAccessKeyRequest
> = {
  key: "create-access-key",
  affected: [QUERY_KEYS.accessKeys],
  run: (api, variables) => api.createAccessKey(variables),
};

export const revokeAccessKeyMutation: MutationDefinition<
  ApiResult<"revokeAccessKey">,
  { id: string }
> = {
  key: "revoke-access-key",
  affected: [QUERY_KEYS.accessKeys],
  run: (api, variables) => api.revokeAccessKey(variables.id),
};

export const revokeSessionMutation: MutationDefinition<
  ApiResult<"revokeSession">,
  { id: string }
> = {
  key: "revoke-session",
  affected: [QUERY_KEYS.sessions, QUERY_KEYS.currentSession],
  run: (api, variables) => api.revokeSession(variables.id),
};

export const createProfileMutation: MutationDefinition<
  ApiResult<"createProfile">,
  { name: string }
> = {
  key: "create-profile",
  affected: [QUERY_KEYS.profiles],
  run: (api, variables) => api.createProfile(variables.name),
};

export const renameProfileMutation: MutationDefinition<
  ApiResult<"renameProfile">,
  { id: string; name: string }
> = {
  key: "rename-profile",
  affected: [...PROFILE_QUERY_KEYS],
  run: (api, variables) => api.renameProfile(variables.id, variables.name),
};

export const initializeSetupMutation: MutationDefinition<
  ApiResult<"initializeSetup">,
  SetupInitializeRequest
> = {
  key: "initialize-setup",
  affected: [
    QUERY_KEYS.profiles,
    QUERY_KEYS.setupStatus,
    QUERY_KEYS.setupSteps,
    QUERY_KEYS.setupCollections,
    QUERY_KEYS.integrations,
    QUERY_KEYS.configuration,
    QUERY_KEYS.status,
    QUERY_KEYS.sections,
    QUERY_KEYS.issueTypes,
    QUERY_KEYS.collections,
    QUERY_KEYS.license,
    QUERY_KEYS.targets,
    QUERY_KEYS.executionTargets,
    QUERY_KEYS.providerKinds,
    QUERY_KEYS.modelServers,
    QUERY_KEYS.delegators,
  ],
  run: (api, variables) => api.initializeSetup(variables),
};

export const installLicenseMutation: MutationDefinition<
  ApiResult<"installLicense">,
  { licenseKey: string }
> = {
  key: "install-license",
  affected: [...LICENSE_QUERY_KEYS],
  sensitive: true,
  run: (api, variables) => api.installLicense(variables.licenseKey),
};

export const removeLicenseMutation: MutationDefinition<
  ApiResult<"removeLicense">,
  Record<string, never>
> = {
  key: "remove-license",
  affected: [...LICENSE_QUERY_KEYS],
  run: (api) => api.removeLicense(),
};

export const saveTargetMutation: MutationDefinition<
  ApiResult<"saveTarget">,
  { target: TargetDef; existingName?: string }
> = {
  key: "save-target",
  affected: [...LICENSE_QUERY_KEYS],
  run: (api, variables) => api.saveTarget(variables.target, variables.existingName),
};

export const removeTargetMutation: MutationDefinition<
  ApiResult<"removeTarget">,
  { name: string }
> = {
  key: "remove-target",
  affected: [...LICENSE_QUERY_KEYS],
  run: (api, variables) => api.removeTarget(variables.name),
};

export const probeTargetMutation: MutationDefinition<ApiResult<"probeTarget">, { name: string }> = {
  key: "probe-target",
  affected: [QUERY_KEYS.targets],
  run: (api, variables) => api.probeTarget(variables.name),
};

export const pauseQueueMutation: MutationDefinition<
  ApiResult<"pauseQueue">,
  Record<string, never>
> = {
  key: "pause-queue",
  affected: [...TICKET_QUERY_KEYS],
  run: (api) => api.pauseQueue(),
};

export const resumeQueueMutation: MutationDefinition<
  ApiResult<"resumeQueue">,
  Record<string, never>
> = {
  key: "resume-queue",
  affected: [...TICKET_QUERY_KEYS],
  run: (api) => api.resumeQueue(),
};

export const syncKanbanMutation: MutationDefinition<void, Record<string, never>> = {
  key: "sync-kanban",
  affected: [...TICKET_QUERY_KEYS],
  run: (api) => api.syncKanban(),
};

export const createTicketMutation: MutationDefinition<
  ApiResult<"createTicket">,
  CreateTicketRequest
> = {
  key: "create-ticket",
  affected: [...TICKET_QUERY_KEYS],
  run: (api, variables) => api.createTicket(variables),
};

export const launchTicketMutation: MutationDefinition<
  ApiResult<"launchTicket">,
  { ticketId: string; options: LaunchTicketRequest }
> = {
  key: "launch-ticket",
  affected: [...TICKET_QUERY_KEYS],
  run: (api, variables) => api.launchTicket(variables.ticketId, variables.options),
};

export const approveReviewMutation: MutationDefinition<void, { agentId: string }> = {
  key: "approve-review",
  affected: [...TICKET_QUERY_KEYS],
  run: (api, variables) => api.approveReview(variables.agentId),
};

export const rejectReviewMutation: MutationDefinition<void, { agentId: string; reason: string }> = {
  key: "reject-review",
  affected: [...TICKET_QUERY_KEYS],
  run: (api, variables) => api.rejectReview(variables.agentId, variables.reason),
};

export const focusSessionMutation: MutationDefinition<void, { agentId: string }> = {
  key: "focus-session",
  affected: [QUERY_KEYS.activeAgents, QUERY_KEYS.agent],
  run: (api, variables) => api.focusSession(variables.agentId),
};

export const createIssueTypeMutation: MutationDefinition<
  ApiResult<"createIssueType">,
  CreateIssueTypeRequest
> = {
  key: "create-issue-type",
  affected: [...ISSUE_TYPE_QUERY_KEYS],
  run: (api, variables) => api.createIssueType(variables),
};

export const updateIssueTypeMutation: MutationDefinition<
  ApiResult<"updateIssueType">,
  { key: string; request: UpdateIssueTypeRequest }
> = {
  key: "update-issue-type",
  affected: [...ISSUE_TYPE_QUERY_KEYS],
  run: (api, variables) => api.updateIssueType(variables.key, variables.request),
};

export const deleteIssueTypeMutation: MutationDefinition<void, { key: string }> = {
  key: "delete-issue-type",
  affected: [...ISSUE_TYPE_QUERY_KEYS],
  run: (api, variables) => api.deleteIssueType(variables.key),
};

export const activateCollectionMutation: MutationDefinition<void, { name: string }> = {
  key: "activate-collection",
  affected: [...ISSUE_TYPE_QUERY_KEYS, QUERY_KEYS.status],
  run: (api, variables) => api.activateCollection(variables.name),
};

export const updateConfigurationMutation: MutationDefinition<
  ApiResult<"updateConfiguration">,
  UpdateConfigurationRequest
> = {
  key: "update-configuration",
  affected: [...PROFILE_QUERY_KEYS],
  run: (api, variables) => api.updateConfiguration(variables),
};

export const createModelServerMutation: MutationDefinition<
  ApiResult<"createModelServer">,
  CreateModelServerRequest
> = {
  key: "create-model-server",
  affected: [...MODEL_QUERY_KEYS],
  sensitive: true,
  run: (api, variables) => api.createModelServer(variables),
};

export const createDelegatorMutation: MutationDefinition<
  ApiResult<"createDelegator">,
  CreateDelegatorRequest
> = {
  key: "create-delegator",
  affected: [...MODEL_QUERY_KEYS],
  run: (api, variables) => api.createDelegator(variables),
};

export const updateDelegatorMutation: MutationDefinition<
  ApiResult<"updateDelegator">,
  { name: string; request: CreateDelegatorRequest }
> = {
  key: "update-delegator",
  affected: [...MODEL_QUERY_KEYS],
  run: (api, variables) => api.updateDelegator(variables.name, variables.request),
};

export const validateGitTokenMutation: MutationDefinition<
  ApiResult<"validateGitToken">,
  ValidateGitTokenRequest
> = {
  key: "validate-git-token",
  affected: [QUERY_KEYS.gitProviders],
  sensitive: true,
  run: (api, variables) => api.validateGitToken(variables),
};

export const writeGitConfigMutation: MutationDefinition<
  ApiResult<"writeGitConfig">,
  WriteGitConfigRequest
> = {
  key: "write-git-config",
  affected: [...PROFILE_QUERY_KEYS, QUERY_KEYS.gitProviders],
  run: (api, variables) => api.writeGitConfig(variables),
};

export const setGitSessionEnvMutation: MutationDefinition<
  ApiResult<"setGitSessionEnv">,
  SetGitSessionEnvRequest
> = {
  key: "set-git-session-env",
  affected: [...PROFILE_QUERY_KEYS],
  sensitive: true,
  run: (api, variables) => api.setGitSessionEnv(variables),
};

export const validateKanbanCredentialsMutation: MutationDefinition<
  ApiResult<"validateKanbanCredentials">,
  ValidateKanbanCredentialsRequest
> = {
  key: "validate-kanban-credentials",
  affected: [QUERY_KEYS.kanbanProviders],
  sensitive: true,
  run: (api, variables) => api.validateKanbanCredentials(variables),
};

export const listKanbanProjectsMutation: MutationDefinition<
  ApiResult<"listKanbanProjects">,
  ListKanbanProjectsRequest
> = {
  key: "list-kanban-projects",
  affected: [],
  sensitive: true,
  run: (api, variables) => api.listKanbanProjects(variables),
};

export const listKanbanStatusesMutation: MutationDefinition<
  ApiResult<"listKanbanStatuses">,
  ListKanbanStatusesRequest
> = {
  key: "list-kanban-statuses",
  affected: [],
  sensitive: true,
  run: (api, variables) => api.listKanbanStatuses(variables),
};

export const writeKanbanConfigMutation: MutationDefinition<
  ApiResult<"writeKanbanConfig">,
  WriteKanbanConfigRequest
> = {
  key: "write-kanban-config",
  affected: [...PROFILE_QUERY_KEYS, QUERY_KEYS.kanbanProviders, ...TICKET_QUERY_KEYS],
  run: (api, variables) => api.writeKanbanConfig(variables),
};

export const setKanbanSessionEnvMutation: MutationDefinition<
  ApiResult<"setKanbanSessionEnv">,
  SetKanbanSessionEnvRequest
> = {
  key: "set-kanban-session-env",
  affected: [...PROFILE_QUERY_KEYS],
  sensitive: true,
  run: (api, variables) => api.setKanbanSessionEnv(variables),
};
