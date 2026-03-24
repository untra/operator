import * as vscode from 'vscode';

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
}

/**
 * TreeItem for status display
 */
export class StatusItem extends vscode.TreeItem {
  public readonly sectionId?: string;
  public readonly provider?: string;
  public readonly workspaceKey?: string;
  public readonly projectKey?: string;

  constructor(opts: StatusItemOptions) {
    super(
      opts.label,
      opts.collapsibleState ?? vscode.TreeItemCollapsibleState.None
    );
    this.sectionId = opts.sectionId;
    this.provider = opts.provider;
    this.workspaceKey = opts.workspaceKey;
    this.projectKey = opts.projectKey;
    if (opts.description !== undefined) {
      this.description = opts.description;
    }
    this.tooltip = opts.tooltip || (opts.description
      ? `${opts.label}: ${opts.description}`
      : opts.label);
    this.iconPath = new vscode.ThemeIcon(opts.icon);
    if (opts.command) {
      this.command = opts.command;
    }
    if (opts.contextValue) {
      this.contextValue = opts.contextValue;
    }
  }
}
