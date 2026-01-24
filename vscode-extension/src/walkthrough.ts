/**
 * Walkthrough module for Operator VS Code extension
 *
 * Provides setup wizard functionality to guide users through:
 * 1. Selecting a working directory
 * 2. Connecting a kanban provider (Jira/Linear)
 * 3. Installing an LLM tool (Claude Code/Codex/Gemini CLI)
 */

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

/** Kanban provider types */
export type KanbanProviderType = 'jira' | 'linear';

/** Detected kanban workspace with connection details */
export interface KanbanWorkspace {
  provider: KanbanProviderType;
  name: string; // Workspace name (e.g., "Untra Operator" or domain)
  url: string; // Linkable URL (e.g., "https://untraoperator.atlassian.net")
  configured: boolean; // Whether API key is set
}

/** Result of kanban environment check */
export interface KanbanEnvResult {
  workspaces: KanbanWorkspace[];
  anyConfigured: boolean;
}

/** State of walkthrough completion */
export interface WalkthroughState {
  workingDirectorySet: boolean;
  kanbanConnected: boolean;
  llmToolInstalled: boolean;
  workingDirectory?: string;
  kanbanWorkspaces: KanbanWorkspace[];
  installedLlmTools: string[];
}

/** Environment variable names for kanban providers */
export const KANBAN_ENV_VARS = {
  jira: {
    apiKey: ['OPERATOR_JIRA_API_KEY', 'JIRA_API_TOKEN'] as const,
    domain: ['OPERATOR_JIRA_DOMAIN'] as const,
    email: ['OPERATOR_JIRA_EMAIL'] as const,
  },
  linear: {
    apiKey: ['OPERATOR_LINEAR_API_KEY', 'LINEAR_API_KEY'] as const,
  },
} as const;

/** Linear GraphQL API URL */
const LINEAR_API_URL = 'https://api.linear.app/graphql';

/**
 * Find the first set environment variable from a list of keys
 */
export function findEnvVar(keys: readonly string[]): string | undefined {
  for (const key of keys) {
    const value = process.env[key];
    if (value) {
      return value;
    }
  }
  return undefined;
}

/** LLM tools to detect */
export const LLM_TOOLS = ['claude', 'codex', 'gemini'] as const;

/**
 * Check if kanban environment variables are set and return workspace info
 */
export function checkKanbanEnvVars(): KanbanEnvResult {
  const workspaces: KanbanWorkspace[] = [];

  // Check Jira - requires domain to build URL
  const jiraApiKey = findEnvVar(KANBAN_ENV_VARS.jira.apiKey);
  const jiraDomain = findEnvVar(KANBAN_ENV_VARS.jira.domain);
  if (jiraApiKey && jiraDomain) {
    workspaces.push({
      provider: 'jira',
      name: jiraDomain,
      url: `https://${jiraDomain}`,
      configured: true,
    });
  }

  // Check Linear - API key only, name/url fetched async
  const linearApiKey = findEnvVar(KANBAN_ENV_VARS.linear.apiKey);
  if (linearApiKey) {
    workspaces.push({
      provider: 'linear',
      name: 'Linear', // Placeholder, updated by fetchLinearWorkspace
      url: 'https://linear.app',
      configured: true,
    });
  }

  return {
    workspaces,
    anyConfigured: workspaces.length > 0,
  };
}

/**
 * Fetch Linear organization details via GraphQL API
 */
export async function fetchLinearWorkspace(
  apiKey: string
): Promise<{ name: string; url: string } | null> {
  const query = `
    query {
      organization {
        id
        name
        urlKey
      }
    }
  `;

  try {
    const response = await fetch(LINEAR_API_URL, {
      method: 'POST',
      headers: {
        Authorization: apiKey,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ query }),
    });

    if (!response.ok) {
      return null;
    }

    const data = (await response.json()) as {
      data?: { organization?: { name?: string; urlKey?: string } };
    };
    const org = data?.data?.organization;
    if (!org?.name || !org?.urlKey) {
      return null;
    }

    return {
      name: org.name,
      url: `https://linear.app/${org.urlKey}`,
    };
  } catch {
    return null;
  }
}

