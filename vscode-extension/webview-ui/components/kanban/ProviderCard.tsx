import React, { useCallback, useState } from "react";
import { Alert, Button, Card, CardContent, Chip, Spinner, TextInput, Toggle } from "../primitives";
import { ProjectRow } from "./ProjectRow";
import { useDraftField } from "../../hooks/useDraftField";
import type { JiraConfig } from "../../../src/generated/JiraConfig";
import type { LinearConfig } from "../../../src/generated/LinearConfig";
import type {
  JiraValidationInfo,
  LinearValidationInfo,
  IssueTypeSummary,
  CollectionResponse,
  ExternalIssueTypeSummary,
} from "../../types/messages";

interface ProviderCardProps {
  type: "jira" | "linear";
  domain: string;
  config: JiraConfig | LinearConfig;
  onUpdate: (section: string, key: string, value: unknown, instanceKey?: string) => void;
  onValidate: (...args: string[]) => void;
  validationResult: JiraValidationInfo | LinearValidationInfo | null;
  validating: boolean;
  collections: CollectionResponse[];
  issueTypes: IssueTypeSummary[];
  externalIssueTypes: Map<string, ExternalIssueTypeSummary[]>;
  onGetExternalIssueTypes: (provider: string, domain: string, projectKey: string) => void;
  kanbanStatuses: Map<string, string[]>;
  onGetKanbanStatuses: (provider: string, projectKey: string) => void;
  onViewIssueType: () => void;
}

