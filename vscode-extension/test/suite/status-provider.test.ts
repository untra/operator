/**
 * Tests for status-provider.ts
 *
 * Tests webhook status icon rendering and working directory item behavior.
 * Uses real temp directories for file-system-dependent checks and sinon
 * stubs for external dependencies (network, binary discovery, etc.).
 */

import * as assert from "node:assert";
import * as sinon from "sinon";
import * as vscode from "vscode";
import fs from "node:fs/promises";
import * as path from "node:path";
import * as os from "node:os";
import type { StatusItem } from "../../src/status-provider";
import { StatusTreeProvider } from "../../src/status-provider";
import { resetConnection, setConnectionOrigin } from "../../src/connection";
import * as configPaths from "../../src/config-paths";
import * as walkthrough from "../../src/walkthrough";
import * as operatorBinary from "../../src/operator-binary";
import * as apiClient from "../../src/api-client";
import * as mcpConnect from "../../src/mcp-connect";

const EMPTY_STUB = {};

/**
 * Create a mock ExtensionContext with stubbed globalState
 */
function createMockContext(
  sandbox: sinon.SinonSandbox,
  workingDir?: string,
): vscode.ExtensionContext {
  const getStub = sandbox.stub();
  getStub.withArgs("operator.workingDirectory").returns(workingDir ?? "");

  return {
    globalState: {
      get: getStub,
      update: sandbox.stub().resolves(),
      keys: sandbox.stub().returns([]),
      setKeysForSync: sandbox.stub(),
    },
    subscriptions: [],
    extensionPath: "/fake/extension",
    extensionUri: vscode.Uri.file("/fake/extension"),
    globalStorageUri: vscode.Uri.file("/fake/storage"),
    storageUri: vscode.Uri.file("/fake/workspace-storage"),
    logUri: vscode.Uri.file("/fake/log"),
    extensionMode: vscode.ExtensionMode.Test,
    extension: EMPTY_STUB as vscode.Extension<unknown>,
    environmentVariableCollection: EMPTY_STUB as vscode.GlobalEnvironmentVariableCollection,
    secrets: EMPTY_STUB as vscode.SecretStorage,
    storagePath: "/fake/workspace-storage",
    globalStoragePath: "/fake/storage",
    logPath: "/fake/log",
    asAbsolutePath: (p: string) => p,
    languageModelAccessInformation: EMPTY_STUB as vscode.LanguageModelAccessInformation,
  } as unknown as vscode.ExtensionContext;
}

/** Helper to find a child item by label */
function findChild(items: StatusItem[], label: string): StatusItem | undefined {
  return items.find((item) => item.label === label);
}

/** Helper to extract section labels from top-level items */
function getSectionLabels(items: StatusItem[]): string[] {
  return items.map((item) => item.label as string);
}