/**
 * Get all kanban workspaces with enhanced details (async for API calls)
 */
export async function getKanbanWorkspaces(): Promise<KanbanWorkspace[]> {
  const envResult = checkKanbanEnvVars();
  const workspaces = [...envResult.workspaces];

  // Enhance Linear workspace with org details
  const linearIdx = workspaces.findIndex((w) => w.provider === 'linear');
  if (linearIdx >= 0) {
    const apiKey = findEnvVar(KANBAN_ENV_VARS.linear.apiKey);
    if (apiKey) {
      const orgInfo = await fetchLinearWorkspace(apiKey);
      if (orgInfo) {
        workspaces[linearIdx] = {
          ...workspaces[linearIdx],
          name: orgInfo.name,
          url: orgInfo.url,
        };
      }
    }
  }

  return workspaces;
}

/**
 * Check if an LLM tool is available in PATH
 */
export async function checkLlmToolInPath(tool: string): Promise<boolean> {
  try {
    const command = process.platform === 'win32' ? 'where' : 'which';
    await execAsync(`${command} ${tool}`);
    return true;
  } catch {
    return false;
  }
}

/**
 * Detect all installed LLM tools
 */
export async function detectInstalledLlmTools(): Promise<string[]> {
  const results: (string | null)[] = await Promise.all(
    LLM_TOOLS.map(async (tool): Promise<string | null> => {
      const installed = await checkLlmToolInPath(tool);
      return installed ? tool : null;
    })
  );
  return results.filter((tool): tool is string => tool !== null);
}

/**
 * Validate that a path is a valid directory
 */
export async function validateWorkingDirectory(dirPath: string): Promise<boolean> {
  try {
    const stat = await fs.stat(dirPath);
    return stat.isDirectory();
  } catch {
    return false;
  }
}

/**
 * Initialize .tickets directory structure
 * Calls operator setup if available, otherwise creates directories manually
 */
export async function initializeTicketsDirectory(
  workingDir: string,
  operatorPath?: string
): Promise<boolean> {
  const ticketsDir = path.join(workingDir, '.tickets');

  try {
    // Try using operator CLI if available
    if (operatorPath) {
      try {
        await execAsync(`"${operatorPath}" setup --working-dir "${workingDir}"`);
        return true;
      } catch {
        // Fall through to manual creation
      }
    }

    // Manual creation of directory structure
    const dirs = [
      ticketsDir,
      path.join(ticketsDir, 'queue'),
      path.join(ticketsDir, 'in-progress'),
      path.join(ticketsDir, 'completed'),
      path.join(ticketsDir, 'operator'),
    ];

    for (const dir of dirs) {
      await fs.mkdir(dir, { recursive: true });
    }

    return true;
  } catch (error) {
    console.error('Failed to initialize tickets directory:', error);
    return false;
  }
}

/**
 * Update all walkthrough context keys in VS Code
 */
export async function updateWalkthroughContext(
  context: vscode.ExtensionContext
): Promise<WalkthroughState> {
  // Check working directory
  const workingDirectory = context.globalState.get<string>('operator.workingDirectory');
  const workingDirectorySet = workingDirectory
    ? await validateWorkingDirectory(workingDirectory)
    : false;

  // Check kanban connection
  const kanbanWorkspaces = await getKanbanWorkspaces();
  const kanbanConnected = kanbanWorkspaces.length > 0;

  // Check LLM tools
  const installedLlmTools = await detectInstalledLlmTools();
  const llmToolInstalled = installedLlmTools.length > 0;

  // Update context keys
  await vscode.commands.executeCommand(
    'setContext',
    'operator.workingDirectorySet',
    workingDirectorySet
  );
  await vscode.commands.executeCommand(
    'setContext',
    'operator.kanbanConnected',
    kanbanConnected
  );
  await vscode.commands.executeCommand(
    'setContext',
    'operator.llmToolInstalled',
    llmToolInstalled
  );

  const state: WalkthroughState = {
    workingDirectorySet,
    kanbanConnected,
    llmToolInstalled,
    workingDirectory: workingDirectorySet ? workingDirectory : undefined,
    kanbanWorkspaces,
    installedLlmTools,
  };

  return state;
}

