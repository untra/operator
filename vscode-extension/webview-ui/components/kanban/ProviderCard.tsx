import React, { useState } from 'react';
import { Alert, Button, Card, CardContent, Chip, Spinner, TextInput, Toggle } from '../primitives';
import { ProjectRow } from './ProjectRow';
import type { JiraConfig } from '../../../src/generated/JiraConfig';
import type { LinearConfig } from '../../../src/generated/LinearConfig';
import type { ProjectSyncConfig } from '../../../src/generated/ProjectSyncConfig';
import type {
  JiraValidationInfo,
  LinearValidationInfo,
  IssueTypeSummary,
  CollectionResponse,
  ExternalIssueTypeSummary,
} from '../../types/messages';

interface ProviderCardProps {
  type: 'jira' | 'linear';
  domain: string;
  config: JiraConfig | LinearConfig;
  onUpdate: (section: string, key: string, value: unknown) => void;
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
  const [apiToken, setApiToken] = useState('');
  const [showCredentials, setShowCredentials] = useState(false);
  const sectionKey = type === 'jira' ? 'kanban.jira' : 'kanban.linear';
  const enabled = config.enabled;
  const projectEntries = Object.entries(config.projects ?? {});

  const isJira = type === 'jira';
  const jiraConfig = isJira ? (config as JiraConfig) : null;
  const providerLabel = isJira ? 'Jira Cloud' : 'Linear';

  const isConnected = validationResult?.valid === true;
  const projectCount = projectEntries.length;

  return (
    <Card accent="terracotta">
      <CardContent>
        {/* Header */}
        <div className="op-row op-space-between op-mb-1">
          <div className="op-row op-gap-1">
            <i
              className={type === 'jira' ? 'opi-atlassian' : 'opi-linear'}
              style={{ fontSize: '1.25rem', lineHeight: 1 }}
            />
            <span className="op-body1" style={{ fontWeight: 600 }}>
              {providerLabel}
            </span>
            <Chip
              label={isConnected ? 'Connected' : 'Not validated'}
              color={isConnected ? 'success' : 'default'}
              variant="outlined"
            />
            {projectCount > 0 && (
              <Chip label={`${projectCount} project${projectCount !== 1 ? 's' : ''}`} variant="outlined" />
            )}
          </div>
          <Toggle
            checked={enabled}
            onChange={(e) => onUpdate(sectionKey, 'enabled', e.target.checked)}
            label="Enabled"
          />
        </div>

        <div style={{ opacity: enabled ? 1 : 0.5 }}>
          {/* Summary line */}
          {!showCredentials && (
            <div className="op-row op-gap-2 op-mb-1">
              <span className="op-body2 op-text-secondary">
                {isJira ? `${domain} · ${jiraConfig?.email || 'no email'}` : domain}
              </span>
              <Button size="small" onClick={() => setShowCredentials(true)} disabled={!enabled}>
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
                    value={domain}
                    onChange={(e) => onUpdate(sectionKey, 'domain', e.target.value)}
                    placeholder="your-org.atlassian.net"
                    disabled={!enabled}
                    helperText="Jira Cloud instance domain"
                  />
                  <TextInput
                    label="Email"
                    value={jiraConfig?.email ?? ''}
                    onChange={(e) => onUpdate(sectionKey, 'email', e.target.value)}
                    placeholder="you@example.com"
                    disabled={!enabled}
                  />
                  <TextInput
                    label="API Key Env Var"
                    value={config.api_key_env}
                    onChange={(e) => onUpdate(sectionKey, 'api_key_env', e.target.value)}
                    disabled={!enabled}
                  />
                </>
              ) : (
                <>
                  <TextInput
                    label="Team ID"
                    value={domain}
                    onChange={(e) => onUpdate(sectionKey, 'team_id', e.target.value)}
                    disabled={!enabled}
                  />
                  <TextInput
                    label="API Key Env Var"
                    value={config.api_key_env}
                    onChange={(e) => onUpdate(sectionKey, 'api_key_env', e.target.value)}
                    disabled={!enabled}
                  />
                </>
              )}

              <div className="op-row op-gap-1">
                <TextInput
                  type="password"
                  label={isJira ? 'API Token' : 'API Key'}
                  value={apiToken}
                  onChange={(e) => setApiToken(e.target.value)}
                  placeholder={isJira ? 'Paste token to validate' : 'lin_api_xxxxx'}
                  disabled={!enabled}
                  style={{ flexGrow: 1 }}
                />
                <Button
                  variant="contained"
                  onClick={() => {
                    if (isJira) {
                      onValidate(domain, jiraConfig?.email ?? '', apiToken);
                    } else {
                      onValidate(apiToken);
                    }
                  }}
                  disabled={!enabled || !apiToken || validating}
                  style={{ minWidth: 'auto', paddingLeft: 16, paddingRight: 16 }}
                >
                  {validating ? <Spinner size={20} /> : 'Validate'}
                </Button>
              </div>

              {validationResult && (
                <Alert severity={validationResult.valid ? 'success' : 'error'}>
                  {validationResult.valid
                    ? isJira
                      ? `Authenticated as ${(validationResult as JiraValidationInfo).displayName}`
                      : `Authenticated as ${(validationResult as LinearValidationInfo).userName} in ${(validationResult as LinearValidationInfo).orgName}`
                    : validationResult.error}
                </Alert>
              )}

              <Button size="small" onClick={() => setShowCredentials(false)}>
                Hide Credentials
              </Button>
            </div>
          )}

          {/* Project list */}
          <div className="op-mt-1">
            {projectEntries.length === 0 ? (
              <p className="op-body2 op-text-secondary" style={{ padding: '8px 0' }}>
                No projects configured. Add a project key above to start syncing.
              </p>
            ) : (
              projectEntries.map(([key, project]) => (
                <ProjectRow
                  key={key}
                  provider={type}
                  domain={domain}
                  projectKey={key}
                  project={project as ProjectSyncConfig}
                  collections={collections}
                  issueTypes={issueTypes}
                  externalTypes={externalIssueTypes.get(`${type}/${key}`)}
                  statuses={kanbanStatuses.get(`${type}/${key}`)}
                  onUpdate={onUpdate}
                  onGetExternalIssueTypes={onGetExternalIssueTypes}
                  onGetKanbanStatuses={onGetKanbanStatuses}
                  onViewIssueType={onViewIssueType}
                  sectionKey={sectionKey}
                />
              ))
            )}

            {/* Add project shortcut */}
            <div className="op-mt-1">
              <AddProjectInput
                disabled={!enabled}
                onAdd={(key) => {
                  onUpdate(sectionKey, `projects.${key}.collection_name`, '');
                }}
              />
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

function AddProjectInput({ disabled, onAdd }: { disabled: boolean; onAdd: (key: string) => void }) {
  const [value, setValue] = useState('');
  return (
    <div className="op-row op-gap-1">
      <TextInput
        label="Add Project Key"
        value={value}
        onChange={(e) => setValue(e.target.value.toUpperCase())}
        placeholder="PROJ"
        disabled={disabled}
        style={{ flex: 1 }}
      />
      <Button
        size="small"
        variant="outlined"
        disabled={disabled || !value.trim()}
        onClick={() => {
          onAdd(value.trim());
          setValue('');
        }}
      >
        Add
      </Button>
    </div>
  );
}
