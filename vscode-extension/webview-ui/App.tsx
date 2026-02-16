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
} from './types/messages';

export function App() {
  const [config, setConfig] = useState<WebviewConfig | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [jiraResult, setJiraResult] = useState<JiraValidationInfo | null>(null);
  const [linearResult, setLinearResult] = useState<LinearValidationInfo | null>(null);
  const [validatingJira, setValidatingJira] = useState(false);
  const [validatingLinear, setValidatingLinear] = useState(false);

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
      }
    });

    // Signal ready and request config
    postMessage({ type: 'ready' });
    postMessage({ type: 'getConfig' });

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

  const handleBrowseFile = useCallback((field: string) => {
    postMessage({ type: 'browseFile', field });
  }, []);

  const handleBrowseFolder = useCallback((field: string) => {
    postMessage({ type: 'browseFolder', field });
  }, []);

  const handleOpenFile = useCallback((filePath: string) => {
    postMessage({ type: 'openFile', filePath });
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
          onValidateJira={handleValidateJira}
          onValidateLinear={handleValidateLinear}
          onDetectTools={handleDetectTools}
          jiraResult={jiraResult}
          linearResult={linearResult}
          validatingJira={validatingJira}
          validatingLinear={validatingLinear}
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
    config: deepMerge(defaults.config as Record<string, unknown>, (incoming.config ?? {}) as Record<string, unknown>) as WebviewConfig['config'],
  };
}

/** Recursively merge source into target (source wins for leaf values) */
function deepMerge(target: Record<string, unknown>, source: Record<string, unknown>): Record<string, unknown> {
  const result: Record<string, unknown> = { ...target };
  for (const key of Object.keys(source)) {
    const srcVal = source[key];
    const tgtVal = target[key];
    if (
      srcVal !== null &&
      srcVal !== undefined &&
      typeof srcVal === 'object' &&
      !Array.isArray(srcVal) &&
      typeof tgtVal === 'object' &&
      tgtVal !== null &&
      !Array.isArray(tgtVal)
    ) {
      result[key] = deepMerge(tgtVal as Record<string, unknown>, srcVal as Record<string, unknown>);
    } else if (srcVal !== undefined) {
      result[key] = srcVal;
    }
  }
  return result;
}

/** Apply an update to the config object by section/key path */
function applyUpdate(
  config: WebviewConfig,
  section: string,
  key: string,
  value: unknown
): WebviewConfig {
  const next = { ...config, config: { ...config.config } };
  const c = next.config as Record<string, unknown>;

  switch (section) {
    case 'primary':
      if (key === 'working_directory') { next.working_directory = value as string; }
      break;

    case 'agents':
      c.agents = { ...(c.agents as Record<string, unknown> ?? {}), [key]: value };
      break;

    case 'sessions':
      c.sessions = { ...(c.sessions as Record<string, unknown> ?? {}), [key]: value };
      break;

    case 'kanban.jira': {
      const kanban = { ...(c.kanban as Record<string, unknown> ?? {}) };
      const jiraMap = { ...(kanban.jira as Record<string, unknown> ?? {}) };
      const domains = Object.keys(jiraMap);
      const domain = domains[0] ?? 'your-org.atlassian.net';
      const ws = { ...(jiraMap[domain] as Record<string, unknown> ?? {}) };

      if (key === 'enabled' || key === 'email' || key === 'api_key_env') {
        ws[key] = value;
        jiraMap[domain] = ws;
      } else if (key === 'domain' && typeof value === 'string' && value !== domain) {
        delete jiraMap[domain];
        jiraMap[value] = ws;
      } else if (key === 'project_key' || key === 'sync_statuses' || key === 'collection_name' || key === 'sync_user_id') {
        const projects = { ...(ws.projects as Record<string, unknown> ?? {}) };
        const pKeys = Object.keys(projects);
        const pKey = pKeys[0] ?? 'default';
        if (key === 'project_key') {
          const oldProject = projects[pKey] ?? {};
          delete projects[pKey];
          projects[value as string] = oldProject;
        } else {
          projects[pKey] = { ...(projects[pKey] as Record<string, unknown> ?? {}), [key]: value };
        }
        ws.projects = projects;
        jiraMap[domain] = ws;
      }
      kanban.jira = jiraMap;
      c.kanban = kanban;
      break;
    }

    case 'kanban.linear': {
      const kanban = { ...(c.kanban as Record<string, unknown> ?? {}) };
      const linearMap = { ...(kanban.linear as Record<string, unknown> ?? {}) };
      const teams = Object.keys(linearMap);
      const teamId = teams[0] ?? 'default-team';
      const ws = { ...(linearMap[teamId] as Record<string, unknown> ?? {}) };

      if (key === 'enabled' || key === 'api_key_env') {
        ws[key] = value;
        linearMap[teamId] = ws;
      } else if (key === 'team_id' && typeof value === 'string' && value !== teamId) {
        delete linearMap[teamId];
        linearMap[value] = ws;
      } else if (key === 'sync_statuses' || key === 'collection_name' || key === 'sync_user_id') {
        const projects = { ...(ws.projects as Record<string, unknown> ?? {}) };
        const pKeys = Object.keys(projects);
        const pKey = pKeys[0] ?? 'default';
        projects[pKey] = { ...(projects[pKey] as Record<string, unknown> ?? {}), [key]: value };
        ws.projects = projects;
        linearMap[teamId] = ws;
      }
      kanban.linear = linearMap;
      c.kanban = kanban;
      break;
    }

    case 'git':
      c.git = { ...(c.git as Record<string, unknown> ?? {}), [key]: value };
      break;

    case 'git.github': {
      const git = { ...(c.git as Record<string, unknown> ?? {}) };
      git.github = { ...(git.github as Record<string, unknown> ?? {}), [key]: value };
      c.git = git;
      break;
    }
  }

  return next;
}
