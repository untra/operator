/**
 * Launch dialogs for Operator VS Code extension
 *
 * QuickPick dialogs for selecting tickets and launch options.
 * Prefers delegators fetched from the Operator API; falls back
 * to hardcoded Claude models when the API is unavailable.
 */

import * as vscode from "vscode";
import type { LaunchOptions, TicketInfo, ModelOption } from "./types";
import type { DelegatorResponse } from "./generated/DelegatorResponse";
import type { DelegatorsResponse } from "./generated/DelegatorsResponse";
import type { ModelServerModelsResponse } from "./generated/ModelServerModelsResponse";
import { discoverApiUrl, OperatorApiClient } from "./api-client";

/**
 * Provider kind probed for the model fallback list when no delegators exist.
 * Matches the hardcoded Claude aliases this fallback replaces.
 */
const FALLBACK_MODEL_KIND = "anthropic-api";

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
async function fetchDelegators(ticketsDir: string | undefined): Promise<DelegatorResponse[]> {
  try {
    const client = new OperatorApiClient(await discoverApiUrl(ticketsDir));
    const data: DelegatorsResponse = await client.listDelegators();
    return data.delegators;
  } catch {
    // API not available
  }
  return [];
}

/**
 * Live text models for the no-delegator fallback, probed from a connected
 * provider. Returns `null` when the daemon is unreachable or the provider isn't
 * connected, so callers fall back to the hardcoded aliases.
 *
 * The probe already filters to LLM text models, so every id is dropdown-ready.
 */
async function fetchFallbackModels(
  ticketsDir: string | undefined,
): Promise<DelegatorPickItem[] | null> {
  try {
    const client = new OperatorApiClient(await discoverApiUrl(ticketsDir));
    const data: ModelServerModelsResponse = await client.providerModels(FALLBACK_MODEL_KIND);
    if (!data.reachable || data.models.length === 0) {
      return null;
    }
    return data.models.map((m) => ({
      label: m.id,
      description: m.display_name ?? undefined,
      delegatorName: undefined,
      // Runtime-free model id; the daemon's launch route accepts any string.
      model: m.id as ModelOption,
    }));
  } catch {
    return null;
  }
}

/** Hardcoded Claude aliases - the offline / not-connected safety net. */
function hardcodedModelItems(): DelegatorPickItem[] {
  return [
    {
      label: "sonnet",
      description: "Claude Sonnet (recommended)",
      delegatorName: undefined,
      model: "sonnet",
    },
    {
      label: "opus",
      description: "Claude Opus (most capable)",
      delegatorName: undefined,
      model: "opus",
    },
    {
      label: "haiku",
      description: "Claude Haiku (fastest)",
      delegatorName: undefined,
      model: "haiku",
    },
  ];
}

/**
 * Build the model/delegator pick list. When delegators exist, list them (plus an
 * "Auto" default). When none exist, prefer live probed models, falling back to
 * hardcoded aliases - resolved by [`resolveFallbackItems`] before this is called.
 */
