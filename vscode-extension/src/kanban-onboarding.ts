/**
 * Interactive Kanban Onboarding for Operator VS Code Extension
 *
 * Provides multi-step QuickPick/InputBox flows for configuring
 * Jira Cloud and Linear kanban integrations. All credential validation,
 * project fetching, TOML config writing, and env var setting is
 * delegated to the Operator REST API — this file is UI-only.
 */

import * as vscode from 'vscode';
import * as path from 'path';
import { updateWalkthroughContext } from './walkthrough';
import { resolveWorkingDirectory } from './config-paths';
import {
  OperatorApiClient,
  discoverApiUrl,
  type KanbanProjectInfo,
} from './api-client';

/**
 * Build an API client pointed at the local Operator server, honoring
 * the session file if present.
 */
async function buildClient(): Promise<OperatorApiClient> {
  const workDir = resolveWorkingDirectory();
  const ticketsDir = workDir ? path.join(workDir, '.tickets') : undefined;
  const apiUrl = await discoverApiUrl(ticketsDir);
  return new OperatorApiClient(apiUrl);
}

/**
 * After onboarding, sync kanban issue types from the provider and nudge
 * the user to configure mappings. Non-fatal -- degrades gracefully if
 * the Operator API is not running.
 */
async function syncAndNudgeIssueTypes(
  provider: 'jira' | 'linear' | 'github',
  projectKey: string,
  displayName: string
): Promise<void> {
  try {
    const client = await buildClient();

    const result = await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: `Syncing ${displayName} issue types...`,
        cancellable: false,
      },
      () => client.syncKanbanIssueTypes(provider, projectKey)
    );

    if (result.synced > 0) {
      const action = await vscode.window.showInformationMessage(
        `Synced ${result.synced} issue type${result.synced === 1 ? '' : 's'} from ${displayName}. Map them to Operator types for better ticket routing.`,
        'Configure Mappings'
      );
      if (action === 'Configure Mappings') {
        await vscode.commands.executeCommand('operator.openSettings');
      }
    }
  } catch {
    // Non-fatal: Operator API may not be running during initial onboarding
    void vscode.window.showWarningMessage(
      'Could not sync issue types automatically. You can sync them later from Settings.'
    );
  }
}

// ─── Input Helpers ─────────────────────────────────────────────────────

/**
 * Show an InputBox with Back button support and step indicators.
 * Returns the entered string, 'back' if Back was pressed, or undefined if cancelled.
 */
export function showInputBoxWithBack(options: {
  title: string;
  prompt: string;
  placeholder?: string;
  step: number;
  totalSteps: number;
  value?: string;
  password?: boolean;
  validate?: (value: string) => string | undefined;
  buttons?: vscode.QuickInputButton[];
}): Promise<string | undefined> {
  return new Promise((resolve) => {
    const input = vscode.window.createInputBox();
    input.title = options.title;
    input.prompt = options.prompt;
    input.placeholder = options.placeholder;
    input.step = options.step;
    input.totalSteps = options.totalSteps;
    input.value = options.value ?? '';
    input.password = options.password ?? false;
    input.ignoreFocusOut = true;

    const buttons: vscode.QuickInputButton[] = [];
    if (options.step > 1) {
      buttons.push(vscode.QuickInputButtons.Back);
    }
    if (options.buttons) {
      buttons.push(...options.buttons);
    }
    input.buttons = buttons;

    let resolved = false;

    input.onDidChangeValue((value) => {
      if (options.validate) {
        const error = options.validate(value);
        input.validationMessage = error ?? '';
      }
    });

    input.onDidAccept(() => {
      const value = input.value.trim();
      if (options.validate) {
        const error = options.validate(value);
        if (error) {
          input.validationMessage = error;
          return;
        }
      }
      resolved = true;
      input.dispose();
      resolve(value);
    });

    input.onDidTriggerButton((button) => {
      if (button === vscode.QuickInputButtons.Back) {
        resolved = true;
        input.dispose();
        resolve('back');
      }
    });

    input.onDidHide(() => {
      if (!resolved) {
        resolved = true;
        resolve(undefined);
      }
    });

    input.show();
  });
}

