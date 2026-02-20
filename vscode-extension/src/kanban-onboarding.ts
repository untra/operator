/**
 * Interactive Kanban Onboarding for Operator VS Code Extension
 *
 * Provides multi-step QuickPick/InputBox flows for configuring
 * Jira Cloud and Linear kanban integrations. Validates credentials
 * against live APIs, writes TOML config, and sets env vars for
 * the current session.
 */

import * as vscode from 'vscode';
import * as fs from 'fs/promises';
import { updateWalkthroughContext } from './walkthrough';
import { getConfigDir, getResolvedConfigPath, resolveWorkingDirectory } from './config-paths';

// smol-toml is ESM-only, must use dynamic import
async function importSmolToml() {
  return await import('smol-toml');
}

/** Linear GraphQL API URL */
const LINEAR_API_URL = 'https://api.linear.app/graphql';

// ─── TOML Config Utilities ─────────────────────────────────────────────

/**
 * Generate TOML config section for a Jira workspace + project
 */
export function generateJiraToml(
  domain: string,
  email: string,
  apiKeyEnv: string,
  projectKey: string,
  accountId: string
): string {
  return [
    `[kanban.jira."${domain}"]`,
    `enabled = true`,
    `email = "${email}"`,
    `api_key_env = "${apiKeyEnv}"`,
    ``,
    `[kanban.jira."${domain}".projects.${projectKey}]`,
    `sync_user_id = "${accountId}"`,
    `collection_name = "dev_kanban"`,
    ``,
  ].join('\n');
}

/**
 * Generate TOML config section for a Linear team + project
 */
export function generateLinearToml(
  teamId: string,
  apiKeyEnv: string,
  userId: string
): string {
  return [
    `[kanban.linear."${teamId}"]`,
    `enabled = true`,
    `api_key_env = "${apiKeyEnv}"`,
    ``,
    `[kanban.linear."${teamId}".projects.default]`,
    `sync_user_id = "${userId}"`,
    `collection_name = "dev_kanban"`,
    ``,
  ].join('\n');
}

/**
 * Read config.toml, append or replace a kanban section, write back.
 *
 * If the section header already exists, prompts user to confirm replacement.
 * Returns true if written successfully.
 */
export async function writeKanbanConfig(section: string): Promise<boolean> {
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
    // file doesn't exist yet, start fresh
  }

  // Extract the section header (first line) to check for duplicates
  const headerLine = section.split('\n')[0];
  if (existing.includes(headerLine)) {
    const replace = await vscode.window.showWarningMessage(
      `Config already contains ${headerLine}. Replace it?`,
      'Replace',
      'Cancel'
    );
    if (replace !== 'Replace') {
      return false;
    }

    // Remove old section: from header line to next top-level section or EOF
    const headerEscaped = headerLine.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const sectionRegex = new RegExp(
      `${headerEscaped}[\\s\\S]*?(?=\\n\\[(?!kanban\\.)|\n*$)`,
      'm'
    );
    existing = existing.replace(sectionRegex, '');
  }

  // Ensure trailing newline before appending
  const separator = existing.length > 0 && !existing.endsWith('\n') ? '\n\n' : '\n';
  const newContent = existing.length > 0 ? existing.trimEnd() + separator + section : section;

  await fs.writeFile(configPath, newContent, 'utf-8');
  return true;
}

// ─── API Validation ────────────────────────────────────────────────────

export interface JiraValidationResult {
  valid: boolean;
  accountId: string;
  displayName: string;
  error?: string;
}

export interface JiraProject {
  key: string;
  name: string;
}

export interface LinearTeam {
  id: string;
  name: string;
  key: string;
}

export interface LinearValidationResult {
  valid: boolean;
  userId: string;
  userName: string;
  orgName: string;
  teams: LinearTeam[];
  error?: string;
}

/**
 * Validate Jira credentials by calling GET /rest/api/3/myself
 */