/**
 * Command: Select working directory
 */
export async function selectWorkingDirectory(
  context: vscode.ExtensionContext,
  operatorPath?: string
): Promise<void> {
  const folders = await vscode.window.showOpenDialog({
    canSelectFiles: false,
    canSelectFolders: true,
    canSelectMany: false,
    openLabel: 'Select Working Directory',
    title: 'Select parent directory for your repositories',
  });

  if (!folders || folders.length === 0) {
    return;
  }

  const selectedPath = folders[0].fsPath;

  // Validate directory
  const isValid = await validateWorkingDirectory(selectedPath);
  if (!isValid) {
    vscode.window.showErrorMessage('Selected path is not a valid directory');
    return;
  }

  // Initialize .tickets structure
  const initialized = await initializeTicketsDirectory(selectedPath, operatorPath);
  if (!initialized) {
    vscode.window.showErrorMessage('Failed to initialize tickets directory structure');
    return;
  }

  // Store in global state
  await context.globalState.update('operator.workingDirectory', selectedPath);

  // Update context
  await updateWalkthroughContext(context);

  vscode.window.showInformationMessage(
    `Working directory set to: ${selectedPath}`
  );
}

/**
 * Command: Check kanban connection
 */
export async function checkKanbanConnection(
  context: vscode.ExtensionContext
): Promise<void> {
  const workspaces = await getKanbanWorkspaces();

  if (workspaces.length === 0) {
    const choice = await vscode.window.showWarningMessage(
      'No kanban provider configured. Set up Jira or Linear environment variables.',
      'Configure Jira',
      'Configure Linear'
    );

    if (choice === 'Configure Jira') {
      await vscode.commands.executeCommand('operator.configureJira');
    } else if (choice === 'Configure Linear') {
      await vscode.commands.executeCommand('operator.configureLinear');
    }
  } else if (workspaces.length === 1) {
    const ws = workspaces[0];
    vscode.window.showInformationMessage(
      `Connected to ${ws.provider}: ${ws.name} (${ws.url})`
    );
  } else {
    const details = workspaces.map((ws) => `${ws.provider}: ${ws.name}`).join(', ');
    vscode.window.showInformationMessage(
      `Connected to ${workspaces.length} workspaces: ${details}`
    );
  }

  await updateWalkthroughContext(context);
}

/**
 * Command: Configure Jira
 */
export async function configureJira(): Promise<void> {
  const panel = vscode.window.createWebviewPanel(
    'operatorJiraSetup',
    'Configure Jira for Operator',
    vscode.ViewColumn.One,
    { enableScripts: false }
  );

  panel.webview.html = `
    <!DOCTYPE html>
    <html>
    <head>
      <style>
        body { font-family: var(--vscode-font-family); padding: 20px; }
        h1 { color: var(--vscode-editor-foreground); }
        code { background: var(--vscode-textCodeBlock-background); padding: 2px 6px; border-radius: 3px; }
        pre { background: var(--vscode-textCodeBlock-background); padding: 12px; border-radius: 6px; overflow-x: auto; }
        .step { margin: 16px 0; }
        a { color: var(--vscode-textLink-foreground); }
      </style>
    </head>
    <body>
      <h1>Configure Jira for Operator</h1>

      <div class="step">
        <h3>Step 1: Create an API Token</h3>
        <p>Go to <a href="https://id.atlassian.com/manage-profile/security/api-tokens">Atlassian API Tokens</a> and create a new token.</p>
      </div>

      <div class="step">
        <h3>Step 2: Set Environment Variables</h3>
        <p>Add the following to your shell profile (<code>~/.zshrc</code> or <code>~/.bashrc</code>):</p>
        <pre>
export OPERATOR_JIRA_API_KEY="your-api-token"
export OPERATOR_JIRA_EMAIL="your-email@example.com"
export OPERATOR_JIRA_URL="https://your-domain.atlassian.net"
        </pre>
      </div>

      <div class="step">
        <h3>Step 3: Restart VS Code</h3>
        <p>After setting the environment variables, restart VS Code for the changes to take effect.</p>
      </div>
    </body>
    </html>
  `;
}

