import React, { useCallback } from "react";
import { TextInput, SelectInput, Toggle } from "../primitives";
import { SectionHeader } from "../SectionHeader";
import type { GitConfig } from "../../../src/generated/GitConfig";

interface GitRepositoriesSectionProps {
  git: GitConfig;
  onUpdate: (section: string, key: string, value: unknown) => void;
}

export function GitRepositoriesSection({ git, onUpdate }: GitRepositoriesSectionProps) {
  const provider = git.provider;
  const githubEnabled = git.github.enabled;
  const githubTokenEnv = git.github.token_env;
  const branchFormat = git.branch_format;
  const useWorktrees = git.use_worktrees;
  const handleProviderChange = useCallback(
    (event: React.ChangeEvent<HTMLSelectElement>) =>
      onUpdate("git", "provider", event.target.value),
    [onUpdate],
  );
  const handleGithubEnabledChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) =>
      onUpdate("git.github", "enabled", event.target.checked),
    [onUpdate],
  );
  const handleGithubTokenChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) =>
      onUpdate("git.github", "token_env", event.target.value),
    [onUpdate],
  );
  const handleBranchFormatChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) =>
      onUpdate("git", "branch_format", event.target.value),
    [onUpdate],
  );
  const handleWorktreesChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) =>
      onUpdate("git", "use_worktrees", event.target.checked),
    [onUpdate],
  );

  return (
    <div className="op-mb-4">
      <SectionHeader id="section-git" title="Git Repositories" />
      <p className="op-body1 op-text-secondary op-mb-1">
        Configure workspace git provider and branch settings. For more details see the{" "}
        <a href="https://operator.untra.io/getting-started/git/">git documentation</a>
      </p>

      <div className="op-col" style={{ gap: 20 }}>
        <SelectInput
          label="Git Provider"
          value={provider || "github"}
          onChange={handleProviderChange}
        >
          <option value="github">GitHub</option>
          <option value="gitlab">GitLab</option>
          <option value="bitbucket">Bitbucket</option>
          <option value="azuredevops">Azure DevOps</option>
        </SelectInput>

        <Toggle
          checked={githubEnabled}
          onChange={handleGithubEnabledChange}
          label="GitHub integration enabled"
        />

        <TextInput
          label="GitHub Token Environment Variable"
          value={githubTokenEnv}
          onChange={handleGithubTokenChange}
          placeholder="GITHUB_TOKEN"
          helperText="Name of the environment variable containing your GitHub personal access token"
          disabled={!githubEnabled}
        />

        <TextInput
          label="Branch Format"
          value={branchFormat}
          onChange={handleBranchFormatChange}
          placeholder="{type}/{ticket_id}-{slug}"
          helperText="Template for branch names. Variables: {type}, {ticket_id}, {slug}"
        />

        <Toggle
          checked={useWorktrees}
          onChange={handleWorktreesChange}
          label="Use git worktrees for parallel agent branches"
        />
      </div>
    </div>
  );
}
