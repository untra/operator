import React from 'react';
import Box from '@mui/material/Box';
import TextField from '@mui/material/TextField';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import Chip from '@mui/material/Chip';
import type { CreateStepRequest } from '../../../src/generated/CreateStepRequest';

interface StepEditorProps {
  step: CreateStepRequest;
  index: number;
  allStepNames: string[];
  onChange: (index: number, step: CreateStepRequest) => void;
  onRemove: (index: number) => void;
  readOnly?: boolean;
}

const PERMISSION_MODES = [
  { value: 'default', label: 'Default (autonomous)' },
  { value: 'plan', label: 'Plan' },
  { value: 'acceptEdits', label: 'Accept Edits' },
  { value: 'delegate', label: 'Delegate' },
];

const REVIEW_TYPES = [
  { value: 'none', label: 'None' },
  { value: 'plan', label: 'Plan Review' },
  { value: 'visual', label: 'Visual Review' },
  { value: 'pr', label: 'PR Review' },
];

const OUTPUT_OPTIONS = ['plan', 'code', 'test', 'pr', 'ticket', 'review', 'report', 'documentation'];

const MODE_COLORS: Record<string, string> = {
  acceptEdits: '#4caf50',
  default: '#4caf50',
  plan: '#2196f3',
  delegate: '#ff9800',
};

export function StepEditor({ step, index, allStepNames, onChange, onRemove, readOnly }: StepEditorProps) {
  const update = (patch: Partial<CreateStepRequest>) => {
    onChange(index, { ...step, ...patch });
  };

  const modeColor = MODE_COLORS[step.permission_mode] || MODE_COLORS.default;

  return (
    <Box sx={{ border: '1px solid', borderColor: 'divider', borderLeft: `3px solid ${modeColor}`, borderRadius: 1, p: 1.5, mb: 1 }}>
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1 }}>
        <Typography variant="caption" color="text.secondary">
          Step {index + 1}
        </Typography>
        {!readOnly && (
          <IconButton size="small" onClick={() => onRemove(index)} sx={{ p: 0.5 }}>
            <Typography variant="caption">✕</Typography>
          </IconButton>
        )}
      </Box>

      <Box sx={{ display: 'flex', gap: 1, mb: 1 }}>
        <TextField
          size="small"
          label="Name"
          value={step.name}
          onChange={(e) => update({ name: e.target.value })}
          disabled={readOnly}
          sx={{ flex: 1 }}
        />
        <TextField
          size="small"
          label="Display Name"
          value={step.display_name ?? ''}
          onChange={(e) => update({ display_name: e.target.value || undefined })}
          disabled={readOnly}
          sx={{ flex: 1 }}
        />
      </Box>

      <TextField
        size="small"
        label="Prompt"
        value={step.prompt}
        onChange={(e) => update({ prompt: e.target.value })}
        disabled={readOnly}
        fullWidth
        multiline
        minRows={2}
        maxRows={6}
        sx={{ mb: 1, '& .MuiInputBase-input': { fontFamily: 'monospace', fontSize: '0.8rem' } }}
      />

      <Box sx={{ display: 'flex', gap: 1, mb: 1 }}>
        <FormControl size="small" sx={{ minWidth: 160 }}>
          <InputLabel>Permission Mode</InputLabel>
          <Select
            value={step.permission_mode}
            label="Permission Mode"
            onChange={(e) => update({ permission_mode: e.target.value })}
            disabled={readOnly}
          >
            {PERMISSION_MODES.map((pm) => (
              <MenuItem key={pm.value} value={pm.value}>{pm.label}</MenuItem>
            ))}
          </Select>
        </FormControl>

        <FormControl size="small" sx={{ minWidth: 140 }}>
          <InputLabel>Review Type</InputLabel>
          <Select
            value={step.review_type}
            label="Review Type"
            onChange={(e) => update({ review_type: e.target.value })}
            disabled={readOnly}
          >
            {REVIEW_TYPES.map((rt) => (
              <MenuItem key={rt.value} value={rt.value}>{rt.label}</MenuItem>
            ))}
          </Select>
        </FormControl>

        <FormControl size="small" sx={{ minWidth: 140 }}>
          <InputLabel>Next Step</InputLabel>
          <Select
            value={step.next_step ?? ''}
            label="Next Step"
            onChange={(e) => update({ next_step: e.target.value || undefined })}
            disabled={readOnly}
          >
            <MenuItem value="">
              <em>None (end)</em>
            </MenuItem>
            {allStepNames.filter(n => n !== step.name).map((name) => (
              <MenuItem key={name} value={name}>{name}</MenuItem>
            ))}
          </Select>
        </FormControl>
      </Box>

      {/* Outputs */}
      <Box sx={{ mb: 1 }}>
        <Typography variant="caption" color="text.secondary" sx={{ mb: 0.5, display: 'block' }}>
          Outputs
        </Typography>
        <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
          {OUTPUT_OPTIONS.map((output) => {
            const selected = (step.outputs ?? []).includes(output);
            return (
              <Chip
                key={output}
                label={output}
                size="small"
                variant={selected ? 'filled' : 'outlined'}
                color={selected ? 'primary' : 'default'}
                onClick={readOnly ? undefined : () => {
                  const outputs = selected
                    ? (step.outputs ?? []).filter(o => o !== output)
                    : [...(step.outputs ?? []), output];
                  update({ outputs });
                }}
                sx={{ cursor: readOnly ? 'default' : 'pointer' }}
              />
            );
          })}
        </Box>
      </Box>
    </Box>
  );
}