/**
 * Show info message with copy-to-clipboard action for shell profile env vars.
 *
 * `exportBlock` is the multi-line `export FOO="<placeholder>"` string
 * returned by the server's `setKanbanSessionEnv` endpoint.
 */
export async function showEnvVarInstructions(exportBlock: string): Promise<void> {
  const action = await vscode.window.showInformationMessage(
    'Add these to your shell profile (~/.zshrc or ~/.bashrc) for persistence across restarts:',
    'Copy to Clipboard'
  );

  if (action === 'Copy to Clipboard') {
    await vscode.env.clipboard.writeText(exportBlock);
    void vscode.window.showInformationMessage('Environment variable exports copied to clipboard.');
  }
}

// ─── Interactive Onboarding Flows ──────────────────────────────────────

/**
 * Collect Jira credentials via a 3-step InputBox wizard.
 * Returns null if the user cancelled.
 */
async function collectJiraCreds(
  title: string,
  initial: { domain: string; email: string; apiToken: string }
): Promise<{ domain: string; email: string; apiToken: string } | null> {
  let { domain, email, apiToken } = initial;
  let step = 1;

  while (step >= 1 && step <= 3) {
    if (step === 1) {
      const result = await showInputBoxWithBack({
        title,
        prompt: 'Enter your Jira Cloud domain ending in .atlassian.net',
        placeholder: 'your-org.atlassian.net',
        step: 1,
        totalSteps: 3,
        value: domain,
        validate: (v) => {
          if (!v) {
            return 'Domain is required';
          }
          if (!v.endsWith('.atlassian.net')) {
            return 'Must be a Jira Cloud domain (ending in .atlassian.net)';
          }
          return undefined;
        },
      });

      if (result === undefined) { return null; }
      if (result === 'back') { step--; continue; }
      domain = result;
      step = 2;
    } else if (step === 2) {
      const result = await showInputBoxWithBack({
        title,
        prompt: `Enter Jira email address authorized for ${domain}`,
        placeholder: 'you@example.com',
        step: 2,
        totalSteps: 3,
        value: email,
        validate: (v) => {
          if (!v) {
            return 'Email is required';
          }
          if (!v.includes('@') || !v.includes('.')) {
            return 'Enter a valid email address';
          }
          return undefined;
        },
      });

      if (result === undefined) { return null; }
      if (result === 'back') { step--; continue; }
      email = result;
      step = 3;
    } else if (step === 3) {
      const openTokenPage: vscode.QuickInputButton = {
        iconPath: new vscode.ThemeIcon('link-external'),
        tooltip: 'Open Atlassian API Tokens page',
      };

      const input = vscode.window.createInputBox();
      input.title = title;
      input.prompt = 'Enter your [Jira API token](https://id.atlassian.com/manage-profile/security/api-tokens)';
      input.placeholder = 'Paste your API token here';
      input.step = 3;
      input.totalSteps = 3;
      input.password = true;
      input.ignoreFocusOut = true;
      input.buttons = [vscode.QuickInputButtons.Back, openTokenPage];

      const result = await new Promise<string | undefined>((resolve) => {
        let resolved = false;

        input.onDidAccept(() => {
          const val = input.value.trim();
          if (!val) {
            input.validationMessage = 'API token is required';
            return;
          }
          resolved = true;
          input.dispose();
          resolve(val);
        });

        input.onDidTriggerButton((button) => {
          if (button === vscode.QuickInputButtons.Back) {
            resolved = true;
            input.dispose();
            resolve('back');
          } else if (button === openTokenPage) {
            void vscode.env.openExternal(
              vscode.Uri.parse('https://id.atlassian.com/manage-profile/security/api-tokens')
            );
          }
        });

        input.onDidHide(() => {
          if (!resolved) {
            resolved = true;
            resolve(undefined);
          }
        });

        input.show();
      });

      if (result === undefined) { return null; }
      if (result === 'back') { step--; continue; }
      apiToken = result;
      step = 4; // done
    }
  }

  return { domain, email, apiToken };
}

/**
 * Jira Cloud onboarding: collect creds -> validate via API -> set session env ->
 * list projects via API -> pick one -> write config via API -> sync issuetypes.
 */
