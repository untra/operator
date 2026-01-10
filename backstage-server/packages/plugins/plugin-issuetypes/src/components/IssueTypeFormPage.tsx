/**
 * Issue Type Form Page
 *
 * Create or edit an issue type.
 * Supports Simple and Advanced modes (URL-persisted).
 * In create mode, defaults come from the TASK issuetype template.
 */

import React, { useState, useEffect, useMemo } from 'react';
import { useParams, useNavigate, useLocation } from 'react-router-dom';
import {
  Content,
  ContentHeader,
  Header,
  Page,
  InfoCard,
} from '@backstage/core-components';
import {
  Button,
  ButtonGroup,
  makeStyles,
} from '@material-ui/core';
import {
  useIssueType,
  useCreateIssueType,
  useUpdateIssueType,
} from '../hooks';
import type {
  CreateIssueTypeRequest,
  CreateFieldRequest,
  CreateStepRequest,
  ExecutionMode,
} from '../api/types';
import { FieldEditor, StepEditor } from './editors';
import { Chip } from './ui';

const GLYPH_OPTIONS = ['*', '#', '>', '?', '!', 'A', 'S', 'I', 'X'];
const COLOR_OPTIONS = [
  'blue',
  'cyan',
  'green',
  'yellow',
  'magenta',
  'red',
] as const;
const MODE_OPTIONS: ExecutionMode[] = ['autonomous', 'paired'];

const useStyles = makeStyles((theme) => ({
  toggleContainer: {
    marginLeft: 'auto',
  },
  toggleButton: {
    textTransform: 'none',
    padding: '6px 16px',
  },
  activeButton: {
    backgroundColor: theme.palette.primary.main,
    color: theme.palette.primary.contrastText,
    '&:hover': {
      backgroundColor: theme.palette.primary.dark,
    },
  },
  headerRow: {
    display: 'flex',
    alignItems: 'center',
    gap: theme.spacing(2),
    marginBottom: theme.spacing(2),
  },
  label: {
    display: 'block',
    marginBottom: 4,
    color: theme.palette.text.primary,
  },
  input: {
    width: '100%',
    padding: 8,
    border: `1px solid ${theme.palette.divider}`,
    borderRadius: 4,
    backgroundColor: theme.palette.background.paper,
    color: theme.palette.text.primary,
    fontSize: 14,
    '&:disabled': {
      backgroundColor: theme.palette.action.disabledBackground,
      color: theme.palette.text.secondary,
    },
  },
  textarea: {
    width: '100%',
    padding: 8,
    border: `1px solid ${theme.palette.divider}`,
    borderRadius: 4,
    backgroundColor: theme.palette.background.paper,
    color: theme.palette.text.primary,
    fontSize: 14,
    resize: 'vertical',
  },
  select: {
    width: '100%',
    padding: 8,
    border: `1px solid ${theme.palette.divider}`,
    borderRadius: 4,
    backgroundColor: theme.palette.background.paper,
    color: theme.palette.text.primary,
    fontSize: 14,
  },
  checkboxLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
    color: theme.palette.text.primary,
  },
  addButton: {
    padding: '8px 16px',
    backgroundColor: theme.palette.primary.main,
    color: theme.palette.primary.contrastText,
    border: 'none',
    borderRadius: 4,
    cursor: 'pointer',
    '&:hover': {
      backgroundColor: theme.palette.primary.dark,
    },
  },
  submitButton: {
    padding: '12px 24px',
    backgroundColor: theme.palette.success.main,
    color: theme.palette.success.contrastText,
    border: 'none',
    borderRadius: 4,
    cursor: 'pointer',
    fontSize: 16,
    '&:disabled': {
      cursor: 'not-allowed',
      opacity: 0.6,
    },
  },
  cancelButton: {
    padding: '12px 24px',
    backgroundColor: theme.palette.action.hover,
    color: theme.palette.text.primary,
    border: 'none',
    borderRadius: 4,
    cursor: 'pointer',
    fontSize: 16,
  },
  errorBox: {
    padding: 16,
    marginBottom: 16,
    backgroundColor: theme.palette.error.light,
    border: `1px solid ${theme.palette.error.main}`,
    borderRadius: 4,
    color: theme.palette.error.contrastText,
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr 1fr',
    gap: 16,
  },
  fullWidth: {
    gridColumn: '1 / -1',
  },
}));

type ViewMode = 'simple' | 'advanced';

/**
 * URL-persisted view mode toggle.
 */
