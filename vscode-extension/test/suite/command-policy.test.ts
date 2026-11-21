import * as assert from "node:assert";
import {
  focusMissAction,
  launchClientAction,
  launchWrapper,
  notReadyHeaderCommand,
  presentRemoteCommand,
  remoteCommandTarget,
  ticketOpenAction,
} from "../../src/command-policy";

suite("command policy", () => {
  test("local mode leaves every command on its current handler", () => {
    assert.deepStrictEqual(remoteCommandTarget("operator.runSetup", false), { kind: "local" });
    assert.deepStrictEqual(remoteCommandTarget("operator.downloadOperator", false), {
      kind: "local",
    });
    assert.strictEqual(launchWrapper(false), "vscode");
    assert.strictEqual(launchClientAction(false, false, "claude"), "run-terminal");
    assert.strictEqual(focusMissAction(false), "relaunch");
    assert.strictEqual(ticketOpenAction(false, "FEAT-1"), "file");
  });

  test("remote setup, settings, git, and llm commands open the hosted UI", () => {
    assert.deepStrictEqual(remoteCommandTarget("operator.runSetup", true), {
      kind: "ui",
      route: "setup",
    });
    assert.deepStrictEqual(remoteCommandTarget("operator.openSettings", true), {
      kind: "ui",
      route: "config",
    });
    assert.deepStrictEqual(remoteCommandTarget("operator.openCreateDelegator", true), {
      kind: "ui",
      route: "delegators",
    });
    assert.deepStrictEqual(remoteCommandTarget("operator.detectLlmTools", true), {
      kind: "ui",
      route: "llm",
    });
    assert.deepStrictEqual(remoteCommandTarget("operator.configureGitHub", true), {
      kind: "ui",
      route: "git",
    });
    assert.deepStrictEqual(remoteCommandTarget("operator.configureGitLab", true), {
      kind: "ui",
      route: "git",
    });
    assert.deepStrictEqual(remoteCommandTarget("operator.startGitOnboarding", true), {
      kind: "ui",
      route: "git",
    });
    assert.deepStrictEqual(remoteCommandTarget("operator.checkKanbanConnection", true), {
      kind: "ui",
      route: "kanban",
    });
    assert.deepStrictEqual(remoteCommandTarget("operator.revealTicketsDir", true), {
      kind: "ui",
      route: "queue",
    });
  });

  test("remote local-machine commands refuse instead of touching this laptop", () => {
    const directory = remoteCommandTarget("operator.selectWorkingDirectory", true);
    const webhook = remoteCommandTarget("operator.startWebhookServer", true);
    const download = remoteCommandTarget("operator.downloadOperator", true);
    const start = remoteCommandTarget("operator.startOperatorServer", true);
    assert.strictEqual(directory.kind, "message");
    assert.strictEqual(webhook.kind, "message");
    assert.strictEqual(download.kind, "message");
    assert.strictEqual(start.kind, "message");
    if (directory.kind === "message") {
      assert.strictEqual(directory.chooseConnection, true);
    }
    if (webhook.kind === "message") {
      assert.strictEqual(webhook.chooseConnection, false);
      assert.match(webhook.message, /does not call this laptop/);
    }
  });

  test("an unlisted command stays on the API path", () => {
    assert.deepStrictEqual(remoteCommandTarget("operator.signIn", true), { kind: "local" });
    assert.deepStrictEqual(remoteCommandTarget("operator.pauseQueue", true), { kind: "local" });
  });

  test("a remote launch never becomes a local terminal", () => {
    assert.strictEqual(launchWrapper(true), null);
    assert.strictEqual(launchClientAction(true, true, ""), "server-session");
    assert.strictEqual(launchClientAction(true, false, "claude --print"), "refuse-local-command");
    assert.strictEqual(launchClientAction(true, false, "  "), "server-session");
    assert.strictEqual(launchClientAction(false, true, ""), "server-session");
  });

  test("a missed remote terminal opens the queue, and a ticket opens by id", () => {
    assert.strictEqual(focusMissAction(true), "open-queue");
    assert.strictEqual(ticketOpenAction(true, "FEAT-1"), "ui");
    assert.strictEqual(ticketOpenAction(true, undefined), "unavailable");
  });

  test("a not-ready remote header asks the user to choose a connection", () => {
    assert.strictEqual(notReadyHeaderCommand(true, true), "operator.chooseConnection");
    assert.strictEqual(notReadyHeaderCommand(true, false), "operator.chooseConnection");
    assert.strictEqual(notReadyHeaderCommand(false, true), "operator.runSetup");
    assert.strictEqual(notReadyHeaderCommand(false, false), "operator.selectWorkingDirectory");
  });

  test("presentRemoteCommand opens the route and does not also inform", async () => {
    const opened: string[] = [];
    const notes: string[] = [];
    const handled = await presentRemoteCommand(
      { kind: "ui", route: "setup" },
      {
        openUi: (route) => {
          opened.push(route);
          return Promise.resolve();
        },
        inform: (message) => {
          notes.push(message);
          return Promise.resolve();
        },
      },
    );
    assert.strictEqual(handled, true);
    assert.deepStrictEqual(opened, ["setup"]);
    assert.deepStrictEqual(notes, []);
  });

  test("presentRemoteCommand lets a local target through", async () => {
    const handled = await presentRemoteCommand(
      { kind: "local" },
      {
        openUi: () => Promise.reject(new Error("should not open")),
        inform: () => Promise.reject(new Error("should not inform")),
      },
    );
    assert.strictEqual(handled, false);
  });
});
