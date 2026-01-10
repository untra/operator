/**
 * Launch dialogs for Operator VS Code extension
 *
 * QuickPick dialogs for selecting tickets and launch options.
 */

import * as vscode from 'vscode';
import { LaunchOptions, TicketInfo, ModelOption } from './types';

interface TicketPickItem extends vscode.QuickPickItem {
  ticket: TicketInfo;
}

interface ModelPickItem extends vscode.QuickPickItem {
  model: ModelOption;
}

/**
 * Show launch options dialog
 */
export async function showLaunchOptionsDialog(
  ticket: TicketInfo,
  hasExistingSession: boolean
): Promise<LaunchOptions | undefined> {
  // Model selection
  const modelItems: ModelPickItem[] = [
    {
      label: 'sonnet',
      description: 'Claude Sonnet (recommended)',
      model: 'sonnet',
    },
    {
      label: 'opus',
      description: 'Claude Opus (most capable)',
      model: 'opus',
    },
    {
      label: 'haiku',
      description: 'Claude Haiku (fastest)',
      model: 'haiku',
    },
  ];

  const modelChoice = await vscode.window.showQuickPick(modelItems, {
    title: `Launch ${ticket.id}: Select Model`,
    placeHolder: 'Choose the model to use',
  });

  if (!modelChoice) {
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
    model: modelChoice.model,
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
    vscode.window.showInformationMessage('No tickets available');
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
 * Show quick model picker (for fast launches)
 */
export async function showQuickModelPicker(): Promise<ModelOption | undefined> {
  const modelItems: ModelPickItem[] = [
    {
      label: '$(sparkle) Sonnet',
      description: 'Recommended balance of speed and capability',
      model: 'sonnet',
    },
    {
      label: '$(star-full) Opus',
      description: 'Most capable, slower',
      model: 'opus',
    },
    {
      label: '$(zap) Haiku',
      description: 'Fastest, simpler tasks',
      model: 'haiku',
    },
  ];

  const choice = await vscode.window.showQuickPick(modelItems, {
    title: 'Select Model',
    placeHolder: 'Choose model for launch',
  });

  return choice?.model;
}
