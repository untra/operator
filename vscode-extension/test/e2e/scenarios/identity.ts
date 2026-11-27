import assert from "node:assert/strict";
import * as fs from "node:fs/promises";
import * as path from "node:path";
import * as vscode from "vscode";
import { credentialProvider } from "../../../src/auth/credentials";
import { OperatorApiClient } from "../../../src/api-client";
import { configureEditor } from "../oidc";
import { requiredEnv } from "../daemon";
import { waitFor } from "../hosted-identity";

export async function run(): Promise<void> {
  const origin = requiredEnv("OPERATOR_E2E_ORIGIN");
  const first = requiredEnv("OPERATOR_E2E_IDENTITY_PHASE") === "sign-in";
  if (first) {
    await configureEditor(origin);
    await vscode.commands.executeCommand("operator.signIn");
  } else {
    await vscode.extensions.getExtension("untra.operator-terminals")!.activate();
  }
  const client = new OperatorApiClient(origin);
  assert.equal((await client.currentSession()).subject, requiredEnv("OPERATOR_E2E_SUBJECT"));
  assert.equal((await client.getTicket("TASK-alpha")).project, "alpha");
  await assert.rejects(client.getTicket("TASK-beta"));
  if (first) {
    return;
  }
  assert.ok(await credentialProvider().refresh(origin));
  assert.equal((await client.currentSession()).subject, requiredEnv("OPERATOR_E2E_SUBJECT"));
  await assert.rejects(client.getTicket("TASK-beta"));
  const dir = requiredEnv("OPERATOR_E2E_TEMP_DIR");
  await fs.writeFile(path.join(dir, "identity-refreshed"), "");
  const wait = (name: string) =>
    waitFor(async () => fs.stat(path.join(dir, name)).catch(() => undefined), name);
  await wait("identity-unassigned");
  await assert.rejects(client.getTicket("TASK-alpha"));
  await wait("identity-disabled");
  await assert.rejects(client.currentSession());
}