export async function onboardJira(
  context: vscode.ExtensionContext
): Promise<void> {
  const title = 'Configure Jira Cloud';
  const client = await buildClient();

  const creds = await collectJiraCreds(title, { domain: '', email: '', apiToken: '' });
  if (!creds) { return; }

  // Validate credentials via the Operator API
  let validation;
  try {
    validation = await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: 'Validating Jira credentials...',
        cancellable: false,
      },
      () => client.validateKanbanCredentials({
        provider: 'jira',
        jira: {
          domain: creds.domain,
          email: creds.email,
          api_token: creds.apiToken,
        },
        linear: null,
        github: null,
      })
    );
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Could not reach Operator API: ${msg}`);
    return;
  }

  if (!validation.valid || !validation.jira) {
    const retry = await vscode.window.showErrorMessage(
      `Jira validation failed: ${validation.error ?? 'unknown error'}`,
      'Retry',
      'Cancel'
    );
    if (retry === 'Retry') {
      return onboardJira(context);
    }
    return;
  }

  void vscode.window.showInformationMessage(
    `Authenticated as ${validation.jira.display_name} (${validation.jira.account_id})`
  );

  // Set session env so subsequent API calls can use the token server-side
  const apiKeyEnv = 'OPERATOR_JIRA_API_KEY';
  let envInfo;
  try {
    envInfo = await client.setKanbanSessionEnv({
      provider: 'jira',
      jira: {
        domain: creds.domain,
        email: creds.email,
        api_token: creds.apiToken,
        api_key_env: apiKeyEnv,
      },
      linear: null,
      github: null,
    });
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to set session env: ${msg}`);
    return;
  }

  // Fetch projects via the API
  let projects: KanbanProjectInfo[];
  try {
    projects = await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: 'Fetching Jira projects...',
        cancellable: false,
      },
      () => client.listKanbanProjects({
        provider: 'jira',
        jira: {
          domain: creds.domain,
          email: creds.email,
          api_token: creds.apiToken,
        },
        linear: null,
        github: null,
      })
    );
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to list Jira projects: ${msg}`);
    return;
  }

  if (projects.length === 0) {
    void vscode.window.showWarningMessage(
      'No projects found. Check your permissions. Config was not written.'
    );
    return;
  }

  const projectItems = projects.map((p) => ({
    label: p.key,
    description: p.name,
  }));

  const selectedProject = await vscode.window.showQuickPick(projectItems, {
    title: 'Select Jira Project',
    placeHolder: 'Choose a project to sync tickets from',
    ignoreFocusOut: true,
  });

  if (!selectedProject) {
    return;
  }

  // Write config via the API
  try {
    await client.writeKanbanConfig({
      provider: 'jira',
      jira: {
        domain: creds.domain,
        email: creds.email,
        api_key_env: apiKeyEnv,
        project_key: selectedProject.label,
        sync_user_id: validation.jira.account_id,
      },
      linear: null,
      github: null,
    });
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to write config: ${msg}`);
    return;
  }

  void vscode.window.showInformationMessage(
    `Jira configured! Run Operator to activate.`
  );

  await showEnvVarInstructions(envInfo.shell_export_block);

  // Update walkthrough context
  await updateWalkthroughContext(context);

  // Auto-sync issue types and nudge user to map them
  await syncAndNudgeIssueTypes('jira', selectedProject.label, `Jira ${selectedProject.label}`);
}

/**
 * Prompt for a Linear API key via InputBox (with external-link button).
 * Returns null if cancelled.
 */
