import * as assert from "node:assert/strict";
import * as fs from "node:fs/promises";
import * as path from "node:path";
import * as vscode from "vscode";
import { OperatorApiClient } from "../../../src/api-client";
import { requiredEnv } from "../daemon";
import { waitFor } from "../hosted-identity";
import { configureEditor } from "../oidc";

export async function run(): Promise<void> {
  const origin = requiredEnv("OPERATOR_E2E_ORIGIN");
  const host = requiredEnv("OPERATOR_E2E_SSH_HOST");
  await configureEditor(origin);
  await vscode.commands.executeCommand("operator.signIn");
  const dir = requiredEnv("OPERATOR_E2E_TEMP_DIR");
  const revokeAssignment = async () => {
    await fs.writeFile(path.join(dir, "terminal-revoke"), "");
    await waitFor(
      () => fs.stat(path.join(dir, "terminal-revoked")).catch(() => undefined),
      "revoked assignment",
    );
  };
  const agent = "attach-e2e";
  const attached = () =>
    fs.stat(path.join(dir, "terminal-attached")).then(
      () => true,
      () => false,
    );
  const localTerminals = vscode.window.terminals.map((t) => t.name);
  const client = new OperatorApiClient(origin);
  const ref = await client.issueAttach(agent);
  assert.equal(ref.destination.authority, `ssh-remote+${host}`);
  await assert.rejects(
    client.redeemAttach(agent, {
      ...ref,
      destination: { ...ref.destination, path: "/different" },
    }),
  );
  assert.equal(await attached(), false);
  await client.redeemAttach(agent, ref);
  await assert.rejects(client.redeemAttach(agent, ref));
  assert.equal(await attached(), false);
  await vscode.commands.executeCommand("operator.attachAgent", agent);
  await waitFor(async () => (await attached()) || undefined, "remote terminal client");
  assert.ok(
    await attached(),
    "Expected tmux to observe a client attachment in the disposable SSH host",
  );
  const beforeRevocation = await client.issueAttach(agent);
  await revokeAssignment();
  await assert.rejects(client.redeemAttach(agent, beforeRevocation));
  assert.deepEqual(
    vscode.window.terminals.map((t) => t.name),
    localTerminals,
  );
}
