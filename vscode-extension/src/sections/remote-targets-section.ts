import * as vscode from "vscode";
import { StatusItem } from "../status-item";
import type { SectionContext, StatusSection } from "./types";
import type { SectionId, SectionHealth } from "../generated";
import type { TargetResponse } from "../generated/TargetResponse";
import { discoverApiUrl, OperatorApiClient } from "../api-client";

/**
 * Remote Targets section - SSH hosts and Coder workspaces, and whether this
 * configuration may currently launch onto them.
 *
 * Stays visible without a licence: hiding it would make Premium look like a
 * missing feature rather than a locked one. Registering and probing targets
 * happens in the hosted Operator UI, which the rows link out to.
 */
export class RemoteTargetsSection implements StatusSection {
  readonly sectionId: SectionId = "remote-targets";
  readonly prerequisites: SectionId[] = [];

  private targets: TargetResponse[] = [];
  private apiAvailable = false;

  health(): SectionHealth {
    if (!this.apiAvailable || this.targets.length === 0) {
      return "Gray";
    }
    return this.targets.every((target) => target.entitled) ? "Green" : "Yellow";
  }

  async check(ctx: SectionContext): Promise<void> {
    try {
      const client = new OperatorApiClient(await discoverApiUrl(ctx.ticketsDir));
      const response = await client.listTargets();
      this.targets = response.targets.filter(
        (target: TargetResponse) => target.kind === "ssh" || target.kind === "coder",
      );
      this.apiAvailable = true;
    } catch {
      this.targets = [];
      this.apiAvailable = false;
    }
  }

  getTopLevelItem(_ctx: SectionContext): StatusItem {
    const description = this.apiAvailable
      ? this.targets.length === 0
        ? "Premium · none configured"
        : `${this.targets.length} configured`
      : "API required";
    return new StatusItem({
      label: "Remote Targets",
      description,
      icon: "server-environment",
      collapsibleState:
        this.targets.length > 0
          ? vscode.TreeItemCollapsibleState.Collapsed
          : vscode.TreeItemCollapsibleState.None,
      sectionId: this.sectionId,
      health: this.health(),
    });
  }

  getChildren(_ctx: SectionContext, _element?: StatusItem): StatusItem[] {
    return this.targets.map(
      (target) =>
        new StatusItem({
          label: target.display_name ?? target.name,
          description: `${target.kind} · ${target.entitled ? "available" : "license required"}`,
          icon: "server",
          collapsibleState: vscode.TreeItemCollapsibleState.None,
          sectionId: this.sectionId,
          health: target.entitled ? "Green" : "Yellow",
          command: {
            command: "operator.openRemoteTargets",
            title: "Open Remote Targets in Operator UI",
          },
        }),
    );
  }
}
