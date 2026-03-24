import * as vscode from 'vscode';
import { StatusItem } from '../status-item';
import type { SectionContext, StatusSection } from './types';
import { discoverApiUrl } from '../api-client';
import type { DelegatorResponse } from '../generated/DelegatorResponse';
import type { DelegatorsResponse } from '../generated/DelegatorsResponse';

interface DelegatorState {
  apiAvailable: boolean;
  delegators: DelegatorResponse[];
}

export class DelegatorSection implements StatusSection {
  readonly sectionId = 'delegators';

  private state: DelegatorState = { apiAvailable: false, delegators: [] };

  async check(ctx: SectionContext): Promise<void> {
    try {
      const apiUrl = await discoverApiUrl(ctx.ticketsDir);
      const response = await fetch(`${apiUrl}/api/v1/delegators`);
      if (response.ok) {
        const data = await response.json() as DelegatorsResponse;
        this.state = { apiAvailable: true, delegators: data.delegators };
        return;
      }
    } catch {
      // API not available
    }
    this.state = { apiAvailable: false, delegators: [] };
  }

  getTopLevelItem(_ctx: SectionContext): StatusItem {
    if (this.state.apiAvailable) {
      const count = this.state.delegators.length;
      return new StatusItem({
        label: 'Delegators',
        description: count > 0
          ? `${count} delegator${count !== 1 ? 's' : ''}`
          : 'None configured',
        icon: 'rocket',
        collapsibleState: count > 0
          ? vscode.TreeItemCollapsibleState.Collapsed
          : vscode.TreeItemCollapsibleState.Expanded,
        sectionId: this.sectionId,
      });
    }

    return new StatusItem({
      label: 'Delegators',
      description: 'API required',
      icon: 'rocket',
      collapsibleState: vscode.TreeItemCollapsibleState.None,
      sectionId: this.sectionId,
    });
  }

  getChildren(_ctx: SectionContext, _element?: StatusItem): StatusItem[] {
    const items: StatusItem[] = [];

    if (!this.state.apiAvailable) {
      return items;
    }

    for (const delegator of this.state.delegators) {
      const label = delegator.display_name || delegator.name;
      const yoloFlag = delegator.launch_config?.yolo ? ' · yolo' : '';

      items.push(new StatusItem({
        label,
        description: `${delegator.llm_tool}:${delegator.model}${yoloFlag}`,
        icon: `operator-${delegator.llm_tool}`,
        tooltip: this.buildTooltip(delegator),
        sectionId: this.sectionId,
      }));
    }

    items.push(new StatusItem({
      label: 'Add Delegator',
      icon: 'add',
      command: {
        command: 'operator.openSettings',
        title: 'Add Delegator',
      },
      sectionId: this.sectionId,
    }));

    return items;
  }

  private buildTooltip(d: DelegatorResponse): string {
    const lines = [`${d.name}: ${d.llm_tool} / ${d.model}`];
    if (d.launch_config) {
      if (d.launch_config.yolo) { lines.push('YOLO mode: enabled'); }
      if (d.launch_config.permission_mode) { lines.push(`Permission: ${d.launch_config.permission_mode}`); }
      if (d.launch_config.flags.length > 0) { lines.push(`Flags: ${d.launch_config.flags.join(' ')}`); }
    }
    return lines.join('\n');
  }
}
