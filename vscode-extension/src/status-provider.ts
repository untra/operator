/**
 * Status TreeDataProvider for Operator VS Code extension
 *
 * Slim orchestrator that delegates to per-section modules in ./sections/.
 * Each section owns its state, check logic, and tree item rendering.
 *
 * Sections use progressive disclosure - they only appear when every prerequisite
 * section is visible and not Red, mirroring the Rust TUI's section registry:
 *   Configuration, Workflows, Remote Targets, License: no prerequisites
 *   Connections <- config; Kanban, LLM Tools, Git <- connections
 *   Model Servers, Delegators <- llm; Issue Types <- kanban; Managed Projects <- git
 */

import * as vscode from "vscode";
import * as fs from "node:fs/promises";
import { currentConnection } from "./connection";
import { getResolvedConfigPath } from "./config-paths";
import type { StatusItem } from "./status-item";
import type { SectionContext, StatusSection } from "./sections/types";
import { ConfigSection } from "./sections/config-section";
import { ConnectionsSection } from "./sections/connections-section";
import { KanbanSection } from "./sections/kanban-section";
import { LlmSection } from "./sections/llm-section";
import { GitSection } from "./sections/git-section";
import { IssueTypeSection } from "./sections/issuetype-section";
import { DelegatorSection } from "./sections/delegator-section";
import { ModelServerSection } from "./sections/modelserver-section";
import { ManagedProjectsSection } from "./sections/managed-projects-section";
import { RemoteTargetsSection } from "./sections/remote-targets-section";
import { LicenseSection } from "./sections/license-section";
import { WorkflowsSection } from "./sections/workflows-section";

// Backward-compatible re-exports
export { StatusItem } from "./status-item";
export type { StatusItemOptions } from "./status-item";
export type { WebhookStatus, ApiStatus } from "./sections/types";

// smol-toml is ESM-only, must use dynamic import
async function importSmolToml() {
  return import("smol-toml");
}

async function readConfigToml(configPath: string): Promise<Record<string, unknown>> {
  if (!configPath) {
    return {};
  }
  try {
    const raw = await fs.readFile(configPath, "utf-8");
    if (!raw.trim()) {
      return {};
    }
    const { parse } = await importSmolToml();
    return parse(raw);
  } catch {
    return {};
  }
}

/**
 * TreeDataProvider for hierarchical status information
 */
export class StatusTreeProvider implements vscode.TreeDataProvider<StatusItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<StatusItem | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  private context: vscode.ExtensionContext;
  private ticketsDir: string | undefined;
  private webhookServerRef?: { isRunning: () => boolean; getPort: () => number };

  private sections = this.createSections();
  private revision = 0;
  private ctx: SectionContext;

  constructor(context: vscode.ExtensionContext) {
    this.context = context;
    this.ctx = this.buildContext();
  }

  private createSections() {
    const config = new ConfigSection();
    const connections = new ConnectionsSection();
    const kanban = new KanbanSection();
    const llm = new LlmSection();
    const git = new GitSection();
    const all: StatusSection[] = [
      config,
      connections,
      kanban,
      llm,
      new ModelServerSection(),
      git,
      new IssueTypeSection(),
      new DelegatorSection(),
      new ManagedProjectsSection(),
      new WorkflowsSection(),
      new RemoteTargetsSection(),
      new LicenseSection(),
    ];
    return {
      config,
      connections,
      kanban,
      llm,
      git,
      all,
      byId: new Map<string, StatusSection>(all.map((section) => [section.sectionId, section])),
    };
  }

  redraw(): void {
    this._onDidChangeTreeData.fire(undefined);
  }

  clearConnection(): void {
    this.revision++;
    this.ticketsDir = undefined;
    this.sections = this.createSections();
    this.ctx = this.buildContext();
    this._onDidChangeTreeData.fire(undefined);
  }

  setWebhookServer(server: { isRunning: () => boolean; getPort: () => number }): void {
    this.webhookServerRef = server;
  }

  async setTicketsDir(dir: string | undefined): Promise<void> {
    this.ticketsDir = dir;
    await this.refresh();
  }

  async refresh(): Promise<void> {
    const ctx = this.buildContext();

    const revision = ++this.revision;
    const sections = this.createSections();
    await Promise.allSettled(sections.all.map((section) => section.check(ctx)));
    if (revision !== this.revision) {
      return;
    }
    ctx.configReady = sections.config.isReady();
    ctx.connectionsReady = sections.connections.isConfigured();
    ctx.kanbanConfigured = sections.kanban.isConfigured();
    ctx.llmConfigured = sections.llm.isConfigured();
    ctx.gitConfigured = sections.git.isConfigured();
    this.sections = sections;
    this.ctx = ctx;

    this._onDidChangeTreeData.fire(undefined);
  }

  private buildContext(): SectionContext {
    const configPath = getResolvedConfigPath();
    let config: Promise<Record<string, unknown>> | undefined;
    return {
      extensionContext: this.context,
      ticketsDir: this.ticketsDir,
      readConfigToml: () => (config ??= readConfigToml(configPath)),
      configReady: false,
      connectionsReady: false,
      kanbanConfigured: false,
      llmConfigured: false,
      gitConfigured: false,
      webhookServer: this.webhookServerRef,
    };
  }

  /**
   * Build the list of sections visible based on prerequisite health.
   *
   * A section is visible when all its prerequisite sections report Green health.
   * This replaces the hardcoded tier system with a declarative, data-driven approach
   * that matches the Rust TUI's `StatusSection` trait prerequisites.
   */
  private getVisibleSections(): StatusSection[] {
    const healthCache = new Map<string, string>();

    const getSectionHealth = (sectionId: string): string => {
      if (healthCache.has(sectionId)) {
        return healthCache.get(sectionId)!;
      }
      const section = this.sections.byId.get(sectionId);
      if (!section) {
        return "Red";
      }
      const h = section.health();
      healthCache.set(sectionId, h);
      return h;
    };

    const prerequisitesMet = (section: StatusSection): boolean => {
      return section.prerequisites.every((prereqId) => {
        // Prerequisite must itself be visible (transitive) and not Red
        const prereqSection = this.sections.byId.get(prereqId);
        if (prereqId === "connections" && !(this.ctx.configReady && this.ctx.connectionsReady)) {
          return false;
        }
        if (!prereqSection) {
          return false;
        }
        return prerequisitesMet(prereqSection) && getSectionHealth(prereqId) !== "Red";
      });
    };

    return this.sections.all.filter((s) => prerequisitesMet(s));
  }

  getTreeItem(element: StatusItem): vscode.TreeItem {
    return element;
  }

  getChildren(element?: StatusItem): StatusItem[] {
    if (!element) {
      if (currentConnection().selection.kind === "undecided") {
        return this.sections.connections.getChildren(this.ctx);
      }
      return this.getVisibleSections().map((s) => s.getTopLevelItem(this.ctx));
    }

    // Route to section by sectionId
    const section = element.sectionId ? this.sections.byId.get(element.sectionId) : undefined;
    if (section) {
      return section.getChildren(this.ctx, element);
    }

    return [];
  }
}
