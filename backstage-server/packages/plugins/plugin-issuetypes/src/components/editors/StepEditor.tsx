/**
 * Step Editor Component
 *
 * Form for editing a single workflow step within an issue type.
 * Uses theme-aware styling for dark mode support.
 */

import React from 'react';
import { makeStyles, alpha } from '@material-ui/core';
import type {
  CreateStepRequest,
  StepOutput,
  PermissionMode,
} from '../../api/types';
import { STEP_OUTPUTS, ALLOWED_TOOLS } from '../../api/types';

const PERMISSION_MODES: PermissionMode[] = [
  'default',
  'plan',
  'acceptEdits',
  'delegate',
];

const useStyles = makeStyles((theme) => ({
  container: {
    border: `1px solid ${theme.palette.divider}`,
    borderRadius: 8,
    padding: 16,
    marginBottom: 16,
    backgroundColor: theme.palette.background.default,
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 16,
  },
  title: {
    margin: 0,
    color: theme.palette.text.primary,
  },
  removeButton: {
    padding: '4px 12px',
    backgroundColor: theme.palette.error.main,
    color: theme.palette.error.contrastText,
    border: 'none',
    borderRadius: 4,
    cursor: 'pointer',
    '&:hover': {
      backgroundColor: theme.palette.error.dark,
    },
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: 16,
  },
  fullWidth: {
    gridColumn: '1 / -1',
  },
  label: {
    display: 'block',
    marginBottom: 4,
    color: theme.palette.text.primary,
  },
  sectionLabel: {
    display: 'block',
    marginBottom: 8,
    color: theme.palette.text.primary,
  },
  checkboxLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
    color: theme.palette.text.primary,
    cursor: 'pointer',
  },
  helperText: {
    color: theme.palette.text.secondary,
    display: 'block',
    marginTop: 4,
    fontSize: '0.75rem',
  },
  input: {
    width: '100%',
    padding: 8,
    border: `1px solid ${theme.palette.divider}`,
    borderRadius: 4,
    backgroundColor: theme.palette.background.paper,
    color: theme.palette.text.primary,
    fontSize: 14,
    '&::placeholder': {
      color: theme.palette.text.secondary,
      opacity: 0.7,
    },
    '&:focus': {
      outline: 'none',
      borderColor: theme.palette.primary.main,
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
    fontFamily: 'monospace',
    resize: 'vertical',
    '&::placeholder': {
      color: theme.palette.text.secondary,
      opacity: 0.7,
    },
    '&:focus': {
      outline: 'none',
      borderColor: theme.palette.primary.main,
    },
  },
  select: {
    width: '100%',
    padding: 8,
    border: `1px solid ${theme.palette.divider}`,
    borderRadius: 4,
    backgroundColor: theme.palette.background.paper,
    color: theme.palette.text.primary,
    fontSize: 14,
    '&:focus': {
      outline: 'none',
      borderColor: theme.palette.primary.main,
    },
  },
  chipContainer: {
    display: 'flex',
    flexWrap: 'wrap',
    gap: 8,
  },
  chip: {
    display: 'flex',
    alignItems: 'center',
    gap: 4,
    padding: '4px 8px',
    borderRadius: 4,
    cursor: 'pointer',
    color: theme.palette.text.primary,
    transition: 'background-color 0.15s ease',
  },
  chipUnselected: {
    backgroundColor: theme.palette.action.hover,
  },
  outputChipSelected: {
    backgroundColor: alpha(theme.palette.primary.main, 0.15),
  },
  toolChipSelected: {
    backgroundColor: alpha(theme.palette.success.main, 0.15),
  },
}));

export interface StepEditorProps {
  step: CreateStepRequest;
  onChange: (step: CreateStepRequest) => void;
  onDelete: () => void;
  index: number;
  stepNames: string[]; // All step names for next_step dropdown
}