async function collectLinearApiKey(title: string): Promise<string | null> {
  const openLinearSettings: vscode.QuickInputButton = {
    iconPath: new vscode.ThemeIcon('link-external'),
    tooltip: 'Open Linear API Settings',
  };

  const input = vscode.window.createInputBox();
  input.title = title;
  input.prompt = 'Enter your Linear API key';
  input.placeholder = 'lin_api_xxxxxxxxxxxxx';
  input.step = 1;
  input.totalSteps = 2;
  input.password = true;
  input.ignoreFocusOut = true;
  input.buttons = [openLinearSettings];

  const apiKey = await new Promise<string | undefined>((resolve) => {
    let resolved = false;

    input.onDidChangeValue((value) => {
      if (value && !value.startsWith('lin_api_')) {
        input.validationMessage = 'Linear API keys start with "lin_api_"';
      } else {
        input.validationMessage = '';
      }
    });

    input.onDidAccept(() => {
      const val = input.value.trim();
      if (!val) {
        input.validationMessage = 'API key is required';
        return;
      }
      if (!val.startsWith('lin_api_')) {
        input.validationMessage = 'Linear API keys start with "lin_api_"';
        return;
      }
      resolved = true;
      input.dispose();
      resolve(val);
    });

    input.onDidTriggerButton((button) => {
      if (button === openLinearSettings) {
        void vscode.env.openExternal(
          vscode.Uri.parse('https://linear.app/settings/api')
        );
      }
    });

    input.onDidHide(() => {
      if (!resolved) {
        resolved = true;
        resolve(undefined);
      }
    });

    input.show();
  });

  return apiKey ?? null;
}

/**
 * Linear onboarding: prompt for API key -> validate via API -> set session env ->
 * pick team -> write config via API -> sync issuetypes.
 */