function buildDelegatorItems(
  delegators: DelegatorResponse[],
  fallback?: DelegatorPickItem[] | null,
): DelegatorPickItem[] {
  if (delegators.length === 0) {
    return fallback && fallback.length > 0 ? fallback : hardcodedModelItems();
  }

  const items: DelegatorPickItem[] = [
    {
      label: "$(rocket) Auto",
      description: "Use default delegator",
      delegatorName: undefined,
      model: "sonnet", // fallback model if backend resolution fails
    },
  ];

  for (const d of delegators) {
    const yoloFlag = d.launch_config?.yolo ? " · yolo" : "";
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
  ticketsDir?: string,
): Promise<LaunchOptions | undefined> {
  // Fetch delegators from API; when none, probe live models for the fallback.
  const delegators = await fetchDelegators(ticketsDir);
  const fallback = delegators.length === 0 ? await fetchFallbackModels(ticketsDir) : null;
  const delegatorItems = buildDelegatorItems(delegators, fallback);

  const delegatorChoice = await vscode.window.showQuickPick(delegatorItems, {
    title: `Launch ${ticket.id}: Select Delegator`,
    placeHolder:
      delegators.length > 0 ? "Choose a delegator or use auto" : "Choose the model to use",
  });

  if (!delegatorChoice) {
    return undefined;
  }

  // Options checkboxes
  const optionItems: vscode.QuickPickItem[] = [
    {
      label: "YOLO Mode",
      description: "Auto-accept all permission prompts",
      picked: false,
    },
  ];

  if (hasExistingSession) {
    optionItems.push({
      label: "Resume Session",
      description: "Continue from previous session",
      picked: true,
    });
  }

  const optionChoices = await vscode.window.showQuickPick(optionItems, {
    title: `Launch ${ticket.id}: Options`,
    placeHolder: "Select launch options (Space to toggle)",
    canPickMany: true,
  });

  if (!optionChoices) {
    return undefined;
  }

  const selectedLabels = new Set(optionChoices.map((c) => c.label));

  // Execution target: offered only when the config declares targets/hosts;
  // Auto keeps the delegator's own resolution.
  const target = await pickTarget(ticket, ticketsDir);
  if (target === null) {
    return undefined; // dismissed
  }

  return {
    delegator: delegatorChoice.delegatorName ?? null,
    model: delegatorChoice.model,
    yoloMode: selectedLabels.has("YOLO Mode"),
    resumeSession: selectedLabels.has("Resume Session"),
    target,
  };
}

/**
 * Pick an execution target by name. Returns:
 * - undefined: no override (Auto, or no targets configured)
 * - a name: override
 * - null: dialog dismissed
 */
async function pickTarget(
  ticket: TicketInfo,
  ticketsDir?: string,
): Promise<string | undefined | null> {
  const names = await fetchTargetNames(ticketsDir);
  if (names.length === 0) {
    return undefined;
  }
  const items: vscode.QuickPickItem[] = [
    {
      label: "$(rocket) Auto",
      description: "Use the delegator's configured target",
    },
    ...names.map((n) => ({ label: n })),
  ];
  const choice = await vscode.window.showQuickPick(items, {
    title: `Launch ${ticket.id}: Execution Target`,
    placeHolder: "Where should the agent run?",
  });
  if (!choice) {
    return null;
  }
  return choice.label.includes("Auto") ? undefined : choice.label;
}

/** The implicit default; offering it as an override would be a no-op. */
const LOCAL_TARGET = "local";

/**
 * Named execution targets: [[targets]] entries, [[hosts]] synths, and docker
 * when configured. Served by the read-scoped targets route rather than the
 * admin-only configuration tree.
 */
async function fetchTargetNames(ticketsDir?: string): Promise<string[]> {
  try {
    const client = new OperatorApiClient(await discoverApiUrl(ticketsDir));
    const { targets } = await client.listExecutionTargets();
    return targets.filter((t) => t.name !== LOCAL_TARGET && t.available).map((t) => t.name);
  } catch {
    return [];
  }
}

/**
 * Show ticket picker for launch command
 */
export async function showTicketPicker(tickets: TicketInfo[]): Promise<TicketInfo | undefined> {
  if (tickets.length === 0) {
    void vscode.window.showInformationMessage("No tickets available");
    return undefined;
  }

  const items: TicketPickItem[] = tickets.map((t) => ({
    label: t.id,
    description: t.title,
    detail: `${t.type} - ${t.status}`,
    ticket: t,
  }));

  const choice = await vscode.window.showQuickPick(items, {
    title: "Select Ticket to Launch",
    placeHolder: "Choose a ticket",
    matchOnDescription: true,
    matchOnDetail: true,
  });

  return choice?.ticket;
}

/**
 * Show quick delegator picker (for fast launches)
 */
export async function showQuickDelegatorPicker(
  ticketsDir?: string,
): Promise<Pick<LaunchOptions, "delegator" | "model"> | undefined> {
  const delegators = await fetchDelegators(ticketsDir);
  const fallback = delegators.length === 0 ? await fetchFallbackModels(ticketsDir) : null;
  const items = buildDelegatorItems(delegators, fallback);

  const choice = await vscode.window.showQuickPick(items, {
    title: "Select Delegator",
    placeHolder:
      delegators.length > 0 ? "Choose a delegator for launch" : "Choose model for launch",
  });

  if (!choice) {
    return undefined;
  }

  return {
    delegator: choice.delegatorName ?? null,
    model: choice.model,
  };
}
