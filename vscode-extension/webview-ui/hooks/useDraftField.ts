import type React from "react";
import { useCallback, useState } from "react";
import { initialDraft, isDirty, syncDraft, withDraft } from "../state/draftField";

const COMMIT_KEY = "Enter";

export interface DraftFieldProps {
  value: string;
  onChange: React.ChangeEventHandler<HTMLInputElement>;
  onBlur: React.FocusEventHandler<HTMLInputElement>;
  onKeyDown: React.KeyboardEventHandler<HTMLInputElement>;
}

const acceptAll = (): boolean => true;

/**
 * Local edit buffer for a config field: keystrokes stay in the webview and
 * `commit` runs once on blur or Enter, instead of a host round-trip per key.
 * A draft failing `isValid` is discarded and the committed value restored.
 */
export function useDraftField(
  value: string,
  commit: (next: string) => void,
  isValid: (next: string) => boolean = acceptAll,
): DraftFieldProps {
  const [stored, setStored] = useState(() => initialDraft(value));
  const state = syncDraft(stored, value);
  if (state !== stored) {
    setStored(state);
  }

  const onChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const draft = event.target.value;
      setStored((prev) => withDraft(syncDraft(prev, value), draft));
    },
    [value],
  );

  const flush = useCallback(() => {
    if (!isDirty(state)) {
      return;
    }
    if (isValid(state.draft)) {
      commit(state.draft);
      setStored(initialDraft(state.draft));
    } else {
      setStored((prev) => withDraft(prev, prev.synced));
    }
  }, [commit, isValid, state]);

  const onKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLInputElement>) => {
      if (event.key === COMMIT_KEY) {
        flush();
      }
    },
    [flush],
  );

  return { value: state.draft, onChange, onBlur: flush, onKeyDown };
}
