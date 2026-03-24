import React, { useState } from 'react';
import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Typography from '@mui/material/Typography';
import Switch from '@mui/material/Switch';
import FormControlLabel from '@mui/material/FormControlLabel';
import TextField from '@mui/material/TextField';
import Button from '@mui/material/Button';
import Chip from '@mui/material/Chip';
import Alert from '@mui/material/Alert';
import CircularProgress from '@mui/material/CircularProgress';
import Collapse from '@mui/material/Collapse';
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
  IssueTypeResponse,
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
  selectedIssueType: IssueTypeResponse | null;
  onGetExternalIssueTypes: (provider: string, domain: string, projectKey: string) => void;
  onViewIssueType: (key: string) => void;
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
  selectedIssueType,
  onGetExternalIssueTypes,
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
    <Card variant="outlined">
      <CardContent>
        {/* Header */}
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1 }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <i
              className={type === 'jira' ? 'opi-atlassian' : 'opi-linear'}
              style={{ fontSize: '1.25rem', lineHeight: 1 }}
            />
            <Typography variant="subtitle1" fontWeight={600}>
              {providerLabel}
            </Typography>
            <Chip
              label={isConnected ? 'Connected' : 'Not validated'}
              size="small"
              color={isConnected ? 'success' : 'default'}
              variant="outlined"
            />
            {projectCount > 0 && (
              <Chip label={`${projectCount} project${projectCount !== 1 ? 's' : ''}`} size="small" variant="outlined" />
            )}
          </Box>
          <FormControlLabel
            control={
              <Switch
                checked={enabled}
                onChange={(e) => onUpdate(sectionKey, 'enabled', e.target.checked)}
                size="small"
              />
            }
            label="Enabled"
          />
        </Box>

        <Box sx={{ opacity: enabled ? 1 : 0.5 }}>
          {/* Summary line */}
          {!showCredentials && (
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 1 }}>
              <Typography variant="body2" color="text.secondary">
                {isJira ? `${domain} · ${jiraConfig?.email || 'no email'}` : domain}
              </Typography>
              <Button size="small" onClick={() => setShowCredentials(true)} disabled={!enabled}>
                Edit Credentials
              </Button>
            </Box>
          )}

          {/* Credentials (collapsible) */}
          <Collapse in={showCredentials}>
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, mb: 2 }}>
              {isJira ? (
                <>
                  <TextField
                    fullWidth
                    size="small"
                    label="Domain"
                    value={domain}
                    onChange={(e) => onUpdate(sectionKey, 'domain', e.target.value)}
                    placeholder="your-org.atlassian.net"
                    disabled={!enabled}
                    helperText="Jira Cloud instance domain"
                  />
                  <TextField
                    fullWidth
                    size="small"
                    label="Email"
                    value={jiraConfig?.email ?? ''}
                    onChange={(e) => onUpdate(sectionKey, 'email', e.target.value)}
                    placeholder="you@example.com"
                    disabled={!enabled}
                  />
                  <TextField
                    fullWidth
                    size="small"
                    label="API Key Env Var"
                    value={config.api_key_env}
                    onChange={(e) => onUpdate(sectionKey, 'api_key_env', e.target.value)}
                    disabled={!enabled}
                  />
                </>
              ) : (
                <>
                  <TextField
                    fullWidth
                    size="small"
                    label="Team ID"
                    value={domain}
                    onChange={(e) => onUpdate(sectionKey, 'team_id', e.target.value)}
                    disabled={!enabled}
                  />
                  <TextField
                    fullWidth
                    size="small"
                    label="API Key Env Var"
                    value={config.api_key_env}
                    onChange={(e) => onUpdate(sectionKey, 'api_key_env', e.target.value)}
                    disabled={!enabled}
                  />
                </>
              )}

              <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                <TextField
                  size="small"
                  type="password"
                  label={isJira ? 'API Token' : 'API Key'}
                  value={apiToken}
                  onChange={(e) => setApiToken(e.target.value)}
                  placeholder={isJira ? 'Paste token to validate' : 'lin_api_xxxxx'}
                  disabled={!enabled}
                  sx={{ flexGrow: 1 }}
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
                  sx={{ minWidth: 'auto', px: 2 }}
                >
                  {validating ? <CircularProgress size={20} /> : 'Validate'}
                </Button>
              </Box>

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
            </Box>
          </Collapse>

          {/* Project list */}
          <Box sx={{ mt: 1 }}>
            {projectEntries.length === 0 ? (
              <Typography variant="body2" color="text.secondary" sx={{ py: 1 }}>
                No projects configured. Add a project key above to start syncing.
              </Typography>
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
                  selectedIssueType={selectedIssueType}
                  onUpdate={onUpdate}
                  onGetExternalIssueTypes={onGetExternalIssueTypes}
                  onViewIssueType={onViewIssueType}
                  sectionKey={sectionKey}
                />
              ))
            )}

            {/* Add project shortcut */}
            <Box sx={{ mt: 1 }}>
              <AddProjectInput
                disabled={!enabled}
                onAdd={(key) => {
                  onUpdate(sectionKey, `projects.${key}.collection_name`, '');
                }}
              />
            </Box>
          </Box>
        </Box>
      </CardContent>
    </Card>
  );
}

function AddProjectInput({ disabled, onAdd }: { disabled: boolean; onAdd: (key: string) => void }) {
  const [value, setValue] = useState('');
  return (
    <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
      <TextField
        size="small"
        label="Add Project Key"
        value={value}
        onChange={(e) => setValue(e.target.value.toUpperCase())}
        placeholder="PROJ"
        disabled={disabled}
        sx={{ flex: 1 }}
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
    </Box>
  );
}
