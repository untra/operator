/**
 * Issue Type create/edit form page component.
 */
import React, { useState, useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import {
  Content,
  ContentHeader,
  Header,
  Page,
  Progress,
} from '@backstage/core-components';
import {
  Box,
  Button,
  Card,
  CardContent,
  FormControl,
  FormControlLabel,
  Grid,
  IconButton,
  InputLabel,
  MenuItem,
  Select,
  Switch,
  TextField,
  Typography,
} from '@material-ui/core';
import { makeStyles } from '@material-ui/core/styles';
import AddIcon from '@material-ui/icons/Add';
import ArrowBackIcon from '@material-ui/icons/ArrowBack';
import DeleteIcon from '@material-ui/icons/Delete';
import { Alert } from '@material-ui/lab';
import {
  useIssueType,
  useCreateIssueType,
  useUpdateIssueType,
} from '../hooks/useIssueTypes';
import type {
  CreateIssueTypeRequest,
  CreateStepRequest,
  ExecutionMode,
  PermissionMode,
} from '../api/types';

const useStyles = makeStyles(theme => ({
  form: {
    display: 'flex',
    flexDirection: 'column',
    gap: theme.spacing(2),
  },
  stepCard: {
    marginBottom: theme.spacing(2),
    position: 'relative',
  },
  stepHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: theme.spacing(1),
  },
  actions: {
    display: 'flex',
    gap: theme.spacing(2),
    marginTop: theme.spacing(3),
  },
}));

interface StepFormData {
  name: string;
  display_name: string;
  prompt: string;
  permission_mode: PermissionMode;
  requires_review: boolean;
  next_step: string;
}

const defaultStep: StepFormData = {
  name: '',
  display_name: '',
  prompt: '',
  permission_mode: 'default',
  requires_review: false,
  next_step: '',
};

/** Step editor component */
function StepEditor({
  step,
  index,
  onChange,
  onDelete,
}: {
  step: StepFormData;
  index: number;
  onChange: (step: StepFormData) => void;
  onDelete: () => void;
}) {
  const classes = useStyles();

  return (
    <Card className={classes.stepCard} variant="outlined">
      <CardContent>
        <Box className={classes.stepHeader}>
          <Typography variant="h6">Step {index + 1}</Typography>
          <IconButton size="small" onClick={onDelete} title="Delete step">
            <DeleteIcon />
          </IconButton>
        </Box>

        <Grid container spacing={2}>
          <Grid item xs={12} sm={6}>
            <TextField
              label="Step Name"
              value={step.name}
              onChange={e => onChange({ ...step, name: e.target.value })}
              fullWidth
              required
              helperText="Internal identifier (e.g., 'plan', 'implement')"
            />
          </Grid>
          <Grid item xs={12} sm={6}>
            <TextField
              label="Display Name"
              value={step.display_name}
              onChange={e => onChange({ ...step, display_name: e.target.value })}
              fullWidth
              helperText="Human-readable name (optional)"
            />
          </Grid>
          <Grid item xs={12}>
            <TextField
              label="Prompt"
              value={step.prompt}
              onChange={e => onChange({ ...step, prompt: e.target.value })}
              fullWidth
              required
              multiline
              rows={4}
              helperText="Instructions for the agent"
            />
          </Grid>
          <Grid item xs={12} sm={4}>
            <FormControl fullWidth>
              <InputLabel>Permission Mode</InputLabel>
              <Select
                value={step.permission_mode}
                onChange={e =>
                  onChange({
                    ...step,
                    permission_mode: e.target.value as PermissionMode,
                  })
                }
                label="Permission Mode"
              >
                <MenuItem value="default">Default</MenuItem>
                <MenuItem value="plan">Plan</MenuItem>
                <MenuItem value="acceptEdits">Accept Edits</MenuItem>
                <MenuItem value="delegate">Delegate</MenuItem>
              </Select>
            </FormControl>
          </Grid>
          <Grid item xs={12} sm={4}>
            <TextField
              label="Next Step"
              value={step.next_step}
              onChange={e => onChange({ ...step, next_step: e.target.value })}
              fullWidth
              helperText="Name of next step (optional)"
            />
          </Grid>
          <Grid item xs={12} sm={4}>
            <FormControlLabel
              control={
                <Switch
                  checked={step.requires_review}
                  onChange={e =>
                    onChange({ ...step, requires_review: e.target.checked })
                  }
                />
              }
              label="Requires Review"
            />
          </Grid>
        </Grid>
      </CardContent>
    </Card>
  );
}

