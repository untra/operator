import * as vscode from 'vscode';
import * as path from 'path';
import { StatusItem } from '../status-item';
import type { SectionContext, StatusSection, ConfigState } from './types';
import type { SectionId, SectionHealth } from '../generated';
import {
  resolveWorkingDirectory,
  configFileExists,
  getResolvedConfigPath,
} from '../config-paths';
import { getOperatorPath, getOperatorVersion } from '../operator-binary';

export class ConfigSection implements StatusSection {
  readonly sectionId: SectionId = 'config';
  readonly prerequisites: SectionId[] = [];

  private state: ConfigState = {
    workingDirSet: false,
    workingDir: '',
    configExists: false,
    configPath: '',
    wrapperType: 'vscode',
    editorVar: process.env.EDITOR || 'vim',
    visualVar: process.env.VISUAL || 'code --wait',
  };

  isReady(): boolean {
    return this.state.workingDirSet && this.state.configExists;
  }

  health(): SectionHealth {
    if (!this.state.configExists) { return 'Red'; }
    if (!this.state.workingDirSet) { return 'Yellow'; }
    return 'Green';
  }

  async check(ctx: SectionContext): Promise<void> {
    const workingDir = ctx.extensionContext.globalState.get<string>('operator.workingDirectory')
      || resolveWorkingDirectory();
    const workingDirSet = !!workingDir;
    const configExists = await configFileExists();
    const configPath = getResolvedConfigPath();

    // Read wrapper type from config
    let wrapperType = 'vscode';
    let operatorVersion: string | undefined;
    try {
      const config = await ctx.readConfigToml();
      const sessions = config.sessions as Record<string, unknown> | undefined;
      if (sessions?.wrapper && typeof sessions.wrapper === 'string') {
        wrapperType = sessions.wrapper;
      }
    } catch {
      // Default to vscode
    }

    // Try to get operator version from binary
    try {
      const operatorPath = await getOperatorPath(ctx.extensionContext);
      if (operatorPath) {
        operatorVersion = await getOperatorVersion(operatorPath) || undefined;
      }
    } catch {
      // Version unknown
    }

    this.state = {
      workingDirSet,
      workingDir: workingDir || '',
      configExists,
      configPath: configPath || '',
      wrapperType,
      operatorVersion,
      editorVar: process.env.EDITOR || 'vim',
      visualVar: process.env.VISUAL || 'code --wait',
    };
  }

  getTopLevelItem(_ctx: SectionContext): StatusItem {
    const configuredBoth = this.state.workingDirSet && this.state.configExists;

    const configCommand = !configuredBoth
      ? this.state.workingDirSet
        ? { command: 'operator.runSetup', title: 'Run Operator Setup' }
        : { command: 'operator.selectWorkingDirectory', title: 'Select Working Directory' }
      : undefined;

    return new StatusItem({
      label: 'Configuration',
      description: configuredBoth
        ? path.basename(this.state.workingDir)
        : 'Setup required',
      icon: configuredBoth ? 'check' : 'debug-configure',
      collapsibleState: configuredBoth
        ? vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.Expanded,
      sectionId: this.sectionId,
      command: configCommand,
    });
  }

  getChildren(ctx: SectionContext, _element?: StatusItem): StatusItem[] {
    const items: StatusItem[] = [];

    if (this.state.workingDirSet) {
      items.push(new StatusItem({
        label: 'Working Directory',
        description: this.state.workingDir,
        icon: 'folder-opened',
        contextValue: 'workingDirConfigured',
        sectionId: this.sectionId,
      }));
    } else {
      items.push(new StatusItem({
        label: 'Working Directory',
        description: 'Not set',
        icon: 'folder',
        command: {
          command: 'operator.selectWorkingDirectory',
          title: 'Select Working Directory',
        },
        sectionId: this.sectionId,
      }));
    }

    items.push(new StatusItem({
      label: 'Config File',
      description: this.state.configExists
        ? this.state.configPath
        : 'Not found',
      icon: this.state.configExists ? 'file' : 'file-add',
      command: {
        command: 'operator.openSettings',
        title: 'Open Settings',
      },
      sectionId: this.sectionId,
    }));

    if (ctx.ticketsDir) {
      items.push(new StatusItem({
        label: 'Tickets',
        description: ctx.ticketsDir,
        icon: 'markdown',
        command: {
          command: 'operator.revealTicketsDir',
          title: 'Reveal in Explorer',
        },
        sectionId: this.sectionId,
      }));
    } else {
      items.push(new StatusItem({
        label: 'Tickets',
        description: 'Not found',
        icon: 'markdown',
        tooltip: 'No .tickets directory found',
        sectionId: this.sectionId,
      }));
    }

    // Session wrapper (readonly)
    items.push(new StatusItem({
      label: 'Wrapper',
      description: this.state.wrapperType,
      icon: 'terminal',
      sectionId: this.sectionId,
    }));

    // Editor environment variables
    items.push(new StatusItem({
      label: '$EDITOR',
      description: this.state.editorVar || 'Not set',
      icon: this.state.editorVar ? 'check' : 'warning',
      sectionId: this.sectionId,
    }));

    items.push(new StatusItem({
      label: '$VISUAL',
      description: this.state.visualVar || 'Not set',
      icon: this.state.visualVar ? 'check' : 'warning',
      sectionId: this.sectionId,
    }));

    // Operator version with update nudge
    const versionDesc = this.state.updateAvailable
      ? `${this.state.operatorVersion ?? 'Unknown'} → ${this.state.updateAvailable} available`
      : this.state.operatorVersion ?? 'Unknown';
    items.push(new StatusItem({
      label: 'Version',
      description: versionDesc,
      icon: this.state.updateAvailable ? 'warning' : 'versions',
      command: {
        command: 'vscode.open',
        title: 'Open Downloads',
        arguments: [vscode.Uri.parse('https://operator.untra.io/downloads/')],
      },
      sectionId: this.sectionId,
    }));

    return items;
  }
}
