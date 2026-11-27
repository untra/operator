import { onMessage, postMessage } from "../../vscodeApi";
import React, { useCallback, useState, useEffect } from "react";
import { Alert, Button, Card, CardContent, Chip, Spinner, TextInput, Toggle } from "../primitives";
import { ProjectRow } from "./ProjectRow";
import { useDraftField } from "../../hooks/useDraftField";
import { addProject } from "../../state/addProject";
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
  instances: Record<string, JiraConfig | LinearConfig>;
  apiReachable: boolean;
  type: "jira" | "linear";
  domain: string;
  config: JiraConfig | LinearConfig;
  onUpdate: (section: string, key: string, value: unknown, instanceKey?: string) => void;
  collections: CollectionResponse[];
  issueTypes: IssueTypeSummary[];
  externalIssueTypes: Map<string, ExternalIssueTypeSummary[]>;
  onGetExternalIssueTypes: (provider: string, domain: string, projectKey: string) => void;
  kanbanStatuses: Map<string, string[]>;
  onGetKanbanStatuses: (provider: string, domain: string, projectKey: string) => void;
  onViewIssueType: () => void;
}

export function ProviderCard({
  instances,
  apiReachable,
  type,
  domain,
  config,
  onUpdate,
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
  const [validation, setValidation] = useState<{
    id: string;
    domain: string;
    email?: string;
    token: string;
    result?: JiraValidationInfo | LinearValidationInfo;
  }>();
  const validationMatches =
    enabled &&
    validation?.domain === domain &&
    validation.email === jiraConfig?.email &&
    validation.token === apiToken;
  const validationResult = validationMatches ? validation?.result : undefined;
  const validating = validationMatches && !validationResult;
  useEffect(
    () =>
      onMessage((message) => {
        if (message.type === "jiraValidationResult" || message.type === "linearValidationResult") {
          setValidation((current) =>
            current?.id === message.requestId ? { ...current, result: message.result } : current,
          );
        }
      }),
    [],
  );
  const handleValidate = useCallback(() => {
    const requestId = crypto.randomUUID();
    setValidation({ id: requestId, domain, email: jiraConfig?.email, token: apiToken });
    if (isJira) {
      postMessage({
        type: "validateJira",
        requestId,
        domain,
        email: jiraConfig?.email ?? "",
        apiToken,
      });
    } else {
      postMessage({ type: "validateLinear", requestId, apiKey: apiToken });
    }
  }, [domain, jiraConfig?.email, apiToken, isJira]);
  const handleAddProject = useCallback(
    (key: string) => {
      const projects = addProject(config.projects, key);
      if (projects !== config.projects) {
        updateProvider("projects", projects);
      }
    },
    [config.projects, updateProvider],
  );

  return (
    <Card accent="terracotta">
      <CardContent>
        {/* Header */}
        <div className="op-row op-space-between op-mb-1">
          <div className="op-row op-gap-1">
            <i
              className={
                type === "jira" ? "opi-atlassian op-provider-icon" : "opi-linear op-provider-icon"
              }
            />
            <span className="op-body1 op-semibold">{providerLabel}</span>
            <Chip
              label={validationResult?.valid ? "Connected" : "Not validated"}
              color={validationResult?.valid ? "success" : "default"}
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

        <div className="op-provider-fields" data-enabled={enabled}>
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
                  className="op-grow"
                />
                <Button
                  variant="contained"
                  onClick={handleValidate}
                  disabled={
                    !apiReachable ||
                    !enabled ||
                    !apiToken ||
                    (isJira && !jiraConfig?.email) ||
                    validating
                  }
                  className="op-validate"
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
              <p className="op-body2 op-text-secondary op-project-empty">
                No projects configured. Add a project key above to start syncing.
              </p>
            ) : (
              projectEntries.map(([key, project]) => (
                <ProjectRow
                  lookupEnabled={
                    enabled &&
                    apiReachable &&
                    Object.values(instances).filter(
                      (entry) => entry.enabled && Object.hasOwn(entry.projects ?? {}, key),
                    ).length === 1
                  }
                  key={key}
                  provider={type}
                  domain={domain}
                  projectKey={key}
                  project={project}
                  collections={collections}
                  issueTypes={issueTypes}
                  externalTypes={externalIssueTypes.get(JSON.stringify([type, domain, key]))}
                  statuses={kanbanStatuses.get(JSON.stringify([type, domain, key]))}
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
        className="op-flex-1"
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