export async function onboardLinear(
  context: vscode.ExtensionContext
): Promise<void> {
  const title = 'Configure Linear';
  const client = await buildClient();

  const apiKey = await collectLinearApiKey(title);
  if (!apiKey) { return; }

  // Validate credentials via the Operator API
  let validation;
  try {
    validation = await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: 'Validating Linear credentials...',
        cancellable: false,
      },
      () => client.validateKanbanCredentials({
        provider: 'linear',
        jira: null,
        linear: { api_key: apiKey },
        github: null,
      })
    );
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Could not reach Operator API: ${msg}`);
    return;
  }

  if (!validation.valid || !validation.linear) {
    const retry = await vscode.window.showErrorMessage(
      `Linear validation failed: ${validation.error ?? 'unknown error'}`,
      'Retry',
      'Cancel'
    );
    if (retry === 'Retry') {
      return onboardLinear(context);
    }
    return;
  }

  void vscode.window.showInformationMessage(
    `Authenticated as ${validation.linear.user_name} in ${validation.linear.org_name}`
  );

  // Set session env
  const apiKeyEnv = 'OPERATOR_LINEAR_API_KEY';
  let envInfo;
  try {
    envInfo = await client.setKanbanSessionEnv({
      provider: 'linear',
      jira: null,
      linear: { api_key: apiKey, api_key_env: apiKeyEnv },
      github: null,
    });
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to set session env: ${msg}`);
    return;
  }

  // Select team from the teams returned by validation
  if (validation.linear.teams.length === 0) {
    void vscode.window.showWarningMessage(
      'No teams found. Check your permissions. Config was not written.'
    );
    return;
  }

  const teamItems = validation.linear.teams.map((t) => ({
    label: t.name,
    description: t.key,
    detail: t.id,
  }));

  const selectedTeam = await vscode.window.showQuickPick(teamItems, {
    title: 'Select Linear Team',
    placeHolder: 'Choose a team to sync tickets from',
    ignoreFocusOut: true,
  });

  if (!selectedTeam) {
    return;
  }

  // Write config via the API. For Linear, we use the org slug / a default
  // workspace key. Since validation doesn't return a workspace slug directly,
  // use the org name (sanitized) as the workspace key.
  const workspaceKey = selectedTeam.detail ?? 'default';
  try {
    await client.writeKanbanConfig({
      provider: 'linear',
      jira: null,
      linear: {
        workspace_key: workspaceKey,
        api_key_env: apiKeyEnv,
        project_key: 'default',
        sync_user_id: validation.linear.user_id,
      },
      github: null,
    });
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to write config: ${msg}`);
    return;
  }

  void vscode.window.showInformationMessage(`Linear configured!`);

  await showEnvVarInstructions(envInfo.shell_export_block);

  // Update walkthrough context
  await updateWalkthroughContext(context);

  // Auto-sync issue types and nudge user to map them
  await syncAndNudgeIssueTypes('linear', 'default', `Linear ${selectedTeam.description ?? selectedTeam.label}`);
}

/**
 * Prompt for a GitHub Projects token via InputBox.
 *
 * IMPORTANT: This is the *projects* token, not the *repo* token. It must
 * have the `project` (or `read:project`) scope. A repo-only token (the kind
 * typically set as `GITHUB_TOKEN` for PR workflows) will be rejected by the
 * server's scope verification with a friendly error pointing to the docs.
 */
async function collectGithubToken(title: string): Promise<string | null> {
  const openGithubSettings: vscode.QuickInputButton = {
    iconPath: new vscode.ThemeIcon('link-external'),
    tooltip: 'Open GitHub Token Settings',
  };

  const input = vscode.window.createInputBox();
  input.title = title;
  input.prompt =
    'Enter a GitHub PAT with the `project` (or `read:project`) scope — NOT a repo-only token\nhttps://github.com/settings/personal-access-tokens';
  input.placeholder = 'ghp_xxxxxxxxxxxxxxxx or github_pat_xxxxxxxx';
  input.step = 1;
  input.totalSteps = 2;
  input.password = true;
  input.ignoreFocusOut = true;
  input.buttons = [openGithubSettings];

  const isRecognizedPrefix = (val: string): boolean =>
    val.startsWith('ghp_') || val.startsWith('github_pat_') || val.startsWith('gho_');

  const token = await new Promise<string | undefined>((resolve) => {
    let resolved = false;

    input.onDidChangeValue((value) => {
      if (value && !isRecognizedPrefix(value)) {
        input.validationMessage =
          'GitHub tokens start with "ghp_", "github_pat_", or "gho_"';
      } else {
        input.validationMessage = '';
      }
    });

    input.onDidAccept(() => {
      const val = input.value.trim();
      if (!val) {
        input.validationMessage = 'Token is required';
        return;
      }
      if (!isRecognizedPrefix(val)) {
        input.validationMessage =
          'GitHub tokens start with "ghp_", "github_pat_", or "gho_"';
        return;
      }
      resolved = true;
      input.dispose();
      resolve(val);
    });

    input.onDidTriggerButton((button) => {
      if (button === openGithubSettings) {
        void vscode.env.openExternal(
          vscode.Uri.parse('https://github.com/settings/tokens')
        );
      }
    });

    input.onDidHide(() => {
      if (!resolved) {
        resolved = true;
        resolve(undefined);
      }
    });

    input.show();
  });

  return token ?? null;
}

/**
 * GitHub Projects v2 onboarding: prompt for token -> validate (with scope
 * verification) -> set session env -> pick project -> write config -> sync
 * issue types.
 *
 * The validate step performs the Token Disambiguation scope check on the
 * server side; if the token is repo-only the user gets a friendly error
 * pointing them at the docs.
 */
export async function onboardGithub(
  context: vscode.ExtensionContext
): Promise<void> {
  const title = 'Configure GitHub Projects';
  const client = await buildClient();

  const token = await collectGithubToken(title);
  if (!token) { return; }

  // Validate credentials via the Operator API (includes scope verification).
  let validation;
  try {
    validation = await vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Notification,
        title: 'Validating GitHub Projects credentials...',
        cancellable: false,
      },
      () => client.validateKanbanCredentials({
        provider: 'github',
        jira: null,
        linear: null,
        github: { token },
      })
    );
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Could not reach Operator API: ${msg}`);
    return;
  }

  if (!validation.valid || !validation.github) {
    const retry = await vscode.window.showErrorMessage(
      `GitHub validation failed: ${validation.error ?? 'unknown error'}`,
      'Retry',
      'Cancel'
    );
    if (retry === 'Retry') {
      return onboardGithub(context);
    }
    return;
  }

  void vscode.window.showInformationMessage(
    `Authenticated as ${validation.github.user_login} (connected via ${validation.github.resolved_env_var})`
  );

  // Set session env so subsequent API calls can use the token server-side.
  const apiKeyEnv = 'OPERATOR_GITHUB_TOKEN';
  let envInfo;
  try {
    envInfo = await client.setKanbanSessionEnv({
      provider: 'github',
      jira: null,
      linear: null,
      github: { token, api_key_env: apiKeyEnv },
    });
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to set session env: ${msg}`);
    return;
  }

  // Project picker: use projects from validation (no extra round-trip).
  if (validation.github.projects.length === 0) {
    void vscode.window.showWarningMessage(
      'No GitHub Projects v2 found for this token. Confirm the token has the `project` scope and that you have access to at least one project. Config was not written.'
    );
    return;
  }

  const projectItems = validation.github.projects.map((p) => ({
    label: `${p.owner_login}/#${p.number} ${p.title}`,
    description: p.owner_kind,
    detail: p.node_id,
    project: p,
  }));

  const selectedProject = await vscode.window.showQuickPick(projectItems, {
    title: 'Select GitHub Project',
    placeHolder: 'Choose a project to sync tickets from',
    ignoreFocusOut: true,
  });

  if (!selectedProject) {
    return;
  }

  // Write config — owner is the workspace key, project node id is the project key.
  try {
    await client.writeKanbanConfig({
      provider: 'github',
      jira: null,
      linear: null,
      github: {
        owner: selectedProject.project.owner_login,
        api_key_env: apiKeyEnv,
        project_key: selectedProject.project.node_id,
        sync_user_id: validation.github.user_id,
      },
    });
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    void vscode.window.showErrorMessage(`Failed to write config: ${msg}`);
    return;
  }

  void vscode.window.showInformationMessage(
    `GitHub Projects configured for ${selectedProject.project.owner_login}/#${selectedProject.project.number}!`
  );

  await showEnvVarInstructions(envInfo.shell_export_block);

  // Update walkthrough context
  await updateWalkthroughContext(context);

  // Auto-sync issue types and nudge user to map them.
  await syncAndNudgeIssueTypes(
    'github',
    selectedProject.project.node_id,
    `GitHub ${selectedProject.project.owner_login}/#${selectedProject.project.number}`
  );
}

