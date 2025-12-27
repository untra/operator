/**
 * Field Editor Component
 *
 * Form for editing a single field within an issue type.
 * Uses theme-aware styling for dark mode support.
 */

import React from 'react';
import { makeStyles } from '@material-ui/core';
import type { CreateFieldRequest, FieldType } from '../../api/types';

const FIELD_TYPES: FieldType[] = ['string', 'text', 'enum', 'bool', 'date'];

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
  checkboxLabel: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
    color: theme.palette.text.primary,
    cursor: 'pointer',
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
}));

export interface FieldEditorProps {
  field: CreateFieldRequest;
  onChange: (field: CreateFieldRequest) => void;
  onDelete: () => void;
  index: number;
  /** Whether to show advanced options like user_editable */
  showAdvanced?: boolean;
}

export const FieldEditor: React.FC<FieldEditorProps> = ({
  field,
  onChange,
  onDelete,
  index,
  showAdvanced = true,
}) => {
  const classes = useStyles();

  const handleChange = (
    key: keyof CreateFieldRequest,
    value: string | boolean | string[] | number | undefined,
  ) => {
    onChange({ ...field, [key]: value });
  };

  const handleOptionsChange = (optionsStr: string) => {
    const options = optionsStr
      .split(',')
      .map((o) => o.trim())
      .filter(Boolean);
    handleChange('options', options);
  };

  return (
    <div className={classes.container}>
      <div className={classes.header}>
        <h4 className={classes.title}>Field {index + 1}</h4>
        <button onClick={onDelete} className={classes.removeButton}>
          Remove
        </button>
      </div>

      <div className={classes.grid}>
        <div>
          <label className={classes.label}>Name *</label>
          <input
            type="text"
            value={field.name}
            onChange={(e) => handleChange('name', e.target.value)}
            placeholder="field_name"
            pattern="^[a-z_]+$"
            className={classes.input}
          />
        </div>

        <div>
          <label className={classes.label}>Type</label>
          <select
            value={field.field_type || 'string'}
            onChange={(e) => handleChange('field_type', e.target.value as FieldType)}
            className={classes.select}
          >
            {FIELD_TYPES.map((type) => (
              <option key={type} value={type}>
                {type}
              </option>
            ))}
          </select>
        </div>

        <div className={classes.fullWidth}>
          <label className={classes.label}>Description *</label>
          <input
            type="text"
            value={field.description}
            onChange={(e) => handleChange('description', e.target.value)}
            placeholder="Description of this field"
            className={classes.input}
          />
        </div>

        <div>
          <label className={classes.checkboxLabel}>
            <input
              type="checkbox"
              checked={field.required || false}
              onChange={(e) => handleChange('required', e.target.checked)}
            />
            Required
          </label>
        </div>

        {showAdvanced && (
          <div>
            <label className={classes.checkboxLabel}>
              <input
                type="checkbox"
                checked={field.user_editable !== false}
                onChange={(e) => handleChange('user_editable', e.target.checked)}
              />
              User Editable
            </label>
          </div>
        )}

        <div>
          <label className={classes.label}>Default Value</label>
          <input
            type="text"
            value={field.default || ''}
            onChange={(e) => handleChange('default', e.target.value || undefined)}
            placeholder="Default value"
            className={classes.input}
          />
        </div>

        <div>
          <label className={classes.label}>Placeholder</label>
          <input
            type="text"
            value={field.placeholder || ''}
            onChange={(e) =>
              handleChange('placeholder', e.target.value || undefined)
            }
            placeholder="Placeholder text"
            className={classes.input}
          />
        </div>

        {field.field_type === 'enum' && (
          <div className={classes.fullWidth}>
            <label className={classes.label}>Options (comma-separated) *</label>
            <input
              type="text"
              value={(field.options || []).join(', ')}
              onChange={(e) => handleOptionsChange(e.target.value)}
              placeholder="option1, option2, option3"
              className={classes.input}
            />
          </div>
        )}

        {(field.field_type === 'string' || field.field_type === 'text') && (
          <div>
            <label className={classes.label}>Max Length</label>
            <input
              type="number"
              value={field.max_length || ''}
              onChange={(e) =>
                handleChange(
                  'max_length',
                  e.target.value ? parseInt(e.target.value, 10) : undefined,
                )
              }
              placeholder="No limit"
              min={1}
              className={classes.input}
            />
          </div>
        )}
      </div>
    </div>
  );
};
