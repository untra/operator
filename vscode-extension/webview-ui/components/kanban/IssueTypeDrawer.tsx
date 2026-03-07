import React, { useState, useEffect } from 'react';
import Box from '@mui/material/Box';
import Drawer from '@mui/material/Drawer';
import Typography from '@mui/material/Typography';
import TextField from '@mui/material/TextField';
import Button from '@mui/material/Button';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import FormControlLabel from '@mui/material/FormControlLabel';
import Switch from '@mui/material/Switch';
import Divider from '@mui/material/Divider';
import Alert from '@mui/material/Alert';
import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import DialogActions from '@mui/material/DialogActions';
import { WorkflowPreview } from './WorkflowPreview';
import { FieldEditor } from './FieldEditor';
import { StepEditor } from './StepEditor';
import type { IssueTypeResponse } from '../../../src/generated/IssueTypeResponse';
import type { CreateIssueTypeRequest } from '../../../src/generated/CreateIssueTypeRequest';
import type { UpdateIssueTypeRequest } from '../../../src/generated/UpdateIssueTypeRequest';
import type { CreateFieldRequest } from '../../../src/generated/CreateFieldRequest';
import type { CreateStepRequest } from '../../../src/generated/CreateStepRequest';

interface IssueTypeDrawerProps {
  open: boolean;
  mode: 'view' | 'edit' | 'create';
  issueType: IssueTypeResponse | null;
  onClose: () => void;
  onCreate: (request: CreateIssueTypeRequest) => void;
  onUpdate: (key: string, request: UpdateIssueTypeRequest) => void;
  onDelete: (key: string) => void;
}

const DEFAULT_FIELD: CreateFieldRequest = {
  name: '',
  description: '',
  field_type: 'string',
  required: false,
  default: null,
  options: [],
  placeholder: null,
  max_length: null,
  user_editable: true,
};

const DEFAULT_STEP: CreateStepRequest = {
  name: '',
  display_name: null,
  prompt: '',
  outputs: [],
  allowed_tools: ['*'],
  review_type: 'none',
  next_step: null,
  permission_mode: 'default',
};

function generateKey(name: string): string {
  return name
    .replace(/[^a-zA-Z0-9]/g, '')
    .toUpperCase()
    .substring(0, 10);
}

