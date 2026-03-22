import * as vscode from 'vscode';
import { StatusItem } from '../status-item';
import type { SectionContext, StatusSection, GitState } from './types';

/** Map provider names to branded ThemeIcon IDs */
const PROVIDER_ICONS: Record<string, string> = {
  github: 'operator-github',
  gitlab: 'operator-gitlab',
  bitbucket: 'repo',
  azuredevops: 'azure-devops',
};

export class GitSection implements StatusSection {
  readonly sectionId = 'git';

  private state: GitState = { configured: false };

  isConfigured(): boolean {
    return this.state.configured;
  }

  async check(ctx: SectionContext): Promise<void> {
    const config = await ctx.readConfigToml();
    const gitSection = config.git as Record<string, unknown> | undefined;

    if (!gitSection) {
      this.state = { configured: false };
      return;
    }

    const provider = gitSection.provider as string | undefined;
    const github = gitSection.github as Record<string, unknown> | undefined;
    const gitlab = gitSection.gitlab as Record<string, unknown> | undefined;
    const githubEnabled = github?.enabled as boolean | undefined;
    const gitlabEnabled = gitlab?.enabled as boolean | undefined;
    const branchFormat = gitSection.branch_format as string | undefined;
    const useWorktrees = gitSection.use_worktrees as boolean | undefined;

    // Determine token status based on active provider
    let tokenSet = false;
    if (provider === 'gitlab' || gitlabEnabled) {
      const tokenEnv = (gitlab?.token_env as string) || 'GITLAB_TOKEN';
      tokenSet = !!process.env[tokenEnv];
    } else {
      const tokenEnv = (github?.token_env as string) || 'GITHUB_TOKEN';
      tokenSet = !!process.env[tokenEnv];
    }

    const configured = !!(provider || githubEnabled || gitlabEnabled);

    this.state = {
      configured,
      provider,
      githubEnabled,
      tokenSet,
      branchFormat,
      useWorktrees,
    };
  }

  getTopLevelItem(_ctx: SectionContext): StatusItem {
    const providerLabel = this.state.provider
      ? this.state.provider.charAt(0).toUpperCase() + this.state.provider.slice(1)
      : 'GitHub';

    return new StatusItem({
      label: 'Git',
      description: this.state.configured ? providerLabel : 'Not configured',
      icon: this.state.configured ? 'check' : 'warning',
      collapsibleState: this.state.configured
        ? vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.Expanded,
      sectionId: this.sectionId,
      command: this.state.configured ? undefined : {
        command: 'operator.startGitOnboarding',
        title: 'Connect Git Provider',
      },
    });
  }

  getChildren(_ctx: SectionContext, _element?: StatusItem): StatusItem[] {
    const items: StatusItem[] = [];

    if (this.state.configured) {
      // Provider with branded icon
      const providerName = this.state.provider || 'github';
      const providerIcon = PROVIDER_ICONS[providerName] || 'source-control';
      const providerLabel = providerName.charAt(0).toUpperCase() + providerName.slice(1);
      items.push(new StatusItem({
        label: 'Provider',
        description: providerLabel,
        icon: providerIcon,
        sectionId: this.sectionId,
      }));

      // Token status — clickable when not set
      const tokenLabel = providerName === 'gitlab' ? 'GitLab Token' : 'GitHub Token';
      items.push(new StatusItem({
        label: tokenLabel,
        description: this.state.tokenSet ? 'Set' : 'Not set',
        icon: this.state.tokenSet ? 'key' : 'warning',
        sectionId: this.sectionId,
        command: this.state.tokenSet ? undefined : {
          command: providerName === 'gitlab' ? 'operator.configureGitLab' : 'operator.configureGitHub',
          title: 'Set Token',
        },
      }));

      // Branch Format
      if (this.state.branchFormat) {
        items.push(new StatusItem({
          label: 'Branch Format',
          description: this.state.branchFormat,
          icon: 'git-branch',
          sectionId: this.sectionId,
        }));
      }

      // Worktrees
      items.push(new StatusItem({
        label: 'Worktrees',
        description: this.state.useWorktrees ? 'Enabled' : 'Disabled',
        icon: 'git-merge',
        sectionId: this.sectionId,
      }));
    } else {
      // Unconfigured: show provider options
      items.push(new StatusItem({
        label: 'GitHub',
        icon: 'operator-github',
        command: {
          command: 'operator.configureGitHub',
          title: 'Connect GitHub',
        },
        sectionId: this.sectionId,
      }));
      items.push(new StatusItem({
        label: 'GitLab',
        icon: 'operator-gitlab',
        command: {
          command: 'operator.configureGitLab',
          title: 'Connect GitLab',
        },
        sectionId: this.sectionId,
      }));
    }

    return items;
  }
}
