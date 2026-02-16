import React, { useState } from 'react';
import Box from '@mui/material/Box';
import TextField from '@mui/material/TextField';
import Button from '@mui/material/Button';
import Switch from '@mui/material/Switch';
import FormControlLabel from '@mui/material/FormControlLabel';
import Typography from '@mui/material/Typography';
import Alert from '@mui/material/Alert';
import Link from '@mui/material/Link';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CircularProgress from '@mui/material/CircularProgress';
import { SectionHeader } from '../SectionHeader';
import type { JiraValidationInfo, LinearValidationInfo } from '../../types/messages';

interface KanbanProvidersSectionProps {
  kanban: Record<string, unknown>;
  onUpdate: (section: string, key: string, value: unknown) => void;
  onValidateJira: (domain: string, email: string, apiToken: string) => void;
  onValidateLinear: (apiKey: string) => void;
  jiraResult: JiraValidationInfo | null;
  linearResult: LinearValidationInfo | null;
  validatingJira: boolean;
  validatingLinear: boolean;
}

/** Extract first entry from a domain-keyed map */
function firstEntry(map: Record<string, unknown>): [string, Record<string, unknown>] {
  const keys = Object.keys(map);
  if (keys.length === 0) {
    return ['', {}];
  }
  return [keys[0], (map[keys[0]] ?? {}) as Record<string, unknown>];
}

/** Extract first project from projects sub-map */
function firstProject(ws: Record<string, unknown>): [string, Record<string, unknown>] {
  const projects = (ws.projects ?? {}) as Record<string, unknown>;
  const keys = Object.keys(projects);
  if (keys.length === 0) {
    return ['', {}];
  }
  return [keys[0], (projects[keys[0]] ?? {}) as Record<string, unknown>];
}