export function ProviderCard({
  type,
  domain,
  config,
  onUpdate,
  onValidate,
  validationResult,
  validating,
  collections,
  issueTypes,
  externalIssueTypes,
  onGetExternalIssueTypes,
  kanbanStatuses,
  onGetKanbanStatuses,
  onViewIssueType,
}: ProviderCardProps) {
  const [apiToken, setApiToken] = useState("");
  const [showCredentials, setShowCredentials] = useState(false);
  const sectionKey = type === "jira" ? "kanban.jira" : "kanban.linear";
  const enabled = config.enabled;
  const projectEntries = Object.entries(config.projects ?? {});

  const isJira = type === "jira";
  const jiraConfig = isJira ? (config as JiraConfig) : null;
  const providerLabel = isJira ? "Jira Cloud" : "Linear";

  const isConnected = validationResult?.valid === true;
  const projectCount = projectEntries.length;
  const updateProvider = useCallback(
    (key: string, value: unknown) => onUpdate(sectionKey, key, value, domain),
    [domain, onUpdate, sectionKey],
  );
  const handleEnabledChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => updateProvider("enabled", event.target.checked),
    [updateProvider],
  );
  const showCredentialFields = useCallback(() => setShowCredentials(true), [setShowCredentials]);
  const hideCredentialFields = useCallback(() => setShowCredentials(false), [setShowCredentials]);
  const commitDomain = useCallback(
    (next: string) => updateProvider(isJira ? "domain" : "team_id", next),
    [isJira, updateProvider],
  );
  const commitEmail = useCallback(
    (next: string) => updateProvider("email", next),
    [updateProvider],
  );
  const commitApiKeyEnv = useCallback(
    (next: string) => updateProvider("api_key_env", next),
    [updateProvider],
  );
  const domainDraft = useDraftField(domain, commitDomain);
  const emailDraft = useDraftField(jiraConfig?.email ?? "", commitEmail);
  const apiKeyEnvDraft = useDraftField(config.api_key_env, commitApiKeyEnv);
  const handleApiTokenChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => setApiToken(event.target.value),
    [setApiToken],
  );
  const handleValidate = useCallback(() => {
    if (isJira) {
      onValidate(domain, jiraConfig?.email ?? "", apiToken);
    } else {
      onValidate(apiToken);
    }
  }, [apiToken, domain, isJira, jiraConfig?.email, onValidate]);
  const handleAddProject = useCallback(
    (key: string) => updateProvider(`projects.${key}.collection_name`, ""),
    [updateProvider],
  );

  return (
    <Card accent="terracotta">
      <CardContent>
        {/* Header */}
        <div className="op-row op-space-between op-mb-1">
          <div className="op-row op-gap-1">
            <i
              className={type === "jira" ? "opi-atlassian" : "opi-linear"}
              style={{ fontSize: "1.25rem", lineHeight: 1 }}
            />
            <span className="op-body1" style={{ fontWeight: 600 }}>
              {providerLabel}
            </span>
            <Chip
              label={isConnected ? "Connected" : "Not validated"}
              color={isConnected ? "success" : "default"}
              variant="outlined"
            />
            {projectCount > 0 && (
              <Chip
                label={`${projectCount} project${projectCount !== 1 ? "s" : ""}`}
                variant="outlined"
              />
            )}
          </div>
          <Toggle checked={enabled} onChange={handleEnabledChange} label="Enabled" />
        </div>

        <div style={{ opacity: enabled ? 1 : 0.5 }}>
          {/* Summary line */}
          {!showCredentials && (
            <div className="op-row op-gap-2 op-mb-1">
              <span className="op-body2 op-text-secondary">
                {isJira ? `${domain} · ${jiraConfig?.email || "no email"}` : domain}
              </span>
              <Button size="small" onClick={showCredentialFields} disabled={!enabled}>
                Edit Credentials
              </Button>
            </div>
          )}

          {/* Credentials (collapsible) */}
          {showCredentials && (
            <div className="op-col op-gap-2 op-mb-2">
              {isJira ? (
                <>
                  <TextInput
                    label="Domain"
                    {...domainDraft}
                    placeholder="your-org.atlassian.net"
                    disabled={!enabled}
                    helperText="Jira Cloud instance domain"
                  />
                  <TextInput
                    label="Email"
                    {...emailDraft}
                    placeholder="you@example.com"
                    disabled={!enabled}
                  />
                  <TextInput label="API Key Env Var" {...apiKeyEnvDraft} disabled={!enabled} />
                </>
              ) : (
                <>
                  <TextInput label="Team ID" {...domainDraft} disabled={!enabled} />
                  <TextInput label="API Key Env Var" {...apiKeyEnvDraft} disabled={!enabled} />
                </>
              )}

              <div className="op-row op-gap-1">
                <TextInput
                  type="password"
                  label={isJira ? "API Token" : "API Key"}
                  value={apiToken}
                  onChange={handleApiTokenChange}
                  placeholder={isJira ? "Paste token to validate" : "lin_api_xxxxx"}
                  disabled={!enabled}
                  style={{ flexGrow: 1 }}
                />
                <Button
                  variant="contained"
                  onClick={handleValidate}
                  disabled={!enabled || !apiToken || validating}
                  style={{ minWidth: "auto", paddingLeft: 16, paddingRight: 16 }}
                >
                  {validating ? <Spinner size={20} /> : "Validate"}
                </Button>
              </div>

              {validationResult && (
                <Alert severity={validationResult.valid ? "success" : "error"}>
                  {validationResult.valid
                    ? isJira
                      ? `Authenticated as ${(validationResult as JiraValidationInfo).displayName}`
                      : `Authenticated as ${(validationResult as LinearValidationInfo).userName} in ${(validationResult as LinearValidationInfo).orgName}`
                    : validationResult.error}
                </Alert>
              )}

              <Button size="small" onClick={hideCredentialFields}>
                Hide Credentials
              </Button>
            </div>
          )}

          {/* Project list */}
          <div className="op-mt-1">
            {projectEntries.length === 0 ? (
              <p className="op-body2 op-text-secondary" style={{ padding: "8px 0" }}>
                No projects configured. Add a project key above to start syncing.
              </p>
            ) : (
              projectEntries.map(([key, project]) => (
                <ProjectRow
                  key={key}
                  provider={type}
                  domain={domain}
                  projectKey={key}
                  project={project}
                  collections={collections}
                  issueTypes={issueTypes}
                  externalTypes={externalIssueTypes.get(`${type}/${key}`)}
                  statuses={kanbanStatuses.get(`${type}/${key}`)}
                  onUpdate={updateProvider}
                  onGetExternalIssueTypes={onGetExternalIssueTypes}
                  onGetKanbanStatuses={onGetKanbanStatuses}
                  onViewIssueType={onViewIssueType}
                />
              ))
            )}

            {/* Add project shortcut */}
            <div className="op-mt-1">
              <AddProjectInput disabled={!enabled} onAdd={handleAddProject} />
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function AddProjectInput({ disabled, onAdd }: { disabled: boolean; onAdd: (key: string) => void }) {
  const [value, setValue] = useState("");
  const handleValueChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => setValue(event.target.value.toUpperCase()),
    [setValue],
  );
  const handleAdd = useCallback(() => {
    onAdd(value.trim());
    setValue("");
  }, [onAdd, value]);
  return (
    <div className="op-row op-gap-1">
      <TextInput
        label="Add Project Key"
        value={value}
        onChange={handleValueChange}
        placeholder="PROJ"
        disabled={disabled}
        style={{ flex: 1 }}
      />
      <Button
        size="small"
        variant="outlined"
        disabled={disabled || !value.trim()}
        onClick={handleAdd}
      >
        Add
      </Button>
    </div>
  );
}
