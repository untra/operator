import * as path from "node:path";
import type * as vscode from "vscode";
import { ApiError, AuthRequiredError, OperatorApiClient } from "./api-client";
import { localAdoptMismatch, type ConnectionSelection } from "./connection";
import { getOperatorPath, getOperatorVersion } from "./operator-binary";
import type { ApiStatus } from "./sections/types";

export async function assessConnection(
  selection: ConnectionSelection,
  url: string,
  ticketsDir: string | undefined,
  context: vscode.ExtensionContext,
): Promise<ApiStatus> {
  const client = new OperatorApiClient(url);
  if (!(await client.isReachable())) {
    return { connected: false, url };
  }
  const answered: ApiStatus = { connected: false, url, reachable: true };
  try {
    if (selection.kind === "hosted") {
      answered.subject = (await client.currentSession()).subject;
    }
    let health;
    try {
      health =
        selection.kind === "local"
          ? await client.localConfigurationHealth()
          : await client.health();
    } catch (error) {
      if (selection.kind === "local" && error instanceof ApiError && error.status === 404) {
        return {
          ...answered,
          mismatch: {
            kind: "project",
            detail: "does not serve the selected Operator configuration",
          },
        };
      }
      if (selection.kind === "hosted" && error instanceof ApiError && error.status === 403) {
        return { ...answered, connected: true };
      }
      throw error;
    }
    if (selection.kind === "local") {
      const binary = await getOperatorPath(context);
      const mismatch = localAdoptMismatch({
        remote: false,
        expectedVersion: binary ? await getOperatorVersion(binary) : undefined,
        foundVersion: health.version,
        localDirectoryName: ticketsDir ? path.basename(path.dirname(ticketsDir)) : undefined,
        foundDirectoryName: health.directory_name,
      });
      if (mismatch) {
        return { ...answered, connected: false, mismatch };
      }
    }
    return {
      ...answered,
      connected: true,
      version: health.version,
      directoryName: health.directory_name,
    };
  } catch (error) {
    const mismatch =
      error instanceof AuthRequiredError
        ? { kind: "auth" as const, detail: `at ${url} requires sign-in` }
        : error instanceof ApiError && error.status === 403
          ? {
              kind: "forbidden" as const,
              detail:
                "Your account lacks access to this Operator profile. Contact your administrator.",
            }
          : undefined;
    return { ...answered, connected: false, mismatch };
  }
}