export function KanbanProvidersSection({
  kanban,
  onUpdate,
  onValidateJira,
  onValidateLinear,
  jiraResult,
  linearResult,
  validatingJira,
  validatingLinear,
}: KanbanProvidersSectionProps) {
  const jiraMap = (kanban.jira ?? {}) as Record<string, unknown>;
  const linearMap = (kanban.linear ?? {}) as Record<string, unknown>;

  const [jiraDomain, jiraWs] = firstEntry(jiraMap);
  const [jiraProjectKey, jiraProject] = firstProject(jiraWs);
  const jiraEnabled = (jiraWs.enabled as boolean) ?? false;
  const jiraEmail = (jiraWs.email as string) ?? '';
  const jiraApiKeyEnv = (jiraWs.api_key_env as string) ?? 'OPERATOR_JIRA_API_KEY';

  const [linearTeamId, linearWs] = firstEntry(linearMap);
  const [, linearProject] = firstProject(linearWs);
  const linearEnabled = (linearWs.enabled as boolean) ?? false;
  const linearApiKeyEnv = (linearWs.api_key_env as string) ?? 'OPERATOR_LINEAR_API_KEY';

  const [jiraApiToken, setJiraApiToken] = useState('');
  const [linearApiKey, setLinearApiKey] = useState('');

  return (
    <Box sx={{ mb: 4 }}>
      <SectionHeader id="section-kanban" title="Kanban Providers" />
      <Typography color="text.secondary" gutterBottom>
        Configure kanban board integrations for ticket management. For more details see the <Link href="https://operator.untra.io/getting-started/kanban/">kanban documentation</Link>
      </Typography>

      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
        {/* Jira Cloud */}
        <Card variant="outlined">
          <CardContent>
            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
              <Typography variant="subtitle1" fontWeight={600}>
                Jira Cloud
              </Typography>
              <FormControlLabel
                control={
                  <Switch
                    checked={jiraEnabled}
                    onChange={(e) =>
                      onUpdate('kanban.jira', 'enabled', e.target.checked)
                    }
                    size="small"
                  />
                }
                label="Enabled"
              />
            </Box>

            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, opacity: jiraEnabled ? 1 : 0.5 }}>
              <TextField
                fullWidth
                size="small"
                label="Domain"
                InputLabelProps={{ margin: 'dense' }}
                value={jiraDomain}
                onChange={(e) => onUpdate('kanban.jira', 'domain', e.target.value)}
                placeholder="your-org.atlassian.net"
                disabled={!jiraEnabled}
                helperText="Your Jira Cloud instance domain (e.g. your-org.atlassian.net)"
              />

              <TextField
                fullWidth
                size="small"
                label="Email"
                InputLabelProps={{ margin: 'dense' }}
                value={jiraEmail}
                onChange={(e) => onUpdate('kanban.jira', 'email', e.target.value)}
                placeholder="you@example.com"
                disabled={!jiraEnabled}
                helperText="Email address associated with your Jira account"
              />

              <TextField
                fullWidth
                size="small"
                label="API Key Environment Variable"
                InputLabelProps={{ margin: 'dense' }}
                value={jiraApiKeyEnv}
                onChange={(e) => onUpdate('kanban.jira', 'api_key_env', e.target.value)}
                placeholder="OPERATOR_JIRA_API_KEY"
                disabled={!jiraEnabled}
                helperText="Name of the environment variable containing your Jira API token"
              />

              <TextField
                fullWidth
                size="small"
                label="Project Key"
                InputLabelProps={{ margin: 'dense' }}
                value={jiraProjectKey}
                onChange={(e) => onUpdate('kanban.jira', 'project_key', e.target.value)}
                placeholder="PROJ"
                disabled={!jiraEnabled}
                helperText="Jira project key to sync issues from (e.g. PROJ)"
              />

              <TextField
                fullWidth
                size="small"
                label="Sync Statuses"
                InputLabelProps={{ margin: 'dense' }}
                value={((jiraProject.sync_statuses as string[]) ?? []).join(', ')}
                onChange={(e) => {
                  const statuses = e.target.value
                    .split(',')
                    .map((s) => s.trim())
                    .filter(Boolean);
                  onUpdate('kanban.jira', 'sync_statuses', statuses);
                }}
                placeholder="To Do, In Progress"
                disabled={!jiraEnabled}
                helperText="Workflow statuses to sync (comma-separated, e.g., To Do, In Progress)"
              />

              <TextField
                fullWidth
                size="small"
                label="Collection Name"
                InputLabelProps={{ margin: 'dense' }}
                value={(jiraProject.collection_name as string) ?? ''}
                onChange={(e) => onUpdate('kanban.jira', 'collection_name', e.target.value)}
                placeholder="dev_kanban"
                disabled={!jiraEnabled}
                helperText="IssueType collection this project maps to"
              />

              <Box>
                <Typography variant="body2" color="text.secondary" sx={{ mb: 0.5 }}>
                  Validate Connection
                </Typography>
                <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                  <TextField
                    size="small"
                    type="password"
                    label="API Token"
                    InputLabelProps={{ margin: 'dense' }}
                    value={jiraApiToken}
                    onChange={(e) => setJiraApiToken(e.target.value)}
                    placeholder="Paste token to validate"
                    disabled={!jiraEnabled}
                    sx={{ flexGrow: 1 }}
                    helperText="Paste your Jira API token to test the connection"
                  />
                  <Button
                    variant="contained"
                    onClick={() => onValidateJira(jiraDomain, jiraEmail, jiraApiToken)}
                    disabled={!jiraEnabled || !jiraDomain || !jiraEmail || !jiraApiToken || validatingJira}
                    sx={{ minWidth: 'auto', px: 2, mt: -2 }}
                  >
                    {validatingJira ? <CircularProgress size={20} /> : 'Validate'}
                  </Button>
                </Box>
              </Box>

              {jiraResult && (
                <Alert severity={jiraResult.valid ? 'success' : 'error'} sx={{ mt: 1 }}>
                  {jiraResult.valid
                    ? `Authenticated as ${jiraResult.displayName} (${jiraResult.accountId})`
                    : jiraResult.error}
                </Alert>
              )}
            </Box>
          </CardContent>
        </Card>

        {/* Linear */}
        <Card variant="outlined">
          <CardContent>
            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
              <Typography variant="subtitle1" fontWeight={600}>
                Linear
              </Typography>
              <FormControlLabel
                control={
                  <Switch
                    checked={linearEnabled}
                    onChange={(e) =>
                      onUpdate('kanban.linear', 'enabled', e.target.checked)
                    }
                    size="small"
                  />
                }
                label="Enabled"
              />
            </Box>

            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, opacity: linearEnabled ? 1 : 0.5 }}>
              <TextField
                fullWidth
                size="small"
                label="Team ID"
                value={linearTeamId}
                onChange={(e) => onUpdate('kanban.linear', 'team_id', e.target.value)}
                placeholder="Team identifier"
                disabled={!linearEnabled}
                helperText="Linear team identifier to sync issues from"
              />

              <TextField
                fullWidth
                size="small"
                label="API Key Environment Variable"
                value={linearApiKeyEnv}
                onChange={(e) => onUpdate('kanban.linear', 'api_key_env', e.target.value)}
                placeholder="OPERATOR_LINEAR_API_KEY"
                disabled={!linearEnabled}
                helperText="Name of the environment variable containing your Linear API key"
              />

              <TextField
                fullWidth
                size="small"
                label="Sync Statuses"
                value={((linearProject.sync_statuses as string[]) ?? []).join(', ')}
                onChange={(e) => {
                  const statuses = e.target.value
                    .split(',')
                    .map((s) => s.trim())
                    .filter(Boolean);
                  onUpdate('kanban.linear', 'sync_statuses', statuses);
                }}
                placeholder="To Do, In Progress"
                disabled={!linearEnabled}
                helperText="Workflow statuses to sync (comma-separated, e.g., To Do, In Progress)"
              />

              <TextField
                fullWidth
                size="small"
                label="Collection Name"
                value={(linearProject.collection_name as string) ?? ''}
                onChange={(e) => onUpdate('kanban.linear', 'collection_name', e.target.value)}
                placeholder="dev_kanban"
                disabled={!linearEnabled}
                helperText="IssueType collection this project maps to"
              />

              <Box>
                <Typography variant="body2" color="text.secondary" sx={{ mb: 0.5 }}>
                  Validate Connection
                </Typography>
                <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                  <TextField
                    size="small"
                    type="password"
                    label="API Key"
                    value={linearApiKey}
                    onChange={(e) => setLinearApiKey(e.target.value)}
                    placeholder="lin_api_xxxxx"
                    disabled={!linearEnabled}
                    sx={{ flexGrow: 1 }}
                    helperText="Paste your Linear API key to test the connection"
                  />
                  <Button
                    variant="contained"
                    onClick={() => onValidateLinear(linearApiKey)}
                    disabled={!linearEnabled || !linearApiKey || validatingLinear}
                    sx={{ minWidth: 'auto', px: 2 , mt: -2 }}
                  >
                    {validatingLinear ? <CircularProgress size={20} /> : 'Validate'}
                  </Button>
                </Box>
              </Box>

              {linearResult && (
                <Alert severity={linearResult.valid ? 'success' : 'error'} sx={{ mt: 1 }}>
                  {linearResult.valid
                    ? `Authenticated as ${linearResult.userName} in ${linearResult.orgName}`
                    : linearResult.error}
                </Alert>
              )}
            </Box>
          </CardContent>
        </Card>
      </Box>
    </Box>
  );
}
