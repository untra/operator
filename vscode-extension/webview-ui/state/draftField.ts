/** `synced` is the last external value seen; the draft is dirty while it differs. */
export interface DraftState {
  readonly draft: string;
  readonly synced: string;
}

export function initialDraft(value: string): DraftState {
  return { draft: value, synced: value };
}

export function withDraft(state: DraftState, draft: string): DraftState {
  return { ...state, draft };
}

export function isDirty(state: DraftState): boolean {
  return state.draft !== state.synced;
}

/**
 * Reconcile a new external value: a clean draft follows it (including a
 * rollback), a dirty draft keeps the user's in-progress text.
 */
export function syncDraft(state: DraftState, value: string): DraftState {
  if (value === state.synced) {
    return state;
  }
  return { synced: value, draft: isDirty(state) ? state.draft : value };
}
