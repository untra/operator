import React from 'react';
import { Button, TextInput, SelectInput } from '../primitives';
import { SectionHeader } from '../SectionHeader';
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
    <div className="op-mb-4">
      <SectionHeader id="section-primary" title="Workspace Configuration" />
      <p className="op-body1 op-text-secondary op-mb-1">
        These are settings for <b>Operator!</b> configuration for the VS Code extension. For more details see the <a href="https://operator.untra.io/configuration/">configuration documentation</a>
      </p>

      <div className="op-mb-2">
        <p className="op-body2 op-text-secondary op-mb-05">
          <OperatorBrand /> Working Directory
        </p>
        <div style={{ display: 'flex', gap: 8 }}>
          <TextInput
            style={{ flex: 1 }}
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
            style={{
              alignSelf: 'flex-start',
              marginTop: 8,
              borderColor: 'var(--op-terracotta)',
              color: 'var(--op-terracotta)',
            }}
          >
            change
          </Button>
        </div>
      </div>

      <SelectInput
        label="Session Wrapper"
        value={sessions_wrapper || 'vscode'}
        onChange={(e) =>
          onUpdate('sessions', 'wrapper', e.target.value)
        }
        helperText="Only VS Code Terminal is available when running from the extension. Other wrappers require running Operator from the CLI."
      >
        <option value="vscode">VS Code Terminal</option>
        <option value="tmux" disabled>tmux</option>
        <option value="cmux" disabled>cmux</option>
        <option value="zellij" disabled>zellij</option>
      </SelectInput>
    </div>
  );
}
