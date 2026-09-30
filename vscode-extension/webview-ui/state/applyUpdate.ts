import type { WebviewConfig } from "../types/messages";
import type { JiraConfig } from "../../src/generated/JiraConfig";
import type { LinearConfig } from "../../src/generated/LinearConfig";
import type { ProjectSyncConfig } from "../../src/generated/ProjectSyncConfig";

const DEFAULT_JIRA: JiraConfig = {
  enabled: false,
  api_key_env: "OPERATOR_JIRA_API_KEY",
  email: "",
  projects: {},
};
const DEFAULT_LINEAR: LinearConfig = {
  enabled: false,
  api_key_env: "OPERATOR_LINEAR_API_KEY",
  projects: {},
};
const DEFAULT_PROJECT_SYNC: ProjectSyncConfig = {
  sync_user_id: "",
  status_mapping: {},
  collection_name: null,
  type_mappings: {},
  bidirectional: false,
};

const PROJECT_PATH_PREFIX = "projects.";
const PROJECT_PATH_MIN_PARTS = 3;

interface KanbanProviderShape<W> {
  instanceKeyField: string;
  defaultWorkspace: W;
}

const JIRA_SHAPE: KanbanProviderShape<JiraConfig> = {
  instanceKeyField: "domain",
  defaultWorkspace: DEFAULT_JIRA,
};
const LINEAR_SHAPE: KanbanProviderShape<LinearConfig> = {
  instanceKeyField: "team_id",
  defaultWorkspace: DEFAULT_LINEAR,
};

type KanbanWorkspace = { projects: Record<string, ProjectSyncConfig> };

function applyKanbanUpdate<W extends KanbanWorkspace>(
  source: Record<string, W>,
  shape: KanbanProviderShape<W>,
  key: string,
  value: unknown,
  instanceKey: string | undefined,
): Record<string, W> {
  const map = { ...source };
  if (!instanceKey) {
    return map;
  }
  const hasInstances = Object.keys(map).length > 0;
  if (hasInstances && !Object.hasOwn(map, instanceKey)) {
    return map;
  }
  const workspace: W = { ...(map[instanceKey] ?? shape.defaultWorkspace) };

  if (key === shape.instanceKeyField) {
    if (typeof value === "string" && value !== instanceKey) {
      if (Object.hasOwn(map, value)) {
        return map;
      }
      delete map[instanceKey];
      map[value] = workspace;
    }
    return map;
  }

  if (key.startsWith(PROJECT_PATH_PREFIX)) {
    const parts = key.split(".");
    const projectKey = parts[1];
    if (parts.length < PROJECT_PATH_MIN_PARTS || !projectKey) {
      return map;
    }
    const field = parts.slice(2).join(".");
    const project = { ...(workspace.projects[projectKey] ?? DEFAULT_PROJECT_SYNC) };
    (project as Record<string, unknown>)[field] = value;
    workspace.projects = { ...workspace.projects, [projectKey]: project };
  } else {
    (workspace as Record<string, unknown>)[key] = value;
  }
  map[instanceKey] = workspace;
  return map;
}

/** Apply an update to the config object by section/key path. Pure: never mutates `config`. */
export function applyUpdate(
  config: WebviewConfig,
  section: string,
  key: string,
  value: unknown,
  instanceKey?: string,
): WebviewConfig {
  const next = { ...config, config: { ...config.config } };

  switch (section) {
    case "primary":
      if (key === "working_directory") {
        next.working_directory = value as string;
      }
      break;

    case "agents":
      next.config.agents = { ...next.config.agents, [key]: value };
      break;

    case "sessions":
      next.config.sessions = { ...next.config.sessions, [key]: value };
      break;

    case "kanban.jira":
      next.config.kanban = {
        ...next.config.kanban,
        jira: applyKanbanUpdate(next.config.kanban.jira, JIRA_SHAPE, key, value, instanceKey),
      };
      break;

    case "kanban.linear":
      next.config.kanban = {
        ...next.config.kanban,
        linear: applyKanbanUpdate(next.config.kanban.linear, LINEAR_SHAPE, key, value, instanceKey),
      };
      break;

    case "git":
      next.config.git = { ...next.config.git, [key]: value };
      break;

    case "git.github":
      next.config.git = {
        ...next.config.git,
        github: { ...next.config.git.github, [key]: value },
      };
      break;
  }

  return next;
}
