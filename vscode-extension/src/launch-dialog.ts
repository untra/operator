/**
 * Launch dialogs for Operator VS Code extension
 *
 * QuickPick dialogs for selecting tickets and launch options.
 * Prefers delegators fetched from the Operator API; falls back
 * to hardcoded Claude models when the API is unavailable.
 */

import * as vscode from 'vscode';
import { LaunchOptions, TicketInfo, ModelOption } from './types';
import type { DelegatorResponse } from './generated/DelegatorResponse';
import type { DelegatorsResponse } from './generated/DelegatorsResponse';
import { discoverApiUrl } from './api-client';

interface TicketPickItem extends vscode.QuickPickItem {
  ticket: TicketInfo;
}

interface DelegatorPickItem extends vscode.QuickPickItem {
  delegatorName: string | undefined;
  model: ModelOption;
}

/**
 * Fetch configured delegators from the Operator API.
 * Returns an empty array if the API is unavailable.
 */
async function fetchDelegators(
  ticketsDir: string | undefined
): Promise<DelegatorResponse[]> {
  try {
    const apiUrl = await discoverApiUrl(ticketsDir);
    const response = await fetch(`${apiUrl}/api/v1/delegators`);
    if (response.ok) {
      const data = (await response.json()) as DelegatorsResponse;
      return data.delegators;
    }
  } catch {
    // API not available
  }
  return [];
}

/**
 * Build delegator QuickPick items from API response.
 * Includes an "Auto" default and falls back to hardcoded models when empty.
 */
function buildDelegatorItems(
  delegators: DelegatorResponse[]
): DelegatorPickItem[] {
  if (delegators.length === 0) {
    // Fallback: hardcoded Claude models
    return [
      {
        label: 'sonnet',
        description: 'Claude Sonnet (recommended)',
        delegatorName: undefined,
        model: 'sonnet',
      },
      {
        label: 'opus',
        description: 'Claude Opus (most capable)',
        delegatorName: undefined,
        model: 'opus',
      },
      {
        label: 'haiku',
        description: 'Claude Haiku (fastest)',
        delegatorName: undefined,
        model: 'haiku',
      },
    ];
  }

  const items: DelegatorPickItem[] = [
    {
      label: '$(rocket) Auto',
      description: 'Use default delegator',
      delegatorName: undefined,
      model: 'sonnet', // fallback model if backend resolution fails
    },
  ];

  for (const d of delegators) {
    const yoloFlag = d.launch_config?.yolo ? ' · yolo' : '';
    items.push({
      label: d.display_name || d.name,
      description: `${d.llm_tool}:${d.model}${yoloFlag}`,
      delegatorName: d.name,
      model: d.model as ModelOption,
    });
  }

  return items;
}

/**
 * Show launch options dialog
 */
export async function showLaunchOptionsDialog(
  ticket: TicketInfo,
  hasExistingSession: boolean,
  ticketsDir?: string
): Promise<LaunchOptions | undefined> {
  // Fetch delegators from API
  const delegators = await fetchDelegators(ticketsDir);
  const delegatorItems = buildDelegatorItems(delegators);

  const delegatorChoice = await vscode.window.showQuickPick(delegatorItems, {
    title: `Launch ${ticket.id}: Select Delegator`,
    placeHolder:
      delegators.length > 0
        ? 'Choose a delegator or use auto'
        : 'Choose the model to use',
  });

  if (!delegatorChoice) {
    return undefined;
  }

  // Options checkboxes
  const optionItems: vscode.QuickPickItem[] = [
    {
      label: 'YOLO Mode',
      description: 'Auto-accept all permission prompts',
      picked: false,
    },
  ];

  if (hasExistingSession) {
    optionItems.push({
      label: 'Resume Session',
      description: 'Continue from previous session',
      picked: true,
    });
  }

  const optionChoices = await vscode.window.showQuickPick(optionItems, {
    title: `Launch ${ticket.id}: Options`,
    placeHolder: 'Select launch options (Space to toggle)',
    canPickMany: true,
  });

  if (!optionChoices) {
    return undefined;
  }

  const selectedLabels = optionChoices.map((c) => c.label);

  return {
    delegator: delegatorChoice.delegatorName ?? null,
    model: delegatorChoice.model,
    yoloMode: selectedLabels.includes('YOLO Mode'),
    resumeSession: selectedLabels.includes('Resume Session'),
  };
}

/**
 * Show ticket picker for launch command
 */
export async function showTicketPicker(
  tickets: TicketInfo[]
): Promise<TicketInfo | undefined> {
  if (tickets.length === 0) {
    void vscode.window.showInformationMessage('No tickets available');
    return undefined;
  }

  const items: TicketPickItem[] = tickets.map((t) => ({
    label: t.id,
    description: t.title,
    detail: `${t.type} - ${t.status}`,
    ticket: t,
  }));

  const choice = await vscode.window.showQuickPick(items, {
    title: 'Select Ticket to Launch',
    placeHolder: 'Choose a ticket',
    matchOnDescription: true,
    matchOnDetail: true,
  });

  return choice?.ticket;
}

/**
 * Show quick delegator picker (for fast launches)
 */
export async function showQuickDelegatorPicker(
  ticketsDir?: string
): Promise<Pick<LaunchOptions, 'delegator' | 'model'> | undefined> {
  const delegators = await fetchDelegators(ticketsDir);
  const items = buildDelegatorItems(delegators);

  const choice = await vscode.window.showQuickPick(items, {
    title: 'Select Delegator',
    placeHolder:
      delegators.length > 0
        ? 'Choose a delegator for launch'
        : 'Choose model for launch',
  });

  if (!choice) {
    return undefined;
  }

  return {
    delegator: choice.delegatorName ?? null,
    model: choice.model,
  };
}
