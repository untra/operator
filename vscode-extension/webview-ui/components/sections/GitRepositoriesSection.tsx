import React, { useCallback } from "react";
import { TextInput, SelectInput, Toggle } from "../primitives";
import { SectionHeader } from "../SectionHeader";
import { useDraftField } from "../../hooks/useDraftField";
import type { GitConfig } from "../../../src/generated/GitConfig";

interface GitRepositoriesSectionProps {
  git: GitConfig;
  onUpdate: (section: string, key: string, value: unknown) => void;
}

export function GitRepositoriesSection({ git, onUpdate }: GitRepositoriesSectionProps) {
  const provider = git.provider;
  const forge =
    provider && provider in git && provider !== "bitbucket" && provider !== "azuredevops"
      ? git[provider]
      : undefined;
  const branchFormat = git.branch_format;
  const useWorktrees = git.use_worktrees;
  const handleProviderChange = useCallback(
    (event: React.ChangeEvent<HTMLSelectElement>) =>
      onUpdate("git", "provider", event.target.value || null),
    [onUpdate],
  );
  const handleEnabledChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) =>
      onUpdate(`git.${provider}`, "enabled", event.target.checked),
    [onUpdate, provider],
  );
  const commitToken = useCallback(
    (next: string) => onUpdate(`git.${provider}`, "token_env", next),
    [onUpdate, provider],
  );
  const commitHost = useCallback(
    (next: string) => onUpdate(`git.${provider}`, "host", next || null),
    [onUpdate, provider],
  );
  const hostDraft = useDraftField(
    forge && "host" in forge && typeof forge.host === "string" ? forge.host : "",
    commitHost,
  );
  const commitBranchFormat = useCallback(
    (next: string) => onUpdate("git", "branch_format", next),
    [onUpdate],
  );
  const tokenDraft = useDraftField(forge?.token_env ?? "", commitToken);
  const branchFormatDraft = useDraftField(branchFormat, commitBranchFormat);
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

      <div className="op-col op-settings-fields">
        <SelectInput label="Git Provider" value={provider ?? ""} onChange={handleProviderChange}>
          <option value="">Auto-detect from remote</option>
          <option value="gitea">Gitea</option>
          <option value="forgejo">Forgejo</option>
          <option value="github">GitHub</option>
          <option value="gitlab">GitLab</option>
          {provider === "bitbucket" && (
            <option value="bitbucket" disabled>
              Bitbucket (detection only)
            </option>
          )}
          {provider === "azuredevops" && (
            <option value="azuredevops" disabled>
              Azure DevOps (detection only)
            </option>
          )}
        </SelectInput>

        {forge && (
          <>
            <Toggle
              checked={forge.enabled}
              onChange={handleEnabledChange}
              label={`Enable ${provider} integration`}
            />

            <TextInput
              label="Token environment variable"
              {...tokenDraft}
              helperText="Name of the environment variable containing the provider token"
              disabled={!forge.enabled}
            />

            {"host" in forge && <TextInput label="Host" {...hostDraft} disabled={!forge.enabled} />}
          </>
        )}
        <TextInput
          label="Branch Format"
          {...branchFormatDraft}
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
