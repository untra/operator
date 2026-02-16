/**
 * Status TreeDataProvider for Operator VS Code extension
 *
 * Displays a hierarchical status tree with 5 collapsible sections:
 * 1. Configuration — working directory + config.toml
 * 2. Kanban — connected providers and workspaces
 * 3. LLM Tools — detected CLI tools
 * 4. Git — provider, token, branch format
 * 5. Connections — API + Webhook status
 *
 * Unconfigured sections expand automatically with nudge items
 * that link to the relevant setup command.
 */

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';
import { SessionInfo } from './types';
import { discoverApiUrl, ApiSessionInfo } from './api-client';
import {
  resolveWorkingDirectory,
  configFileExists,
  getResolvedConfigPath,
} from './config-paths';
import {
  detectInstalledLlmTools,
  getKanbanWorkspaces,
  DetectedToolResult,
} from './walkthrough';

// smol-toml is ESM-only, must use dynamic import
async function importSmolToml() {
  return await import('smol-toml');
}

/**
 * Webhook server connection status
 */
export interface WebhookStatus {
  running: boolean;
  version?: string;
  port?: number;
  workspace?: string;
  sessionFile?: string;
}

/**
 * Operator REST API connection status
 */
export interface ApiStatus {
  connected: boolean;
  version?: string;
  port?: number;
  url?: string;
}

/** Internal state for the Configuration section */
interface ConfigState {
  workingDirSet: boolean;
  workingDir: string;
  configExists: boolean;
  configPath: string;
}

/** Config-driven state for a single kanban provider */
interface KanbanProviderState {
  provider: 'jira' | 'linear';
  key: string;              // domain for jira, teamId for linear
  enabled: boolean;
  displayName: string;      // domain for jira, or team name
  url: string;              // e.g. "https://myorg.atlassian.net"
  projects: Array<{
    key: string;            // project key or "default"
    collectionName: string;
    url: string;            // e.g. "https://myorg.atlassian.net/browse/PROJ"
  }>;
}

/** Internal state for the Kanban section */
interface KanbanState {
  configured: boolean;
  providers: KanbanProviderState[];
}

/** Internal state for the LLM Tools section */
interface LlmState {
  detected: boolean;
  tools: DetectedToolResult[];
  configDetected: Array<{ name: string; version?: string }>;
}

/** Internal state for the Git section */
interface GitState {
  configured: boolean;
  provider?: string;
  githubEnabled?: boolean;
  tokenSet?: boolean;
  branchFormat?: string;
  useWorktrees?: boolean;
}

/**
 * TreeDataProvider for hierarchical status information
 */
