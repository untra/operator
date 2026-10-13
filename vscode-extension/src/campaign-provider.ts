/**
 * Campaigns view: campaign → wave → ticket, read from the Operator REST API.
 *
 * Campaigns are Premium. Without a license the view stays visible with a
 * single row linking to the license page, so the feature reads as locked
 * rather than missing.
 */

import * as vscode from "vscode";
import type { OperatorApiClient } from "./api-client";
import type {
  BlockReason,
  CampaignMember,
  CampaignStatus,
  CampaignStatusView,
  CampaignSummary,
} from "./generated";

const LAUNCHABLE: ReadonlySet<CampaignStatus> = new Set(["draft", "ready", "paused"]);

export type CampaignNodeKind = "paywall" | "message" | "campaign" | "wave" | "ticket";

export class CampaignNode extends vscode.TreeItem {
  constructor(
    readonly kind: CampaignNodeKind,
    label: string,
    collapsible: vscode.TreeItemCollapsibleState,
    readonly campaignId?: string,
    readonly wave?: number,
  ) {
    super(label, collapsible);
  }
}

/** Context value driving the launch and pause menu entries. */
export function campaignContextValue(status: CampaignStatus): string {
  if (status === "running") {
    return "campaign-running";
  }
  return LAUNCHABLE.has(status) ? "campaign-launchable" : "campaign";
}

export function blockedLabel(reason: BlockReason): string {
  if (reason.kind === "gate") {
    return `waiting on ${reason.refs.join(", ")}`;
  }
  if (reason.kind === "unsynced") {
    return `waiting on unsynced ${reason.refs.join(", ")}`;
  }
  return BLOCKED_LABELS[reason.kind];
}

const BLOCKED_LABELS = {
  dependencies: "waiting on dependencies",
  license: "needs a Premium license",
  capacity: "waiting for a free slot",
} as const;

function memberNote(member: CampaignMember, view: CampaignStatusView): string | undefined {
  const blocked = view.report.blocked[member.id];
  if (blocked) {
    return blockedLabel(blocked);
  }
  if (view.report.awaiting_operator.includes(member.id)) {
    return "awaiting operator";
  }
  if (view.report.stalled.includes(member.id)) {
    return "stalled";
  }
  return undefined;
}

function campaignNode({ campaign, members }: CampaignSummary): CampaignNode {
  const node = new CampaignNode(
    "campaign",
    `${campaign.id} · ${campaign.title}`,
    vscode.TreeItemCollapsibleState.Collapsed,
    campaign.id,
  );
  node.description = `${campaign.status} · ${members} ${members === 1 ? "ticket" : "tickets"}`;
  node.iconPath = new vscode.ThemeIcon("milestone");
  node.contextValue = campaignContextValue(campaign.status);
  return node;
}

function waveNode(campaignId: string, index: number, size: number): CampaignNode {
  const node = new CampaignNode(
    "wave",
    `Wave ${index + 1}`,
    vscode.TreeItemCollapsibleState.Expanded,
    campaignId,
    index,
  );
  node.description = `${size} ${size === 1 ? "ticket" : "tickets"}`;
  node.iconPath = new vscode.ThemeIcon("layers");
  return node;
}

function ticketNode(member: CampaignMember, view: CampaignStatusView): CampaignNode {
  const node = new CampaignNode("ticket", member.id, vscode.TreeItemCollapsibleState.None);
  const note = memberNote(member, view);
  node.description = note ? `${member.column} · ${note}` : member.column;
  node.tooltip = member.summary;
  node.iconPath = new vscode.ThemeIcon(member.column === "completed" ? "pass" : "note");
  return node;
}

function paywallNode(): CampaignNode {
  const node = new CampaignNode("paywall", "Campaigns", vscode.TreeItemCollapsibleState.None);
  node.description = "Premium license required";
  node.iconPath = new vscode.ThemeIcon("lock");
  node.command = { command: "operator.openLicense", title: "Open License" };
  return node;
}

function messageNode(text: string): CampaignNode {
  const node = new CampaignNode("message", text, vscode.TreeItemCollapsibleState.None);
  node.iconPath = new vscode.ThemeIcon("warning");
  return node;
}

export class CampaignTreeProvider implements vscode.TreeDataProvider<CampaignNode> {
  private readonly changed = new vscode.EventEmitter<CampaignNode | undefined>();
  readonly onDidChangeTreeData = this.changed.event;

  private client: OperatorApiClient | undefined;
  private roots: CampaignNode[] = [];
  private readonly views = new Map<string, CampaignStatusView>();

  setApiClient(client: OperatorApiClient | undefined): void {
    this.client = client;
  }

  async refresh(): Promise<void> {
    this.views.clear();
    this.roots = await this.loadRoots();
    this.changed.fire(undefined);
  }

  private async loadRoots(): Promise<CampaignNode[]> {
    if (!this.client) {
      return [];
    }
    try {
      const license = await this.client.license();
      if (!license.premium) {
        return [paywallNode()];
      }
      const list = await this.client.listCampaigns();
      return list.campaigns.map(campaignNode);
    } catch (err) {
      return [messageNode(err instanceof Error ? err.message : "Campaigns unavailable")];
    }
  }

  private async view(campaignId: string): Promise<CampaignStatusView | undefined> {
    const cached = this.views.get(campaignId);
    if (cached || !this.client) {
      return cached;
    }
    const view = await this.client.campaignStatus(campaignId);
    this.views.set(campaignId, view);
    return view;
  }

  getTreeItem(element: CampaignNode): vscode.TreeItem {
    return element;
  }

  async getChildren(element?: CampaignNode): Promise<CampaignNode[]> {
    if (!element) {
      return this.roots;
    }
    if (!element.campaignId || (element.kind !== "campaign" && element.kind !== "wave")) {
      return [];
    }
    try {
      const view = await this.view(element.campaignId);
      if (!view) {
        return [];
      }
      if (element.kind === "campaign") {
        return view.waves.map((wave, index) => waveNode(view.id, index, wave.length));
      }
      const byId = new Map(view.tickets.map((m) => [m.id, m]));
      return (view.waves[element.wave ?? 0] ?? [])
        .map((id) => byId.get(id))
        .filter((m): m is CampaignMember => m !== undefined)
        .map((m) => ticketNode(m, view));
    } catch (err) {
      return [messageNode(err instanceof Error ? err.message : "Campaign unavailable")];
    }
  }
}
