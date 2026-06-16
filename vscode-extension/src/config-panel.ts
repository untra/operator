/**
 * ConfigPanel - WebviewPanel for the Operator settings page
 *
 * Manages a singleton React-based webview that renders config.toml
 * settings with live editing, validation, and theme synchronization.
 */

import * as vscode from 'vscode';
import * as fs from 'fs/promises';
import * as path from 'path';
// smol-toml is ESM-only, must use dynamic import
async function importSmolToml() {
  return await import('smol-toml');
}
import { detectInstalledLlmTools } from './walkthrough';
import {
  getConfigDir,
  getResolvedConfigPath,
  resolveWorkingDirectory,
} from './config-paths';
import { OperatorApiClient, discoverApiUrl } from './api-client';
import type { CreateDelegatorRequest } from './generated';

/** Message types from the webview */
interface WebviewMessage {
  type: string;
  [key: string]: unknown;
}

/** WebviewConfig shape matching the webview types */
interface WebviewConfig {
  config_path: string;
  working_directory: string;
  config_exists: boolean;
  config: Record<string, unknown>;
}

export class ConfigPanel {
  public static currentPanel: ConfigPanel | undefined;

  private readonly _panel: vscode.WebviewPanel;
  private readonly _extensionUri: vscode.Uri;
  private _disposables: vscode.Disposable[] = [];

  private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
    this._panel = panel;
    this._extensionUri = extensionUri;

    this._panel.webview.html = this._getHtmlContent();

    this._panel.onDidDispose(() => this._dispose(), null, this._disposables);

    this._panel.webview.onDidReceiveMessage(
      (msg: WebviewMessage) => this._handleMessage(msg),
      null,
      this._disposables
    );
  }

  /** Show existing panel or create a new one */
  public static createOrShow(extensionUri: vscode.Uri): void {
    const column = vscode.window.activeTextEditor
      ? vscode.window.activeTextEditor.viewColumn
      : undefined;

    if (ConfigPanel.currentPanel) {
      ConfigPanel.currentPanel._panel.reveal(column);
      return;
    }

    const panel = vscode.window.createWebviewPanel(
      'operatorSettings',
      'Operator Settings',
      column || vscode.ViewColumn.One,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [
          vscode.Uri.joinPath(extensionUri, 'dist', 'webview'),
        ],
      }
    );

    ConfigPanel.currentPanel = new ConfigPanel(panel, extensionUri);
  }

  /** Send a navigation message to the webview to scroll to a section */
  public static navigateTo(section: string, prefill?: Record<string, unknown>): void {
    if (ConfigPanel.currentPanel) {
      void ConfigPanel.currentPanel._panel.webview.postMessage({
        type: 'navigateTo',
        section,
        prefill,
      });
    }
  }

  private _getHtmlContent(): string {
    const webview = this._panel.webview;

    const scriptUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this._extensionUri, 'dist', 'webview', 'configPage.js')
    );

    const nonce = getNonce();

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}'; font-src ${webview.cspSource};">
  <title>Operator Settings</title>
</head>
<body>
  <div id="root"></div>
  <script nonce="${nonce}" src="${scriptUri.toString()}"></script>
