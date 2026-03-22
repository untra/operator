import * as vscode from 'vscode';
import { StatusItem } from '../status-item';
import type { SectionContext, StatusSection } from './types';
import type { IssueTypeSummary } from '../generated/IssueTypeSummary';
import { DEFAULT_ISSUE_TYPES, GLYPH_TO_ICON, COLOR_TO_THEME } from '../issuetype-service';
import { discoverApiUrl } from '../api-client';

interface IssueTypeState {
  apiAvailable: boolean;
  types: IssueTypeSummary[];
}

export class IssueTypeSection implements StatusSection {
  readonly sectionId = 'issuetypes';

  private state: IssueTypeState = { apiAvailable: false, types: [] };

  async check(ctx: SectionContext): Promise<void> {
    // Try fetching from API
    try {
      const apiUrl = await discoverApiUrl(ctx.ticketsDir);
      const response = await fetch(`${apiUrl}/api/v1/issuetypes`);
      if (response.ok) {
        const types = await response.json() as IssueTypeSummary[];
        this.state = { apiAvailable: true, types };
        return;
      }
    } catch {
      // API not available
    }

    // Fall back to defaults
    this.state = { apiAvailable: false, types: [...DEFAULT_ISSUE_TYPES] };
  }

  getTopLevelItem(_ctx: SectionContext): StatusItem {
    const count = this.state.types.length;
    if (this.state.apiAvailable) {
      return new StatusItem({
        label: 'Issue Types',
        description: `${count} type${count !== 1 ? 's' : ''}`,
        icon: 'check',
        collapsibleState: vscode.TreeItemCollapsibleState.Collapsed,
        sectionId: this.sectionId,
      });
    }

    return new StatusItem({
      label: 'Issue Types',
      description: `${count} defaults (API offline)`,
      icon: 'warning',
      collapsibleState: vscode.TreeItemCollapsibleState.Collapsed,
      sectionId: this.sectionId,
    });
  }

  getChildren(_ctx: SectionContext, _element?: StatusItem): StatusItem[] {
    const items: StatusItem[] = [];

    for (const type of this.state.types) {
      const iconName = GLYPH_TO_ICON[type.glyph] ?? 'file';
      const themeColorId = type.color ? COLOR_TO_THEME[type.color] : undefined;
      const modeLabel = type.mode === 'autonomous' ? 'autonomous' : 'paired';

      items.push(new StatusItem({
        label: type.key,
        description: `${type.name} · ${modeLabel}`,
        icon: iconName,
        tooltip: `${type.description}\nSource: ${type.source} · ${type.stepCount} steps`,
        sectionId: this.sectionId,
      }));

      // Apply color to the icon if available
      const item = items[items.length - 1]!;
      if (themeColorId) {
        item.iconPath = new vscode.ThemeIcon(iconName, new vscode.ThemeColor(themeColorId));
      }
    }

    if (this.state.apiAvailable) {
      items.push(new StatusItem({
        label: 'Manage Issue Types',
        icon: 'gear',
        command: {
          command: 'operator.openSettings',
          title: 'Open Settings',
        },
        sectionId: this.sectionId,
      }));
    }

    return items;
  }
}
