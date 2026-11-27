import React, { useCallback } from "react";
import { Button, TextInput } from "../primitives";
import { SectionHeader } from "../SectionHeader";
import { useDraftField } from "../../hooks/useDraftField";

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
  const commitWorkingDirectory = useCallback(
    (next: string) => onUpdate("primary", "working_directory", next),
    [onUpdate],
  );
  const workingDirectoryDraft = useDraftField(working_directory, commitWorkingDirectory);
  const handleBrowseFolder = useCallback(
    () => onBrowseFolder("workingDirectory"),
    [onBrowseFolder],
  );
  return (
    <div className="op-mb-4">
      <SectionHeader id="section-primary" title="Workspace Configuration" />
      <p className="op-body1 op-text-secondary op-mb-1">
        These are settings for <b>Operator!</b> configuration for the VS Code extension. For more
        details see the{" "}
        <a href="https://operator.untra.io/configuration/">configuration documentation</a>
      </p>

      <div className="op-mb-2">
        <div className="op-folder-picker">
          <TextInput
            className="op-flex-1"
            label="Working directory"
            {...workingDirectoryDraft}
            placeholder="/path/to/your/repos"
            helperText="Parent directory of Operator! managed code repositories containing .tickets/ working directory"
          />
          <Button variant="outlined" onClick={handleBrowseFolder} className="op-browse-folder">
            Browse…
          </Button>
        </div>
      </div>

      <p>Extension launches use the VS Code terminal.</p>
      {sessions_wrapper !== "vscode" && <p>CLI session wrapper: {sessions_wrapper}</p>}
    </div>
  );
}
