/**
 * Campaign commands: preview-then-launch, pause, and open in the Operator UI.
 */

import * as vscode from "vscode";
import { discoverApiUrl, OperatorApiClient } from "./api-client";
import type { CampaignNode } from "./campaign-provider";
import type { CampaignLaunchOutcome } from "./generated";
import { openOperatorUi } from "./open-operator-ui";

export interface CampaignCommandContext {
  getCurrentTicketsDir: () => string | undefined;
  refreshCampaigns: () => Promise<void>;
}

/** One-paragraph summary of a dry run, shown before launching. */
export function launchSummary(outcome: CampaignLaunchOutcome): string {
  const waves = `${outcome.waves.length} ${outcome.waves.length === 1 ? "wave" : "waves"}`;
  const first =
    outcome.report.launched.length > 0 ? outcome.report.launched.join(", ") : "nothing yet";
  const policy = `Governance: ${outcome.policy.name}.`;
  return `${outcome.id}: ${waves}; the first launches ${first}. ${policy}`;
}

async function client(ctx: CampaignCommandContext): Promise<OperatorApiClient> {
  return new OperatorApiClient(await discoverApiUrl(ctx.getCurrentTicketsDir()));
}

function failed(action: string, err: unknown): void {
  const msg = err instanceof Error ? err.message : "Unknown error";
  void vscode.window.showErrorMessage(`Failed to ${action}: ${msg}`);
}

export async function launchCampaignCommand(
  ctx: CampaignCommandContext,
  node: CampaignNode | undefined,
): Promise<void> {
  const id = node?.campaignId;
  if (!id) {
    return;
  }
  try {
    const api = await client(ctx);
    const preview = await api.launchCampaign(id, true);
    if (preview.violations.length > 0) {
      void vscode.window.showErrorMessage(`${id} cannot launch: ${preview.violations.join("; ")}`);
      return;
    }
    const choice = await vscode.window.showInformationMessage(
      launchSummary(preview),
      { modal: true },
      "Launch",
    );
    if (choice !== "Launch") {
      return;
    }
    const outcome = await api.launchCampaign(id, false);
    void vscode.window.showInformationMessage(`${id} is ${outcome.status}.`);
    await ctx.refreshCampaigns();
  } catch (err) {
    failed("launch campaign", err);
  }
}

export async function pauseCampaignCommand(
  ctx: CampaignCommandContext,
  node: CampaignNode | undefined,
): Promise<void> {
  const id = node?.campaignId;
  if (!id) {
    return;
  }
  try {
    await (await client(ctx)).pauseCampaign(id);
    void vscode.window.showInformationMessage(`${id} paused.`);
    await ctx.refreshCampaigns();
  } catch (err) {
    failed("pause campaign", err);
  }
}

export function openCampaignCommand(
  ctx: CampaignCommandContext,
  node: CampaignNode | undefined,
): Promise<void> {
  return openOperatorUi(ctx.getCurrentTicketsDir(), "campaigns", node?.campaignId);
}
