/**
 * Status TreeDataProvider for Operator VS Code extension
 *
 * Slim orchestrator that delegates to per-section modules in ./sections/.
 * Each section owns its state, check logic, and tree item rendering.
 *
 * Sections use progressive disclosure — they only appear when prerequisites are met:
 *   Tier 0: Configuration (always visible)
 *   Tier 1: Connections (requires configReady)
 *   Tier 2: Kanban, LLM Tools, Git (requires connectionsReady)
 *   Tier 3: Issue Types (kanbanConfigured), Delegators (llmConfigured), Managed Projects (gitConfigured)
 */

import * as vscode from 'vscode';
import * as fs from 'fs/promises';
import { getResolvedConfigPath } from './config-paths';
import { StatusItem } from './status-item';
import type { SectionContext, StatusSection } from './sections/types';
import { ConfigSection } from './sections/config-section';
import { ConnectionsSection } from './sections/connections-section';
import { KanbanSection } from './sections/kanban-section';
import { LlmSection } from './sections/llm-section';
import { GitSection } from './sections/git-section';
import { IssueTypeSection } from './sections/issuetype-section';
import { DelegatorSection } from './sections/delegator-section';
import { ManagedProjectsSection } from './sections/managed-projects-section';

// Backward-compatible re-exports
export { StatusItem } from './status-item';
export type { StatusItemOptions } from './status-item';
export type { WebhookStatus, ApiStatus } from './sections/types';

// smol-toml is ESM-only, must use dynamic import
async function importSmolToml() {
  return await import('smol-toml');
}

/**
 * TreeDataProvider for hierarchical status information
 */
export class StatusTreeProvider implements vscode.TreeDataProvider<StatusItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<
    StatusItem | undefined
  >();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private context: vscode.ExtensionContext;
  private parsedConfig: Record<string, unknown> | null = null;
  private ticketsDir: string | undefined;
  private webhookServerRef?: { isRunning: () => boolean; getPort: () => number };

  // Named section references for progressive disclosure
  private configSection: ConfigSection;
  private connectionsSection: ConnectionsSection;
  private kanbanSection: KanbanSection;
  private llmSection: LlmSection;
  private gitSection: GitSection;
  private issueTypeSection: IssueTypeSection;
  private delegatorSection: DelegatorSection;
  private managedProjectsSection: ManagedProjectsSection;

  // All sections for check() and routing
  private allSections: StatusSection[];
  private sectionMap: Map<string, StatusSection>;
  private ctx: SectionContext;

  constructor(context: vscode.ExtensionContext) {
    this.context = context;
    this.configSection = new ConfigSection();
    this.connectionsSection = new ConnectionsSection();
    this.kanbanSection = new KanbanSection();
    this.llmSection = new LlmSection();
    this.gitSection = new GitSection();
    this.issueTypeSection = new IssueTypeSection();
    this.delegatorSection = new DelegatorSection();
    this.managedProjectsSection = new ManagedProjectsSection();

    this.allSections = [
      this.configSection,
      this.connectionsSection,
      this.kanbanSection,
      this.llmSection,
      this.gitSection,
      this.issueTypeSection,
      this.delegatorSection,
      this.managedProjectsSection,
    ];
    this.sectionMap = new Map(this.allSections.map(s => [s.sectionId, s]));
    this.ctx = this.buildContext();
  }

  setWebhookServer(server: { isRunning: () => boolean; getPort: () => number }): void {
    this.webhookServerRef = server;
  }

  async setTicketsDir(dir: string | undefined): Promise<void> {
    this.ticketsDir = dir;
    await this.refresh();
  }

  async refresh(): Promise<void> {
    this.parsedConfig = null;
    const ctx = this.buildContext();

    // All sections run check() regardless of visibility
    await Promise.allSettled(this.allSections.map(s => s.check(ctx)));

    // Set readiness flags after checks complete
    ctx.configReady = this.configSection.isReady();
    ctx.connectionsReady = this.connectionsSection.isConfigured();
    ctx.kanbanConfigured = this.kanbanSection.isConfigured();
    ctx.llmConfigured = this.llmSection.isConfigured();
    ctx.gitConfigured = this.gitSection.isConfigured();
    this.ctx = ctx;

    this._onDidChangeTreeData.fire(undefined);
  }

  /**
   * Read and cache config.toml
   */
  private async readConfigToml(): Promise<Record<string, unknown>> {
    if (this.parsedConfig) {
      return this.parsedConfig;
    }

    const configPath = getResolvedConfigPath();
    if (!configPath) {
      this.parsedConfig = {};
      return this.parsedConfig;
    }

    try {
      const raw = await fs.readFile(configPath, 'utf-8');
      if (raw.trim()) {
        const { parse } = await importSmolToml();
        this.parsedConfig = parse(raw) as Record<string, unknown>;
      } else {
        this.parsedConfig = {};
      }
    } catch {
      this.parsedConfig = {};
    }

    return this.parsedConfig;
  }

  private buildContext(): SectionContext {
    return {
      extensionContext: this.context,
      ticketsDir: this.ticketsDir,
      readConfigToml: () => this.readConfigToml(),
      configReady: false,
      connectionsReady: false,
      kanbanConfigured: false,
      llmConfigured: false,
      gitConfigured: false,
      webhookServer: this.webhookServerRef,
    };
  }

  /**
   * Build the list of sections visible based on current readiness flags.
   */
  private getVisibleSections(): StatusSection[] {
    const visible: StatusSection[] = [this.configSection];

    // Tier 1: requires config ready
    if (!this.ctx.configReady) { return visible; }
    visible.push(this.connectionsSection);

    // Tier 2: requires connections ready (API or webhook)
    if (!this.ctx.connectionsReady) { return visible; }
    visible.push(this.kanbanSection, this.llmSection, this.gitSection);

    // Tier 3: each requires its parent tier-2 section configured
    if (this.ctx.kanbanConfigured) { visible.push(this.issueTypeSection); }
    if (this.ctx.llmConfigured) { visible.push(this.delegatorSection); }
    if (this.ctx.gitConfigured) { visible.push(this.managedProjectsSection); }

    return visible;
  }

  getTreeItem(element: StatusItem): vscode.TreeItem {
    return element;
  }

  getChildren(element?: StatusItem): StatusItem[] {
    if (!element) {
      return this.getVisibleSections().map(s => s.getTopLevelItem(this.ctx));
    }

    // Route to section by sectionId
    const section = element.sectionId ? this.sectionMap.get(element.sectionId) : undefined;
    if (section) {
      return section.getChildren(this.ctx, element);
    }

    return [];
  }
}
