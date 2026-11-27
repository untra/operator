import * as assert from "node:assert/strict";
import * as fs from "node:fs/promises";
import * as http from "node:http";
import { once } from "node:events";
import type { AddressInfo } from "node:net";
import * as path from "node:path";
import * as vscode from "vscode";
import { credentialProvider } from "../../../src/auth/credentials";
import { OperatorApiClient } from "../../../src/api-client";
import { startLocal } from "../daemon";

export async function run(): Promise<void> {
  const daemon = await startLocal();
  let observations = 0;
  const other = http.createServer((request, response) => {
    observations++;
    response
      .writeHead(request.headers.authorization ? 403 : 200, { "content-type": "application/json" })
      .end("{}");
  });
  try {
    await vscode.extensions.getExtension("untra.operator-terminals")!.activate();
    await vscode.commands.executeCommand("operator.chooseLocalConnection");
    const checkQueue = async () => {
      await vscode.commands.executeCommand("operator.refreshTickets");
      const board = await new OperatorApiClient(daemon.origin).getKanban();
      assert.ok(board.queue.some((card) => card.id === "TASK-alpha"));
    };
    await checkQueue();
    const initial = await credentialProvider().bearer(daemon.origin);
    assert.ok(initial);
    await daemon.restart();
    await checkQueue();
    const current = await credentialProvider().bearer(daemon.origin);
    assert.ok(current && current !== initial);
    other.listen(0, "127.0.0.1");
    await once(other, "listening");
    const otherOrigin = `http://127.0.0.1:${(other.address() as AddressInfo).port}`;
    await new OperatorApiClient(otherOrigin).health();
    assert.equal(observations, 1);
    assert.ok(
      (await fs.readFile(path.join(daemon.stateDir, "local-token"), "utf8")) === current,
      "Extension credential did not rotate with the daemon",
    );
  } finally {
    other.close();
    await daemon.stop();
  }
}