/** Main form page component */
export function IssueTypeFormPage() {
  const classes = useStyles();
  const navigate = useNavigate();
  const { key } = useParams<{ key: string }>();
  const isEditMode = Boolean(key && key !== 'new');

  const { issueType, loading: loadingType } = useIssueType(
    isEditMode ? key! : '',
  );
  const { createIssueType, creating, error: createError } = useCreateIssueType();
  const { updateIssueType, updating, error: updateError } = useUpdateIssueType();

  // Form state
  const [formKey, setFormKey] = useState('');
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [glyph, setGlyph] = useState('');
  const [mode, setMode] = useState<ExecutionMode>('autonomous');
  const [projectRequired, setProjectRequired] = useState(true);
  const [color, setColor] = useState('');
  const [steps, setSteps] = useState<StepFormData[]>([{ ...defaultStep }]);

  // Populate form when editing
  useEffect(() => {
    if (isEditMode && issueType) {
      setFormKey(issueType.key);
      setName(issueType.name);
      setDescription(issueType.description);
      setGlyph(issueType.glyph);
      setMode(issueType.mode);
      setProjectRequired(issueType.project_required);
      setColor(issueType.color || '');
      setSteps(
        issueType.steps.map(s => ({
          name: s.name,
          display_name: s.display_name || '',
          prompt: s.prompt,
          permission_mode: s.permission_mode,
          requires_review: s.requires_review,
          next_step: s.next_step || '',
        })),
      );
    }
  }, [isEditMode, issueType]);

  if (isEditMode && loadingType) {
    return <Progress />;
  }

  const error = createError || updateError;
  const saving = creating || updating;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const stepsData: CreateStepRequest[] = steps.map(s => ({
      name: s.name,
      display_name: s.display_name || undefined,
      prompt: s.prompt,
      permission_mode: s.permission_mode,
      requires_review: s.requires_review,
      next_step: s.next_step || undefined,
    }));

    const request: CreateIssueTypeRequest = {
      key: formKey.toUpperCase(),
      name,
      description,
      glyph,
      mode,
      project_required: projectRequired,
      color: color || undefined,
      steps: stepsData,
    };

    try {
      if (isEditMode) {
        await updateIssueType(key!, {
          name,
          description,
          glyph,
          mode,
          project_required: projectRequired,
          color: color || undefined,
          steps: stepsData,
        });
      } else {
        await createIssueType(request);
      }
      navigate('..');
    } catch {
      // Error is handled by the hook
    }
  };

  const handleStepChange = (index: number, step: StepFormData) => {
    const newSteps = [...steps];
    newSteps[index] = step;
    setSteps(newSteps);
  };

  const handleAddStep = () => {
    setSteps([...steps, { ...defaultStep }]);
  };

  const handleDeleteStep = (index: number) => {
    if (steps.length > 1) {
      setSteps(steps.filter((_, i) => i !== index));
    }
  };

  return (
    <Page themeId="tool">
      <Header
        title={isEditMode ? `Edit ${key}` : 'Create Issue Type'}
        subtitle={isEditMode ? 'Modify issue type configuration' : 'Define a new issue type template'}
      />
      <Content>
        <ContentHeader title="">
          <Button startIcon={<ArrowBackIcon />} onClick={() => navigate('..')}>
            Cancel
          </Button>
        </ContentHeader>

        {error && (
          <Alert severity="error" style={{ marginBottom: 16 }}>
            {error.message}
          </Alert>
        )}

        <form onSubmit={handleSubmit}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Basic Information
              </Typography>
              <Grid container spacing={2}>
                <Grid item xs={12} sm={4}>
                  <TextField
                    label="Key"
                    value={formKey}
                    onChange={e => setFormKey(e.target.value.toUpperCase())}
                    fullWidth
                    required
                    disabled={isEditMode}
                    helperText="Unique identifier (e.g., FEAT, FIX)"
                    inputProps={{ maxLength: 10 }}
                  />
                </Grid>
                <Grid item xs={12} sm={4}>
                  <TextField
                    label="Name"
                    value={name}
                    onChange={e => setName(e.target.value)}
                    fullWidth
                    required
                    helperText="Display name"
                  />
                </Grid>
                <Grid item xs={12} sm={2}>
                  <TextField
                    label="Glyph"
                    value={glyph}
                    onChange={e => setGlyph(e.target.value)}
                    fullWidth
                    required
                    helperText="Icon character"
                    inputProps={{ maxLength: 2 }}
                  />
                </Grid>
                <Grid item xs={12} sm={2}>
                  <TextField
                    label="Color"
                    value={color}
                    onChange={e => setColor(e.target.value)}
                    fullWidth
                    helperText="Hex color (optional)"
                    placeholder="#3B82F6"
                  />
                </Grid>
                <Grid item xs={12}>
                  <TextField
                    label="Description"
                    value={description}
                    onChange={e => setDescription(e.target.value)}
                    fullWidth
                    required
                    multiline
                    rows={2}
                  />
                </Grid>
                <Grid item xs={12} sm={4}>
                  <FormControl fullWidth>
                    <InputLabel>Execution Mode</InputLabel>
                    <Select
                      value={mode}
                      onChange={e => setMode(e.target.value as ExecutionMode)}
                      label="Execution Mode"
                    >
                      <MenuItem value="autonomous">Autonomous</MenuItem>
                      <MenuItem value="paired">Paired</MenuItem>
                    </Select>
                  </FormControl>
                </Grid>
                <Grid item xs={12} sm={4}>
                  <TextField
                    label="Branch Prefix"
                    value={branchPrefix}
                    onChange={e => setBranchPrefix(e.target.value)}
                    fullWidth
                    helperText="Git branch prefix"
                    placeholder={formKey.toLowerCase() || 'task'}
                  />
                </Grid>
                <Grid item xs={12} sm={4}>
                  <FormControlLabel
                    control={
                      <Switch
                        checked={projectRequired}
                        onChange={e => setProjectRequired(e.target.checked)}
                      />
                    }
                    label="Project Required"
                  />
                </Grid>
              </Grid>
            </CardContent>
          </Card>

          <Box mt={3}>
            <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
              <Typography variant="h6">Steps</Typography>
              <Button
                variant="outlined"
                startIcon={<AddIcon />}
                onClick={handleAddStep}
              >
                Add Step
              </Button>
            </Box>

            {steps.map((step, index) => (
              <StepEditor
                key={index}
                step={step}
                index={index}
                onChange={s => handleStepChange(index, s)}
                onDelete={() => handleDeleteStep(index)}
              />
            ))}
          </Box>

          <Box className={classes.actions}>
            <Button
              type="submit"
              variant="contained"
              color="primary"
              disabled={saving}
            >
              {saving ? 'Saving...' : isEditMode ? 'Update' : 'Create'}
            </Button>
            <Button onClick={() => navigate('..')}>Cancel</Button>
          </Box>
        </form>
      </Content>
    </Page>
  );
}
