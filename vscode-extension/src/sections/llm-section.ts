import * as vscode from 'vscode';
import { StatusItem } from '../status-item';
import type { SectionContext, StatusSection, LlmState, LlmToolInfo } from './types';
import { detectInstalledLlmTools } from '../walkthrough';
import { discoverApiUrl } from '../api-client';
import type { DetectedTool } from '../generated/DetectedTool';

export class LlmSection implements StatusSection {
  readonly sectionId = 'llm';

  private state: LlmState = { detected: false, tools: [], configDetected: [], toolDetails: [] };

  isConfigured(): boolean {
    return this.state.detected;
  }

  async check(ctx: SectionContext): Promise<void> {
    const toolDetails: LlmToolInfo[] = [];
    const seen = new Set<string>();

    // Priority 1: Try API (has model_aliases from embedded tool configs)
    try {
      const apiUrl = await discoverApiUrl(ctx.ticketsDir);
      const response = await fetch(`${apiUrl}/api/v1/llm-tools`);
      if (response.ok) {
        const data = await response.json() as { tools: DetectedTool[] };
        for (const tool of data.tools) {
          seen.add(tool.name);
          toolDetails.push({
            name: tool.name,
            version: tool.version,
            models: tool.model_aliases,
          });
        }
      }
    } catch {
      // API not available
    }

    // Priority 2: Config TOML llm_tools.detected (may have model_aliases)
    if (toolDetails.length === 0) {
      const config = await ctx.readConfigToml();
      const llmTools = config.llm_tools as Record<string, unknown> | undefined;
      const detectedArray = Array.isArray(llmTools?.detected) ? llmTools.detected as Array<Record<string, unknown>> : [];
      for (const entry of detectedArray) {
        if (typeof entry === 'object' && entry !== null && typeof entry.name === 'string') {
          const name = entry.name;
          if (seen.has(name)) { continue; }
          seen.add(name);
          const models = Array.isArray(entry.model_aliases) ? entry.model_aliases as string[] : [];
          const version = typeof entry.version === 'string' ? entry.version : undefined;
          toolDetails.push({ name, version, models });
        }
      }
    }

    // Priority 3: PATH detection (no model info — tools won't be expandable)
    const tools = await detectInstalledLlmTools();
    for (const tool of tools) {
      if (!seen.has(tool.name)) {
        seen.add(tool.name);
        toolDetails.push({
          name: tool.name,
          version: tool.version !== 'unknown' ? tool.version : undefined,
          models: [],
        });
      }
    }

    // Build legacy configDetected for backward compat
    const config = await ctx.readConfigToml();
    const llmTools = config.llm_tools as Record<string, unknown> | undefined;
    const configDetected = Array.isArray(llmTools?.detected)
      ? (llmTools.detected as Array<string | { name: string; version?: string }>).map(
          (entry) => {
            if (typeof entry === 'string') {
              return { name: entry };
            }
            return { name: entry.name, version: entry.version };
          }
        )
      : [];

    this.state = {
      detected: toolDetails.length > 0,
      tools,
      configDetected,
      toolDetails,
    };
  }

  getTopLevelItem(_ctx: SectionContext): StatusItem {
    return new StatusItem({
      label: 'LLM Tools',
      description: this.state.detected
        ? this.getLlmSummary()
        : 'No tools detected',
      icon: this.state.detected ? 'check' : 'warning',
      collapsibleState: this.state.detected
        ? vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.Expanded,
      sectionId: this.sectionId,
      command: this.state.detected ? undefined : {
        command: 'operator.detectLlmTools',
        title: 'Detect LLM Tools',
      },
    });
  }

  getChildren(_ctx: SectionContext, element?: StatusItem): StatusItem[] {
    // Expanding a tool item: show model aliases
    if (element?.contextValue?.startsWith('llmTool:')) {
      const toolName = element.contextValue.slice('llmTool:'.length);
      return this.getModelChildren(toolName);
    }

    const items: StatusItem[] = [];

    if (this.state.detected) {
      for (const tool of this.state.toolDetails) {
        const hasModels = tool.models.length > 0;
        items.push(new StatusItem({
          label: tool.name,
          description: tool.version,
          icon: `operator-${tool.name}`,
          collapsibleState: hasModels
            ? vscode.TreeItemCollapsibleState.Collapsed
            : vscode.TreeItemCollapsibleState.None,
          contextValue: `llmTool:${tool.name}`,
          sectionId: this.sectionId,
        }));
      }

      items.push(new StatusItem({
        label: 'Detect Tools',
        icon: 'search',
        command: {
          command: 'operator.detectLlmTools',
          title: 'Detect LLM Tools',
        },
        sectionId: this.sectionId,
      }));
    } else {
      items.push(new StatusItem({
        label: 'Detect Tools',
        icon: 'search',
        command: {
          command: 'operator.detectLlmTools',
          title: 'Detect LLM Tools',
        },
        sectionId: this.sectionId,
      }));
      items.push(new StatusItem({
        label: 'Install Claude Code',
        icon: 'link-external',
        command: {
          command: 'vscode.open',
          title: 'Install Claude Code',
          arguments: [vscode.Uri.parse('https://docs.anthropic.com/en/docs/claude-code')],
        },
        sectionId: this.sectionId,
      }));
    }

    return items;
  }

  private getModelChildren(toolName: string): StatusItem[] {
    const tool = this.state.toolDetails.find(t => t.name === toolName);
    if (!tool) { return []; }

    return tool.models.map(model => new StatusItem({
      label: model,
      icon: 'symbol-field',
      tooltip: `Create delegator for ${toolName}:${model}`,
      command: {
        command: 'operator.openCreateDelegator',
        title: 'Create Delegator',
        arguments: [toolName, model],
      },
      sectionId: this.sectionId,
    }));
  }

  private getLlmSummary(): string {
    const count = this.state.toolDetails.length;
    if (count === 0) { return ''; }
    const first = this.state.toolDetails[0]!;
    const label = first.version ? `${first.name} v${first.version}` : first.name;
    return count > 1 ? `${label} +${count - 1}` : label;
  }
}
