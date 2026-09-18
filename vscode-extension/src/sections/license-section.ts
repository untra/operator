import * as vscode from "vscode";
import { StatusItem } from "../status-item";
import type { SectionContext, StatusSection } from "./types";
import type { SectionId, SectionHealth } from "../generated";
import type { LicenseResponse } from "../generated/LicenseResponse";
import { discoverApiUrl, OperatorApiClient } from "../api-client";

/**
 * License section - which tier this configuration runs at.
 *
 * Read-only, like the rest of the extension's status tree: installing a licence
 * happens in the hosted Operator UI, which the rows link out to. A missing
 * licence is the free tier working as intended, so it reads Gray rather than
 * as a fault.
 */
const STATUS_LABELS: Record<LicenseResponse["status"], string> = {
  missing: "Free",
  valid: "Premium",
  expired: "Expired",
  not_yet_valid: "Not yet valid",
  invalid: "Invalid",
};

const STATUS_HEALTH: Record<LicenseResponse["status"], SectionHealth> = {
  missing: "Gray",
  valid: "Green",
  expired: "Yellow",
  not_yet_valid: "Yellow",
  invalid: "Red",
};

const day = (seconds: bigint): string =>
  new Date(Number(seconds) * 1000).toISOString().slice(0, 10);

export class LicenseSection implements StatusSection {
  readonly sectionId: SectionId = "license";
  readonly prerequisites: SectionId[] = [];

  private license: LicenseResponse | null = null;

  health(): SectionHealth {
    return this.license ? STATUS_HEALTH[this.license.status] : "Gray";
  }

  async check(ctx: SectionContext): Promise<void> {
    try {
      const client = new OperatorApiClient(await discoverApiUrl(ctx.ticketsDir));
      this.license = await client.license();
    } catch {
      this.license = null;
    }
  }

  getTopLevelItem(_ctx: SectionContext): StatusItem {
    const label = this.license ? STATUS_LABELS[this.license.status] : "API required";
    return new StatusItem({
      label: "License",
      description: label,
      icon: "key",
      collapsibleState: this.license
        ? vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.None,
      sectionId: this.sectionId,
      health: this.health(),
    });
  }

  getChildren(_ctx: SectionContext, _element?: StatusItem): StatusItem[] {
    if (!this.license) {
      return [];
    }
    const rows: StatusItem[] = [
      this.row("Configuration", this.license.profile_id, "symbol-namespace"),
    ];
    if (this.license.terms) {
      rows.push(
        this.row("Licensed to", this.license.terms.sub, "account"),
        this.row("License ID", this.license.terms.jti, "file"),
        this.row(
          "Valid",
          `${day(this.license.terms.nbf)} to ${day(this.license.terms.exp)}`,
          "calendar",
        ),
      );
    } else {
      rows.push(this.row("Included", "Multiple local agents and local containers", "check"));
    }
    return rows;
  }

  private row(label: string, description: string, icon: string): StatusItem {
    return new StatusItem({
      label,
      description,
      icon,
      collapsibleState: vscode.TreeItemCollapsibleState.None,
      sectionId: this.sectionId,
      health: this.health(),
      command: { command: "operator.openLicense", title: "Open License in Operator UI" },
    });
  }
}
