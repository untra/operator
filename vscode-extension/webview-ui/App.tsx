import React, { useEffect, useState, useCallback } from 'react';
import Box from '@mui/material/Box';
import CircularProgress from '@mui/material/CircularProgress';
import Typography from '@mui/material/Typography';
import Alert from '@mui/material/Alert';
import { ThemeWrapper } from './theme/ThemeWrapper';
import { ConfigPage } from './components/ConfigPage';
import { postMessage, onMessage } from './vscodeApi';
import { DEFAULT_WEBVIEW_CONFIG } from './types/defaults';
import type {
  WebviewConfig,
  ExtensionToWebviewMessage,
  JiraValidationInfo,
  LinearValidationInfo,
  IssueTypeSummary,
  CollectionResponse,
  ExternalIssueTypeSummary,
} from './types/messages';
import type { JiraConfig } from '../src/generated/JiraConfig';
import type { LinearConfig } from '../src/generated/LinearConfig';
import type { ProjectSyncConfig } from '../src/generated/ProjectSyncConfig';

export function App() {
  const [config, setConfig] = useState<WebviewConfig | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [jiraResult, setJiraResult] = useState<JiraValidationInfo | null>(null);
  const [linearResult, setLinearResult] = useState<LinearValidationInfo | null>(null);
  const [validatingJira, setValidatingJira] = useState(false);
  const [validatingLinear, setValidatingLinear] = useState(false);
  const [apiReachable, setApiReachable] = useState(false);
  const [issueTypes, setIssueTypes] = useState<IssueTypeSummary[]>([]);
  const [collections, setCollections] = useState<CollectionResponse[]>([]);
  const [externalIssueTypes, setExternalIssueTypes] = useState<Map<string, ExternalIssueTypeSummary[]>>(new Map());

  useEffect(() => {
    const cleanup = onMessage((msg: ExtensionToWebviewMessage) => {
      switch (msg.type) {
        case 'configLoaded':
        case 'configUpdated':
          setConfig(mergeWithDefaults(msg.config));
          setError(null);
          break;
        case 'configError':
          setError(msg.error);
          break;
        case 'browseResult':
          setConfig((prev) => {
            if (!prev) { return prev; }
            if (msg.field === 'workingDirectory') {
              return { ...prev, working_directory: msg.path };
            }
            return prev;
          });
          break;
        case 'jiraValidationResult':
          setJiraResult(msg.result);
          setValidatingJira(false);
          break;
        case 'linearValidationResult':
          setLinearResult(msg.result);
          setValidatingLinear(false);
          break;
        case 'llmToolsDetected':
          setConfig(mergeWithDefaults(msg.config));
          break;
        case 'apiHealthResult':
          setApiReachable(msg.reachable);
          if (msg.reachable) {
            postMessage({ type: 'getIssueTypes' });
            postMessage({ type: 'getCollections' });
          }
          break;
        case 'issueTypesLoaded':
          setIssueTypes(msg.issueTypes);
          break;
        case 'collectionsLoaded':
          setCollections(msg.collections);
          break;
        case 'externalIssueTypesLoaded':
          setExternalIssueTypes(prev => {
            const next = new Map(prev);
            next.set(`${msg.provider}/${msg.projectKey}`, msg.types);
            return next;
          });
          break;
        case 'externalIssueTypesError':
          // External issue type lookup failed; the mapping panel renders an
          // empty/unmapped state, so no extra handling is required here.
          break;
      }
    });

    // Signal ready and request config
    postMessage({ type: 'ready' });
    postMessage({ type: 'getConfig' });
    postMessage({ type: 'checkApiHealth' });

    return cleanup;
  }, []);

  const handleUpdate = useCallback(
    (section: string, key: string, value: unknown) => {
      postMessage({ type: 'updateConfig', section, key, value });

      // Optimistic update for responsiveness
      setConfig((prev) => {
        if (!prev) { return prev; }
        return applyUpdate(prev, section, key, value);
      });
    },
    []
  );

  const handleBrowseFolder = useCallback((field: string) => {
    postMessage({ type: 'browseFolder', field });
  }, []);

  const handleOpenFile = useCallback((filePath: string) => {
    postMessage({ type: 'openFile', filePath });
  }, []);

  const handleStartSetup = useCallback(() => {
    postMessage({ type: 'openWalkthrough' });
  }, []);

  const handleValidateJira = useCallback(
    (domain: string, email: string, apiToken: string) => {
      setValidatingJira(true);
      setJiraResult(null);
      postMessage({ type: 'validateJira', domain, email, apiToken });
    },
    []
  );

  const handleValidateLinear = useCallback((apiKey: string) => {
    setValidatingLinear(true);
    setLinearResult(null);
    postMessage({ type: 'validateLinear', apiKey });
  }, []);

  const handleDetectTools = useCallback(() => {
    postMessage({ type: 'detectLlmTools' });
  }, []);

  const handleGetExternalIssueTypes = useCallback((provider: string, domain: string, projectKey: string) => {
    postMessage({ type: 'getExternalIssueTypes', provider, domain, projectKey });
  }, []);

  const handleOpenOperatorUi = useCallback((route: 'issuetypes' | 'projects') => {
    postMessage({ type: 'openOperatorUi', route });
  }, []);

  return (
    <ThemeWrapper>
      {error && (
        <Alert severity="error" sx={{ m: 2 }}>
          {error}
        </Alert>
      )}
      {config ? (
        <ConfigPage
          config={config}
          onUpdate={handleUpdate}
          onBrowseFolder={handleBrowseFolder}
          onOpenFile={handleOpenFile}
          onStartSetup={handleStartSetup}
          onValidateJira={handleValidateJira}
          onValidateLinear={handleValidateLinear}
          onDetectTools={handleDetectTools}
          jiraResult={jiraResult}
          linearResult={linearResult}
          validatingJira={validatingJira}
          validatingLinear={validatingLinear}
          apiReachable={apiReachable}
          issueTypes={issueTypes}
          collections={collections}
          externalIssueTypes={externalIssueTypes}
          onGetExternalIssueTypes={handleGetExternalIssueTypes}
          onOpenOperatorUi={handleOpenOperatorUi}
        />
      ) : (
        <Box sx={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100vh', gap: 2 }}>
          <CircularProgress />
          <Typography variant="body2" color="text.secondary">
            Loading configuration...
          </Typography>
        </Box>
      )}
    </ThemeWrapper>
  );
}

/** Deep-merge incoming config with defaults so all fields exist */
function mergeWithDefaults(incoming: WebviewConfig): WebviewConfig {
  const defaults = DEFAULT_WEBVIEW_CONFIG;
  return {
    config_path: incoming.config_path || defaults.config_path,
    working_directory: incoming.working_directory || defaults.working_directory,
    config_exists: incoming.config_exists ?? defaults.config_exists,
    config: deepMerge(defaults.config, incoming.config),
  };
}

/** Recursively merge source into target (source wins for leaf values) */
function deepMerge<T extends Record<string, unknown>>(target: T, source: T): T {
  const result: Record<string, unknown> = { ...target };
  for (const key of Object.keys(source)) {
    const srcVal = (source as Record<string, unknown>)[key];
    const tgtVal = (target as Record<string, unknown>)[key];
    if (
      srcVal !== null &&
      srcVal !== undefined &&
      typeof srcVal === 'object' &&
      !Array.isArray(srcVal) &&
      typeof tgtVal === 'object' &&
      tgtVal !== null &&
      !Array.isArray(tgtVal)
    ) {
      result[key] = deepMerge(
        tgtVal as Record<string, unknown>,
        srcVal as Record<string, unknown>,
      );
    } else if (srcVal !== undefined) {
      result[key] = srcVal;
    }
  }
  return result as T;
}

const DEFAULT_JIRA: JiraConfig = { enabled: false, api_key_env: 'OPERATOR_JIRA_API_KEY', email: '', projects: {} };
const DEFAULT_LINEAR: LinearConfig = { enabled: false, api_key_env: 'OPERATOR_LINEAR_API_KEY', projects: {} };
const DEFAULT_PROJECT_SYNC: ProjectSyncConfig = { sync_user_id: '', sync_statuses: [], collection_name: null, type_mappings: {}, bidirectional: false };

/** Apply an update to the config object by section/key path */
function applyUpdate(
  config: WebviewConfig,
  section: string,
  key: string,
  value: unknown
): WebviewConfig {
  const next = { ...config, config: { ...config.config } };

  switch (section) {
    case 'primary':
      if (key === 'working_directory') { next.working_directory = value as string; }
      break;

    case 'agents': {
      const updated = { ...next.config.agents };
      (updated as Record<string, unknown>)[key] = value;
      next.config.agents = updated;
      break;
    }

    case 'sessions': {
      const updated = { ...next.config.sessions };
      (updated as Record<string, unknown>)[key] = value;
      next.config.sessions = updated;
      break;
    }

    case 'kanban.jira': {
      const jiraMap = { ...next.config.kanban.jira };
      const domains = Object.keys(jiraMap);
      const domain = domains[0] ?? 'your-org.atlassian.net';
      const ws: JiraConfig = { ...(jiraMap[domain] ?? DEFAULT_JIRA) };

      if (key === 'enabled' || key === 'email' || key === 'api_key_env') {
        (ws as Record<string, unknown>)[key] = value;
        jiraMap[domain] = ws;
      } else if (key === 'domain' && typeof value === 'string' && value !== domain) {
        delete jiraMap[domain];
        jiraMap[value] = ws;
      } else if (key === 'project_key' || key === 'sync_statuses' || key === 'collection_name' || key === 'sync_user_id' || key === 'type_mappings') {
        const projects = { ...ws.projects };
        const pKeys = Object.keys(projects);
        const pKey = pKeys[0] ?? 'default';
        if (key === 'project_key') {
          const oldProject = projects[pKey] ?? DEFAULT_PROJECT_SYNC;
          delete projects[pKey];
          projects[value as string] = oldProject;
        } else {
          const existing = { ...(projects[pKey] ?? DEFAULT_PROJECT_SYNC) };
          (existing as Record<string, unknown>)[key] = value;
          projects[pKey] = existing;
        }
        ws.projects = projects;
        jiraMap[domain] = ws;
      } else if (key.startsWith('projects.')) {
        // Multi-project writes: projects.{projectKey}.{field}
        const parts = key.split('.');
        if (parts.length >= 3) {
          const pKey = parts[1];
          const field = parts.slice(2).join('.');
          const projects = { ...ws.projects };
          const existing = { ...(projects[pKey] ?? DEFAULT_PROJECT_SYNC) };
          (existing as Record<string, unknown>)[field] = value;
          projects[pKey] = existing;
          ws.projects = projects;
          jiraMap[domain] = ws;
        }
      }
      next.config.kanban = { ...next.config.kanban, jira: jiraMap };
      break;
    }

    case 'kanban.linear': {
      const linearMap = { ...next.config.kanban.linear };
      const teams = Object.keys(linearMap);
      const teamId = teams[0] ?? 'default-team';
      const ws: LinearConfig = { ...(linearMap[teamId] ?? DEFAULT_LINEAR) };

      if (key === 'enabled' || key === 'api_key_env') {
        (ws as Record<string, unknown>)[key] = value;
        linearMap[teamId] = ws;
      } else if (key === 'team_id' && typeof value === 'string' && value !== teamId) {
        delete linearMap[teamId];
        linearMap[value] = ws;
      } else if (key === 'sync_statuses' || key === 'collection_name' || key === 'sync_user_id' || key === 'type_mappings') {
        const projects = { ...ws.projects };
        const pKeys = Object.keys(projects);
        const pKey = pKeys[0] ?? 'default';
        const existing = { ...(projects[pKey] ?? DEFAULT_PROJECT_SYNC) };
        (existing as Record<string, unknown>)[key] = value;
        projects[pKey] = existing;
        ws.projects = projects;
        linearMap[teamId] = ws;
      } else if (key.startsWith('projects.')) {
        // Multi-project writes: projects.{projectKey}.{field}
        const parts = key.split('.');
        if (parts.length >= 3) {
          const pKey = parts[1];
          const field = parts.slice(2).join('.');
          const projects = { ...ws.projects };
          const existing = { ...(projects[pKey] ?? DEFAULT_PROJECT_SYNC) };
          (existing as Record<string, unknown>)[field] = value;
          projects[pKey] = existing;
          ws.projects = projects;
          linearMap[teamId] = ws;
        }
      }
      next.config.kanban = { ...next.config.kanban, linear: linearMap };
      break;
    }

    case 'git': {
      const updated = { ...next.config.git };
      (updated as Record<string, unknown>)[key] = value;
      next.config.git = updated;
      break;
    }

    case 'git.github': {
      const github = { ...next.config.git.github };
      (github as Record<string, unknown>)[key] = value;
      next.config.git = { ...next.config.git, github };
      break;
    }
  }

  return next;
}
