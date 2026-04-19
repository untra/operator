import * as vscode from 'vscode';
import type { SectionHealth } from './generated';

/**
 * Map a SectionHealth value to a VS Code theme color id used to tint the
 * row icon. Yellow/Red nudge the user to address the item; Green and Gray
 * leave the icon at its default theme color.
 */
const HEALTH_THEME_COLOR: Partial<Record<SectionHealth, string>> = {
  Yellow: 'list.warningForeground',
  Red: 'list.errorForeground',
};

/**
 * StatusItem options
 */
export interface StatusItemOptions {
  label: string;
  description?: string;
  icon: string;
  tooltip?: string;
  collapsibleState?: vscode.TreeItemCollapsibleState;
  command?: vscode.Command;
  sectionId?: string;
  contextValue?: string;    // for view/item/context when clause
  provider?: string;        // 'jira' | 'linear'
  workspaceKey?: string;    // domain or teamId (config key)
  projectKey?: string;      // project/team sync config key
  /** X button (Shift+Enter) — special/tertiary action */
  specialCommand?: vscode.Command;
  /** Y button (Ctrl+Enter) — contextual refresh */
  refreshCommand?: vscode.Command;
  /**
   * Optional health state. When `Yellow` or `Red`, the row icon is tinted
   * with the corresponding semantic theme color so the user is nudged to
   * address the item. Mirrors the Rust TUI `kanban_section.rs` header
   * colorization driven by `SectionHealth::to_color()`.
   */
  health?: SectionHealth;
}

/**
 * TreeItem for status display
 */
export class StatusItem extends vscode.TreeItem {
  public readonly sectionId?: string;
  public readonly provider?: string;
  public readonly workspaceKey?: string;
  public readonly projectKey?: string;
  /** X button (Shift+Enter) — special/tertiary action */
  public readonly specialCommand?: vscode.Command;
  /** Y button (Ctrl+Enter) — contextual refresh */
  public readonly refreshCommand?: vscode.Command;

  constructor(opts: StatusItemOptions) {
    super(
      opts.label,
      opts.collapsibleState ?? vscode.TreeItemCollapsibleState.None
    );
    this.sectionId = opts.sectionId;
    this.provider = opts.provider;
    this.workspaceKey = opts.workspaceKey;
    this.projectKey = opts.projectKey;
    this.specialCommand = opts.specialCommand;
    this.refreshCommand = opts.refreshCommand;

    // Build description with action indicator titles
    let desc = opts.description ?? '';
    const indicators: string[] = [];
    if (opts.specialCommand) {
      indicators.push(opts.specialCommand.title || '*');
    }
    if (opts.refreshCommand) {
      indicators.push(opts.refreshCommand.title || '\u27F3');
    }
    if (indicators.length > 0) {
      desc = desc ? `${desc} ${indicators.join(' ')}` : indicators.join(' ');
    }

    if (desc) {
      this.description = desc;
    }

    // Build rich tooltip with action hints
    const tooltipLines: string[] = [];
    const baseTooltip = opts.tooltip || (opts.description
      ? `${opts.label}: ${opts.description}`
      : opts.label);
    tooltipLines.push(baseTooltip);
    if (opts.command) {
      tooltipLines.push(`Enter: ${opts.command.title}`);
    }
    if (opts.specialCommand?.tooltip) {
      tooltipLines.push(`Shift+Enter: ${opts.specialCommand.tooltip}`);
    }
    if (opts.refreshCommand?.tooltip) {
      tooltipLines.push(`Ctrl+Enter: ${opts.refreshCommand.tooltip}`);
    }
    this.tooltip = tooltipLines.join('\n');

    const themeColorId = opts.health ? HEALTH_THEME_COLOR[opts.health] : undefined;
    this.iconPath = themeColorId
      ? new vscode.ThemeIcon(opts.icon, new vscode.ThemeColor(themeColorId))
      : new vscode.ThemeIcon(opts.icon);
    if (opts.command) {
      this.command = opts.command;
    }
    if (opts.contextValue) {
      this.contextValue = opts.contextValue;
    }
  }
}
