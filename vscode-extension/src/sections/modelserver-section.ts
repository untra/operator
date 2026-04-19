import * as vscode from 'vscode';
import { StatusItem } from '../status-item';
import type { SectionContext, StatusSection } from './types';
import type { SectionId, SectionHealth } from '../generated';
import { discoverApiUrl } from '../api-client';
import type { ModelServerResponse } from '../generated/ModelServerResponse';
import type { ModelServersResponse } from '../generated/ModelServersResponse';

interface ModelServerState {
  apiAvailable: boolean;
  servers: ModelServerResponse[];
}

export class ModelServerSection implements StatusSection {
  readonly sectionId: SectionId = 'model-servers';
  readonly prerequisites: SectionId[] = ['llm'];

  private state: ModelServerState = { apiAvailable: false, servers: [] };

  health(): SectionHealth {
    if (!this.state.apiAvailable) { return 'Yellow'; }
    return this.state.servers.some((s) => s.user_declared) ? 'Green' : 'Gray';
  }

  async check(ctx: SectionContext): Promise<void> {
    try {
      const apiUrl = await discoverApiUrl(ctx.ticketsDir);
      const response = await fetch(`${apiUrl}/api/v1/model-servers`);
      if (response.ok) {
        const data = await response.json() as ModelServersResponse;
        this.state = { apiAvailable: true, servers: data.servers };
        return;
      }
    } catch {
      // API not available
    }
    this.state = { apiAvailable: false, servers: [] };
  }

  getTopLevelItem(_ctx: SectionContext): StatusItem {
    if (this.state.apiAvailable) {
      const declared = this.state.servers.filter((s) => s.user_declared).length;
      const description = declared > 0
        ? `${declared} declared`
        : 'builtins only';
      return new StatusItem({
        label: 'Model Servers',
        description,
        icon: 'server',
        collapsibleState: this.state.servers.length > 0
          ? vscode.TreeItemCollapsibleState.Collapsed
          : vscode.TreeItemCollapsibleState.None,
        sectionId: this.sectionId,
        health: this.health(),
      });
    }

    return new StatusItem({
      label: 'Model Servers',
      description: 'API required',
      icon: 'server',
      collapsibleState: vscode.TreeItemCollapsibleState.None,
      sectionId: this.sectionId,
      health: this.health(),
    });
  }

  getChildren(_ctx: SectionContext, _element?: StatusItem): StatusItem[] {
    const items: StatusItem[] = [];

    if (!this.state.apiAvailable) {
      return items;
    }

    for (const server of this.state.servers) {
      const label = server.display_name || server.name;
      const descriptionParts: string[] = [server.kind];
      if (server.base_url) { descriptionParts.push(server.base_url); }
      if (!server.user_declared) { descriptionParts.push('builtin'); }

      items.push(new StatusItem({
        label,
        description: descriptionParts.join(' · '),
        icon: server.user_declared ? 'server' : 'circle-outline',
        tooltip: this.buildTooltip(server),
        sectionId: this.sectionId,
      }));
    }

    items.push(new StatusItem({
      label: 'Add Model Server',
      icon: 'add',
      command: {
        command: 'operator.openSettings',
        title: 'Add Model Server',
      },
      sectionId: this.sectionId,
    }));

    return items;
  }

  private buildTooltip(s: ModelServerResponse): string {
    const lines = [`${s.name} (${s.kind})`];
    if (s.base_url) { lines.push(`URL: ${s.base_url}`); }
    if (s.api_key_env) { lines.push(`API key env: ${s.api_key_env}`); }
    if (!s.user_declared) { lines.push('Implicit builtin — cannot be deleted.'); }
    const extraKeys = Object.keys(s.extra_env);
    if (extraKeys.length > 0) {
      lines.push(`Extra env: ${extraKeys.join(', ')}`);
    }
    return lines.join('\n');
  }
}