</body>
</html>`;
  }

  /** Resolve the running operator REST base URL from the current workspace. */
  private async apiUrl(): Promise<string> {
    const workDir = resolveWorkingDirectory();
    const ticketsDir = workDir ? path.join(workDir, '.tickets') : undefined;
    return discoverApiUrl(ticketsDir);
  }

  private async _handleMessage(message: WebviewMessage): Promise<void> {
    switch (message.type) {
      case 'ready':
        // Webview is ready, send config
        await this._sendConfig();
        break;

      case 'getConfig':
        await this._sendConfig();
        break;

      case 'updateConfig':
        await this._updateConfig(
          message.section as string,
          message.key as string,
          message.value
        );
        break;

      case 'browseFile': {
        const fileUri = await vscode.window.showOpenDialog({
          canSelectFiles: true,
          canSelectFolders: false,
          canSelectMany: false,
          openLabel: 'Select File',
        });
        if (fileUri && fileUri.length > 0) {
          void this._panel.webview.postMessage({
            type: 'browseResult',
            field: message.field,
            path: fileUri[0]!.fsPath,
          });
        }
        break;
      }

      case 'browseFolder': {
        const folderUri = await vscode.window.showOpenDialog({
          canSelectFiles: false,
          canSelectFolders: true,
          canSelectMany: false,
          openLabel: 'Select Folder',
        });
        if (folderUri && folderUri.length > 0) {
          void this._panel.webview.postMessage({
            type: 'browseResult',
            field: message.field,
            path: folderUri[0]!.fsPath,
          });
          // Also persist to VS Code settings
          await vscode.workspace
            .getConfiguration('operator')
            .update('workingDirectory', folderUri[0]!.fsPath, vscode.ConfigurationTarget.Global);
        }
        break;
      }

      case 'validateJira': {
        // Delegate credential validation to the Operator REST API.
        const workDir = resolveWorkingDirectory();
        const ticketsDir = workDir ? path.join(workDir, '.tickets') : undefined;
        const apiUrl = await discoverApiUrl(ticketsDir);
        const client = new OperatorApiClient(apiUrl);

        let displayName = '';
        let accountId = '';
        let errorMsg: string | undefined;
        let projects: Array<{ key: string; name: string }> = [];
        let valid: boolean;

        try {
          const result = await client.validateKanbanCredentials({
            provider: 'jira',
            jira: {
              domain: message.domain as string,
              email: message.email as string,
              api_token: message.apiToken as string,
            },
            linear: null,
            github: null,
          });
          valid = result.valid;
          if (result.jira) {
            displayName = result.jira.display_name;
            accountId = result.jira.account_id;
          }
          errorMsg = result.error ?? undefined;

          if (valid) {
            const projs = await client.listKanbanProjects({
              provider: 'jira',
              jira: {
                domain: message.domain as string,
                email: message.email as string,
                api_token: message.apiToken as string,
              },
              linear: null,
              github: null,
            });
            projects = projs.map((p) => ({ key: p.key, name: p.name }));
          }
        } catch (err) {
          valid = false;
          errorMsg = err instanceof Error ? err.message : 'Unknown error';
        }

        void this._panel.webview.postMessage({
          type: 'jiraValidationResult',
          result: {
            valid,
            displayName,
            accountId,
            error: errorMsg,
            projects,
          },
        });
        break;
      }

      case 'validateLinear': {
        const workDir = resolveWorkingDirectory();
        const ticketsDir = workDir ? path.join(workDir, '.tickets') : undefined;
        const apiUrl = await discoverApiUrl(ticketsDir);
        const client = new OperatorApiClient(apiUrl);

        let userName = '';
        let orgName = '';
        let userId = '';
        let teams: Array<{ id: string; name: string; key: string }> = [];
        let errorMsg: string | undefined;
        let valid: boolean;

        try {
          const result = await client.validateKanbanCredentials({
            provider: 'linear',
            jira: null,
            linear: { api_key: message.apiKey as string },
            github: null,
          });
          valid = result.valid;
          if (result.linear) {
            userName = result.linear.user_name;
            orgName = result.linear.org_name;
            userId = result.linear.user_id;
            teams = result.linear.teams.map((t) => ({
              id: t.id,
              name: t.name,
              key: t.key,
            }));
          }
          errorMsg = result.error ?? undefined;
        } catch (err) {
          valid = false;
          errorMsg = err instanceof Error ? err.message : 'Unknown error';
        }

        void this._panel.webview.postMessage({
          type: 'linearValidationResult',
          result: {
            valid,
            userName,
            orgName,
            userId,
            error: errorMsg,
            teams,
          },
        });
        break;
      }

      case 'detectLlmTools': {
        const tools = await detectInstalledLlmTools();
        // Update config with detected tools and send back full WebviewConfig
        const configPath = getResolvedConfigPath();
        if (configPath) {
          try {
            await writeConfigField('llm_tools', 'detected', tools);
          } catch {
            // Non-fatal: config may not exist yet
          }
        }
        try {
          const config = await readConfig();
          void this._panel.webview.postMessage({
            type: 'llmToolsDetected',
            config,
          });
        } catch {
          // If we can't read config, just send tool names for compatibility
          void this._panel.webview.postMessage({
            type: 'llmToolsDetected',
            config: {
              config_path: configPath || '',
              working_directory: resolveWorkingDirectory(),
              config_exists: Boolean(configPath),
              config: { llm_tools: { detected: tools, providers: [], detection_complete: true } },
            },
          });
        }
        break;
      }

      case 'checkApiHealth': {
        try {
          const workDir = resolveWorkingDirectory();
          const ticketsDir = workDir ? path.join(workDir, '.tickets') : undefined;
          const apiUrl = await discoverApiUrl(ticketsDir);
          const client = new OperatorApiClient(apiUrl);
          await client.health();
          void this._panel.webview.postMessage({ type: 'apiHealthResult', reachable: true });
        } catch {
          void this._panel.webview.postMessage({ type: 'apiHealthResult', reachable: false });
        }
        break;
      }

      case 'openOperatorUi': {
        // The webview now links out to the daemon-hosted Operator UI for
        // surfaces it no longer reimplements (issue types, projects, …).
        const route = message.route as string;
        const command =
          route === 'projects'
            ? 'operator.openProjects'
            : route === 'issuetypes'
              ? 'operator.openIssueTypes'
              : 'operator.openUi';
        await vscode.commands.executeCommand(command);
        break;
      }

      case 'openExternal':
        void vscode.env.openExternal(vscode.Uri.parse(message.url as string));
        break;

      case 'openWalkthrough':
        await vscode.commands.executeCommand('operator.openWalkthrough');
        break;

      case 'openFile': {
        const filePath = message.filePath as string;
        if (filePath) {
          const doc = await vscode.workspace.openTextDocument(filePath);
          await vscode.window.showTextDocument(doc);
        }
        break;
      }

      case 'getIssueTypes': {
        try {
          const workDir = resolveWorkingDirectory();
          const ticketsDir = workDir ? path.join(workDir, '.tickets') : undefined;
          const apiUrl = await discoverApiUrl(ticketsDir);
          const client = new OperatorApiClient(apiUrl);
          const issueTypes = await client.listIssueTypes();
          void this._panel.webview.postMessage({ type: 'issueTypesLoaded', issueTypes });
        } catch (err) {
          void this._panel.webview.postMessage({
            type: 'issueTypeError',
            error: err instanceof Error ? err.message : 'Failed to load issue types',
          });
        }
        break;
      }

      case 'getCollections': {
        try {
          const workDir = resolveWorkingDirectory();
          const ticketsDir = workDir ? path.join(workDir, '.tickets') : undefined;
          const apiUrl = await discoverApiUrl(ticketsDir);
          const client = new OperatorApiClient(apiUrl);
          const collections = await client.listCollections();
          void this._panel.webview.postMessage({ type: 'collectionsLoaded', collections });
        } catch (err) {
          void this._panel.webview.postMessage({
            type: 'collectionsError',
            error: err instanceof Error ? err.message : 'Failed to load collections',
          });
        }
        break;
      }

      case 'getModelProviders': {
        try {
          const client = new OperatorApiClient(await this.apiUrl());
          const [kinds, delegatorsResp] = await Promise.all([
            client.listProviderKinds(),
            client.listDelegators(),
          ]);
          void this._panel.webview.postMessage({
            type: 'modelProvidersLoaded',
            kinds,
            delegators: delegatorsResp.delegators,
          });
        } catch (err) {
          void this._panel.webview.postMessage({
            type: 'modelProvidersError',
            error: err instanceof Error ? err.message : 'Failed to load model providers',
          });
        }
        break;
      }

      case 'probeProvider': {
        const slug = message.slug as string;
        try {
          const client = new OperatorApiClient(await this.apiUrl());
          const result = await client.providerModels(slug);
          void this._panel.webview.postMessage({ type: 'providerProbed', slug, result });
        } catch {
          void this._panel.webview.postMessage({
            type: 'providerProbed',
            slug,
            result: { server: slug, reachable: false, models: [], error: 'probe failed' },
          });
        }
        break;
      }

      case 'connectProvider': {
        const slug = message.slug as string;
        try {
          const client = new OperatorApiClient(await this.apiUrl());
          const kinds = await client.listProviderKinds();
          const kind = kinds.find((k) => k.slug === slug);
          await client.createModelServer({
            name: slug,
            kind: slug,
            base_url: kind?.default_base_url ?? null,
            api_key_env: kind?.default_api_key_env ?? null,
            extra_env: {},
            display_name: kind?.display_name ?? null,
          });
          const result = await client.providerModels(slug);
          void this._panel.webview.postMessage({ type: 'providerProbed', slug, result });
        } catch (err) {
          void this._panel.webview.postMessage({
            type: 'modelProvidersError',
            error: err instanceof Error ? err.message : 'Failed to connect provider',
          });
        }
        break;
      }

      case 'createDelegator': {
        try {
          const client = new OperatorApiClient(await this.apiUrl());
          const created = await client.createDelegator(message.request as CreateDelegatorRequest);
          void this._panel.webview.postMessage({ type: 'delegatorCreated', name: created.name });
        } catch (err) {
          void this._panel.webview.postMessage({
            type: 'modelProvidersError',
            error: err instanceof Error ? err.message : 'Failed to create delegator',
          });
        }
        break;
      }

      case 'getExternalIssueTypes': {
        const provider = message.provider as string;
        const projectKey = message.projectKey as string;
        try {
          const workDir = resolveWorkingDirectory();
          const ticketsDir = workDir ? path.join(workDir, '.tickets') : undefined;
          const apiUrl = await discoverApiUrl(ticketsDir);
          const client = new OperatorApiClient(apiUrl);
          const types = await client.getExternalIssueTypes(provider, projectKey);
          void this._panel.webview.postMessage({
            type: 'externalIssueTypesLoaded',
            provider,
            projectKey,
            types,
          });
        } catch (err) {
          void this._panel.webview.postMessage({
            type: 'externalIssueTypesError',
            provider,
            projectKey,
            error: err instanceof Error ? err.message : 'Failed to load external issue types',
          });
        }
        break;
      }
    }
  }

  /** Read config.toml and send as WebviewConfig to webview */
  private async _sendConfig(): Promise<void> {
    try {
      const config = await readConfig();
      void this._panel.webview.postMessage({
        type: 'configLoaded',
        config,
      });
    } catch (err) {
      void this._panel.webview.postMessage({
        type: 'configError',
        error: err instanceof Error ? err.message : 'Failed to load config',
      });
    }
  }

  /** Apply a field update to config.toml and send updated config back */
  private async _updateConfig(
    section: string,
    key: string,
    value: unknown
  ): Promise<void> {
    try {
      await writeConfigField(section, key, value);

      const config = await readConfig();
      void this._panel.webview.postMessage({
        type: 'configUpdated',
        config,
      });
    } catch (err) {
      void this._panel.webview.postMessage({
        type: 'configError',
        error: err instanceof Error ? err.message : 'Failed to update config',
      });
    }
  }

  private _dispose(): void {
    ConfigPanel.currentPanel = undefined;
    this._panel.dispose();
    while (this._disposables.length) {
      const d = this._disposables.pop();
      if (d) { d.dispose(); }
    }
  }
}

// ─── Config TOML Helpers ────────────────────────────────────────────────

interface TomlConfig {
  [key: string]: unknown;
}

// ─── Kanban Providers ───────────────────────────────────────────────────
//
// The canonical list of kanban providers lives in the Rust
// `KanbanProviderType::ALL` catalog and is projected into the generated
// `KanbanConfig` type (one keyed sub-table per provider). This table mirrors
// that catalog for the webview's config write path. Every provider in the
// generated schema MUST have an entry here — `config-panel.test.ts` enforces
// it so a new provider can't be added to the schema without being wired up.

/** Per-provider metadata for the kanban config write path. */
export interface KanbanProviderMeta {
  /**
   * The form field whose value renames the provider's HashMap key — the Jira
   * domain, Linear workspace slug, or GitHub owner login.
   */
  instanceKeyField: string;
  /** Placeholder instance key used before the user names a real instance. */
  defaultInstanceKey: string;
}

/**
 * Canonical kanban providers keyed by their lowercase slug (matching the
 * generated `KanbanConfig` keys and `KanbanProviderType::slug()`).
 */
export const KANBAN_PROVIDERS: Record<string, KanbanProviderMeta> = {
  jira: { instanceKeyField: 'domain', defaultInstanceKey: 'your-org.atlassian.net' },
  linear: { instanceKeyField: 'team_id', defaultInstanceKey: 'default-team' },
  github: { instanceKeyField: 'owner', defaultInstanceKey: 'your-org' },
};

/** Slugs of every supported kanban provider, in catalog order. */
export const KANBAN_PROVIDER_SLUGS: string[] = Object.keys(KANBAN_PROVIDERS);

/** Project-level fields written into the first project sub-table by shorthand. */
const KANBAN_PROJECT_LEVEL_KEYS = [
  'sync_statuses',
  'collection_name',
  'sync_user_id',
  'type_mappings',
];

/**
 * Apply a single field update to a kanban provider's sub-table (mutates
 * `kanban`). Shared by every provider so adding a provider is a one-line entry
 * in {@link KANBAN_PROVIDERS} — no new branch here or in the message handler.
 *
 * @param kanban the `[kanban]` table from the parsed config
 * @param slug   provider slug (`jira`, `linear`, `github`, …)
 * @param key    field key emitted by the webview (`enabled`, `api_key_env`,
 *               the instance-key field, `project_key`, a project-level key, or
 *               a `projects.<id>.<field>` path)
 */
export function applyKanbanProviderField(
  kanban: TomlConfig,
  slug: string,
  key: string,
  value: unknown
): void {
  const meta = KANBAN_PROVIDERS[slug];
  if (!meta) {
    throw new Error(`Unknown kanban provider: ${slug}`);
  }

  if (!kanban[slug]) { kanban[slug] = {}; }
  const providerMap = kanban[slug] as TomlConfig;

  const instanceKeys = Object.keys(providerMap);
  const instanceKey = instanceKeys[0] ?? meta.defaultInstanceKey;
  if (!providerMap[instanceKey]) { providerMap[instanceKey] = {}; }
  const ws = providerMap[instanceKey] as TomlConfig;

  if (key === meta.instanceKeyField && typeof value === 'string' && value !== instanceKey) {
    // Rename the instance key (e.g. the Jira domain / GitHub owner)
    const existing = providerMap[instanceKey];
    delete providerMap[instanceKey];
    providerMap[value] = existing;
  } else if (key === 'project_key') {
    // Rename (or create) the first project sub-table
    if (!ws.projects) { ws.projects = {}; }
    const projects = ws.projects as TomlConfig;
    const oldKeys = Object.keys(projects);
    if (oldKeys.length > 0 && oldKeys[0]) {
      const oldProject = projects[oldKeys[0]];
      delete projects[oldKeys[0]];
      projects[value as string] = oldProject;
    } else {
      projects[value as string] = { sync_user_id: '' };
    }
  } else if (KANBAN_PROJECT_LEVEL_KEYS.includes(key)) {
    // Write to the first project sub-table
    if (!ws.projects) { ws.projects = {}; }
    const projects = ws.projects as TomlConfig;
    const projectKeys = Object.keys(projects);
    const projectKey = projectKeys[0] ?? 'default';
    if (!projects[projectKey]) { projects[projectKey] = {}; }
    (projects[projectKey] as TomlConfig)[key] = value;
  } else if (key.startsWith('projects.')) {
    // Multi-project writes: projects.{projectKey}.{field}
    const parts = key.split('.');
    if (parts.length >= 3 && parts[1]) {
      const pKey = parts[1];
      const field = parts.slice(2).join('.');
      if (!ws.projects) { ws.projects = {}; }
      const projects = ws.projects as TomlConfig;
      if (!projects[pKey]) { projects[pKey] = { sync_user_id: '' }; }
      (projects[pKey] as TomlConfig)[field] = value;
    }
  } else {
    // Instance-level scalar (enabled, api_key_env, email, …)
    ws[key] = value;
  }
}

/** Read config.toml and return as WebviewConfig (snake_case, no transformation) */
async function readConfig(): Promise<WebviewConfig> {
  const configPath = getResolvedConfigPath();
  const workDir = resolveWorkingDirectory();

  let raw = '';
  if (configPath) {
    try {
      raw = await fs.readFile(configPath, 'utf-8');
    } catch {
      // File doesn't exist — return defaults
    }
  }

  // The config is "present" only when the file exists with non-empty content.
  // An empty or missing file means the user hasn't run setup yet.
  const configExists = raw.trim().length > 0;

  let parsed: TomlConfig = {};
  if (configExists) {
    const { parse } = await importSmolToml();
    parsed = parse(raw);
  }

  // Return the parsed TOML directly — field names already match generated types
  return {
    config_path: configPath || '',
    working_directory: workDir,
    config_exists: configExists,
    config: parsed,
  };
}

/** Write a single field update to config.toml */
async function writeConfigField(
  section: string,
  key: string,
  value: unknown
): Promise<void> {
  const configPath = getResolvedConfigPath();
  if (!configPath) {
    throw new Error('No working directory configured');
  }

  const configDir = getConfigDir(resolveWorkingDirectory());
  await fs.mkdir(configDir, { recursive: true });

  let raw = '';
  try {
    raw = await fs.readFile(configPath, 'utf-8');
  } catch {
    // file doesn't exist yet
  }

  const { parse, stringify } = await importSmolToml();
  let parsed: TomlConfig = {};
  if (raw.trim()) {
    parsed = parse(raw);
  }

  // Kanban providers share one write path so every provider in
  // KANBAN_PROVIDERS — current and future — is handled identically. This
  // covers `kanban.jira`, `kanban.linear`, `kanban.github`, and any provider
  // added to the catalog later.
  if (section.startsWith('kanban.')) {
    if (!parsed.kanban) { parsed.kanban = {}; }
    applyKanbanProviderField(
      parsed.kanban as TomlConfig,
      section.slice('kanban.'.length),
      key,
      value
    );
  }

  // Apply the update based on section
  switch (section) {
    case 'primary':
      if (key === 'working_directory') {
        // Update VS Code setting, not the TOML file
        await vscode.workspace
          .getConfiguration('operator')
          .update('workingDirectory', value, vscode.ConfigurationTarget.Global);
        return; // Don't write to TOML
      }
      parsed[key] = value;
      break;

    case 'agents':
      if (!parsed.agents) { parsed.agents = {}; }
      (parsed.agents as TomlConfig)[key] = value;
      break;

    case 'sessions':
      if (!parsed.sessions) { parsed.sessions = {}; }
      (parsed.sessions as TomlConfig)[key] = value;
      break;

    case 'llm_tools':
      if (!parsed.llm_tools) { parsed.llm_tools = {}; }
      (parsed.llm_tools as TomlConfig)[key] = value;
      break;

    // kanban.* providers are handled by applyKanbanProviderField above.

    case 'git':
      if (!parsed.git) { parsed.git = {}; }
      (parsed.git as TomlConfig)[key] = value;
      break;

    case 'git.github':
      if (!parsed.git) { parsed.git = {}; }
      if (!(parsed.git as TomlConfig).github) { (parsed.git as TomlConfig).github = {}; }
      ((parsed.git as TomlConfig).github as TomlConfig)[key] = value;
      break;
  }

  const output = stringify(parsed);
  await fs.writeFile(configPath, output, 'utf-8');
}

/** Generate a random nonce for CSP */
function getNonce(): string {
  let text = '';
  const possible = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  for (let i = 0; i < 32; i++) {
    text += possible.charAt(Math.floor(Math.random() * possible.length));
  }
  return text;
}