export async function validateJiraCredentials(
  domain: string,
  email: string,
  apiToken: string
): Promise<JiraValidationResult> {
  const auth = Buffer.from(`${email}:${apiToken}`).toString('base64');
  try {
    const response = await fetch(`https://${domain}/rest/api/3/myself`, {
      headers: {
        Authorization: `Basic ${auth}`,
        Accept: 'application/json',
      },
    });

    if (!response.ok) {
      const status = response.status;
      if (status === 401) {
        return { valid: false, accountId: '', displayName: '', error: 'Invalid credentials (401). Check email and API token.' };
      }
      if (status === 403) {
        return { valid: false, accountId: '', displayName: '', error: 'Access forbidden (403). Token may lack permissions.' };
      }
      return { valid: false, accountId: '', displayName: '', error: `Jira API error: ${status}` };
    }

    const data = (await response.json()) as {
      accountId?: string;
      displayName?: string;
    };

    if (!data.accountId) {
      return { valid: false, accountId: '', displayName: '', error: 'No accountId in response' };
    }

    return {
      valid: true,
      accountId: data.accountId,
      displayName: data.displayName ?? '',
    };
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    return { valid: false, accountId: '', displayName: '', error: `Connection failed: ${msg}` };
  }
}

/**
 * Fetch Jira projects for QuickPick selection
 */
export async function fetchJiraProjects(
  domain: string,
  email: string,
  apiToken: string
): Promise<JiraProject[]> {
  const auth = Buffer.from(`${email}:${apiToken}`).toString('base64');
  try {
    const response = await fetch(
      `https://${domain}/rest/api/3/project/search?maxResults=50&orderBy=name`,
      {
        headers: {
          Authorization: `Basic ${auth}`,
          Accept: 'application/json',
        },
      }
    );

    if (!response.ok) {
      return [];
    }

    const data = (await response.json()) as {
      values?: Array<{ key?: string; name?: string }>;
    };

    return (data.values ?? [])
      .filter((p): p is { key: string; name: string } => !!p.key && !!p.name)
      .map((p) => ({ key: p.key, name: p.name }));
  } catch {
    return [];
  }
}

/**
 * Validate Linear credentials by querying viewer + organization + teams
 */