export class StatusTreeProvider implements vscode.TreeDataProvider<StatusItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<
    StatusItem | undefined
  >();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private context: vscode.ExtensionContext;
  private parsedConfig: Record<string, unknown> | null = null;

  private webhookStatus: WebhookStatus = { running: false };
  private apiStatus: ApiStatus = { connected: false };
  private ticketsDir: string | undefined;

  private configState: ConfigState = {
    workingDirSet: false,
    workingDir: '',
    configExists: false,
    configPath: '',
  };
  private kanbanState: KanbanState = { configured: false, providers: [] };
  private llmState: LlmState = { detected: false, tools: [], configDetected: [] };
  private gitState: GitState = { configured: false };

  constructor(context: vscode.ExtensionContext) {
    this.context = context;
  }

  async setTicketsDir(dir: string | undefined): Promise<void> {
    this.ticketsDir = dir;
    await this.refresh();
  }

  async refresh(): Promise<void> {
    this.parsedConfig = null;

    await Promise.all([
      this.checkConfigState(),
      this.checkKanbanState(),
      this.checkLlmState(),
      this.checkGitState(),
      this.checkWebhookStatus(),
      this.checkApiStatus(),
    ]);

    this._onDidChangeTreeData.fire(undefined);
  }

  /**
   * Read and cache config.toml
   */
  private async readConfigToml(): Promise<Record<string, unknown>> {
    if (this.parsedConfig) {
      return this.parsedConfig;
    }

    const configPath = getResolvedConfigPath();
    if (!configPath) {
      this.parsedConfig = {};
      return this.parsedConfig;
    }

    try {
      const raw = await fs.readFile(configPath, 'utf-8');
      if (raw.trim()) {
        const { parse } = await importSmolToml();
        this.parsedConfig = parse(raw) as Record<string, unknown>;
      } else {
        this.parsedConfig = {};
      }
    } catch {
      this.parsedConfig = {};
    }

    return this.parsedConfig;
  }

  /**
   * Check Configuration section state
   */
  private async checkConfigState(): Promise<void> {
    const workingDir = this.context.globalState.get<string>('operator.workingDirectory')
      || resolveWorkingDirectory();
    const workingDirSet = !!workingDir;
    const configExists = await configFileExists();
    const configPath = getResolvedConfigPath();

    this.configState = {
      workingDirSet,
      workingDir: workingDir || '',
      configExists,
      configPath: configPath || '',
    };
  }

  /**
   * Check Kanban section state from config.toml, falling back to env vars
   */
  private async checkKanbanState(): Promise<void> {
    const config = await this.readConfigToml();
    const kanbanSection = config.kanban as Record<string, unknown> | undefined;
    const providers: KanbanProviderState[] = [];

    if (kanbanSection) {
      // Parse Jira providers from config.toml
      const jiraSection = kanbanSection.jira as Record<string, unknown> | undefined;
      if (jiraSection) {
        for (const [domain, wsConfig] of Object.entries(jiraSection)) {
          const ws = wsConfig as Record<string, unknown>;
          if (ws.enabled === false) { continue; }
          const projects: KanbanProviderState['projects'] = [];
          const projectsSection = ws.projects as Record<string, unknown> | undefined;
          if (projectsSection) {
            for (const [projectKey, projConfig] of Object.entries(projectsSection)) {
              const proj = projConfig as Record<string, unknown>;
              projects.push({
                key: projectKey,
                collectionName: (proj.collection_name as string) || 'dev_kanban',
                url: `https://${domain}/browse/${projectKey}`,
              });
            }
          }
          providers.push({
            provider: 'jira',
            key: domain,
            enabled: ws.enabled !== false,
            displayName: domain,
            url: `https://${domain}`,
            projects,
          });
        }
      }

      // Parse Linear providers from config.toml
      const linearSection = kanbanSection.linear as Record<string, unknown> | undefined;
      if (linearSection) {
        for (const [teamId, wsConfig] of Object.entries(linearSection)) {
          const ws = wsConfig as Record<string, unknown>;
          if (ws.enabled === false) { continue; }
          const projects: KanbanProviderState['projects'] = [];
          const projectsSection = ws.projects as Record<string, unknown> | undefined;
          if (projectsSection) {
            for (const [projectKey, projConfig] of Object.entries(projectsSection)) {
              const proj = projConfig as Record<string, unknown>;
              projects.push({
                key: projectKey,
                collectionName: (proj.collection_name as string) || 'dev_kanban',
                url: `https://linear.app/team/${projectKey}`,
              });
            }
          }
          providers.push({
            provider: 'linear',
            key: teamId,
            enabled: ws.enabled !== false,
            displayName: teamId,
            url: 'https://linear.app',
            projects,
          });
        }
      }
    }

    // Fall back to env-var-based detection if config.toml has no kanban section
    if (providers.length === 0) {
      const workspaces = await getKanbanWorkspaces();
      for (const ws of workspaces) {
        providers.push({
          provider: ws.provider,
          key: ws.name,
          enabled: ws.configured,
          displayName: ws.name,
          url: ws.url,
          projects: [],
        });
      }
    }

    this.kanbanState = {
      configured: providers.length > 0,
      providers,
    };
  }

  /**
   * Check LLM Tools section state
   */
  private async checkLlmState(): Promise<void> {
    const tools = await detectInstalledLlmTools();

    // Also check config.toml for richer detected tool info
    const config = await this.readConfigToml();
    const llmTools = config.llm_tools as Record<string, unknown> | undefined;
    const configDetected = Array.isArray(llmTools?.detected)
      ? (llmTools.detected as Array<string | { name: string; version?: string }>).map(
          (entry) => {
            if (typeof entry === 'string') {
              return { name: entry };
            }
            return { name: entry.name, version: entry.version };
          }
        )
      : [];

    this.llmState = {
      detected: tools.length > 0 || configDetected.length > 0,
      tools,
      configDetected,
    };
  }

  /**
   * Check Git section state
   */
  private async checkGitState(): Promise<void> {
    const config = await this.readConfigToml();
    const gitSection = config.git as Record<string, unknown> | undefined;

    if (!gitSection) {
      this.gitState = { configured: false };
      return;
    }

    const provider = gitSection.provider as string | undefined;
    const github = gitSection.github as Record<string, unknown> | undefined;
    const githubEnabled = github?.enabled as boolean | undefined;
    const branchFormat = gitSection.branch_format as string | undefined;
    const useWorktrees = gitSection.use_worktrees as boolean | undefined;

    // Check GitHub token from env
    const tokenEnv = (github?.token_env as string) || 'GITHUB_TOKEN';
    const tokenSet = !!process.env[tokenEnv];

    const configured = !!(provider || githubEnabled);

    this.gitState = {
      configured,
      provider,
      githubEnabled,
      tokenSet,
      branchFormat,
      useWorktrees,
    };
  }

  /**
   * Check webhook server status via session file
   */
  private async checkWebhookStatus(): Promise<void> {
    if (!this.ticketsDir) {
      this.webhookStatus = { running: false };
      return;
    }

    const webhookSessionFile = path.join(this.ticketsDir, 'operator', 'vscode-session.json');
    try {
      const content = await fs.readFile(webhookSessionFile, 'utf-8');
      const session: SessionInfo = JSON.parse(content);

      this.webhookStatus = {
        running: true,
        version: session.version,
        port: session.port,
        workspace: session.workspace,
        sessionFile: webhookSessionFile,
      };
    } catch {
      this.webhookStatus = { running: false };
    }
  }

  /**
   * Check API status
   */
  private async checkApiStatus(): Promise<void> {
    if (this.ticketsDir) {
      const apiSessionFile = path.join(this.ticketsDir, 'operator', 'api-session.json');
      try {
        const content = await fs.readFile(apiSessionFile, 'utf-8');
        const session: ApiSessionInfo = JSON.parse(content);
        const apiUrl = `http://localhost:${session.port}`;

        if (await this.tryHealthCheck(apiUrl, session.version)) {
          return;
        }
      } catch {
        // Fall through
      }
    }

    const apiUrl = await discoverApiUrl(this.ticketsDir);
    await this.tryHealthCheck(apiUrl);
  }

  /**
   * Attempt a health check against the given API URL
   */
  private async tryHealthCheck(apiUrl: string, sessionVersion?: string): Promise<boolean> {
    try {
      const response = await fetch(`${apiUrl}/api/v1/health`);
      if (response.ok) {
        const health = await response.json() as { version?: string };
        const port = new URL(apiUrl).port;
        this.apiStatus = {
          connected: true,
          version: health.version || sessionVersion,
          port: port ? parseInt(port, 10) : 7008,
          url: apiUrl,
        };
        return true;
      }
    } catch {
      // Health check failed
    }
    this.apiStatus = { connected: false };
    return false;
  }

  getTreeItem(element: StatusItem): vscode.TreeItem {
    return element;
  }

  getChildren(element?: StatusItem): StatusItem[] {
    if (!element) {
      return this.getTopLevelSections();
    }

    switch (element.sectionId) {
      case 'config':
        return this.getConfigChildren();
      case 'kanban':
        return this.getKanbanChildren();
      case 'llm':
        return this.getLlmChildren();
      case 'git':
        return this.getGitChildren();
      case 'connections':
        return this.getConnectionsChildren();
    }

    // Workspace-level items return their project children
    if (element.provider && element.workspaceKey && !element.projectKey) {
      return this.getKanbanProjectChildren(element.provider, element.workspaceKey);
    }

    return [];
  }

  /**
   * Top-level collapsible sections
   */
  private getTopLevelSections(): StatusItem[] {
    const configuredBoth = this.configState.workingDirSet && this.configState.configExists;

    return [
      // 1. Configuration
      new StatusItem({
        label: 'Configuration',
        description: configuredBoth
          ? path.basename(this.configState.workingDir)
          : 'Setup required',
        icon: configuredBoth ? 'check' : 'debug-configure',
        collapsibleState: configuredBoth
          ? vscode.TreeItemCollapsibleState.Collapsed
          : vscode.TreeItemCollapsibleState.Expanded,
        sectionId: 'config',
        command: configuredBoth ? undefined : {
          command: 'operator.selectWorkingDirectory',
          title: 'Select Working Directory',
        },
      }),
      // 2. Connections
      new StatusItem({
        label: 'Connections',
        description: this.getConnectionsSummary(),
        icon: this.getConnectionsIcon(),
        collapsibleState: vscode.TreeItemCollapsibleState.Collapsed,
        sectionId: 'connections',
      }),

      // 3. Kanban Providers
      new StatusItem({
        label: 'Kanban',
        description: this.kanbanState.configured
          ? this.getKanbanSummary()
          : 'No provider connected',
        icon: this.kanbanState.configured ? 'check' : 'warning',
        collapsibleState: this.kanbanState.configured
          ? vscode.TreeItemCollapsibleState.Collapsed
          : vscode.TreeItemCollapsibleState.Expanded,
        sectionId: 'kanban',
        command: this.kanbanState.configured ? undefined : {
          command: 'operator.startKanbanOnboarding',
          title: 'Configure Kanban',
        },
      }),

      // 4. LLM Tools
      new StatusItem({
        label: 'LLM Tools',
        description: this.llmState.detected
          ? this.getLlmSummary()
          : 'No tools detected',
        icon: this.llmState.detected ? 'check' : 'warning',
        collapsibleState: this.llmState.detected
          ? vscode.TreeItemCollapsibleState.Collapsed
          : vscode.TreeItemCollapsibleState.Expanded,
        sectionId: 'llm',
        command: this.llmState.detected ? undefined : {
          command: 'operator.detectLlmTools',
          title: 'Detect LLM Tools',
        },
      }),

      // 5. Git Provider
      new StatusItem({
        label: 'Git',
        description: this.gitState.configured
          ? (this.gitState.provider || 'GitHub')
          : 'Not configured',
        icon: this.gitState.configured ? 'check' : 'warning',
        collapsibleState: this.gitState.configured
          ? vscode.TreeItemCollapsibleState.Collapsed
          : vscode.TreeItemCollapsibleState.Expanded,
        sectionId: 'git',
        command: this.gitState.configured ? undefined : {
          command: 'operator.openSettings',
          title: 'Open Settings',
        },
      }),
    ];
  }

  // ─── Section Children ──────────────────────────────────────────────────

  private getConfigChildren(): StatusItem[] {
    const items: StatusItem[] = [];

    // Working Directory
    items.push(new StatusItem({
      label: 'Working Directory',
      description: this.configState.workingDirSet
        ? this.configState.workingDir
        : 'Not set',
      icon: this.configState.workingDirSet ? 'folder-opened' : 'folder',
      command: {
        command: 'operator.selectWorkingDirectory',
        title: 'Select Working Directory',
      },
    }));

    // Config File
    items.push(new StatusItem({
      label: 'Config File',
      description: this.configState.configExists
        ? this.configState.configPath
        : 'Not found',
      icon: this.configState.configExists ? 'file' : 'file-add',
      command: {
        command: 'operator.openSettings',
        title: 'Open Settings',
      },
    }));

    // Tickets directory — click reveals in file explorer
    if (this.ticketsDir) {
      items.push(new StatusItem({
        label: 'Tickets',
        description: this.ticketsDir,
        icon: 'markdown',
        command: {
          command: 'operator.revealTicketsDir',
          title: 'Reveal in Explorer',
        },
      }));
    } else {
      items.push(new StatusItem({
        label: 'Tickets',
        description: 'Not found',
        icon: 'markdown',
        tooltip: 'No .tickets directory found',
      }));
    }

    return items;
  }

  private getKanbanChildren(): StatusItem[] {
    const items: StatusItem[] = [];

    if (this.kanbanState.configured) {
      // One collapsible item per workspace
      for (const prov of this.kanbanState.providers) {
        const providerLabel = prov.provider === 'jira' ? 'Jira' : 'Linear';
        const hasProjects = prov.projects.length > 0;
        items.push(new StatusItem({
          label: providerLabel,
          description: prov.displayName,
          icon: 'cloud',
          tooltip: prov.url,
          collapsibleState: hasProjects
            ? vscode.TreeItemCollapsibleState.Collapsed
            : vscode.TreeItemCollapsibleState.None,
          command: {
            command: 'vscode.open',
            title: 'Open in Browser',
            arguments: [vscode.Uri.parse(prov.url)],
          },
          contextValue: 'kanbanWorkspace',
          provider: prov.provider,
          workspaceKey: prov.key,
        }));
      }

      // Add provider action
      items.push(new StatusItem({
        label: 'Add Provider',
        icon: 'add',
        command: {
          command: 'operator.startKanbanOnboarding',
          title: 'Add Kanban Provider',
        },
      }));
    } else {
      // Nudge items
      items.push(new StatusItem({
        label: 'Configure Jira',
        icon: 'cloud',
        command: {
          command: 'operator.configureJira',
          title: 'Configure Jira',
        },
      }));
      items.push(new StatusItem({
        label: 'Configure Linear',
        icon: 'cloud',
        command: {
          command: 'operator.configureLinear',
          title: 'Configure Linear',
        },
      }));
    }

    return items;
  }

  private getKanbanProjectChildren(provider: string, workspaceKey: string): StatusItem[] {
    const items: StatusItem[] = [];
    const prov = this.kanbanState.providers.find(
      (p) => p.provider === provider && p.key === workspaceKey
    );
    if (!prov) { return items; }

    // Project/team sync config items
    for (const proj of prov.projects) {
      items.push(new StatusItem({
        label: proj.key,
        description: proj.collectionName,
        icon: 'package',
        tooltip: proj.url,
        command: {
          command: 'vscode.open',
          title: 'Open in Browser',
          arguments: [vscode.Uri.parse(proj.url)],
        },
        contextValue: 'kanbanSyncConfig',
        provider: prov.provider,
        workspaceKey: prov.key,
        projectKey: proj.key,
      }));
    }

    // Add Project / Add Team button
    const addLabel = provider === 'jira' ? 'Add Jira Project' : 'Add Linear Team';
    const addCommand = provider === 'jira' ? 'operator.addJiraProject' : 'operator.addLinearTeam';
    items.push(new StatusItem({
      label: addLabel,
      icon: 'add',
      command: {
        command: addCommand,
        title: addLabel,
        arguments: [workspaceKey],
      },
    }));

    return items;
  }

  private getLlmChildren(): StatusItem[] {
    const items: StatusItem[] = [];

    if (this.llmState.detected) {
      // Show config-detected tools first (richer info)
      const shown = new Set<string>();

      for (const tool of this.llmState.configDetected) {
        shown.add(tool.name);
        items.push(new StatusItem({
          label: tool.name,
          description: tool.version,
          icon: 'terminal',
        }));
      }

      // Show PATH-detected tools not already in config
      for (const tool of this.llmState.tools) {
        if (!shown.has(tool.name)) {
          items.push(new StatusItem({
            label: tool.name,
            description: tool.version !== 'unknown' ? tool.version : undefined,
            icon: 'terminal',
          }));
        }
      }

      // Detect action
      items.push(new StatusItem({
        label: 'Detect Tools',
        icon: 'search',
        command: {
          command: 'operator.detectLlmTools',
          title: 'Detect LLM Tools',
        },
      }));
    } else {
      // Nudge items
      items.push(new StatusItem({
        label: 'Detect Tools',
        icon: 'search',
        command: {
          command: 'operator.detectLlmTools',
          title: 'Detect LLM Tools',
        },
      }));
      items.push(new StatusItem({
        label: 'Install Claude Code',
        icon: 'link-external',
        command: {
          command: 'vscode.open',
          title: 'Install Claude Code',
          arguments: [vscode.Uri.parse('https://docs.anthropic.com/en/docs/claude-code')],
        },
      }));
    }

    return items;
  }

  private getGitChildren(): StatusItem[] {
    const items: StatusItem[] = [];

    if (this.gitState.configured) {
      // Provider
      items.push(new StatusItem({
        label: 'Provider',
        description: this.gitState.provider || 'GitHub',
        icon: 'source-control',
      }));

      // GitHub Token
      items.push(new StatusItem({
        label: 'GitHub Token',
        description: this.gitState.tokenSet ? 'Set' : 'Not set',
        icon: 'key',
      }));

      // Branch Format
      if (this.gitState.branchFormat) {
        items.push(new StatusItem({
          label: 'Branch Format',
          description: this.gitState.branchFormat,
          icon: 'git-branch',
        }));
      }

      // Worktrees
      items.push(new StatusItem({
        label: 'Worktrees',
        description: this.gitState.useWorktrees ? 'Enabled' : 'Disabled',
        icon: 'git-merge',
      }));
    } else {
      // Nudge item
      items.push(new StatusItem({
        label: 'Open Settings',
        icon: 'gear',
        command: {
          command: 'operator.openSettings',
          title: 'Open Settings',
        },
      }));
    }

    return items;
  }

  private getConnectionsChildren(): StatusItem[] {
    const items: StatusItem[] = [];

    // REST API
    if (this.apiStatus.connected) {
      items.push(new StatusItem({
        label: 'API',
        description: this.apiStatus.url || '',
        icon: 'pass',
        tooltip: `Operator REST API at ${this.apiStatus.url}`,
      }));
      if (this.apiStatus.version) {
        items.push(new StatusItem({
          label: 'API Version',
          description: this.apiStatus.version,
          icon: 'versions',
        }));
      }
      if (this.apiStatus.port) {
        items.push(new StatusItem({
          label: 'API Port',
          description: this.apiStatus.port.toString(),
          icon: 'plug',
        }));
      }
    } else {
      items.push(new StatusItem({
        label: 'API',
        description: 'Disconnected',
        icon: 'error',
        tooltip: 'Operator REST API not running. Use "Operator: Download Operator" command if not installed.',
      }));
    }

    // Webhook
    if (this.webhookStatus.running) {
      items.push(new StatusItem({
        label: 'Webhook',
        description: 'Running',
        icon: 'pass',
        tooltip: 'Local webhook server for terminal management',
      }));
      if (this.webhookStatus.port) {
        items.push(new StatusItem({
          label: 'Webhook Port',
          description: this.webhookStatus.port.toString(),
          icon: 'plug',
        }));
      }
    } else {
      items.push(new StatusItem({
        label: 'Webhook',
        description: 'Stopped',
        icon: 'circle-slash',
        tooltip: 'Local webhook server not running',
      }));
    }

    return items;
  }

  // ─── Summary Helpers ───────────────────────────────────────────────────

  private getKanbanSummary(): string {
    const prov = this.kanbanState.providers[0];
    if (!prov) {
      return '';
    }
    const provider = prov.provider === 'jira' ? 'Jira' : 'Linear';
    return `${provider}: ${prov.displayName}`;
  }

  private getLlmSummary(): string {
    // Prefer config-detected (has version info)
    if (this.llmState.configDetected.length > 0) {
      const first = this.llmState.configDetected[0];
      return first.version ? `${first.name} v${first.version}` : first.name;
    }
    // Fall back to PATH-detected
    if (this.llmState.tools.length > 0) {
      const first = this.llmState.tools[0];
      return first.version !== 'unknown' ? `${first.name} v${first.version}` : first.name;
    }
    return '';
  }

  private getConnectionsSummary(): string {
    if (this.apiStatus.connected && this.webhookStatus.running) {
      return 'All connected';
    }
    if (this.apiStatus.connected || this.webhookStatus.running) {
      return 'Partial';
    }
    return 'Disconnected';
  }

  private getConnectionsIcon(): string {
    if (this.apiStatus.connected && this.webhookStatus.running) {
      return 'pass';
    }
    if (this.apiStatus.connected || this.webhookStatus.running) {
      return 'warning';
    }
    return 'error';
  }
}

