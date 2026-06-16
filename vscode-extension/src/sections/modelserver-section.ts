import * as vscode from 'vscode';
import { StatusItem } from '../status-item';
import type { SectionContext, StatusSection } from './types';
import type { SectionId, SectionHealth } from '../generated';
import { discoverApiUrl } from '../api-client';
import type { ModelServerResponse } from '../generated/ModelServerResponse';
import type { ModelServersResponse } from '../generated/ModelServersResponse';
import type { ModelServerKindEntry } from '../generated/ModelServerKindEntry';
import type { ModelServerModelsResponse } from '../generated/ModelServerModelsResponse';

interface ModelServerState {
  apiAvailable: boolean;
  servers: ModelServerResponse[];
  kinds: ModelServerKindEntry[];
  /** Live model-list probe results, keyed by server name. */
  models: Record<string, ModelServerModelsResponse>;
}

/**
 * Resolve a row's icon: the brand ThemeIcon (`operator-{brand}`) when the kind
 * carries a `brand_icon` basename, else the semantic codicon fallback. Mirrors
 * the PROVIDER_ICONS pattern used by the git/kanban sections.
 */
function iconForKind(brand: string | null | undefined, fallback: string): string {
  return brand ? `operator-${brand}` : fallback;
}

export class ModelServerSection implements StatusSection {
  readonly sectionId: SectionId = 'model-servers';
  readonly prerequisites: SectionId[] = ['llm'];

  private state: ModelServerState = {
    apiAvailable: false,
    servers: [],
    kinds: [],
    models: {},
  };

  health(): SectionHealth {
    if (!this.state.apiAvailable) { return 'Yellow'; }
    return this.state.servers.some((s) => s.user_declared) ? 'Green' : 'Gray';
  }

