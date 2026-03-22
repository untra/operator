import * as vscode from 'vscode';
import * as path from 'path';
import { StatusItem } from '../status-item';
import type { SectionContext, StatusSection, ConfigState } from './types';
import {
  resolveWorkingDirectory,
  configFileExists,
  getResolvedConfigPath,
} from '../config-paths';

export class ConfigSection implements StatusSection {
  readonly sectionId = 'config';

  private state: ConfigState = {
    workingDirSet: false,
    workingDir: '',
    configExists: false,
    configPath: '',
  };

  isReady(): boolean {
    return this.state.workingDirSet && this.state.configExists;
  }

  async check(ctx: SectionContext): Promise<void> {
    const workingDir = ctx.extensionContext.globalState.get<string>('operator.workingDirectory')
      || resolveWorkingDirectory();
    const workingDirSet = !!workingDir;
    const configExists = await configFileExists();
    const configPath = getResolvedConfigPath();

    this.state = {
      workingDirSet,
      workingDir: workingDir || '',
      configExists,
      configPath: configPath || '',
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

    return items;
  }
}