export function IssueTypeDrawer({
  open,
  mode,
  issueType,
  onClose,
  onCreate,
  onUpdate,
  onDelete,
}: IssueTypeDrawerProps) {
  const [key, setKey] = useState('');
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [issueMode, setIssueMode] = useState('autonomous');
  const [glyph, setGlyph] = useState('');
  const [color, setColor] = useState('');
  const [projectRequired, setProjectRequired] = useState(true);
  const [fields, setFields] = useState<CreateFieldRequest[]>([]);
  const [steps, setSteps] = useState<CreateStepRequest[]>([]);
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false);
  const [autoKey, setAutoKey] = useState(true);

  const isBuiltin = issueType?.source === 'builtin';
  const readOnly = mode === 'view' || isBuiltin;
  const isCreate = mode === 'create';

  useEffect(() => {
    if (issueType && mode !== 'create') {
      setKey(issueType.key);
      setName(issueType.name);
      setDescription(issueType.description);
      setIssueMode(issueType.mode);
      setGlyph(issueType.glyph);
      setColor(issueType.color ?? '');
      setProjectRequired(issueType.project_required);
      setFields(issueType.fields.map(f => ({
        name: f.name,
        description: f.description,
        field_type: f.field_type,
        required: f.required,
        default: f.default ?? null,
        options: f.options,
        placeholder: f.placeholder ?? null,
        max_length: f.max_length ?? null,
        user_editable: f.user_editable,
      })));
      setSteps(issueType.steps.map(s => ({
        name: s.name,
        display_name: s.display_name ?? null,
        prompt: s.prompt,
        outputs: s.outputs,
        allowed_tools: s.allowed_tools,
        review_type: s.review_type,
        next_step: s.next_step ?? null,
        permission_mode: s.permission_mode,
      })));
      setAutoKey(false);
    } else if (isCreate) {
      setKey('');
      setName('');
      setDescription('');
      setIssueMode('autonomous');
      setGlyph('');
      setColor('');
      setProjectRequired(true);
      setFields([]);
      setSteps([{ ...DEFAULT_STEP, name: 'execute', prompt: '' }]);
      setAutoKey(true);
    }
  }, [issueType, mode, isCreate]);

  const handleSave = () => {
    if (isCreate) {
      const request: CreateIssueTypeRequest = {
        key,
        name,
        description,
        mode: issueMode,
        glyph: glyph || name.charAt(0).toUpperCase(),
        color: color || null,
        project_required: projectRequired,
        fields,
        steps,
      };
      onCreate(request);
    } else if (issueType) {
      const request: UpdateIssueTypeRequest = {
        name,
        description,
        mode: issueMode,
        glyph: glyph || null,
        color: color || null,
        project_required: projectRequired,
        fields,
        steps,
      };
      onUpdate(issueType.key, request);
    }
    onClose();
  };

  const handleFieldChange = (index: number, field: CreateFieldRequest) => {
    setFields(prev => prev.map((f, i) => i === index ? field : f));
  };

  const handleStepChange = (index: number, step: CreateStepRequest) => {
    setSteps(prev => prev.map((s, i) => i === index ? step : s));
  };

  return (
    <Drawer
      anchor="right"
      open={open}
      onClose={onClose}
      PaperProps={{ sx: { width: 500, p: 2, overflow: 'auto' } }}
    >
      {/* Header */}
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 2 }}>
        <Typography variant="h6" sx={{ flex: 1 }}>
          {isCreate ? 'Create Issue Type' : readOnly ? 'Issue Type Details' : 'Edit Issue Type'}
        </Typography>
        {issueType && (
          <Typography variant="caption" color="text.secondary">
            Source: {issueType.source}
          </Typography>
        )}
      </Box>

      {isBuiltin && mode === 'view' && (
        <Alert severity="info" sx={{ mb: 2 }}>
          Builtin types are read-only.
        </Alert>
      )}

      {/* Overview */}
      <Box sx={{ display: 'flex', gap: 1, mb: 2 }}>
        <TextField
          size="small"
          label="Key"
          value={key}
          onChange={(e) => {
            setKey(e.target.value.toUpperCase());
            setAutoKey(false);
          }}
          disabled={!isCreate}
          sx={{ flex: 1 }}
        />
        <TextField
          size="small"
          label="Name"
          value={name}
          onChange={(e) => {
            setName(e.target.value);
            if (isCreate && autoKey) {
              setKey(generateKey(e.target.value));
            }
          }}
          disabled={readOnly}
          sx={{ flex: 2 }}
        />
        <TextField
          size="small"
          label="Glyph"
          value={glyph}
          onChange={(e) => setGlyph(e.target.value)}
          disabled={readOnly}
          sx={{ width: 60 }}
          inputProps={{ maxLength: 2 }}
        />
      </Box>

      <TextField
        size="small"
        label="Description"
        value={description}
        onChange={(e) => setDescription(e.target.value)}
        disabled={readOnly}
        fullWidth
        multiline
        minRows={2}
        sx={{ mb: 2 }}
      />

      <Box sx={{ display: 'flex', gap: 2, mb: 2, alignItems: 'center' }}>
        <FormControl size="small" sx={{ minWidth: 140 }}>
          <InputLabel>Mode</InputLabel>
          <Select
            value={issueMode}
            label="Mode"
            onChange={(e) => setIssueMode(e.target.value)}
            disabled={readOnly}
          >
            <MenuItem value="autonomous">Autonomous</MenuItem>
            <MenuItem value="paired">Paired</MenuItem>
          </Select>
        </FormControl>

        <TextField
          size="small"
          label="Color"
          value={color}
          onChange={(e) => setColor(e.target.value)}
          disabled={readOnly}
          placeholder="#66AA99"
          sx={{ width: 120 }}
        />

        <FormControlLabel
          control={
            <Switch
              size="small"
              checked={projectRequired}
              onChange={(e) => setProjectRequired(e.target.checked)}
              disabled={readOnly}
            />
          }
          label={<Typography variant="body2">Project Required</Typography>}
        />
      </Box>

      <Divider sx={{ my: 2 }} />

      {/* Fields */}
      <Box sx={{ mb: 2 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1 }}>
          <Typography variant="subtitle2">Fields ({fields.length})</Typography>
          {!readOnly && (
            <Button
              size="small"
              onClick={() => setFields([...fields, { ...DEFAULT_FIELD }])}
            >
              Add Field
            </Button>
          )}
        </Box>
        {fields.map((field, i) => (
          <FieldEditor
            key={i}
            field={field}
            index={i}
            onChange={handleFieldChange}
            onRemove={(idx) => setFields(fields.filter((_, j) => j !== idx))}
            readOnly={readOnly}
          />
        ))}
        {fields.length === 0 && (
          <Typography variant="caption" color="text.secondary">No fields defined</Typography>
        )}
      </Box>

      <Divider sx={{ my: 2 }} />

      {/* Steps / Workflow */}
      <Box sx={{ mb: 2 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 1 }}>
          <Typography variant="subtitle2">Workflow Steps ({steps.length})</Typography>
          {!readOnly && (
            <Button
              size="small"
              onClick={() => setSteps([...steps, { ...DEFAULT_STEP }])}
            >
              Add Step
            </Button>
          )}
        </Box>

        {/* Preview */}
        {steps.length > 0 && readOnly && (
          <Box sx={{ mb: 2 }}>
            <WorkflowPreview
              steps={steps.map(s => ({
                name: s.name,
                display_name: s.display_name ?? null,
                prompt: s.prompt,
                outputs: s.outputs,
                allowed_tools: s.allowed_tools,
                review_type: s.review_type,
                next_step: s.next_step ?? null,
                permission_mode: s.permission_mode,
              }))}
            />
          </Box>
        )}

        {/* Editors */}
        {!readOnly && steps.map((step, i) => (
          <StepEditor
            key={i}
            step={step}
            index={i}
            allStepNames={steps.map(s => s.name)}
            onChange={handleStepChange}
            onRemove={(idx) => setSteps(steps.filter((_, j) => j !== idx))}
            readOnly={readOnly}
          />
        ))}
        {steps.length === 0 && (
          <Typography variant="caption" color="text.secondary">No workflow steps defined</Typography>
        )}
      </Box>

      {/* Footer */}
      <Box sx={{ display: 'flex', gap: 1, justifyContent: 'flex-end', mt: 'auto', pt: 2, borderTop: '1px solid', borderColor: 'divider' }}>
        {!readOnly && !isBuiltin && issueType && (
          <Button
            color="error"
            onClick={() => setDeleteConfirmOpen(true)}
            sx={{ mr: 'auto' }}
          >
            Delete
          </Button>
        )}
        <Button onClick={onClose}>
          {readOnly ? 'Close' : 'Cancel'}
        </Button>
        {!readOnly && (
          <Button
            variant="contained"
            onClick={handleSave}
            disabled={!key || !name || steps.length === 0}
          >
            {isCreate ? 'Create' : 'Save'}
          </Button>
        )}
      </Box>

      {/* Delete confirmation */}
      <Dialog open={deleteConfirmOpen} onClose={() => setDeleteConfirmOpen(false)}>
        <DialogTitle>Delete Issue Type</DialogTitle>
        <DialogContent>
          <Typography>
            Are you sure you want to delete <strong>{issueType?.key}</strong>? This cannot be undone.
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteConfirmOpen(false)}>Cancel</Button>
          <Button
            color="error"
            variant="contained"
            onClick={() => {
              if (issueType) {
                onDelete(issueType.key);
                setDeleteConfirmOpen(false);
                onClose();
              }
            }}
          >
            Delete
          </Button>
        </DialogActions>
      </Dialog>
    </Drawer>
  );
}