  async check(ctx: SectionContext): Promise<void> {
    try {
      const apiUrl = await discoverApiUrl(ctx.ticketsDir);
      const response = await fetch(`${apiUrl}/api/v1/model-servers`);
      if (!response.ok) { throw new Error('servers fetch failed'); }
      const data = await response.json() as ModelServersResponse;

      // Catalog of supported kinds (single source of truth, served by REST).
      let kinds: ModelServerKindEntry[] = [];
      try {
        const kindsResp = await fetch(`${apiUrl}/api/v1/model-servers/kinds`);
        if (kindsResp.ok) { kinds = await kindsResp.json() as ModelServerKindEntry[]; }
      } catch { /* kinds are optional decoration */ }

      // Probe each server with a base_url for its model list (and reachability).
      // Builtins without a base_url are skipped — they'd be unreachable.
      const models: Record<string, ModelServerModelsResponse> = {};
      await Promise.all(
        data.servers
          .filter((s) => !!s.base_url)
          .map(async (s) => {
            try {
              const r = await fetch(
                `${apiUrl}/api/v1/model-servers/${encodeURIComponent(s.name)}/models`,
              );
              if (r.ok) { models[s.name] = await r.json() as ModelServerModelsResponse; }
            } catch { /* leave unprobed */ }
          }),
      );

      this.state = { apiAvailable: true, servers: data.servers, kinds, models };
      return;
    } catch {
      // API not available
    }
    this.state = { apiAvailable: false, servers: [], kinds: [], models: {} };
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

  getChildren(_ctx: SectionContext, element?: StatusItem): StatusItem[] {
    if (!this.state.apiAvailable) {
      return [];
    }

    // Server-level expansion: list the models that server offers (or its status).
    if (element && element.sectionId === this.sectionId && element.workspaceKey) {
      return this.getModelChildren(element.workspaceKey);
    }

    const items: StatusItem[] = [];

    for (const server of this.state.servers) {
      const label = server.display_name || server.name;
      const descriptionParts: string[] = [server.kind];
      if (server.base_url) { descriptionParts.push(server.base_url); }
      if (!server.user_declared) { descriptionParts.push('builtin'); }

      const probe = this.state.models[server.name];
      const hasModels = !!probe && probe.reachable && probe.models.length > 0;

      // Brand the server row from its kind (looked up in the shared catalog),
      // falling back to the declared/builtin codicon.
      const kindEntry = this.state.kinds.find((k) => k.slug === server.kind);
      const serverIcon = iconForKind(
        kindEntry?.brand_icon,
        server.user_declared ? 'server' : 'circle-outline',
      );

      items.push(new StatusItem({
        label,
        description: descriptionParts.join(' · '),
        icon: serverIcon,
        tooltip: this.buildTooltip(server, probe),
        collapsibleState: hasModels
          ? vscode.TreeItemCollapsibleState.Collapsed
          : vscode.TreeItemCollapsibleState.None,
        sectionId: this.sectionId,
        // Carry the server name so getChildren can resolve its models on expand.
        workspaceKey: server.name,
      }));
    }

    // Per-kind "Setup <kind>" rows from the shared kinds endpoint (non-builtin
    // kinds only — vendor builtins always exist). Each links to the kind's
    // credential/setup page so the user can obtain what they need. Rows are
    // grouped under a category header (the *Model Provider* vertical) so the
    // catalog reads the same way as the README/docs/web surfaces.
    let lastCategory: string | undefined;
    for (const kind of this.state.kinds.filter((k) => !k.is_builtin)) {
      if (kind.category !== lastCategory) {
        items.push(new StatusItem({
          label: kind.category_label,
          icon: 'list-tree',
          sectionId: this.sectionId,
        }));
        lastCategory = kind.category;
      }
      items.push(new StatusItem({
        label: `Setup ${kind.display_name}`,
        description: kind.description,
        icon: iconForKind(kind.brand_icon, kind.icon),
        tooltip: `${kind.description}\nSetup: ${kind.setup_url}`,
        command: {
          command: 'vscode.open',
          title: 'Open setup page',
          arguments: [vscode.Uri.parse(kind.setup_url)],
        },
        sectionId: this.sectionId,
      }));
    }

    // The create affordance: declare a new server by editing config. (Interactive
    // create/edit/delete forms are not yet wired; the REST API backs them.)
    items.push(new StatusItem({
      label: 'Add Model Server',
      icon: 'add',
      tooltip: 'Open settings to declare a [[model_servers]] entry',
      command: {
        command: 'operator.openSettings',
        title: 'Add Model Server',
      },
      sectionId: this.sectionId,
    }));

    return items;
  }

  /** Children shown when a server node is expanded: its models, or a status line. */
  private getModelChildren(serverName: string): StatusItem[] {
    const probe = this.state.models[serverName];
    if (!probe) {
      return [];
    }
    if (!probe.reachable) {
      return [new StatusItem({
        label: 'Unreachable',
        description: probe.error ?? '',
        icon: 'warning',
        sectionId: this.sectionId,
      })];
    }
    if (probe.models.length === 0) {
      return [new StatusItem({
        label: 'No models reported',
        icon: 'info',
        sectionId: this.sectionId,
      })];
    }
    return probe.models.map((m) => new StatusItem({
      label: m.display_name || m.id,
      description: m.display_name ? m.id : undefined,
      icon: 'symbol-enum',
      sectionId: this.sectionId,
    }));
  }

  private buildTooltip(s: ModelServerResponse, probe?: ModelServerModelsResponse): string {
    const lines = [`${s.name} (${s.kind})`];
    if (s.base_url) { lines.push(`URL: ${s.base_url}`); }
    if (s.api_key_env) { lines.push(`API key env: ${s.api_key_env}`); }
    if (!s.user_declared) { lines.push('Implicit builtin — cannot be deleted.'); }
    const extraKeys = Object.keys(s.extra_env);
    if (extraKeys.length > 0) {
      lines.push(`Extra env: ${extraKeys.join(', ')}`);
    }
    if (probe) {
      lines.push(probe.reachable
        ? `Reachable — ${probe.models.length} model(s)`
        : `Unreachable${probe.error ? `: ${probe.error}` : ''}`);
    }
    return lines.join('\n');
  }
}