function useViewMode(): [ViewMode, (mode: ViewMode) => void] {
  const location = useLocation();
  const navigate = useNavigate();

  const viewMode = useMemo(() => {
    const params = new URLSearchParams(location.search);
    return params.get('mode') === 'advanced' ? 'advanced' : 'simple';
  }, [location.search]);

  const setViewMode = (mode: ViewMode) => {
    const params = new URLSearchParams(location.search);
    if (mode === 'advanced') {
      params.set('mode', 'advanced');
    } else {
      params.delete('mode');
    }
    const newSearch = params.toString();
    navigate(
      {
        pathname: location.pathname,
        search: newSearch ? `?${newSearch}` : '',
      },
      { replace: true },
    );
  };

  return [viewMode, setViewMode];
}

/**
 * Generate KEY from name + timestamp.
 * Format: {NAME}-{YYYYMMDDHHMMSS}
 */
function generateKey(name: string): string {
  if (!name.trim()) {return '';}

  const prefix = name
    .replace(/[^a-zA-Z]/g, '') // Remove non-letters
    .toUpperCase()
    .slice(0, 8); // First 8 chars

  if (!prefix) {return '';}

  const timestamp = new Date()
    .toISOString()
    .replace(/[-:T.Z]/g, '') // YYYYMMDDHHMMSS
    .slice(0, 14);

  return `${prefix}-${timestamp}`;
}

const createEmptyField = (): CreateFieldRequest => ({
  name: '',
  description: '',
  field_type: 'string',
  required: false,
  user_editable: true,
});

const createEmptyStep = (): CreateStepRequest => ({
  name: '',
  prompt: '',
  outputs: [],
  allowed_tools: ['Read', 'Write', 'Edit', 'Glob', 'Grep', 'Bash'],
  requires_review: false,
  permission_mode: 'default',
});

