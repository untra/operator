/**
 * A config snapshot is stale when it was produced before the webview's most
 * recent `updateConfig`; applying it would clobber newer optimistic edits.
 * Snapshots without a rev (e.g. from older hosts) are always applied.
 */
export function isStaleSnapshot(snapshotRev: number | undefined, lastSentRev: number): boolean {
  return snapshotRev !== undefined && snapshotRev < lastSentRev;
}

/** Webview→host messages are JSON-serialized, which cannot encode bigint. */
export function toWireValue(value: unknown): unknown {
  return typeof value === "bigint" ? Number(value) : value;
}

export type ConfigErrorSource = "write" | "load";

export interface ConfigErrorState {
  readonly message: string;
  readonly source: ConfigErrorSource;
}

export type ConfigSnapshotType = "configLoaded" | "configUpdated";

/** Host errors stamped with a rev came from a failed `updateConfig`. */
export function configErrorFrom(message: string, rev: number | undefined): ConfigErrorState {
  return { message, source: rev === undefined ? "load" : "write" };
}

/**
 * Only a successful write clears a write error; the `configLoaded` that rolls
 * back a failed write must leave it visible. A snapshot's arrival proves the
 * host succeeded, so this applies whether or not the snapshot itself is stale.
 */
export function errorAfterSnapshot(
  error: ConfigErrorState | null,
  snapshot: ConfigSnapshotType,
): ConfigErrorState | null {
  if (error === null || snapshot === "configUpdated" || error.source === "load") {
    return null;
  }
  return error;
}
