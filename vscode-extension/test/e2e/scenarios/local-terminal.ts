import * as assert from "node:assert/strict";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import * as vscode from "vscode";
import { credentialProvider } from "../../../src/auth/credentials";
import { startLocal } from "../daemon";
import { TerminalManager } from "../../../src/terminal-manager";
import { WebhookServer } from "../../../src/webhook-server";
import type { SessionInfo } from "../../../src/types";

async function registration(server: WebhookServer): Promise<SessionInfo> {
  const file = server.getRegistrationPath();
  assert.ok(file);
  if (process.platform !== "win32") {
    assert.equal((await fs.stat(file)).mode & 0o777, 0o600);
  }
  return JSON.parse(await fs.readFile(file, "utf8")) as SessionInfo;
}

async function verifyWindowIsolation(): Promise<void> {
  const dir = await fs.mkdtemp(path.join(os.tmpdir(), "operator-terminal-"));
  const platform = process.platform === "darwin" ? "osx" : "linux";
  if (process.platform !== "win32") {
    const config = vscode.workspace.getConfiguration("terminal.integrated");
    await config.update(
      `profiles.${platform}`,
      { "Operator E2E": { path: "/bin/bash", args: ["--noprofile", "--norc"] } },
      vscode.ConfigurationTarget.Global,
    );
    await config.update(
      `defaultProfile.${platform}`,
      "Operator E2E",
      vscode.ConfigurationTarget.Global,
    );
  }
  const manager = new TerminalManager();
  const first = new WebhookServer(manager, dir);
  const otherManager = new TerminalManager();
  const second = new WebhookServer(otherManager, dir);
  try {
    await first.start(dir);
    await second.start(dir);
    const [one, two] = await Promise.all([registration(first), registration(second)]);
    assert.notEqual(one.bearer, two.bearer);
    const name = "operator-terminal-e2e";
    const url = `http://127.0.0.1:${one.port}/terminal/create`;
    const marker = path.join(dir, "operator-marker.txt");
    const body = JSON.stringify({ name, workingDir: dir, env: { OPERATOR_E2E_MARKER: marker } });
    const headers = { Authorization: `Bearer ${one.bearer}` };
    for (const denied of [
      {},
      { Authorization: `Bearer ${two.bearer}` },
      { ...headers, Origin: "https://example.test" },
    ]) {
      const response = await fetch(url, {
        method: "POST",
        headers: denied as Record<string, string>,
        body,
      });
      assert.ok([401, 403].includes(response.status));
      assert.equal(manager.exists(name), false);
    }
    assert.equal((await fetch(url, { method: "POST", headers, body })).status, 200);
    assert.ok(manager.exists(name));
    const isolated = await fetch(`http://127.0.0.1:${two.port}/terminal/${name}/exists`, {
      headers: { Authorization: `Bearer ${two.bearer}` },
    });
    assert.equal(isolated.status, 200);
    assert.deepEqual(await isolated.json(), { exists: false });
    const started = Date.now() + 15000;
    let terminal = vscode.window.terminals.find((t) => t.name === name);
    while (!terminal && Date.now() < started) {
      await delay(100);
      terminal = vscode.window.terminals.find((t) => t.name === name);
    }
    assert.ok(terminal && (await terminal.processId), "VS Code must start the terminal shell");
    const send = await fetch(`http://127.0.0.1:${one.port}/terminal/${name}/send`, {
      method: "POST",
      headers,
      body: JSON.stringify({
        command:
          process.platform === "win32"
            ? "Set-Content -LiteralPath $env:OPERATOR_E2E_MARKER -Value connected"
            : 'echo connected > "$OPERATOR_E2E_MARKER"',
      }),
    });
    assert.equal(send.status, 200);
    const deadline = Date.now() + 15000;
    while (Date.now() < deadline && !(await fs.stat(marker).catch(() => undefined))) {
      await delay(100);
    }
    assert.match(await fs.readFile(marker, "utf8"), /connected/);
    const oldFile = first.getRegistrationPath();
    await first.stop();
    assert.ok(oldFile);
    await assert.rejects(fs.stat(oldFile));
    await first.start(dir);
    const next = await registration(first);
    assert.notEqual(next.bearer, one.bearer);
    assert.equal(
      (await fetch(`http://127.0.0.1:${next.port}/terminal/list`, { headers })).status,
      401,
    );
    assert.equal(
      (
        await fetch(`http://127.0.0.1:${two.port}/terminal/list`, {
          headers: { Authorization: `Bearer ${two.bearer}` },
        })
      ).status,
      200,
    );
  } finally {
    await Promise.all([first.stop(), second.stop()]);
    manager.dispose();
    otherManager.dispose();
    await fs.rm(dir, { recursive: true, force: true });
  }
}

export async function run(): Promise<void> {
  await verifyWindowIsolation();
  const daemon = await startLocal();
  try {
    await vscode.extensions.getExtension("untra.operator-terminals")!.activate();
    await vscode.commands.executeCommand("operator.chooseLocalConnection");
    const registrations = path.join(os.homedir(), ".operator", "vscode-sessions");
    const deadline = Date.now() + 15000;
    let active: SessionInfo | undefined;
    while (!active && Date.now() < deadline) {
      for (const file of await fs.readdir(registrations).catch(() => [])) {
        const candidate = JSON.parse(
          await fs.readFile(path.join(registrations, file), "utf8"),
        ) as SessionInfo;
        if ((await fs.realpath(candidate.workspace)) === (await fs.realpath(daemon.workspace))) {
          active = candidate;
        }
      }
      if (!active) {
        await delay(100);
      }
    }
    assert.ok(active, "The connected local window must register its webhook");
    const name = "operator-local-focus";
    const control = `http://127.0.0.1:${active.port}`;
    const headers = { Authorization: `Bearer ${active.bearer}` };
    assert.equal(
      (
        await fetch(`${control}/terminal/create`, {
          method: "POST",
          headers,
          body: JSON.stringify({ name, workingDir: daemon.workspace }),
        })
      ).status,
      200,
    );
    const now = new Date().toISOString();
    await fs.writeFile(
      path.join(daemon.stateDir, "state.json"),
      JSON.stringify({
        paused: true,
        completed: [],
        agents: [
          {
            id: "local-focus",
            ticket_id: "TASK-alpha",
            ticket_type: "TASK",
            project: "alpha",
            status: "running",
            started_at: now,
            last_activity: now,
            paired: false,
            session_wrapper: "vscode",
            session_name: name,
          },
        ],
      }),
    );
    const bearer = await credentialProvider().bearer(daemon.origin);
    assert.ok(bearer && bearer !== active.bearer);
    assert.equal(
      (
        await fetch(`${daemon.origin}/api/v1/agents/local-focus/focus`, {
          method: "POST",
          headers: { Authorization: `Bearer ${bearer}` },
        })
      ).status,
      200,
    );
    assert.equal(vscode.window.activeTerminal?.name, name);
    await vscode.commands.executeCommand("operator.disconnectWorkspace");
    const timeout = Date.now() + 10000;
    while (Date.now() < timeout && (await fetch(`${control}/health`).catch(() => undefined))) {
      await delay(100);
    }
    await assert.rejects(fetch(`${control}/health`));
  } finally {
    await daemon.stop();
  }
}
