import React from 'react';
import Box from '@mui/material/Box';
import TextField from '@mui/material/TextField';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import Switch from '@mui/material/Switch';
import FormControlLabel from '@mui/material/FormControlLabel';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import Link from '@mui/material/Link';
import { SectionHeader } from '../SectionHeader';

interface GitRepositoriesSectionProps {
  git: Record<string, unknown>;
  projects: string[];
  onUpdate: (section: string, key: string, value: unknown) => void;
}

export function GitRepositoriesSection({
  git,
  projects,
  onUpdate,
}: GitRepositoriesSectionProps) {
  const provider = (git.provider as string) ?? '';
  const github = (git.github ?? {}) as Record<string, unknown>;
  const githubEnabled = (github.enabled as boolean) ?? true;
  const githubTokenEnv = (github.token_env as string) ?? 'GITHUB_TOKEN';
  const branchFormat = (git.branch_format as string) ?? '{type}/{ticket_id}-{slug}';
  const useWorktrees = (git.use_worktrees as boolean) ?? false;

  return (
    <Box sx={{ mb: 4 }}>
      <SectionHeader id="section-git" title="Git Repositories" />
      <Typography color="text.secondary" gutterBottom>
        Configure git provider and branch settings. For more details see the <Link href="https://operator.untra.io/getting-started/git/">git documentation</Link>
      </Typography>

      <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2.5 }}>
        <FormControl fullWidth size="small" margin="dense">
          <InputLabel margin='dense'>Git Provider</InputLabel>
          <Select
            value={provider || 'github'}
            label="Git Provider"
            onChange={(e) => onUpdate('git', 'provider', e.target.value)}
          >
            <MenuItem value="github">GitHub</MenuItem>
            <MenuItem value="gitlab">GitLab</MenuItem>
            <MenuItem value="bitbucket">Bitbucket</MenuItem>
            <MenuItem value="azuredevops">Azure DevOps</MenuItem>
          </Select>
        </FormControl>

        <FormControlLabel
          control={
            <Switch
              checked={githubEnabled}
              onChange={(e) => onUpdate('git.github', 'enabled', e.target.checked)}
            />
          }
          label="GitHub integration enabled"
        />

        <TextField
          fullWidth
          size="small"
          label="GitHub Token Environment Variable"
          value={githubTokenEnv}
          onChange={(e) => onUpdate('git.github', 'token_env', e.target.value)}
          placeholder="GITHUB_TOKEN"
          helperText="Name of the environment variable containing your GitHub personal access token"
          disabled={!githubEnabled}
        />

        <TextField
          fullWidth
          size="small"
          label="Branch Format"
          value={branchFormat}
          onChange={(e) => onUpdate('git', 'branch_format', e.target.value)}
          placeholder="{type}/{ticket_id}-{slug}"
          helperText="Template for branch names. Variables: {type}, {ticket_id}, {slug}"
        />

        <FormControlLabel
          control={
            <Switch
              checked={useWorktrees}
              onChange={(e) => onUpdate('git', 'use_worktrees', e.target.checked)}
            />
          }
          label="Use git worktrees for parallel agent branches"
        />

        <Box>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
            Projects
          </Typography>
          {projects.length > 0 ? (
            <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
              {projects.map((project) => (
                <Chip key={project} label={project} size="small" variant="outlined" />
              ))}
            </Stack>
          ) : (
            <Typography variant="body2" color="text.secondary">
              No projects configured. Set a working directory to discover projects.
            </Typography>
          )}
        </Box>
      </Box>
    </Box>
  );
}