/**
 * Command: Configure Linear
 */
export async function configureLinear(): Promise<void> {
  const panel = vscode.window.createWebviewPanel(
    'operatorLinearSetup',
    'Configure Linear for Operator',
    vscode.ViewColumn.One,
    { enableScripts: false }
  );

  panel.webview.html = `
    <!DOCTYPE html>
    <html>
    <head>
      <style>
        body { font-family: var(--vscode-font-family); padding: 20px; }
        h1 { color: var(--vscode-editor-foreground); }
        code { background: var(--vscode-textCodeBlock-background); padding: 2px 6px; border-radius: 3px; }
        pre { background: var(--vscode-textCodeBlock-background); padding: 12px; border-radius: 6px; overflow-x: auto; }
        .step { margin: 16px 0; }
        a { color: var(--vscode-textLink-foreground); }
      </style>
    </head>
    <body>
      <h1>Configure Linear for Operator</h1>

      <div class="step">
        <h3>Step 1: Create an API Key</h3>
        <p>Go to <a href="https://linear.app/settings/api">Linear API Settings</a> and create a new personal API key.</p>
      </div>

      <div class="step">
        <h3>Step 2: Set Environment Variable</h3>
        <p>Add the following to your shell profile (<code>~/.zshrc</code> or <code>~/.bashrc</code>):</p>
        <pre>
export OPERATOR_LINEAR_API_KEY="lin_api_xxxxxxxxxxxxx"
        </pre>
      </div>

      <div class="step">
        <h3>Step 3: Restart VS Code</h3>
        <p>After setting the environment variable, restart VS Code for the changes to take effect.</p>
      </div>
    </body>
    </html>
  `;
}

/**
 * Command: Detect LLM tools
 */
export async function detectLlmTools(
  context: vscode.ExtensionContext
): Promise<void> {
  const tools = await detectInstalledLlmTools();

  if (tools.length === 0) {
    const choice = await vscode.window.showWarningMessage(
      'No LLM tools detected. Install Claude Code, Codex, or Gemini CLI.',
      'Install Claude Code',
      'Install Codex',
      'Install Gemini CLI'
    );

    if (choice === 'Install Claude Code') {
      vscode.env.openExternal(vscode.Uri.parse('https://docs.anthropic.com/en/docs/claude-code'));
    } else if (choice === 'Install Codex') {
      vscode.env.openExternal(vscode.Uri.parse('https://github.com/openai/codex'));
    } else if (choice === 'Install Gemini CLI') {
      vscode.env.openExternal(vscode.Uri.parse('https://github.com/google/generative-ai-docs'));
    }
  } else {
    const toolList = tools.join(', ');
    vscode.window.showInformationMessage(
      `Detected LLM tools: ${toolList}`
    );
  }

  await updateWalkthroughContext(context);
}

/**
 * Command: Open the walkthrough
 */
export async function openWalkthrough(): Promise<void> {
  await vscode.commands.executeCommand(
    'workbench.action.openWalkthrough',
    'untra.operator-terminals#operator-setup',
    false
  );
}