/**
 * Entry-point: let user pick Jira, Linear, or GitHub Projects, then route to
 * the right flow.
 */
export async function startKanbanOnboarding(
  context: vscode.ExtensionContext
): Promise<void> {
  const choice = await vscode.window.showQuickPick(
    [
      {
        label: '$(operator-atlassian) Jira Cloud',
        description: 'Connect to Jira Cloud with API token',
        provider: 'jira' as const,
      },
      {
        label: '$(operator-linear) Linear',
        description: 'Connect to Linear with API key',
        provider: 'linear' as const,
      },
      {
        label: '$(github) GitHub Projects',
        description: 'Connect to GitHub Projects v2 with a personal access token',
        provider: 'github' as const,
      },
      {
        label: '$(close) Skip for now',
        description: 'You can configure this later',
        provider: 'skip' as const,
      },
    ],
    {
      title: 'Connect Kanban Provider',
      placeHolder: 'Which kanban provider do you use?',
      ignoreFocusOut: true,
    }
  );

  if (!choice || choice.provider === 'skip') {
    return;
  }

  switch (choice.provider) {
    case 'jira':
      await onboardJira(context);
      break;
    case 'linear':
      await onboardLinear(context);
      break;
    case 'github':
      await onboardGithub(context);
      break;
  }
}

// ─── Add Project/Team Flows ───────────────────────────────────────────

/**
 * Add a new Jira project. Since all credential state lives on the server
 * now, this flow is a simplified version of `onboardJira` — it collects
 * credentials again, validates, picks a project, and writes config.
 */
export async function addJiraProject(
  context: vscode.ExtensionContext,
  domain?: string
): Promise<void> {
  // Delegate to the full onboarding flow. The domain hint isn't used —
  // the user re-enters credentials. Future enhancement: load existing
  // workspace config via a GET /api/v1/kanban/config endpoint to skip
  // the domain/email steps.
  void domain;
  await onboardJira(context);
}

/**
 * Add a new Linear team. Same simplification as `addJiraProject`.
 */
export async function addLinearTeam(
  context: vscode.ExtensionContext,
  workspaceKey?: string
): Promise<void> {
  void workspaceKey;
  await onboardLinear(context);
}

/**
 * Add a new GitHub Project. Same simplification as `addJiraProject`.
 */
export async function addGithubProject(
  context: vscode.ExtensionContext,
  owner?: string
): Promise<void> {
  void owner;
  await onboardGithub(context);
}