/**
 * StatusItem options
 */
interface StatusItemOptions {
  label: string;
  description?: string;
  icon: string;
  tooltip?: string;
  collapsibleState?: vscode.TreeItemCollapsibleState;
  command?: vscode.Command;
  sectionId?: string;
  contextValue?: string;    // for view/item/context when clause
  provider?: string;        // 'jira' | 'linear'
  workspaceKey?: string;    // domain or teamId (config key)
  projectKey?: string;      // project/team sync config key
}

/**
 * TreeItem for status display
 */
export class StatusItem extends vscode.TreeItem {
  public readonly sectionId?: string;
  public readonly provider?: string;
  public readonly workspaceKey?: string;
  public readonly projectKey?: string;

  constructor(opts: StatusItemOptions) {
    super(
      opts.label,
      opts.collapsibleState ?? vscode.TreeItemCollapsibleState.None
    );
    this.sectionId = opts.sectionId;
    this.provider = opts.provider;
    this.workspaceKey = opts.workspaceKey;
    this.projectKey = opts.projectKey;
    if (opts.description !== undefined) {
      this.description = opts.description;
    }
    this.tooltip = opts.tooltip || (opts.description
      ? `${opts.label}: ${opts.description}`
      : opts.label);
    this.iconPath = new vscode.ThemeIcon(opts.icon);
    if (opts.command) {
      this.command = opts.command;
    }
    if (opts.contextValue) {
      this.contextValue = opts.contextValue;
    }
  }
}
