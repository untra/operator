import { useState } from "react";
import { EMPTY_STABLE_KEYS, reconcileStableKeys } from "../state/stableKeys";

/** Render keys that stay stable when a map key is renamed in place. */
export function useStableKeys(keys: readonly string[]): ReadonlyMap<string, string> {
  const [stored, setStored] = useState(() => reconcileStableKeys(EMPTY_STABLE_KEYS, keys));
  const state = reconcileStableKeys(stored, keys);
  if (state !== stored) {
    setStored(state);
  }
  return state.ids;
}
