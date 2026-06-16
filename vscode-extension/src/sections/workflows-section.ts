import * as vscode from 'vscode';
import { StatusItem } from '../status-item';
import type { SectionContext, StatusSection } from './types';
import type { SectionId, SectionHealth } from '../generated';
import { discoverApiUrl } from '../api-client';
import type { WorkflowFormatDto } from '../generated/WorkflowFormatDto';

/**
 * Workflows section — the export formats a ticket + issue type can be rendered
 * into (Claude `.js`, AGNT `.json`). Info-only (`Gray`), gated on `connections`:
 * preview/export run against the hosted API (`/api/v1/workflow-formats`), so the
 * section only appears once connections are ready. Rows link out to the hosted
 * Operator UI's Workflows page, where preview/export run — the extension does
 * not reimplement that surface.
 */
interface WorkflowsState {
  apiAvailable: boolean;
  formats: WorkflowFormatDto[];
}

export class WorkflowsSection implements StatusSection {
  readonly sectionId: SectionId = 'workflows';
  readonly prerequisites: SectionId[] = ['connections'];

  private state: WorkflowsState = { apiAvailable: false, formats: [] };

  health(): SectionHealth {
    return 'Gray';
  }

  async check(ctx: SectionContext): Promise<void> {
    try {
      const apiUrl = await discoverApiUrl(ctx.ticketsDir);
      const response = await fetch(`${apiUrl}/api/v1/workflow-formats`);
      if (response.ok) {
        const formats = await response.json() as WorkflowFormatDto[];
        this.state = { apiAvailable: true, formats };
        return;
      }
    } catch {
      // API not available — fall through to the unavailable state.
    }
    this.state = { apiAvailable: false, formats: [] };
  }

  getTopLevelItem(_ctx: SectionContext): StatusItem {
    const count = this.state.formats.length;
    return new StatusItem({
      label: 'Workflows',
      description: this.state.apiAvailable ? `${count} export formats` : 'API required',
      icon: 'type-hierarchy',
      collapsibleState: count > 0
        ? vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.None,
      sectionId: this.sectionId,
      health: this.health(),
    });
  }

  getChildren(_ctx: SectionContext, _element?: StatusItem): StatusItem[] {
    return this.state.formats.map((fmt) => new StatusItem({
      label: fmt.label,
      description: `${fmt.status} · .${fmt.extension}`,
      icon: 'tools',
      collapsibleState: vscode.TreeItemCollapsibleState.None,
      sectionId: this.sectionId,
      health: this.health(),
      // Link out to the hosted UI's Workflows page (preview/export live there).
      command: { command: 'operator.openWorkflows', title: 'Open Workflows in Operator UI' },
    }));
  }
}
