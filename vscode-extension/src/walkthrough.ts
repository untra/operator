/**
 * Walkthrough module for Operator VS Code extension
 *
 * Provides setup wizard functionality to guide users through:
 * 1. Selecting a working directory
 * 2. Connecting a kanban provider (Jira/Linear)
 * 3. Installing an LLM tool (Claude Code/Codex/Gemini CLI)
 */

import * as vscode from "vscode";
import { resolveTicketsDirectory, resolveWorkingDirectory } from "./config-paths";
import { currentConnection } from "./connection";
import { guardRemoteCommand } from "./command-policy";
import * as path from "node:path";
import * as fs from "node:fs/promises";
import { exec, execFile } from "node:child_process";
import { promisify } from "node:util";

const execAsync = promisify(exec);

/** Kanban provider types */
export type KanbanProviderType = "jira" | "linear" | "github" | "openspec";

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
  installedLlmTools: DetectedToolResult[];
}

/** Environment variable names for kanban providers */
export const KANBAN_ENV_VARS = {
  jira: {
    apiKey: ["OPERATOR_JIRA_API_KEY", "JIRA_API_TOKEN"] as const,
    domain: ["OPERATOR_JIRA_DOMAIN"] as const,
    email: ["OPERATOR_JIRA_EMAIL"] as const,
  },
  linear: {
    apiKey: ["OPERATOR_LINEAR_API_KEY", "LINEAR_API_KEY"] as const,
  },
  github: {
    // Token Disambiguation: ONLY OPERATOR_GITHUB_TOKEN is checked here.
    // We deliberately do NOT fall through to GITHUB_TOKEN - that env var
    // belongs to the git provider (PR/branch workflows) and detecting it
    // here would surface a spurious "GitHub kanban detected" prompt.
    apiKey: ["OPERATOR_GITHUB_TOKEN"] as const,
  },
} as const;

/** Linear GraphQL API URL */
const LINEAR_API_URL = "https://api.linear.app/graphql";

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

/** LLM tools to detect. Must match `shipped_llm_tools()` binaries in src/config/llm_tools.rs. */
export const LLM_TOOLS = ["claude", "codex", "gemini", "grok"] as const;

/** Minimal detected tool info (mirrors Rust DetectedTool subset) */
export interface DetectedToolResult {
  name: string;
  path: string;
  version: string;
  version_ok: boolean;
  health_ok: boolean;
}

/** Tool metadata for version detection (mirrors src/llm/tools/*.json) */
const TOOL_META: Record<string, { versionCmd: string; minVersion: string }> = {
  claude: { versionCmd: "claude --version", minVersion: "2.1.0" },
  codex: { versionCmd: "codex --version", minVersion: "0.1.0" },
  gemini: { versionCmd: "gemini --version", minVersion: "0.1.0" },
  grok: { versionCmd: "grok --version", minVersion: "0.1.0" },
};

function parseVersion(value: string): number[] {
  const match = value.match(/(\d+(?:\.\d+)*)/);
  if (!match?.[1]) {
    return [0];
  }
  return match[1].split(".").map(Number);
}

/**
 * Compare two semver-like version strings.
 * Returns true if `version` >= `minVersion`.
 */
export function compareVersions(version: string, minVersion: string): boolean {
  const a = parseVersion(version);
  const b = parseVersion(minVersion);
  const len = Math.max(a.length, b.length);
  for (let i = 0; i < len; i++) {
    const av = a[i] ?? 0;
    const bv = b[i] ?? 0;
    if (av > bv) {
      return true;
    }
    if (av < bv) {
      return false;
    }
  }
  return true; // equal
}

/**
 * Detect a single LLM tool: resolve path, version, and version_ok
 */
