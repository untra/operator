/**
 * Git Provider Onboarding for Operator VS Code extension
 *
 * Guides users through connecting GitHub or GitLab as their git provider.
 * Auto-detects CLI tools (gh, glab) for silent token grab, falls back to
 * manual PAT entry. Smart-merges config into config.toml preserving
 * existing settings like branch_format and use_worktrees.
 */

import * as vscode from 'vscode';
import * as fs from 'fs/promises';
import { exec } from 'child_process';
import { promisify } from 'util';
import { getConfigDir, getResolvedConfigPath, resolveWorkingDirectory } from './config-paths';
import { showEnvVarInstructions } from './kanban-onboarding';

const execAsync = promisify(exec);

/**
 * Detect a CLI tool in PATH, return its path or null
 */
async function findCliTool(tool: string): Promise<string | null> {
  const whichCmd = process.platform === 'win32' ? 'where' : 'which';
  try {
    const { stdout } = await execAsync(`${whichCmd} ${tool}`);
    return stdout.trim().split('\n')[0] ?? null;
  } catch {
    return null;
  }
}

/**
 * Try to get a token from a CLI tool (gh auth token, glab auth token)
 */
async function getCliToken(command: string): Promise<string | null> {
  try {
    const { stdout } = await execAsync(command);
    const token = stdout.trim();
    return token || null;
  } catch {
    return null;
  }
}

/**
 * Smart-merge git config into config.toml.
 *
 * Reads existing config, updates [git] and provider sub-sections,
 * preserves branch_format and use_worktrees if already set.
 */
async function writeGitConfig(
  provider: string,
  providerSection: Record<string, unknown>
): Promise<boolean> {
  try {
    const configDir = getConfigDir(resolveWorkingDirectory());
    await fs.mkdir(configDir, { recursive: true });
  } catch {
    // directory may already exist
  }

  const configPath = getResolvedConfigPath();
  let existing = '';
  try {
    existing = await fs.readFile(configPath, 'utf-8');
  } catch {
    // file doesn't exist yet
  }

  try {
    const { parse, stringify } = await import('smol-toml');
    const config = existing.trim() ? parse(existing) as Record<string, unknown> : {};

    // Preserve existing git settings
    const existingGit = (config.git ?? {}) as Record<string, unknown>;
    const mergedGit: Record<string, unknown> = {
      ...existingGit,
      provider,
    };

    // Merge provider sub-section
    const existingProvider = (existingGit[provider] ?? {}) as Record<string, unknown>;
    mergedGit[provider] = { ...existingProvider, ...providerSection };

    config.git = mergedGit;
    const output = stringify(config);
    await fs.writeFile(configPath, output, 'utf-8');
    return true;
  } catch (err) {
    void vscode.window.showErrorMessage(
      `Failed to write git config: ${err instanceof Error ? err.message : String(err)}`
    );
    return false;
  }
}

/**
 * GitHub onboarding flow
 *
 * 1. Detect gh CLI → grab token silently
 * 2. Fall back to manual PAT input
 * 3. Validate via GitHub API
 * 4. Write config
 */
export async function onboardGitHub(): Promise<void> {
  let token: string | undefined;

  // Try gh CLI first
  const ghPath = await findCliTool('gh');
  if (ghPath) {
    token = await getCliToken('gh auth token') ?? undefined;
    if (token) {
      void vscode.window.showInformationMessage('Found GitHub token from gh CLI.');
    }
  }

  // Fall back to manual input
  if (!token) {
    const message = ghPath
      ? 'gh CLI found but not authenticated. Enter a GitHub Personal Access Token:'
      : 'Enter a GitHub Personal Access Token (or install gh CLI for auto-detection):';

    token = await vscode.window.showInputBox({
      title: 'GitHub Authentication',
      prompt: message,
      password: true,
      ignoreFocusOut: true,
      placeHolder: 'ghp_...',
    }) ?? undefined;

    if (!token) { return; }
  }

  // Validate token
  const user = await vscode.window.withProgress(
    { location: vscode.ProgressLocation.Notification, title: 'Validating GitHub token...' },
    async () => {
      try {
        const response = await fetch('https://api.github.com/user', {
          headers: { Authorization: `Bearer ${token}` },
        });
        if (response.ok) {
          return await response.json() as { login: string };
        }
      } catch {
        // validation failed
      }
      return null;
    }
  );

  if (!user) {
    void vscode.window.showErrorMessage('GitHub token validation failed. Check your token and try again.');
    return;
  }

  // Write config
  const written = await writeGitConfig('github', {
    enabled: true,
    token_env: 'GITHUB_TOKEN',
  });
  if (!written) { return; }

  // Set env var for current session
  process.env['GITHUB_TOKEN'] = token;

  void vscode.window.showInformationMessage(
    `GitHub connected as ${user.login}! Config written to ${getResolvedConfigPath()}`
  );

  await showEnvVarInstructions([
    `export GITHUB_TOKEN="<your-token>"`,
  ]);
}