export const StepEditor: React.FC<StepEditorProps> = ({
  step,
  onChange,
  onDelete,
  index,
  stepNames,
}) => {
  const classes = useStyles();

  const handleChange = <K extends keyof CreateStepRequest>(
    key: K,
    value: CreateStepRequest[K],
  ) => {
    onChange({ ...step, [key]: value });
  };

  const handleOutputToggle = (output: StepOutput) => {
    const current = step.outputs || [];
    const newOutputs = current.includes(output)
      ? current.filter((o) => o !== output)
      : [...current, output];
    handleChange('outputs', newOutputs as StepOutput[]);
  };

  const handleToolToggle = (tool: string) => {
    const current = step.allowed_tools || [];
    const newTools = current.includes(tool)
      ? current.filter((t) => t !== tool)
      : [...current, tool];
    handleChange('allowed_tools', newTools);
  };

  // Filter out current step from next_step options
  const availableNextSteps = stepNames.filter((name) => name !== step.name);

  return (
    <div className={classes.container}>
      <div className={classes.header}>
        <h4 className={classes.title}>
          Step {index + 1}
          {step.name && `: ${step.name}`}
        </h4>
        <button onClick={onDelete} className={classes.removeButton}>
          Remove
        </button>
      </div>

      <div className={classes.grid}>
        <div>
          <label className={classes.label}>Name *</label>
          <input
            type="text"
            value={step.name}
            onChange={(e) => handleChange('name', e.target.value)}
            placeholder="step_name"
            pattern="^[a-z_]+$"
            className={classes.input}
          />
        </div>

        <div>
          <label className={classes.label}>Display Name</label>
          <input
            type="text"
            value={step.display_name || ''}
            onChange={(e) =>
              handleChange('display_name', e.target.value || undefined)
            }
            placeholder="Human-readable name"
            className={classes.input}
          />
        </div>

        <div className={classes.fullWidth}>
          <label className={classes.label}>Prompt *</label>
          <textarea
            value={step.prompt}
            onChange={(e) => handleChange('prompt', e.target.value)}
            placeholder="Instructions for the agent..."
            rows={4}
            className={classes.textarea}
          />
          <small className={classes.helperText}>
            Supports Handlebars templates: {'{{ id }}'}, {'{{ project }}'},{' '}
            {'{{ summary }}'}
          </small>
        </div>

        <div>
          <label className={classes.label}>Permission Mode</label>
          <select
            value={step.permission_mode || 'default'}
            onChange={(e) =>
              handleChange('permission_mode', e.target.value as PermissionMode)
            }
            className={classes.select}
          >
            {PERMISSION_MODES.map((mode) => (
              <option key={mode} value={mode}>
                {mode}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className={classes.label}>Next Step</label>
          <select
            value={step.next_step || ''}
            onChange={(e) =>
              handleChange('next_step', e.target.value || undefined)
            }
            className={classes.select}
          >
            <option value="">(End of workflow)</option>
            {availableNextSteps.map((name) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </select>
        </div>

        <div>
          <label className={classes.checkboxLabel}>
            <input
              type="checkbox"
              checked={step.requires_review || false}
              onChange={(e) => handleChange('requires_review', e.target.checked)}
            />
            Requires Review
          </label>
          <small className={classes.helperText}>
            Pause workflow for human approval
          </small>
        </div>

        {step.requires_review && (
          <div>
            <label className={classes.label}>On Reject (go to step)</label>
            <select
              value={step.on_reject?.goto_step || ''}
              onChange={(e) =>
                handleChange(
                  'on_reject',
                  e.target.value
                    ? { goto_step: e.target.value }
                    : undefined,
                )
              }
              className={classes.select}
            >
              <option value="">(End workflow on reject)</option>
              {stepNames.map((name) => (
                <option key={name} value={name}>
                  {name}
                </option>
              ))}
            </select>
          </div>
        )}

        <div className={classes.fullWidth}>
          <label className={classes.sectionLabel}>Outputs</label>
          <div className={classes.chipContainer}>
            {STEP_OUTPUTS.map((output) => {
              const isSelected = (step.outputs || []).includes(output);
              return (
                <label
                  key={output}
                  className={`${classes.chip} ${
                    isSelected ? classes.outputChipSelected : classes.chipUnselected
                  }`}
                >
                  <input
                    type="checkbox"
                    checked={isSelected}
                    onChange={() => handleOutputToggle(output)}
                  />
                  {output}
                </label>
              );
            })}
          </div>
        </div>

        <div className={classes.fullWidth}>
          <label className={classes.sectionLabel}>Allowed Tools</label>
          <div className={classes.chipContainer}>
            {ALLOWED_TOOLS.map((tool) => {
              const isSelected = (step.allowed_tools || []).includes(tool);
              return (
                <label
                  key={tool}
                  className={`${classes.chip} ${
                    isSelected ? classes.toolChipSelected : classes.chipUnselected
                  }`}
                >
                  <input
                    type="checkbox"
                    checked={isSelected}
                    onChange={() => handleToolToggle(tool)}
                  />
                  {tool}
                </label>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
};