suite("Status Provider Test Suite", () => {
  let sandbox: sinon.SinonSandbox;
  let tempDir: string;

  setup(async () => {
    sandbox = sinon.createSandbox();
    resetConnection({ kind: "local" });
    setConnectionOrigin("http://localhost:7008");

    // Create temp directory for session files
    tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "status-provider-test-"));

    // Stub external dependencies that make network calls or spawn processes
    sandbox.stub(walkthrough, "detectInstalledLlmTools").resolves([]);
    sandbox.stub(walkthrough, "getKanbanWorkspaces").resolves([]);
    sandbox.stub(operatorBinary, "getOperatorPath").resolves(undefined);
    sandbox.stub(operatorBinary, "getOperatorVersion").resolves(undefined);
    sandbox.stub(apiClient, "discoverApiUrl").resolves("http://localhost:7008");
    sandbox.stub(global, "fetch").rejects(new Error("no network in tests"));
  });

  teardown(async () => {
    sandbox.restore();
    resetConnection({ kind: "undecided" });
    try {
      await fs.rm(tempDir, { recursive: true });
    } catch {
      // Ignore cleanup errors
    }
  });

  test("overlapping refreshes read each config once and publish only the newer tree", async () => {
    const olderPath = path.join(tempDir, "older.toml");
    const newerPath = path.join(tempDir, "newer.toml");
    const configPath = sandbox.stub(configPaths, "getResolvedConfigPath").returns(olderPath);
    sandbox.stub(configPaths, "configFileExists").resolves(true);
    sandbox.stub(configPaths, "resolveWorkingDirectory").returns(tempDir);

    const olderRead = sinon.promise<string>();
    const newerRead = sinon.promise<string>();
    const readFile = sandbox.stub(fs, "readFile").callThrough();
    const olderCalls = readFile.withArgs(olderPath, "utf-8").returns(olderRead);
    const newerCalls = readFile.withArgs(newerPath, "utf-8").returns(newerRead);

    const provider = new StatusTreeProvider(createMockContext(sandbox, tempDir));
    provider.setWebhookServer({ isRunning: () => true, getPort: () => 7009 });
    const published = sandbox.spy();
    provider.onDidChangeTreeData(published);

    const older = provider.setTicketsDir(tempDir);
    assert.ok(olderCalls.called);
    configPath.returns(newerPath);
    const newer = provider.refresh();
    await newerRead.resolve('[sessions]\nwrapper = "tmux"\n[git]\nprovider = "gitlab"');

    const assertNewerTree = () => {
      const items = provider.getChildren();
      assert.strictEqual(findChild(items, "Git")?.description, "Gitlab");
      const connections = findChild(items, "Connections");
      assert.ok(connections);
      assert.strictEqual(
        findChild(provider.getChildren(connections), "Session Wrapper")?.description,
        "tmux",
      );
      sinon.assert.calledOnce(published);
    };
    try {
      await newer;
      assertNewerTree();
    } finally {
      await olderRead.resolve('[sessions]\nwrapper = "zellij"\n[git]\nprovider = "github"');
      await older;
    }
    assertNewerTree();
    sinon.assert.calledOnce(olderCalls);
    sinon.assert.calledOnce(newerCalls);
  });

  suite("webhook status rendering", () => {
    test("shows pass icon and Running description when webhook is running", async () => {
      const mockContext = createMockContext(sandbox, "/fake/working-dir");
      sandbox.stub(configPaths, "configFileExists").resolves(true);
      sandbox.stub(configPaths, "getResolvedConfigPath").returns("");
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("/fake/working-dir");

      const provider = new StatusTreeProvider(mockContext);
      provider.setWebhookServer({ isRunning: () => true, getPort: () => 7009 });
      await provider.setTicketsDir(tempDir);

      const sections = provider.getChildren();
      const connections = findChild(sections, "Connections");
      assert.ok(connections, "Should have Connections section");

      const children = provider.getChildren(connections);
      const webhook = findChild(children, "Webhook");
      assert.ok(webhook, "Should have Webhook item");

      const icon = webhook.iconPath as vscode.ThemeIcon;
      assert.strictEqual(icon.id, "pass", "Webhook icon should be pass when running");
      assert.ok(
        (webhook.description as string).includes("Running"),
        `Description "${webhook.description}" should contain "Running"`,
      );
      assert.ok(
        (webhook.description as string).includes(":7009"),
        `Description "${webhook.description}" should contain port ":7009"`,
      );
    });

    test("shows circle-slash icon and Stopped when webhook is not running", async () => {
      const mockContext = createMockContext(sandbox, "/fake/working-dir");
      sandbox.stub(configPaths, "configFileExists").resolves(true);
      sandbox.stub(configPaths, "getResolvedConfigPath").returns("");
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("/fake/working-dir");

      await fs.mkdir(path.join(tempDir, "operator"), { recursive: true });
      await fs.writeFile(
        path.join(tempDir, "operator", "vscode-session.json"),
        JSON.stringify({ port: 7009 }),
      );
      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const sections = provider.getChildren();
      const connections = findChild(sections, "Connections");
      assert.ok(connections, "Should have Connections section");

      const children = provider.getChildren(connections);
      const webhook = findChild(children, "Webhook");
      assert.ok(webhook, "Should have Webhook item");

      const icon = webhook.iconPath as vscode.ThemeIcon;
      assert.strictEqual(
        icon.id,
        "circle-slash",
        "Webhook icon should be circle-slash when stopped",
      );
      assert.strictEqual(webhook.description, "Stopped", 'Description should be "Stopped"');
    });
  });

  suite("working directory item", () => {
    test("has contextValue and no command when working directory is set", async () => {
      const mockContext = createMockContext(sandbox, "/fake/working-dir");
      sandbox.stub(configPaths, "configFileExists").resolves(true);
      sandbox.stub(configPaths, "getResolvedConfigPath").returns("");
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("/fake/working-dir");

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const sections = provider.getChildren();
      const config = findChild(sections, "Configuration");
      assert.ok(config, "Should have Configuration section");

      const children = provider.getChildren(config);
      const workDir = findChild(children, "Working Directory");
      assert.ok(workDir, "Should have Working Directory item");

      assert.strictEqual(
        workDir.contextValue,
        "workingDirConfigured",
        'Should have contextValue "workingDirConfigured"',
      );
      assert.strictEqual(
        workDir.command,
        undefined,
        "Should not have a click command when directory is set",
      );
      assert.strictEqual(workDir.description, "/fake/working-dir");
    });

    test("has click command and no contextValue when working directory is not set", async () => {
      const mockContext = createMockContext(sandbox);
      sandbox.stub(configPaths, "configFileExists").resolves(false);
      sandbox.stub(configPaths, "getResolvedConfigPath").returns("");
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("");

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const sections = provider.getChildren();
      const config = findChild(sections, "Configuration");
      assert.ok(config, "Should have Configuration section");

      const children = provider.getChildren(config);
      const workDir = findChild(children, "Working Directory");
      assert.ok(workDir, "Should have Working Directory item");

      assert.ok(workDir.command, "Should have a click command when directory is not set");
      assert.strictEqual(
        workDir.command?.command,
        "operator.selectWorkingDirectory",
        "Command should be selectWorkingDirectory",
      );
      assert.strictEqual(workDir.contextValue, undefined, "Should not have contextValue");
      assert.strictEqual(workDir.description, "Not set");
    });
  });

  suite("session wrapper item", () => {
    test("shows pass icon with VS Code Terminal when wrapper defaults to vscode and webhook running", async () => {
      const mockContext = createMockContext(sandbox, "/fake/working-dir");
      sandbox.stub(configPaths, "configFileExists").resolves(true);
      sandbox.stub(configPaths, "getResolvedConfigPath").returns("");
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("/fake/working-dir");

      const provider = new StatusTreeProvider(mockContext);
      provider.setWebhookServer({ isRunning: () => true, getPort: () => 7009 });
      await provider.setTicketsDir(tempDir);

      const sections = provider.getChildren();
      const connections = findChild(sections, "Connections");
      assert.ok(connections, "Should have Connections section");

      const children = provider.getChildren(connections);
      const wrapper = findChild(children, "Session Wrapper");
      assert.ok(wrapper, "Should have Session Wrapper item");

      const icon = wrapper.iconPath as vscode.ThemeIcon;
      assert.strictEqual(
        icon.id,
        "pass",
        "Should show pass icon when vscode wrapper and webhook running",
      );
      assert.strictEqual(wrapper.description, "VS Code Terminal");
    });

    test("identifies the configured CLI wrapper", async () => {
      const mockContext = createMockContext(sandbox, "/fake/working-dir");
      sandbox.stub(configPaths, "configFileExists").resolves(true);
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("/fake/working-dir");

      // Write a config.toml with sessions.wrapper = "tmux"
      const configPath = path.join(tempDir, "config.toml");
      await fs.writeFile(configPath, '[sessions]\nwrapper = "tmux"\n');
      sandbox.stub(configPaths, "getResolvedConfigPath").returns(configPath);

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const sections = provider.getChildren();
      const connections = findChild(sections, "Connections");
      assert.ok(connections, "Should have Connections section");

      const children = provider.getChildren(connections);
      const wrapper = findChild(children, "Session Wrapper");
      assert.ok(wrapper, "Should have Session Wrapper item");

      const icon = wrapper.iconPath as vscode.ThemeIcon;
      assert.strictEqual(icon.id, "terminal", "CLI wrapper should have a terminal icon");
      assert.strictEqual(wrapper.description, "tmux");
    });
  });

  suite("MCP status row", () => {
    test("requires API verification before claiming an editor registration", async () => {
      const mockContext = createMockContext(sandbox, "/fake/working-dir");
      sandbox.stub(configPaths, "configFileExists").resolves(true);
      sandbox.stub(configPaths, "getResolvedConfigPath").returns("");
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("/fake/working-dir");
      sandbox.stub(mcpConnect, "inspectMcpRegistration").resolves({
        vscodeWorkspace: true,
        workspacePortable: false,
        copilotGlobal: false,
        cursor: false,
      });
      sandbox.stub(mcpConnect, "isVscodeWorkspaceStdio").returns(true);
      sandbox.stub(mcpConnect, "detectHostApp").returns("vscode");

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const sections = provider.getChildren();
      const connections = findChild(sections, "Connections");
      assert.ok(connections, "Should have Connections section");

      const children = provider.getChildren(connections);
      const mcp = findChild(children, "MCP");
      assert.ok(mcp, "Should have MCP item");
      assert.strictEqual(mcp.description, "API required");
    });
  });

  suite("progressive disclosure", () => {
    test("tier 0: connection choice is available before setup", async () => {
      const mockContext = createMockContext(sandbox);
      sandbox.stub(configPaths, "configFileExists").resolves(false);
      sandbox.stub(configPaths, "getResolvedConfigPath").returns("");
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("");

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.deepStrictEqual(labels, [
        "Configuration",
        "Connections",
        "Workflow exports",
        "Remote Targets",
        "License",
      ]);
    });

    test("tier 1: Configuration + Connections when config ready but no connections", async () => {
      const mockContext = createMockContext(sandbox, "/fake/working-dir");
      sandbox.stub(configPaths, "configFileExists").resolves(true);
      sandbox.stub(configPaths, "getResolvedConfigPath").returns("");
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("/fake/working-dir");

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.deepStrictEqual(labels, [
        "Configuration",
        "Connections",
        "Workflow exports",
        "Remote Targets",
        "License",
      ]);
    });

    test("tier 2: adds Kanban, LLM Tools, Model Servers, Git, Issue Types when connections ready", async () => {
      const mockContext = createMockContext(sandbox, "/fake/working-dir");
      sandbox.stub(configPaths, "configFileExists").resolves(true);
      sandbox.stub(configPaths, "getResolvedConfigPath").returns("");
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("/fake/working-dir");

      const provider = new StatusTreeProvider(mockContext);
      provider.setWebhookServer({ isRunning: () => true, getPort: () => 7009 });
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.deepStrictEqual(labels, [
        "Configuration",
        "Connections",
        "Kanban",
        "LLM Tools",
        "Model Servers",
        "Git",
        "Issue Types",
        "Delegators",
        "Workflow exports",
        "Remote Targets",
        "License",
      ]);
    });

    test("tier 3: Issue Types appears when kanban configured", async () => {
      const mockContext = createMockContext(sandbox, "/fake/working-dir");
      sandbox.stub(configPaths, "configFileExists").resolves(true);
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("/fake/working-dir");

      // Write config.toml with kanban section
      const configPath = path.join(tempDir, "config.toml");
      await fs.writeFile(configPath, '[kanban.jira."test.atlassian.net"]\nenabled = true\n');
      sandbox.stub(configPaths, "getResolvedConfigPath").returns(configPath);

      const provider = new StatusTreeProvider(mockContext);
      provider.setWebhookServer({ isRunning: () => true, getPort: () => 7009 });
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.ok(
        labels.includes("Issue Types"),
        "Should include Issue Types when kanban configured",
      );
      assert.ok(
        !labels.includes("Managed Projects"),
        "Should not include Managed Projects when git not configured",
      );
    });

    test("tier 3: Managed Projects appears when git configured", async () => {
      const mockContext = createMockContext(sandbox, "/fake/working-dir");
      sandbox.stub(configPaths, "configFileExists").resolves(true);
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("/fake/working-dir");

      // Write config.toml with git section
      const configPath = path.join(tempDir, "config.toml");
      await fs.writeFile(configPath, '[git]\nprovider = "github"\n');
      sandbox.stub(configPaths, "getResolvedConfigPath").returns(configPath);

      const provider = new StatusTreeProvider(mockContext);
      provider.setWebhookServer({ isRunning: () => true, getPort: () => 7009 });
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.ok(
        labels.includes("Managed Projects"),
        "Should include Managed Projects when git configured",
      );
      assert.ok(
        labels.includes("Issue Types"),
        "Issue Types stays visible with no kanban config: the built-in board keeps Kanban green",
      );
    });

    test("all tiers: all sections visible when fully configured", async () => {
      const mockContext = createMockContext(sandbox, "/fake/working-dir");
      sandbox.stub(configPaths, "configFileExists").resolves(true);
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("/fake/working-dir");

      // Write config.toml with kanban + git sections
      const configPath = path.join(tempDir, "config.toml");
      await fs.writeFile(
        configPath,
        '[kanban.jira."test.atlassian.net"]\nenabled = true\n\n[git]\nprovider = "github"\n',
      );
      sandbox.stub(configPaths, "getResolvedConfigPath").returns(configPath);

      const provider = new StatusTreeProvider(mockContext);
      provider.setWebhookServer({ isRunning: () => true, getPort: () => 7009 });
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.deepStrictEqual(labels, [
        "Configuration",
        "Connections",
        "Kanban",
        "LLM Tools",
        "Model Servers",
        "Git",
        "Issue Types",
        "Delegators",
        "Managed Projects",
        "Workflow exports",
        "Remote Targets",
        "License",
      ]);
    });

    test("tier 3 not visible when connections disconnected even if kanban/git configured", async () => {
      const mockContext = createMockContext(sandbox, "/fake/working-dir");
      sandbox.stub(configPaths, "configFileExists").resolves(true);
      sandbox.stub(configPaths, "resolveWorkingDirectory").returns("/fake/working-dir");

      // Write config.toml with kanban + git - but NO webhook session
      const configPath = path.join(tempDir, "config.toml");
      await fs.writeFile(
        configPath,
        '[kanban.jira."test.atlassian.net"]\nenabled = true\n\n[git]\nprovider = "github"\n',
      );
      sandbox.stub(configPaths, "getResolvedConfigPath").returns(configPath);

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.deepStrictEqual(
        labels,
        ["Configuration", "Connections", "Workflow exports", "Remote Targets", "License"],
        "Should only show tier 0+1 plus the prerequisite-free sections when connections not ready",
      );
    });
  });
});