async function detectSingleTool(tool: string): Promise<DetectedToolResult | null> {
  const whichCmd = process.platform === "win32" ? "where" : "which";
  let toolPath: string;
  try {
    const { stdout } = await execAsync(`${whichCmd} ${tool}`);
    toolPath = stdout.trim().split("\n")[0] ?? "";
  } catch {
    return null;
  }

  const meta = TOOL_META[tool];
  let version = "unknown";
  let versionOk = false;

  if (meta) {
    try {
      const { stdout } = await execAsync(meta.versionCmd);
      version = stdout.trim();
      versionOk = compareVersions(version, meta.minVersion);
    } catch {
      // Tool exists but version command failed
    }
  }

  // The `which` gate above succeeded, so presence verified.
  return { name: tool, path: toolPath, version, version_ok: versionOk, health_ok: true };
}

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
      provider: "jira",
      name: jiraDomain,
      url: `https://${jiraDomain}`,
      configured: true,
    });
  }

  // Check Linear - API key only, name/url fetched async
  const linearApiKey = findEnvVar(KANBAN_ENV_VARS.linear.apiKey);
  if (linearApiKey) {
    workspaces.push({
      provider: "linear",
      name: "Linear", // Placeholder, updated by fetchLinearWorkspace
      url: "https://linear.app",
      configured: true,
    });
  }

  // Check GitHub Projects - token only, name resolved server-side at validation
  const githubToken = findEnvVar(KANBAN_ENV_VARS.github.apiKey);
  if (githubToken) {
    workspaces.push({
      provider: "github",
      name: "GitHub Projects",
      url: "https://github.com",
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
  apiKey: string,
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
      method: "POST",
      headers: {
        Authorization: apiKey,
        "Content-Type": "application/json",
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
  const linearIdx = workspaces.findIndex((w) => w.provider === "linear");
  if (linearIdx >= 0) {
    const apiKey = findEnvVar(KANBAN_ENV_VARS.linear.apiKey);
    if (apiKey) {
      const orgInfo = await fetchLinearWorkspace(apiKey);
      if (orgInfo) {
        workspaces[linearIdx] = {
          ...workspaces[linearIdx]!,
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
    const command = process.platform === "win32" ? "where" : "which";
    await execAsync(`${command} ${tool}`);
    return true;
  } catch {
    return false;
  }
}

/**
 * Detect all installed LLM tools with path, version, and version check
 */
export async function detectInstalledLlmTools(): Promise<DetectedToolResult[]> {
  const results = await Promise.all(LLM_TOOLS.map((tool) => detectSingleTool(tool)));
  return results.filter((r): r is DetectedToolResult => r !== null);
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
  operatorPath?: string,
  ticketsDir = path.join(workingDir, ".tickets"),
): Promise<boolean> {
  try {
    // Try using operator CLI if available
    if (operatorPath && ticketsDir === path.join(workingDir, ".tickets")) {
      try {
        await promisify(execFile)(operatorPath, ["setup", "--working-dir", workingDir]);
        // Ensure config.toml exists even if CLI didn't create it
        const configPath = path.join(ticketsDir, "operator", "config.toml");
        try {
          await fs.access(configPath);
        } catch {
          await fs.writeFile(configPath, "[agents]\n", { flag: "wx" });
        }
        return true;
      } catch {
        // Fall through to manual creation
      }
    }

    // Manual creation of directory structure
    const dirs = [
      ticketsDir,
      path.join(ticketsDir, "queue"),
      path.join(ticketsDir, "in-progress"),
      path.join(ticketsDir, "completed"),
      path.join(ticketsDir, "operator"),
    ];

    for (const dir of dirs) {
      await fs.mkdir(dir, { recursive: true });
    }

    // Create empty config.toml if it doesn't exist
    const configPath = path.join(ticketsDir, "operator", "config.toml");
    try {
      await fs.access(configPath);
    } catch {
      await fs.writeFile(configPath, "[agents]\n", { flag: "wx" });
    }

    return true;
  } catch (error) {
    console.error("Failed to initialize tickets directory:", error);
    return false;
  }
}

/**
 * Update all walkthrough context keys in VS Code
 */
export async function updateWalkthroughContext(
  context: vscode.ExtensionContext,
): Promise<WalkthroughState> {
  const { selection, signal } = currentConnection();
  const local = selection.kind === "local";
  const workingDirectory = local
    ? resolveWorkingDirectory() || context.globalState.get<string>("operator.workingDirectory")
    : undefined;
  const workingDirectorySet = workingDirectory
    ? await validateWorkingDirectory(workingDirectory)
    : false;

  // Check kanban connection
  const kanbanWorkspaces = local ? await getKanbanWorkspaces() : [];
  const kanbanConnected = kanbanWorkspaces.length > 0;

  // Check LLM tools
  const installedLlmTools = local ? await detectInstalledLlmTools() : [];
  const llmToolInstalled = installedLlmTools.length > 0;

  if (!signal.aborted) {
    await Promise.all([
      vscode.commands.executeCommand(
        "setContext",
        "operator.workingDirectorySet",
        workingDirectorySet,
      ),
      vscode.commands.executeCommand(
        "setContext",
        "operator.connectionChosen",
        local || selection.kind === "hosted",
      ),
      vscode.commands.executeCommand("setContext", "operator.connectionMode", selection.kind),
      vscode.commands.executeCommand("setContext", "operator.kanbanConnected", kanbanConnected),
      vscode.commands.executeCommand("setContext", "operator.llmToolInstalled", llmToolInstalled),
    ]);
  }

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
  operatorPath?: string,
): Promise<void> {
  if (await guardRemoteCommand("operator.selectWorkingDirectory", undefined)) {
    return;
  }

  const folders = await vscode.window.showOpenDialog({
    canSelectFiles: false,
    canSelectFolders: true,
    canSelectMany: false,
    openLabel: "Select Working Directory",
    title: "Select parent directory for your repositories",
  });

  if (!folders || folders.length === 0) {
    return;
  }

  const selectedPath = folders[0]!.fsPath;

  // Validate directory
  const isValid = await validateWorkingDirectory(selectedPath);
  if (!isValid) {
    void vscode.window.showErrorMessage("Selected path is not a valid directory");
    return;
  }

  // Initialize .tickets structure
  const initialized = await initializeTicketsDirectory(
    selectedPath,
    operatorPath,
    resolveTicketsDirectory(selectedPath),
  );
  if (!initialized) {
    void vscode.window.showErrorMessage("Failed to initialize tickets directory structure");
    return;
  }

  const config = vscode.workspace.getConfiguration("operator");
  await config.update("workingDirectory", selectedPath, vscode.ConfigurationTarget.Workspace);

  // Update context
  await updateWalkthroughContext(context);

  void vscode.window.showInformationMessage(`Working directory set to: ${selectedPath}`);
}

/**
 * Command: Check kanban connection
 */
export async function checkKanbanConnection(context: vscode.ExtensionContext): Promise<void> {
  if (await guardRemoteCommand("operator.checkKanbanConnection", undefined)) {
    return;
  }

  const workspaces = await getKanbanWorkspaces();

  if (workspaces.length === 0) {
    const choice = await vscode.window.showWarningMessage(
      "No kanban provider configured. Set up Jira or Linear environment variables.",
      "Configure Jira",
      "Configure Linear",
    );

    if (choice === "Configure Jira") {
      await vscode.commands.executeCommand("operator.configureJira");
    } else if (choice === "Configure Linear") {
      await vscode.commands.executeCommand("operator.configureLinear");
    }
  } else if (workspaces.length === 1) {
    const ws = workspaces[0]!;
    void vscode.window.showInformationMessage(
      `Connected to ${ws.provider}: ${ws.name} (${ws.url})`,
    );
  } else {
    const details = workspaces.map((ws) => `${ws.provider}: ${ws.name}`).join(", ");
    void vscode.window.showInformationMessage(
      `Connected to ${workspaces.length} workspaces: ${details}`,
    );
  }

  await updateWalkthroughContext(context);
}

// Re-export interactive onboarding flows (replaces old webview-based configureJira/configureLinear)
export {
  onboardJira as configureJira,
  onboardLinear as configureLinear,
  startKanbanOnboarding,
} from "./kanban-onboarding";

export async function detectLlmTools(context: vscode.ExtensionContext): Promise<void> {
  if (await guardRemoteCommand("operator.detectLlmTools", undefined)) {
    return;
  }
  const { signal } = currentConnection();
  const tools = await detectInstalledLlmTools();
  if (signal.aborted) {
    return;
  }

  if (tools.length === 0) {
    const choice = await vscode.window.showWarningMessage(
      "No LLM tools detected. Install Claude Code, Codex, or Gemini CLI.",
      "Install Claude Code",
      "Install Codex",
      "Install Gemini CLI",
    );

    if (choice === "Install Claude Code") {
      void vscode.env.openExternal(
        vscode.Uri.parse("https://docs.anthropic.com/en/docs/claude-code"),
      );
    } else if (choice === "Install Codex") {
      void vscode.env.openExternal(vscode.Uri.parse("https://github.com/openai/codex"));
    } else if (choice === "Install Gemini CLI") {
      void vscode.env.openExternal(
        vscode.Uri.parse("https://github.com/google/generative-ai-docs"),
      );
    }
  } else {
    const tool = await vscode.window.showQuickPick(
      tools.map(({ name }) => name),
      {
        title: "Choose the preferred LLM tool",
        placeHolder: "Set the default tool for this Operator configuration",
      },
    );
    if (tool && !signal.aborted) {
      await vscode.commands.executeCommand("operator.setDefaultLlm", tool);
    }
  }

  if (!signal.aborted) {
    await updateWalkthroughContext(context);
  }
}

/**
 * Command: Open the walkthrough
 */
export async function openWalkthrough(): Promise<void> {
  await vscode.commands.executeCommand(
    "workbench.action.openWalkthrough",
    "untra.operator-terminals#operator-setup",
    false,
  );
}
