import * as vscode from 'vscode';
import { StatusItem } from '../status-item';
import type { SectionContext, StatusSection, KanbanState, KanbanProviderState } from './types';
import { getKanbanWorkspaces } from '../walkthrough';

export class KanbanSection implements StatusSection {
  readonly sectionId = 'kanban';

  private state: KanbanState = { configured: false, providers: [] };

  isConfigured(): boolean {
    return this.state.configured;
  }

  async check(ctx: SectionContext): Promise<void> {
    const config = await ctx.readConfigToml();
    const kanbanSection = config.kanban as Record<string, unknown> | undefined;
    const providers: KanbanProviderState[] = [];

    if (kanbanSection) {
      // Parse Jira providers from config.toml
      const jiraSection = kanbanSection.jira as Record<string, unknown> | undefined;
      if (jiraSection) {
        for (const [domain, wsConfig] of Object.entries(jiraSection)) {
          const ws = wsConfig as Record<string, unknown>;
          if (ws.enabled === false) { continue; }
          const projects: KanbanProviderState['projects'] = [];
          const projectsSection = ws.projects as Record<string, unknown> | undefined;
          if (projectsSection) {
            for (const [projectKey, projConfig] of Object.entries(projectsSection)) {
              const proj = projConfig as Record<string, unknown>;
              projects.push({
                key: projectKey,
                collectionName: (proj.collection_name as string) || 'dev_kanban',
                url: `https://${domain}/browse/${projectKey}`,
              });
            }
          }
          providers.push({
            provider: 'jira',
            key: domain,
            enabled: ws.enabled !== false,
            displayName: domain,
            url: `https://${domain}`,
            projects,
          });
        }
      }

      // Parse Linear providers from config.toml
      const linearSection = kanbanSection.linear as Record<string, unknown> | undefined;
      if (linearSection) {
        for (const [teamId, wsConfig] of Object.entries(linearSection)) {
          const ws = wsConfig as Record<string, unknown>;
          if (ws.enabled === false) { continue; }
          const projects: KanbanProviderState['projects'] = [];
          const projectsSection = ws.projects as Record<string, unknown> | undefined;
          if (projectsSection) {
            for (const [projectKey, projConfig] of Object.entries(projectsSection)) {
              const proj = projConfig as Record<string, unknown>;
              projects.push({
                key: projectKey,
                collectionName: (proj.collection_name as string) || 'dev_kanban',
                url: `https://linear.app/team/${projectKey}`,
              });
            }
          }
          providers.push({
            provider: 'linear',
            key: teamId,
            enabled: ws.enabled !== false,
            displayName: teamId,
            url: 'https://linear.app',
            projects,
          });
        }
      }
    }

    // Fall back to env-var-based detection if config.toml has no kanban section
    if (providers.length === 0) {
      const workspaces = await getKanbanWorkspaces();
      for (const ws of workspaces) {
        providers.push({
          provider: ws.provider,
          key: ws.name,
          enabled: ws.configured,
          displayName: ws.name,
          url: ws.url,
          projects: [],
        });
      }
    }

    this.state = {
      configured: providers.length > 0,
      providers,
    };
  }

  getTopLevelItem(_ctx: SectionContext): StatusItem {
    return new StatusItem({
      label: 'Kanban',
      description: this.state.configured
        ? this.getKanbanSummary()
        : 'No provider connected',
      icon: this.state.configured ? 'check' : 'warning',
      collapsibleState: this.state.configured
        ? vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.Expanded,
      sectionId: this.sectionId,
      command: this.state.configured ? undefined : {
        command: 'operator.startKanbanOnboarding',
        title: 'Configure Kanban',
      },
    });
  }

  getChildren(_ctx: SectionContext, element?: StatusItem): StatusItem[] {
    // Workspace-level expansion: show project children
    if (element && element.provider && element.workspaceKey && !element.projectKey) {
      return this.getKanbanProjectChildren(element.provider, element.workspaceKey);
    }

    // Top-level kanban children
    const items: StatusItem[] = [];

    if (this.state.configured) {
      for (const prov of this.state.providers) {
        const providerLabel = prov.provider === 'jira' ? 'Jira' : 'Linear';
        const providerIcon = prov.provider === 'jira' ? 'operator-atlassian' : 'operator-linear';
        items.push(new StatusItem({
          label: providerLabel,
          description: prov.displayName,
          icon: providerIcon,
          tooltip: prov.url,
          collapsibleState: vscode.TreeItemCollapsibleState.Collapsed,
          command: {
            command: 'vscode.open',
            title: 'Open in Browser',
            arguments: [vscode.Uri.parse(prov.url)],
          },
          contextValue: 'kanbanWorkspace',
          provider: prov.provider,
          workspaceKey: prov.key,
          sectionId: this.sectionId,
        }));
      }

      items.push(new StatusItem({
        label: 'Add Provider',
        icon: 'add',
        command: {
          command: 'operator.startKanbanOnboarding',
          title: 'Add Kanban Provider',
        },
        sectionId: this.sectionId,
      }));
    } else {
      items.push(new StatusItem({
        label: 'Configure Jira',
        icon: 'operator-atlassian',
        command: {
          command: 'operator.configureJira',
          title: 'Configure Jira',
        },
        sectionId: this.sectionId,
      }));
      items.push(new StatusItem({
        label: 'Configure Linear',
        icon: 'operator-linear',
        command: {
          command: 'operator.configureLinear',
          title: 'Configure Linear',
        },
        sectionId: this.sectionId,
      }));
    }

    return items;
  }

  private getKanbanProjectChildren(provider: string, workspaceKey: string): StatusItem[] {
    const items: StatusItem[] = [];
    const prov = this.state.providers.find(
      (p) => p.provider === provider && p.key === workspaceKey
    );
    if (!prov) { return items; }

    for (const proj of prov.projects) {
      items.push(new StatusItem({
        label: proj.key,
        description: proj.collectionName,
        icon: 'project',
        tooltip: proj.url,
        command: {
          command: 'vscode.open',
          title: 'Open in Browser',
          arguments: [vscode.Uri.parse(proj.url)],
        },
        contextValue: 'kanbanSyncConfig',
        provider: prov.provider,
        workspaceKey: prov.key,
        projectKey: proj.key,
        sectionId: this.sectionId,
      }));
    }

    const addLabel = provider === 'jira' ? 'Add Jira Project' : 'Add Linear Workspace';
    const addCommand = provider === 'jira' ? 'operator.addJiraProject' : 'operator.addLinearTeam';
    items.push(new StatusItem({
      label: addLabel,
      icon: 'add',
      command: {
        command: addCommand,
        title: addLabel,
        arguments: [workspaceKey],
      },
      sectionId: this.sectionId,
    }));

    return items;
  }

  private getKanbanSummary(): string {
    const prov = this.state.providers[0];
    if (!prov) {
      return '';
    }
    const provider = prov.provider === 'jira' ? 'Jira' : 'Linear';
    return `${provider}: ${prov.displayName}`;
  }
}