export const IssueTypeFormPage = () => {
  const classes = useStyles();
  const { key } = useParams<{ key?: string }>();
  const navigate = useNavigate();
  const isEditing = Boolean(key);
  const [viewMode, setViewMode] = useViewMode();

  // Hooks for loading and saving
  const { issueType, loading: loadingIssueType } = useIssueType(key || '');
  const { issueType: taskTemplate, loading: loadingTask } = useIssueType('TASK');
  const { createIssueType, creating, error: createError } = useCreateIssueType();
  const { updateIssueType, updating, error: updateError } = useUpdateIssueType();

  // Track if form has been initialized with defaults
  const [formInitialized, setFormInitialized] = useState(false);

  // Form state - defaults to paired mode
  const [formData, setFormData] = useState<CreateIssueTypeRequest>({
    key: '',
    name: '',
    description: '',
    glyph: '>',
    mode: 'paired', // Default to paired
    project_required: true,
    fields: [],
    steps: [],
  });

  const [validationErrors, setValidationErrors] = useState<string[]>([]);

  // Populate form with TASK defaults when creating new
  useEffect(() => {
    if (!isEditing && taskTemplate && !formInitialized) {
      // Get user-editable fields only
      const userEditableFields = taskTemplate.fields
        .filter((f) => f.user_editable !== false)
        .map((f) => ({
          name: f.name,
          description: f.description,
          field_type: f.field_type,
          required: f.required,
          default: f.default,
          options: f.options,
          placeholder: f.placeholder,
          max_length: f.max_length,
          user_editable: true,
        }));

      setFormData((prev) => ({
        ...prev,
        fields: userEditableFields,
        steps: taskTemplate.steps.map((s) => ({
          name: s.name,
          display_name: s.display_name,
          prompt: s.prompt,
          outputs: s.outputs,
          allowed_tools: s.allowed_tools,
          requires_review: s.requires_review,
          next_step: s.next_step,
          permission_mode: s.permission_mode,
        })),
      }));
      setFormInitialized(true);
    }
  }, [isEditing, taskTemplate, formInitialized]);

  // Populate form when editing
  useEffect(() => {
    if (isEditing && issueType) {
      setFormData({
        key: issueType.key,
        name: issueType.name,
        description: issueType.description,
        glyph: issueType.glyph,
        mode: issueType.mode,
        color: issueType.color,
        project_required: issueType.project_required,
        fields: issueType.fields.map((f) => ({
          name: f.name,
          description: f.description,
          field_type: f.field_type,
          required: f.required,
          default: f.default,
          options: f.options,
          placeholder: f.placeholder,
          max_length: f.max_length,
          user_editable: f.user_editable,
        })),
        steps: issueType.steps.map((s) => ({
          name: s.name,
          display_name: s.display_name,
          prompt: s.prompt,
          outputs: s.outputs,
          allowed_tools: s.allowed_tools,
          requires_review: s.requires_review,
          next_step: s.next_step,
          on_reject: s.on_reject,
          permission_mode: s.permission_mode,
        })),
      });
      setFormInitialized(true);
    }
  }, [isEditing, issueType]);

  const isBuiltin = issueType?.source === 'builtin';

  const handleChange = (
    field: keyof CreateIssueTypeRequest,
    value: string | boolean | undefined,
  ) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
  };

  // Handle name change - auto-generate KEY in create mode
  const handleNameChange = (name: string) => {
    setFormData((prev) => ({
      ...prev,
      name,
      key: isEditing ? prev.key : generateKey(name),
    }));
  };

  const handleFieldChange = (index: number, field: CreateFieldRequest) => {
    setFormData((prev) => ({
      ...prev,
      fields: prev.fields?.map((f, i) => (i === index ? field : f)),
    }));
  };

  const handleFieldDelete = (index: number) => {
    setFormData((prev) => ({
      ...prev,
      fields: prev.fields?.filter((_, i) => i !== index),
    }));
  };

  const handleAddField = () => {
    setFormData((prev) => ({
      ...prev,
      fields: [...(prev.fields || []), createEmptyField()],
    }));
  };

  const handleStepChange = (index: number, step: CreateStepRequest) => {
    setFormData((prev) => ({
      ...prev,
      steps: prev.steps.map((s, i) => (i === index ? step : s)),
    }));
  };

  const handleStepDelete = (index: number) => {
    setFormData((prev) => ({
      ...prev,
      steps: prev.steps.filter((_, i) => i !== index),
    }));
  };

  const handleAddStep = () => {
    setFormData((prev) => ({
      ...prev,
      steps: [...prev.steps, createEmptyStep()],
    }));
  };

  const validate = (): boolean => {
    const errors: string[] = [];

    if (!formData.key) {
      errors.push('Key is required - enter a name to generate it');
    }
    if (!formData.name) {
      errors.push('Name is required');
    }
    if (!formData.description) {
      errors.push('Description is required');
    }
    if (!formData.glyph) {
      errors.push('Glyph is required');
    }
    if (formData.steps.length === 0) {
      errors.push('At least one step is required');
    }

    // Validate steps
    formData.steps.forEach((step, i) => {
      if (!step.name || !/^[a-z_]+$/.test(step.name)) {
        errors.push(`Step ${i + 1}: Name must be lowercase with underscores only`);
      }
      if (!step.prompt) {
        errors.push(`Step ${i + 1}: Prompt is required`);
      }
    });

    // Validate fields
    formData.fields?.forEach((field, i) => {
      if (!field.name || !/^[a-z_]+$/.test(field.name)) {
        errors.push(
          `Field ${i + 1}: Name must be lowercase with underscores only`,
        );
      }
      if (!field.description) {
        errors.push(`Field ${i + 1}: Description is required`);
      }
      if (field.field_type === 'enum' && (!field.options || field.options.length === 0)) {
        errors.push(`Field ${i + 1}: Enum fields require at least one option`);
      }
      // Required fields must have a non-falsey default
      if (field.required && !field.default) {
        errors.push(`Field ${i + 1}: Required fields must have a default value`);
      }
    });

    setValidationErrors(errors);
    return errors.length === 0;
  };

  const handleSubmit = async () => {
    if (!validate()) {
      return;
    }

    try {
      if (isEditing && key) {
        await updateIssueType(key, {
          name: formData.name,
          description: formData.description,
          glyph: formData.glyph,
          mode: formData.mode,
          color: formData.color,
          project_required: formData.project_required,
          fields: formData.fields,
          steps: formData.steps,
        });
      } else {
        await createIssueType(formData);
      }
      navigate('..');
    } catch {
      // Error is handled by the hook
    }
  };

  const stepNames = formData.steps.map((s) => s.name).filter(Boolean);
  const isSaving = creating || updating;
  const saveError = createError || updateError;
  const isLoading = (isEditing && loadingIssueType) || (!isEditing && loadingTask && !formInitialized);

  if (isLoading) {
    return (
      <Page themeId="tool">
        <Content>Loading...</Content>
      </Page>
    );
  }

  return (
    <Page themeId="tool">
      <Header
        title={isEditing ? `Edit ${key}` : 'New Issue Type'}
        subtitle="Configure issue type settings"
      />
      <Content>
        <div className={classes.headerRow}>
          <ContentHeader title={isEditing ? 'Edit Issue Type' : 'Create Issue Type'}>
            {isBuiltin && (
              <Chip
                label="Read-only (builtin types cannot be modified)"
                variant="default"
                size="small"
              />
            )}
          </ContentHeader>
          <div className={classes.toggleContainer}>
            <ButtonGroup size="small" variant="outlined">
              <Button
                className={`${classes.toggleButton} ${viewMode === 'simple' ? classes.activeButton : ''}`}
                onClick={() => setViewMode('simple')}
              >
                Simple
              </Button>
              <Button
                className={`${classes.toggleButton} ${viewMode === 'advanced' ? classes.activeButton : ''}`}
                onClick={() => setViewMode('advanced')}
              >
                Advanced
              </Button>
            </ButtonGroup>
          </div>
        </div>

        {validationErrors.length > 0 && (
          <div className={classes.errorBox}>
            <strong>Validation Errors:</strong>
            <ul style={{ margin: '8px 0 0 0', paddingLeft: '20px' }}>
              {validationErrors.map((error, i) => (
                <li key={i}>{error}</li>
              ))}
            </ul>
          </div>
        )}

        {saveError && (
          <div className={classes.errorBox}>
            <strong>Error:</strong> {saveError.message}
          </div>
        )}

        <InfoCard title="Basic Information">
          <div className={classes.grid}>
            <div>
              <label className={classes.label}>Name *</label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => handleNameChange(e.target.value)}
                placeholder="My Custom Type"
                disabled={isBuiltin}
                className={classes.input}
              />
            </div>

            <div>
              <label className={classes.label}>Key (auto-generated)</label>
              <input
                type="text"
                value={formData.key}
                readOnly
                disabled
                placeholder="Generated from name"
                className={classes.input}
              />
            </div>

            {viewMode === 'advanced' && (
              <div>
                <label className={classes.label}>Glyph *</label>
                <select
                  value={formData.glyph}
                  onChange={(e) => handleChange('glyph', e.target.value)}
                  disabled={isBuiltin}
                  className={classes.select}
                >
                  {GLYPH_OPTIONS.map((g) => (
                    <option key={g} value={g}>
                      {g}
                    </option>
                  ))}
                </select>
              </div>
            )}

            <div className={classes.fullWidth}>
              <label className={classes.label}>Description *</label>
              <textarea
                value={formData.description}
                onChange={(e) => handleChange('description', e.target.value)}
                placeholder="Describe what this issue type is for..."
                disabled={isBuiltin}
                rows={2}
                className={classes.textarea}
              />
            </div>

            {viewMode === 'advanced' && (
              <>
                <div>
                  <label className={classes.label}>Mode</label>
                  <select
                    value={formData.mode || 'paired'}
                    onChange={(e) =>
                      handleChange('mode', e.target.value as ExecutionMode)
                    }
                    disabled={isBuiltin}
                    className={classes.select}
                  >
                    {MODE_OPTIONS.map((m) => (
                      <option key={m} value={m}>
                        {m}
                      </option>
                    ))}
                  </select>
                </div>

                <div>
                  <label className={classes.label}>Color</label>
                  <select
                    value={formData.color || ''}
                    onChange={(e) =>
                      handleChange('color', e.target.value || undefined)
                    }
                    disabled={isBuiltin}
                    className={classes.select}
                  >
                    <option value="">None</option>
                    {COLOR_OPTIONS.map((c) => (
                      <option key={c} value={c}>
                        {c}
                      </option>
                    ))}
                  </select>
                </div>

                <div>
                  <label className={classes.checkboxLabel}>
                    <input
                      type="checkbox"
                      checked={formData.project_required !== false}
                      onChange={(e) =>
                        handleChange('project_required', e.target.checked)
                      }
                      disabled={isBuiltin}
                    />
                    Project Required
                  </label>
                </div>
              </>
            )}
          </div>
        </InfoCard>

        <div style={{ marginTop: '16px' }}>
          <InfoCard title="Fields">
            {formData.fields?.map((field, index) => (
              <FieldEditor
                key={index}
                field={field}
                onChange={(f) => handleFieldChange(index, f)}
                onDelete={() => handleFieldDelete(index)}
                index={index}
                showAdvanced={viewMode === 'advanced'}
              />
            ))}
            {!isBuiltin && (
              <button onClick={handleAddField} className={classes.addButton}>
                + Add Field
              </button>
            )}
          </InfoCard>
        </div>

        <div style={{ marginTop: '16px' }}>
          <InfoCard title="Workflow Steps">
            {formData.steps.map((step, index) => (
              <StepEditor
                key={index}
                step={step}
                onChange={(s) => handleStepChange(index, s)}
                onDelete={() => handleStepDelete(index)}
                index={index}
                stepNames={stepNames}
              />
            ))}
            {!isBuiltin && (
              <button onClick={handleAddStep} className={classes.addButton}>
                + Add Step
              </button>
            )}
          </InfoCard>
        </div>

        <div style={{ marginTop: '24px', display: 'flex', gap: '16px' }}>
          {!isBuiltin && (
            <button
              onClick={handleSubmit}
              disabled={isSaving}
              className={classes.submitButton}
            >
              {isSaving
                ? 'Saving...'
                : isEditing
                  ? 'Update Issue Type'
                  : 'Create Issue Type'}
            </button>
          )}
          <button onClick={() => navigate('..')} className={classes.cancelButton}>
            Cancel
          </button>
        </div>
      </Content>
    </Page>
  );
};
