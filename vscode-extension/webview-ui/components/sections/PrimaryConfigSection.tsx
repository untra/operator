import React from 'react';
import Box from '@mui/material/Box';
import TextField from '@mui/material/TextField';
import Button from '@mui/material/Button';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import Typography from '@mui/material/Typography';
import { SectionHeader } from '../SectionHeader';
import Link from '@mui/material/Link';
import { OperatorBrand } from '../OperatorBrand';

interface PrimaryConfigSectionProps {
  working_directory: string;
  sessions_wrapper: string;
  onUpdate: (section: string, key: string, value: unknown) => void;
  onBrowseFolder: (field: string) => void;
}

export function PrimaryConfigSection({
  working_directory,
  sessions_wrapper,
  onUpdate,
  onBrowseFolder,
}: PrimaryConfigSectionProps) {
  return (
    <Box sx={{ mb: 4 }}>
      <SectionHeader id="section-primary" title="Workspace Configuration" />
      <Typography color="text.secondary" gutterBottom>
        These are settings for <b>Operator!</b> configuration for the VS Code extension. For more details see the <Link href="https://operator.untra.io/configuration/">configuration documentation</Link>
      </Typography>

      <Box sx={{ mb: 2 }}>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 0.5 }}>
          <OperatorBrand /> Working Directory
        </Typography>
        <Box sx={{ display: 'flex', gap: 1 }}>
          <TextField
            fullWidth
            size="small"
            value={working_directory}
            onChange={(e) =>
              onUpdate('primary', 'working_directory', e.target.value)
            }
            placeholder="/path/to/your/repos"
            helperText="Parent directory of Operator! managed code repositories containing .tickets/ working directory"
          />
          <Button
            variant="outlined"
            onClick={() => onBrowseFolder('workingDirectory')}
            sx={{
              minWidth: 'auto',
              px: 2,
              alignSelf: 'flex-start',
              mt: 1,
              borderColor: '#E05D44',
              color: '#E05D44',
              '&:hover': {
                borderColor: '#E05D44',
                backgroundColor: 'rgba(224, 93, 68, 0.08)',
              },
            }}
          >
            change
          </Button>
        </Box>
      </Box>

      <FormControl fullWidth size="small" margin="dense">
        <InputLabel margin='dense'>Session Wrapper</InputLabel>
        <Select
          value={sessions_wrapper || 'vscode'}
          label="Session Wrapper"
          onChange={(e) =>
            onUpdate('sessions', 'wrapper', e.target.value)
          }
        >
          <MenuItem value="vscode">VS Code Terminal</MenuItem>
          <MenuItem value="tmux">tmux</MenuItem>
        </Select>
        <Typography color="text.secondary">
          Designates how launched ticket work is wrapped when started from VS Code
        </Typography>
      </FormControl>
    </Box>
  );
}