export async function validateLinearCredentials(
  apiKey: string
): Promise<LinearValidationResult> {
  const query = `
    query {
      viewer { id name email }
      organization { name urlKey }
      teams { nodes { id name key } }
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
      const status = response.status;
      if (status === 401) {
        return { valid: false, userId: '', userName: '', orgName: '', teams: [], error: 'Invalid API key (401).' };
      }
      return { valid: false, userId: '', userName: '', orgName: '', teams: [], error: `Linear API error: ${status}` };
    }

    const data = (await response.json()) as {
      data?: {
        viewer?: { id?: string; name?: string };
        organization?: { name?: string };
        teams?: { nodes?: Array<{ id?: string; name?: string; key?: string }> };
      };
    };

    const viewer = data?.data?.viewer;
    const org = data?.data?.organization;
    const teamNodes = data?.data?.teams?.nodes ?? [];

    if (!viewer?.id) {
      return { valid: false, userId: '', userName: '', orgName: '', teams: [], error: 'Could not retrieve user info' };
    }

    const teams: LinearTeam[] = teamNodes
      .filter((t): t is { id: string; name: string; key: string } => !!t.id && !!t.name && !!t.key)
      .map((t) => ({ id: t.id, name: t.name, key: t.key }));

    return {
      valid: true,
      userId: viewer.id,
      userName: viewer.name ?? '',
      orgName: org?.name ?? '',
      teams,
    };
  } catch (err) {
    const msg = err instanceof Error ? err.message : 'Unknown error';
    return { valid: false, userId: '', userName: '', orgName: '', teams: [], error: `Connection failed: ${msg}` };
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
}): Promise<string | 'back' | undefined> {
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
 * Show info message with copy-to-clipboard action for shell profile env vars
 */
export async function showEnvVarInstructions(envLines: string[]): Promise<void> {
  const exportBlock = envLines.join('\n');

  const action = await vscode.window.showInformationMessage(
    'Add these to your shell profile (~/.zshrc or ~/.bashrc) for persistence across restarts:',
    'Copy to Clipboard'
  );

  if (action === 'Copy to Clipboard') {
    await vscode.env.clipboard.writeText(exportBlock);
    vscode.window.showInformationMessage('Environment variable exports copied to clipboard.');
  }
}

// ─── Interactive Onboarding Flows ──────────────────────────────────────

/**
 * Jira Cloud onboarding: domain -> email -> API token -> validate -> pick project -> write config
 */
export async function onboardJira(
  context: vscode.ExtensionContext
): Promise<void> {
  const title = 'Configure Jira Cloud';
  let step = 1;

  // Collect credentials with back navigation
  let domain = '';
  let email = '';
  let apiToken = '';

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

      if (result === undefined) { return; }
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

      if (result === undefined) { return; }
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

      const result = await new Promise<string | 'back' | undefined>((resolve) => {
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
            vscode.env.openExternal(
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

      if (result === undefined) { return; }
      if (result === 'back') { step--; continue; }
      apiToken = result;
      step = 4; // proceed to validation
    }
  }

  // Validate credentials
  const validation = await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: 'Validating Jira credentials...',
      cancellable: false,
    },
    () => validateJiraCredentials(domain, email, apiToken)
  );

  if (!validation.valid) {
    const retry = await vscode.window.showErrorMessage(
      `Jira validation failed: ${validation.error}`,
      'Retry',
      'Cancel'
    );
    if (retry === 'Retry') {
      return onboardJira(context);
    }
    return;
  }

  vscode.window.showInformationMessage(
    `Authenticated as ${validation.displayName} (${validation.accountId})`
  );

  // Fetch and select project
  const projects = await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: 'Fetching Jira projects...',
      cancellable: false,
    },
    () => fetchJiraProjects(domain, email, apiToken)
  );

  if (projects.length === 0) {
    vscode.window.showWarningMessage(
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

  // Write config
  const envVarName = 'OPERATOR_JIRA_API_KEY';
  const toml = generateJiraToml(
    domain,
    email,
    envVarName,
    selectedProject.label,
    validation.accountId
  );

  const written = await writeKanbanConfig(toml);
  if (!written) {
    return;
  }

  // Set env vars for current session
  process.env['OPERATOR_JIRA_API_KEY'] = apiToken;
  process.env['OPERATOR_JIRA_DOMAIN'] = domain;
  process.env['OPERATOR_JIRA_EMAIL'] = email;

  // Show success + env var instructions
  vscode.window.showInformationMessage(
    `Jira configured! Config written to ${getResolvedConfigPath()}`
  );

  await showEnvVarInstructions([
    `export OPERATOR_JIRA_API_KEY="<your-api-token>"`,
  ]);

  // Update walkthrough context
  await updateWalkthroughContext(context);
}

/**
 * Linear onboarding: API key -> validate -> pick team -> write config
 */
export async function onboardLinear(
  context: vscode.ExtensionContext
): Promise<void> {
  const title = 'Configure Linear';

  // Step 1: API key
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
        vscode.env.openExternal(
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

  if (!apiKey) {
    return;
  }

  // Validate credentials
  const validation = await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: 'Validating Linear credentials...',
      cancellable: false,
    },
    () => validateLinearCredentials(apiKey)
  );

  if (!validation.valid) {
    const retry = await vscode.window.showErrorMessage(
      `Linear validation failed: ${validation.error}`,
      'Retry',
      'Cancel'
    );
    if (retry === 'Retry') {
      return onboardLinear(context);
    }
    return;
  }

  vscode.window.showInformationMessage(
    `Authenticated as ${validation.userName} in ${validation.orgName}`
  );

  // Step 2: Select team
  if (validation.teams.length === 0) {
    vscode.window.showWarningMessage(
      'No teams found. Check your permissions. Config was not written.'
    );
    return;
  }

  const teamItems = validation.teams.map((t) => ({
    label: t.name,
    description: t.key,
    detail: t.id,
  }));

  const selectedTeam = await vscode.window.showQuickPick(teamItems, {
    title: 'Select Linear Team',
    placeHolder: 'Choose a team to sync tickets from',
    step: 2,
    totalSteps: 2,
    ignoreFocusOut: true,
  } as vscode.QuickPickOptions & { step: number; totalSteps: number });

  if (!selectedTeam) {
    return;
  }

  // Write config
  const envVarName = 'OPERATOR_LINEAR_API_KEY';
  const toml = generateLinearToml(
    selectedTeam.detail!,
    envVarName,
    validation.userId
  );

  const written = await writeKanbanConfig(toml);
  if (!written) {
    return;
  }

  // Set env var for current session
  process.env['OPERATOR_LINEAR_API_KEY'] = apiKey;

  // Show success + env var instructions
  vscode.window.showInformationMessage(
    `Linear configured! Config written to ${getResolvedConfigPath()}`
  );

  await showEnvVarInstructions([
    `export OPERATOR_LINEAR_API_KEY="<your-api-key>"`,
  ]);

  // Update walkthrough context
  await updateWalkthroughContext(context);
}

/**
 * Entry-point: let user pick Jira or Linear, then route to the right flow
 */
export async function startKanbanOnboarding(
  context: vscode.ExtensionContext
): Promise<void> {
  const choice = await vscode.window.showQuickPick(
    [
      {
        label: '$(cloud) Jira Cloud',
        description: 'Connect to Jira Cloud with API token',
        provider: 'jira' as const,
      },
      {
        label: '$(cloud) Linear',
        description: 'Connect to Linear with API key',
        provider: 'linear' as const,
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

  if (choice.provider === 'jira') {
    await onboardJira(context);
  } else {
    await onboardLinear(context);
  }
}

// ─── Add Project/Team Flows ───────────────────────────────────────────

/**
 * Read and parse config.toml
 */
async function readParsedConfig(): Promise<Record<string, unknown>> {
  const configPath = getResolvedConfigPath();
  if (!configPath) { return {}; }
  try {
    const raw = await fs.readFile(configPath, 'utf-8');
    if (!raw.trim()) { return {}; }
    const { parse } = await importSmolToml();
    return parse(raw) as Record<string, unknown>;
  } catch {
    return {};
  }
}

/**
 * Generate TOML for a single Jira project section to append
 */
function generateJiraProjectToml(
  domain: string,
  projectKey: string,
  accountId: string,
  collectionName: string
): string {
  return [
    `[kanban.jira."${domain}".projects.${projectKey}]`,
    `sync_user_id = "${accountId}"`,
    `collection_name = "${collectionName}"`,
    ``,
  ].join('\n');
}

/**
 * Generate TOML for a single Linear team section to append
 */
function generateLinearTeamToml(
  workspaceKey: string,
  teamKey: string,
  userId: string,
  collectionName: string
): string {
  return [
    `[kanban.linear."${workspaceKey}".projects.${teamKey}]`,
    `sync_user_id = "${userId}"`,
    `collection_name = "${collectionName}"`,
    ``,
  ].join('\n');
}

/**
 * Add a new Jira project to an existing workspace in config.toml
 *
 * Reads existing Jira workspace config (email, api_key_env), fetches available
 * projects from the Jira API, shows a QuickPick, and writes the new project section.
 */
export async function addJiraProject(
  context: vscode.ExtensionContext,
  domain?: string
): Promise<void> {
  if (!domain) {
    vscode.window.showErrorMessage('No Jira domain specified.');
    return;
  }

  // Read config.toml to get workspace credentials
  const config = await readParsedConfig();
  const kanban = config.kanban as Record<string, unknown> | undefined;
  const jiraSection = kanban?.jira as Record<string, unknown> | undefined;
  const wsConfig = jiraSection?.[domain] as Record<string, unknown> | undefined;

  if (!wsConfig) {
    vscode.window.showErrorMessage(`No Jira workspace configured for ${domain}.`);
    return;
  }

  const email = wsConfig.email as string | undefined;
  const apiKeyEnv = (wsConfig.api_key_env as string) || 'OPERATOR_JIRA_API_KEY';
  let apiToken = process.env[apiKeyEnv];

  if (!email) {
    vscode.window.showErrorMessage(`No email configured for Jira workspace ${domain}.`);
    return;
  }

  // Prompt for API token if not in env
  if (!apiToken) {
    apiToken = await vscode.window.showInputBox({
      title: 'Jira API Token',
      prompt: `Enter API token for ${domain} (env var ${apiKeyEnv} not set)`,
      password: true,
      ignoreFocusOut: true,
    }) ?? undefined;
    if (!apiToken) { return; }
    // Set for current session
    process.env[apiKeyEnv] = apiToken;
  }

  // Find already-configured project keys
  const existingProjects = new Set<string>();
  const projectsSection = wsConfig.projects as Record<string, unknown> | undefined;
  if (projectsSection) {
    for (const key of Object.keys(projectsSection)) {
      existingProjects.add(key);
    }
  }

  // Fetch available projects
  const projects = await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: 'Fetching Jira projects...',
      cancellable: false,
    },
    () => fetchJiraProjects(domain, email, apiToken!)
  );

  if (projects.length === 0) {
    vscode.window.showWarningMessage('No projects found. Check your permissions.');
    return;
  }

  // Filter out already-configured projects
  const available = projects.filter((p) => !existingProjects.has(p.key));
  if (available.length === 0) {
    vscode.window.showInformationMessage('All available projects are already configured.');
    return;
  }

  const selected = await vscode.window.showQuickPick(
    available.map((p) => ({ label: p.key, description: p.name })),
    {
      title: `Add Jira Project to ${domain}`,
      placeHolder: 'Select a project to sync',
      ignoreFocusOut: true,
    }
  );

  if (!selected) { return; }

  // Get the user's account ID from validation
  const validation = await validateJiraCredentials(domain, email, apiToken);
  if (!validation.valid) {
    vscode.window.showErrorMessage(`Jira validation failed: ${validation.error}`);
    return;
  }

  // Write project section to config.toml
  const toml = generateJiraProjectToml(domain, selected.label, validation.accountId, 'dev_kanban');
  const written = await writeKanbanConfig(toml);
  if (!written) { return; }

  vscode.window.showInformationMessage(
    `Added Jira project ${selected.label} to ${domain}`
  );

  await updateWalkthroughContext(context);
}

/**
 * Add a new Linear team to an existing workspace in config.toml
 *
 * Reads existing Linear workspace config (api_key_env), fetches available
 * teams from the Linear API, shows a QuickPick, and writes the new team section.
 */
export async function addLinearTeam(
  context: vscode.ExtensionContext,
  workspaceKey?: string
): Promise<void> {
  if (!workspaceKey) {
    vscode.window.showErrorMessage('No Linear workspace specified.');
    return;
  }

  // Read config.toml to get workspace credentials
  const config = await readParsedConfig();
  const kanban = config.kanban as Record<string, unknown> | undefined;
  const linearSection = kanban?.linear as Record<string, unknown> | undefined;
  const wsConfig = linearSection?.[workspaceKey] as Record<string, unknown> | undefined;

  if (!wsConfig) {
    vscode.window.showErrorMessage(`No Linear workspace configured for ${workspaceKey}.`);
    return;
  }

  const apiKeyEnv = (wsConfig.api_key_env as string) || 'OPERATOR_LINEAR_API_KEY';
  let apiKey = process.env[apiKeyEnv];

  // Prompt for API key if not in env
  if (!apiKey) {
    apiKey = await vscode.window.showInputBox({
      title: 'Linear API Key',
      prompt: `Enter API key for Linear (env var ${apiKeyEnv} not set)`,
      password: true,
      ignoreFocusOut: true,
    }) ?? undefined;
    if (!apiKey) { return; }
    // Set for current session
    process.env[apiKeyEnv] = apiKey;
  }

  // Find already-configured team keys
  const existingTeams = new Set<string>();
  const projectsSection = wsConfig.projects as Record<string, unknown> | undefined;
  if (projectsSection) {
    for (const key of Object.keys(projectsSection)) {
      existingTeams.add(key);
    }
  }

  // Fetch available teams
  const validation = await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: 'Fetching Linear teams...',
      cancellable: false,
    },
    () => validateLinearCredentials(apiKey!)
  );

  if (!validation.valid) {
    vscode.window.showErrorMessage(`Linear validation failed: ${validation.error}`);
    return;
  }

  if (validation.teams.length === 0) {
    vscode.window.showWarningMessage('No teams found. Check your permissions.');
    return;
  }

  // Filter out already-configured teams
  const available = validation.teams.filter((t) => !existingTeams.has(t.key));
  if (available.length === 0) {
    vscode.window.showInformationMessage('All available teams are already configured.');
    return;
  }

  const selected = await vscode.window.showQuickPick(
    available.map((t) => ({ label: t.key, description: t.name, detail: t.id })),
    {
      title: 'Add Linear Team',
      placeHolder: 'Select a team to sync',
      ignoreFocusOut: true,
    }
  );

  if (!selected) { return; }

  // Write team section to config.toml
  const toml = generateLinearTeamToml(workspaceKey, selected.label, validation.userId, 'dev_kanban');
  const written = await writeKanbanConfig(toml);
  if (!written) { return; }

  vscode.window.showInformationMessage(
    `Added Linear team ${selected.label} (${selected.description})`
  );

  await updateWalkthroughContext(context);
}
