export interface StableKeyState {
  readonly ids: ReadonlyMap<string, string>;
  readonly nextSeq: number;
}

export const EMPTY_STABLE_KEYS: StableKeyState = { ids: new Map(), nextSeq: 0 };

/**
 * Map each key to an id that survives renames: when exactly one key vanished
 * and exactly one appeared, the new key inherits the old id. Returns `state`
 * itself when nothing changed so callers can detect a no-op by identity.
 */
export function reconcileStableKeys(
  state: StableKeyState,
  keys: readonly string[],
): StableKeyState {
  const added = keys.filter((key) => !state.ids.has(key));
  const removed = [...state.ids.keys()].filter((key) => !keys.includes(key));
  if (added.length === 0 && removed.length === 0) {
    return state;
  }

  const [renamedFrom] = removed;
  const [renamedTo] = added;
  const isRename = added.length === 1 && removed.length === 1 && renamedFrom !== undefined;

  const ids = new Map<string, string>();
  let nextSeq = state.nextSeq;
  for (const key of keys) {
    const inherited = isRename && key === renamedTo ? state.ids.get(renamedFrom) : undefined;
    const id = state.ids.get(key) ?? inherited ?? String(nextSeq++);
    ids.set(key, id);
  }
  return { ids, nextSeq };
}