/**
 * GitLab onboarding flow
 *
 * 1. Ask for host (default gitlab.com)
 * 2. Detect glab CLI → grab token silently
 * 3. Fall back to manual PAT input
 * 4. Validate via GitLab API
 * 5. Write config
 */
export async function onboardGitLab(): Promise<void> {
  // Ask for host
  const host = await vscode.window.showInputBox({
    title: 'GitLab Host',
    prompt: 'Enter your GitLab instance URL',
    value: 'gitlab.com',
    ignoreFocusOut: true,
    placeHolder: 'gitlab.com or your self-hosted domain',
  }) ?? undefined;

  if (!host) { return; }

  let token: string | undefined;

  // Try glab CLI first
  const glabPath = await findCliTool('glab');
  if (glabPath) {
    token = await getCliToken('glab auth token') ?? undefined;
    if (token) {
      void vscode.window.showInformationMessage('Found GitLab token from glab CLI.');
    }
  }

  // Fall back to manual input
  if (!token) {
    const message = glabPath
      ? 'glab CLI found but not authenticated. Enter a GitLab Personal Access Token:'
      : 'Enter a GitLab Personal Access Token (or install glab CLI for auto-detection):';

    token = await vscode.window.showInputBox({
      title: 'GitLab Authentication',
      prompt: message,
      password: true,
      ignoreFocusOut: true,
      placeHolder: 'glpat-...',
    }) ?? undefined;

    if (!token) { return; }
  }

  // Validate token
  const apiHost = host.includes('://') ? host : `https://${host}`;
  const user = await vscode.window.withProgress(
    { location: vscode.ProgressLocation.Notification, title: 'Validating GitLab token...' },
    async () => {
      try {
        const response = await fetch(`${apiHost}/api/v4/user`, {
          headers: { 'Private-Token': token },
        });
        if (response.ok) {
          return await response.json() as { username: string };
        }
      } catch {
        // validation failed
      }
      return null;
    }
  );

  if (!user) {
    void vscode.window.showErrorMessage('GitLab token validation failed. Check your token and host, then try again.');
    return;
  }

  // Write config
  const written = await writeGitConfig('gitlab', {
    enabled: true,
    token_env: 'GITLAB_TOKEN',
    host,
  });
  if (!written) { return; }

  // Set env var for current session
  process.env['GITLAB_TOKEN'] = token;

  void vscode.window.showInformationMessage(
    `GitLab connected as ${user.username}! Config written to ${getResolvedConfigPath()}`
  );

  await showEnvVarInstructions([
    `export GITLAB_TOKEN="<your-token>"`,
  ]);
}

/**
 * Entry point: let user pick GitHub or GitLab, then route to the right flow
 */
export async function startGitOnboarding(): Promise<void> {
  const choice = await vscode.window.showQuickPick(
    [
      { label: 'GitHub', description: 'Connect to github.com', detail: 'github' },
      { label: 'GitLab', description: 'Connect to gitlab.com or self-hosted', detail: 'gitlab' },
      { label: 'Skip', description: 'Configure later' },
    ],
    {
      title: 'Connect Git Provider',
      placeHolder: 'Select a git hosting provider',
      ignoreFocusOut: true,
    }
  );

  if (!choice || choice.label === 'Skip') { return; }

  if (choice.detail === 'github') {
    await onboardGitHub();
  } else if (choice.detail === 'gitlab') {
    await onboardGitLab();
  }
}
