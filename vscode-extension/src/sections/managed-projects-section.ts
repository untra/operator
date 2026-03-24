import * as vscode from 'vscode';
import { StatusItem } from '../status-item';
import type { SectionContext, StatusSection } from './types';
import { discoverApiUrl } from '../api-client';
import type { ProjectSummary } from '../generated/ProjectSummary';

interface ManagedProjectsState {
  configured: boolean;
  projects: ProjectSummary[];
}

export class ManagedProjectsSection implements StatusSection {
  readonly sectionId = 'projects';

  private state: ManagedProjectsState = { configured: false, projects: [] };

  async check(ctx: SectionContext): Promise<void> {
    try {
      const apiUrl = await discoverApiUrl(ctx.ticketsDir);
      const response = await fetch(`${apiUrl}/api/v1/projects`);
      if (response.ok) {
        const projects = await response.json() as ProjectSummary[];
        this.state = { configured: true, projects };
        return;
      }
    } catch {
      // API not available
    }
    this.state = { configured: false, projects: [] };
  }

  getTopLevelItem(_ctx: SectionContext): StatusItem {
    if (this.state.configured) {
      const count = this.state.projects.length;
      return new StatusItem({
        label: 'Managed Projects',
        description: `${count} project${count !== 1 ? 's' : ''}`,
        icon: 'project',
        collapsibleState: count > 0
          ? vscode.TreeItemCollapsibleState.Collapsed
          : vscode.TreeItemCollapsibleState.None,
        sectionId: this.sectionId,
      });
    }

    return new StatusItem({
      label: 'Managed Projects',
      description: 'API required',
      icon: 'project',
      collapsibleState: vscode.TreeItemCollapsibleState.None,
      sectionId: this.sectionId,
    });
  }

  getChildren(_ctx: SectionContext, _element?: StatusItem): StatusItem[] {
    if (!this.state.configured) {
      return [];
    }

    return this.state.projects.map((proj) => {
      const details: string[] = [];
      if (proj.kind) { details.push(proj.kind); }
      if (proj.languages.length > 0) { details.push(proj.languages.join(', ')); }

      return new StatusItem({
        label: proj.project_name,
        description: details.join(' · ') || undefined,
        icon: proj.exists ? 'folder' : 'folder-library',
        tooltip: proj.project_path,
        command: proj.exists ? {
          command: 'vscode.openFolder',
          title: 'Open Project',
          arguments: [vscode.Uri.file(proj.project_path), { forceNewWindow: false }],
        } : undefined,
        sectionId: this.sectionId,
      });
    });
  }
}
