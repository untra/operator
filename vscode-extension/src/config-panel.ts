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
import {
  validateJiraCredentials,
  fetchJiraProjects,
  validateLinearCredentials,
} from './kanban-onboarding';
import { detectInstalledLlmTools } from './walkthrough';
import {
  getConfigDir,
  getResolvedConfigPath,
  resolveWorkingDirectory,
} from './config-paths';
import { OperatorApiClient, discoverApiUrl } from './api-client';

/** Message types from the webview */
interface WebviewMessage {
  type: string;
  [key: string]: unknown;
}

/** WebviewConfig shape matching the webview types */
interface WebviewConfig {
  config_path: string;
  working_directory: string;
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
  <script nonce="${nonce}" src="${scriptUri}"></script>
</body>
</html>`;
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
          this._panel.webview.postMessage({
            type: 'browseResult',
            field: message.field,
            path: fileUri[0].fsPath,
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
          this._panel.webview.postMessage({
            type: 'browseResult',
            field: message.field,
            path: folderUri[0].fsPath,
          });
          // Also persist to VS Code settings
          await vscode.workspace
            .getConfiguration('operator')
            .update('workingDirectory', folderUri[0].fsPath, vscode.ConfigurationTarget.Global);
        }
        break;
      }

      case 'validateJira': {
        const result = await validateJiraCredentials(
          message.domain as string,
          message.email as string,
          message.apiToken as string
        );

        let projects: Array<{ key: string; name: string }> = [];
        if (result.valid) {
          projects = await fetchJiraProjects(
            message.domain as string,
            message.email as string,
            message.apiToken as string
          );
        }

        this._panel.webview.postMessage({
          type: 'jiraValidationResult',
          result: {
            valid: result.valid,
            displayName: result.displayName,
            accountId: result.accountId,
            error: result.error,
            projects,
          },
        });
        break;
      }

      case 'validateLinear': {
        const result = await validateLinearCredentials(
          message.apiKey as string
        );

        this._panel.webview.postMessage({
          type: 'linearValidationResult',
          result: {
            valid: result.valid,
            userName: result.userName,
            orgName: result.orgName,
            userId: result.userId,
            error: result.error,
            teams: result.teams,
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
          this._panel.webview.postMessage({
            type: 'llmToolsDetected',
            config,
          });
        } catch {
          // If we can't read config, just send tool names for compatibility
          this._panel.webview.postMessage({
            type: 'llmToolsDetected',
            config: {
              config_path: configPath || '',
              working_directory: resolveWorkingDirectory(),
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
          this._panel.webview.postMessage({ type: 'apiHealthResult', reachable: true });
        } catch {
          this._panel.webview.postMessage({ type: 'apiHealthResult', reachable: false });
        }
        break;
      }

      case 'getProjects': {
        try {
          const workDir = resolveWorkingDirectory();
          const ticketsDir = workDir ? path.join(workDir, '.tickets') : undefined;
          const apiUrl = await discoverApiUrl(ticketsDir);
          const client = new OperatorApiClient(apiUrl);
          const projects = await client.getProjects();
          this._panel.webview.postMessage({ type: 'projectsLoaded', projects });
        } catch (err) {
          this._panel.webview.postMessage({
            type: 'projectsError',
            error: err instanceof Error ? err.message : 'Failed to load projects',
          });
        }
        break;
      }

      case 'assessProject': {
        const projectName = message.projectName as string;
        try {
          const workDir = resolveWorkingDirectory();
          const ticketsDir = workDir ? path.join(workDir, '.tickets') : undefined;
          const apiUrl = await discoverApiUrl(ticketsDir);
          const client = new OperatorApiClient(apiUrl);
          const result = await client.assessProject(projectName);
          this._panel.webview.postMessage({
            type: 'assessTicketCreated',
            ticketId: result.ticket_id,
            projectName: result.project_name,
          });
        } catch (err) {
          this._panel.webview.postMessage({
            type: 'assessTicketError',
            error: err instanceof Error ? err.message : 'Failed to create ASSESS ticket',
            projectName,
          });
        }
        break;
      }

      case 'openProjectFolder': {
        const projectPath = message.projectPath as string;
        if (projectPath) {
          const uri = vscode.Uri.file(projectPath);
          await vscode.commands.executeCommand('vscode.openFolder', uri, { forceNewWindow: true });
        }
        break;
      }

      case 'openExternal':
        vscode.env.openExternal(vscode.Uri.parse(message.url as string));
        break;

      case 'openFile': {
        const filePath = message.filePath as string;
        if (filePath) {
          const doc = await vscode.workspace.openTextDocument(filePath);
          await vscode.window.showTextDocument(doc);
        }
        break;
      }
    }
  }

  /** Read config.toml and send as WebviewConfig to webview */
  private async _sendConfig(): Promise<void> {
    try {
      const config = await readConfig();
      this._panel.webview.postMessage({
        type: 'configLoaded',
        config,
      });
    } catch (err) {
      this._panel.webview.postMessage({
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
      this._panel.webview.postMessage({
        type: 'configUpdated',
        config,
      });
    } catch (err) {
      this._panel.webview.postMessage({
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

  let parsed: TomlConfig = {};
  if (raw.trim()) {
    const { parse } = await importSmolToml();
    parsed = parse(raw) as TomlConfig;
  }

  // Return the parsed TOML directly — field names already match generated types
  return {
    config_path: configPath || '',
    working_directory: workDir,
    config: parsed as Record<string, unknown>,
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
    parsed = parse(raw) as TomlConfig;
  }

  // Apply the update based on section
  switch (section) {
    case 'primary':
      if (key === 'working_directory') {
        // Update VS Code setting, not the TOML file
        await vscode.workspace
          .getConfiguration('operator')
          .update('workingDirectory', value as string, vscode.ConfigurationTarget.Global);
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

    case 'kanban.jira': {
      if (!parsed.kanban) { parsed.kanban = {}; }
      const kanban = parsed.kanban as TomlConfig;
      if (!kanban.jira) { kanban.jira = {}; }
      const jira = kanban.jira as TomlConfig;

      // Get existing domain or create placeholder
      const jiraKeys = Object.keys(jira);
      const domain = jiraKeys[0] ?? 'your-org.atlassian.net';

      if (!jira[domain]) { jira[domain] = {}; }
      const ws = jira[domain] as TomlConfig;

      if (key === 'domain' && typeof value === 'string' && value !== domain) {
        // Rename the domain key
        const existing = jira[domain];
        delete jira[domain];
        jira[value] = existing;
      } else if (key === 'project_key') {
        // Handle project key under projects sub-table
        if (!ws.projects) { ws.projects = {}; }
        const projects = ws.projects as TomlConfig;
        const oldKeys = Object.keys(projects);
        if (oldKeys.length > 0) {
          const oldProject = projects[oldKeys[0]];
          delete projects[oldKeys[0]];
          projects[value as string] = oldProject;
        } else {
          projects[value as string] = { sync_user_id: '', collection_name: 'dev_kanban' };
        }
      } else if (key === 'sync_statuses' || key === 'collection_name' || key === 'sync_user_id') {
        // Write to the first project sub-table
        if (!ws.projects) { ws.projects = {}; }
        const projects = ws.projects as TomlConfig;
        const projectKeys = Object.keys(projects);
        const projectKey = projectKeys[0] ?? 'default';
        if (!projects[projectKey]) { projects[projectKey] = {}; }
        (projects[projectKey] as TomlConfig)[key] = value;
      } else {
        ws[key] = value;
      }
      break;
    }

    case 'kanban.linear': {
      if (!parsed.kanban) { parsed.kanban = {}; }
      const kanban = parsed.kanban as TomlConfig;
      if (!kanban.linear) { kanban.linear = {}; }
      const linear = kanban.linear as TomlConfig;

      const linearKeys = Object.keys(linear);
      const teamId = linearKeys[0] ?? 'default-team';

      if (!linear[teamId]) { linear[teamId] = {}; }
      const ws = linear[teamId] as TomlConfig;

      if (key === 'team_id' && typeof value === 'string' && value !== teamId) {
        const existing = linear[teamId];
        delete linear[teamId];
        linear[value] = existing;
      } else if (key === 'sync_statuses' || key === 'collection_name' || key === 'sync_user_id') {
        // Write to the first project sub-table
        if (!ws.projects) { ws.projects = {}; }
        const projects = ws.projects as TomlConfig;
        const projectKeys = Object.keys(projects);
        const projectKey = projectKeys[0] ?? 'default';
        if (!projects[projectKey]) { projects[projectKey] = {}; }
        (projects[projectKey] as TomlConfig)[key] = value;
      } else {
        ws[key] = value;
      }
      break;
    }

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

  const output = stringify(parsed as Record<string, unknown>);
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
