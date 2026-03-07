import React from 'react';
import Box from '@mui/material/Box';
import TextField from '@mui/material/TextField';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import FormControlLabel from '@mui/material/FormControlLabel';
import Switch from '@mui/material/Switch';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import type { CreateFieldRequest } from '../../../src/generated/CreateFieldRequest';

interface FieldEditorProps {
  field: CreateFieldRequest;
  index: number;
  onChange: (index: number, field: CreateFieldRequest) => void;
  onRemove: (index: number) => void;
  readOnly?: boolean;
}

const FIELD_TYPES = [
  { value: 'string', label: 'String' },
  { value: 'text', label: 'Text (multiline)' },
  { value: 'enum', label: 'Enum (options)' },
  { value: 'bool', label: 'Boolean' },
  { value: 'date', label: 'Date' },
  { value: 'integer', label: 'Integer' },
];

export function FieldEditor({ field, index, onChange, onRemove, readOnly }: FieldEditorProps) {
  const update = (patch: Partial<CreateFieldRequest>) => {
    onChange(index, { ...field, ...patch });
  };

  return (
    <Box sx={{ border: '1px solid', borderColor: 'divider', borderRadius: 1, p: 1.5, mb: 1 }}>
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1 }}>
        <Typography variant="caption" color="text.secondary">
          Field {index + 1}
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
          value={field.name}
          onChange={(e) => update({ name: e.target.value })}
          disabled={readOnly}
          sx={{ flex: 1 }}
        />
        <FormControl size="small" sx={{ minWidth: 120 }}>
          <InputLabel>Type</InputLabel>
          <Select
            value={field.field_type}
            label="Type"
            onChange={(e) => update({ field_type: e.target.value })}
            disabled={readOnly}
          >
            {FIELD_TYPES.map((ft) => (
              <MenuItem key={ft.value} value={ft.value}>{ft.label}</MenuItem>
            ))}
          </Select>
        </FormControl>
      </Box>

      <TextField
        size="small"
        label="Description"
        value={field.description}
        onChange={(e) => update({ description: e.target.value })}
        disabled={readOnly}
        fullWidth
        sx={{ mb: 1 }}
      />

      {field.field_type === 'enum' && (
        <TextField
          size="small"
          label="Options (comma-separated)"
          value={(field.options ?? []).join(', ')}
          onChange={(e) => update({ options: e.target.value.split(',').map(s => s.trim()).filter(Boolean) })}
          disabled={readOnly}
          fullWidth
          sx={{ mb: 1 }}
        />
      )}

      <Box sx={{ display: 'flex', gap: 2, alignItems: 'center' }}>
        <TextField
          size="small"
          label="Default"
          value={field.default ?? ''}
          onChange={(e) => update({ default: e.target.value || undefined })}
          disabled={readOnly}
          sx={{ flex: 1 }}
        />
        <TextField
          size="small"
          label="Placeholder"
          value={field.placeholder ?? ''}
          onChange={(e) => update({ placeholder: e.target.value || undefined })}
          disabled={readOnly}
          sx={{ flex: 1 }}
        />
      </Box>

      <Box sx={{ display: 'flex', gap: 2, mt: 1 }}>
        <FormControlLabel
          control={
            <Switch
              size="small"
              checked={field.required}
              onChange={(e) => update({ required: e.target.checked })}
              disabled={readOnly}
            />
          }
          label={<Typography variant="caption">Required</Typography>}
        />
        <FormControlLabel
          control={
            <Switch
              size="small"
              checked={field.user_editable}
              onChange={(e) => update({ user_editable: e.target.checked })}
              disabled={readOnly}
            />
          }
          label={<Typography variant="caption">User Editable</Typography>}
        />
      </Box>
    </Box>
  );
}
