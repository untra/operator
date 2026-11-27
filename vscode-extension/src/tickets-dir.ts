import { existsSync } from "node:fs";
import { resolveTicketsDirectory, resolveWorkingDirectory } from "./config-paths";

export function findTicketsDir(): Promise<string | undefined> {
  const directory = resolveTicketsDirectory();
  return Promise.resolve(directory && existsSync(directory) ? directory : undefined);
}

export { findTicketsDir as findParentTicketsDir };

export function findOperatorServerDir(): Promise<string | undefined> {
  return Promise.resolve(resolveWorkingDirectory() || undefined);
}
