/**
 * Tests for mcp-connect.ts
 *
 * Covers:
 * - `fetchMcpDescriptor` HTTP behavior
 * - `detectHostApp` IDE branch detection
 * - `registerInVscodeWorkspaceConfig` (stdio-preferred, SSE fallback)
 * - `registerInCursorUserConfig` (~/.cursor/mcp.json merge semantics)
 * - `connectMcpServer` end-to-end dispatch
 */

import * as assert from "node:assert";
import * as sinon from "sinon";
import * as vscode from "vscode";
import * as fs from "node:fs/promises";
import * as path from "node:path";
import * as os from "node:os";
import * as mcpConnect from "../../src/mcp-connect";
import * as apiClient from "../../src/api-client";
import type { McpDescriptorResponse } from "../../src/mcp-connect";
import {
  fetchMcpDescriptor,
  detectHostApp,
  registerInCursorUserConfig,
  registerInVscodeWorkspaceConfig,
  registerInVscodeWorkspacePortableConfig,
  registerInCopilotGlobalConfig,
  copilotGlobalConfigPath,
  connectMcpServer,
  writeMcpServersOperatorEntry,
  COPILOT_GLOBAL_ACTION,
  inspectMcpRegistration,
  mcpStatusDescription,
  _testable,
} from "../../src/mcp-connect";

const fixturesDir = path.join(__dirname, "..", "..", "..", "test", "fixtures", "api");

async function loadFixture(name: string): Promise<McpDescriptorResponse> {
  return JSON.parse(
    await fs.readFile(path.join(fixturesDir, name), "utf-8"),
  ) as McpDescriptorResponse;
}

import { clearCredentialProvider, setCredentialProvider } from "../../src/auth/credentials";
import { fakeCredentials } from "./helpers/credentials";

suite("MCP Connect Test Suite", () => {
  let fetchStub: sinon.SinonStub;

  setup(() => {
    fetchStub = sinon.stub(global, "fetch");
    setCredentialProvider(fakeCredentials());
  });

  teardown(() => {
    sinon.restore();
    clearCredentialProvider();
  });

  suite("fetchMcpDescriptor()", () => {
    test("fetches descriptor from correct URL", async () => {
      const descriptorResponse = await loadFixture("mcp-descriptor-response.json");

      fetchStub.resolves(new Response(JSON.stringify(descriptorResponse), { status: 200 }));

      const result = await fetchMcpDescriptor("http://localhost:7008");

      assert.ok(fetchStub.calledOnce);
      assert.strictEqual(
        fetchStub.firstCall.args[0],
        "http://localhost:7008/api/v1/mcp/descriptor",
      );
      assert.strictEqual(result.server_name, "operator");
      assert.strictEqual(result.server_id, "operator-mcp");
      assert.strictEqual(result.version, "0.1.26");
      assert.strictEqual(result.transport_url, "http://localhost:7008/api/v1/mcp/sse");
    });

    test("throws on network failure", async () => {
      fetchStub.rejects(new Error("Connection refused"));

      await assert.rejects(
        () => fetchMcpDescriptor("http://localhost:7008"),
        /Operator API is not running/,
      );
    });

    test("throws on HTTP 404", async () => {
      fetchStub.resolves(new Response("Not Found", { status: 404 }));

      await assert.rejects(
        () => fetchMcpDescriptor("http://localhost:7008"),
        /MCP descriptor unavailable/,
      );
    });

    test("throws on HTTP 500", async () => {
      fetchStub.resolves(new Response("Internal Server Error", { status: 500 }));

      await assert.rejects(
        () => fetchMcpDescriptor("http://localhost:7008"),
        /MCP descriptor unavailable/,
      );
    });

    test("uses custom API URL", async () => {
      const descriptorResponse: McpDescriptorResponse = {
        server_name: "operator",
        server_id: "operator-mcp",
        version: "0.1.26",
        transport_url: "http://localhost:9999/api/v1/mcp/sse",
        label: "Operator MCP Server",
        openapi_url: null,
      };

      fetchStub.resolves(new Response(JSON.stringify(descriptorResponse), { status: 200 }));

      await fetchMcpDescriptor("http://localhost:9999");

      assert.strictEqual(
        fetchStub.firstCall.args[0],
        "http://localhost:9999/api/v1/mcp/descriptor",
      );
    });

    test("parses stdio field when present", async () => {
      const descriptorResponse = await loadFixture("mcp-descriptor-response-stdio.json");

      fetchStub.resolves(new Response(JSON.stringify(descriptorResponse), { status: 200 }));

      const result = await fetchMcpDescriptor("http://localhost:7008");

      assert.ok(result.stdio, "stdio field should be populated");
      assert.strictEqual(result.stdio?.command, "/usr/local/bin/operator");
      assert.deepStrictEqual(result.stdio?.args, ["mcp"]);
      assert.strictEqual(result.stdio?.cwd, "/Users/dev/work");
    });
  });

  suite("detectHostApp()", () => {
    let rawAppNameStub: sinon.SinonStub;

    setup(() => {
      rawAppNameStub = sinon.stub(_testable, "rawAppName");
    });

    test("returns 'cursor' for exact 'Cursor'", () => {
      rawAppNameStub.returns("Cursor");
      assert.strictEqual(detectHostApp(), "cursor");
    });

    test("returns 'cursor' for 'Cursor (Anysphere)'", () => {
      rawAppNameStub.returns("Cursor (Anysphere)");
      assert.strictEqual(detectHostApp(), "cursor");
    });

    test("returns 'vscode' for 'Visual Studio Code'", () => {
      rawAppNameStub.returns("Visual Studio Code");
      assert.strictEqual(detectHostApp(), "vscode");
    });

    test("returns 'vscode' for 'Visual Studio Code - Insiders'", () => {
      rawAppNameStub.returns("Visual Studio Code - Insiders");
      assert.strictEqual(detectHostApp(), "vscode");
    });

    test("returns 'other' for empty string", () => {
      rawAppNameStub.returns("");
      assert.strictEqual(detectHostApp(), "other");
    });

    test("returns 'other' for unknown host (e.g. 'Theia IDE')", () => {
      rawAppNameStub.returns("Theia IDE");
      assert.strictEqual(detectHostApp(), "other");
    });
  });

  suite("writeMcpServersOperatorEntry()", () => {
    let tmpDir: string;
    let configPath: string;

    setup(async () => {
      tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "op-mcp-servers-"));
      configPath = path.join(tmpDir, ".mcp.json");
    });

    teardown(async () => {
      await fs.rm(tmpDir, { recursive: true, force: true });
    });

    const stdio = {
      command: "/usr/local/bin/operator",
      args: ["mcp"],
      cwd: "/Users/dev/work",
    };

    test("creates parent directory and writes mcpServers.operator", async () => {
      const result = await writeMcpServersOperatorEntry(configPath, stdio);
      assert.deepStrictEqual(result, { ok: true, path: configPath });
      const parsed = JSON.parse(await fs.readFile(configPath, "utf-8")) as {
        mcpServers: { operator: { command: string; args: string[]; cwd: string } };
      };
      assert.deepStrictEqual(parsed.mcpServers.operator, stdio);
    });

    test("copies descriptor args including --profile", async () => {
      const profileStdio = {
        command: "/usr/local/bin/operator",
        args: ["--profile", "work", "mcp"],
        cwd: "/Users/dev/work",
      };
      await writeMcpServersOperatorEntry(configPath, profileStdio);
      const parsed = JSON.parse(await fs.readFile(configPath, "utf-8")) as {
        mcpServers: { operator: { args: string[] } };
      };
      assert.deepStrictEqual(parsed.mcpServers.operator.args, ["--profile", "work", "mcp"]);
    });

    test("preserves sibling mcpServers entries and other top-level keys", async () => {
      await fs.writeFile(
        configPath,
        JSON.stringify({
          customKey: { foo: "bar" },
          mcpServers: { other: { command: "/usr/bin/other", args: [] } },
        }),
        "utf-8",
      );
      await writeMcpServersOperatorEntry(configPath, stdio);
      const parsed = JSON.parse(await fs.readFile(configPath, "utf-8")) as {
        customKey: { foo: string };
        mcpServers: Record<string, unknown>;
      };
      assert.deepStrictEqual(parsed.customKey, { foo: "bar" });
      assert.ok(parsed.mcpServers.other);
      assert.ok(parsed.mcpServers.operator);
    });

    test("does not write type or url fields", async () => {
      await writeMcpServersOperatorEntry(configPath, stdio);
      const parsed = JSON.parse(await fs.readFile(configPath, "utf-8")) as {
        mcpServers: { operator: Record<string, unknown> };
      };
      assert.strictEqual(parsed.mcpServers.operator.type, undefined);
      assert.strictEqual(parsed.mcpServers.operator.url, undefined);
    });

    test("returns parse failure and leaves malformed JSON untouched", async () => {
      await fs.writeFile(configPath, "{not valid json", "utf-8");
      const result = await writeMcpServersOperatorEntry(configPath, stdio);
      assert.strictEqual(result.ok, false);
      if (result.ok) {
        return;
      }
      assert.strictEqual(result.reason, "parse");
      assert.strictEqual(await fs.readFile(configPath, "utf-8"), "{not valid json");
    });
  });

  suite("registerInCursorUserConfig()", () => {
    let tmpDir: string;
    let cursorConfigPath: string;
    let infoStub: sinon.SinonStub;
    let errorStub: sinon.SinonStub;

    setup(async () => {
      tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "op-cursor-test-"));
      cursorConfigPath = path.join(tmpDir, ".cursor", "mcp.json");
      infoStub = sinon.stub(vscode.window, "showInformationMessage");
      errorStub = sinon.stub(vscode.window, "showErrorMessage");
    });

    teardown(async () => {
      await fs.rm(tmpDir, { recursive: true, force: true });
    });

    test("creates ~/.cursor directory when missing", async () => {
      const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");

      await registerInCursorUserConfig(descriptor, cursorConfigPath);

      const stat = await fs.stat(path.dirname(cursorConfigPath));
      assert.ok(stat.isDirectory(), "~/.cursor should exist");
    });

    test("writes mcpServers.operator with stdio shape", async () => {
      const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");

      await registerInCursorUserConfig(descriptor, cursorConfigPath);

      const raw = await fs.readFile(cursorConfigPath, "utf-8");
      const parsed = JSON.parse(raw) as {
        mcpServers: Record<string, { command: string; args: string[]; cwd: string }>;
      };
      assert.deepStrictEqual(parsed.mcpServers.operator, {
        command: "/usr/local/bin/operator",
        args: ["mcp"],
        cwd: "/Users/dev/work",
      });
      assert.ok(infoStub.calledOnce, "showInformationMessage should be called");
    });

    test("preserves existing mcpServers.* entries during merge", async () => {
      await fs.mkdir(path.dirname(cursorConfigPath), { recursive: true });
      await fs.writeFile(
        cursorConfigPath,
        JSON.stringify({
          mcpServers: {
            "other-server": { command: "/usr/bin/other", args: [] },
          },
        }),
        "utf-8",
      );
      const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");

      await registerInCursorUserConfig(descriptor, cursorConfigPath);

      const parsed = JSON.parse(await fs.readFile(cursorConfigPath, "utf-8")) as {
        mcpServers: Record<string, unknown>;
      };
      assert.ok(parsed.mcpServers["other-server"], "existing other-server should survive");
      assert.ok(parsed.mcpServers["operator"], "new operator entry should be written");
    });

    test("preserves other top-level keys during merge", async () => {
      await fs.mkdir(path.dirname(cursorConfigPath), { recursive: true });
      await fs.writeFile(
        cursorConfigPath,
        JSON.stringify({
          customKey: { foo: "bar" },
          mcpServers: {},
        }),
        "utf-8",
      );
      const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");

      await registerInCursorUserConfig(descriptor, cursorConfigPath);

      const parsed = JSON.parse(await fs.readFile(cursorConfigPath, "utf-8")) as {
        customKey: { foo: string };
        mcpServers: Record<string, unknown>;
      };
      assert.deepStrictEqual(parsed.customKey, { foo: "bar" });
      assert.ok(parsed.mcpServers["operator"]);
    });

    test("shows error and skips write when descriptor lacks stdio", async () => {
      const descriptor: McpDescriptorResponse = {
        server_name: "operator",
        server_id: "operator-mcp",
        version: "0.1.32",
        transport_url: "http://localhost:7008/api/v1/mcp/sse",
        label: "Operator MCP Server",
        openapi_url: null,
        // no stdio
      };

      await registerInCursorUserConfig(descriptor, cursorConfigPath);

      assert.ok(errorStub.calledOnce, "error message should be shown");
      const errorMsg = errorStub.firstCall.args[0] as string;
      assert.ok(errorMsg.includes("stdio_advertised"), "error should name the config knob");
      await assert.rejects(
        () => fs.access(cursorConfigPath),
        "mcp.json should NOT have been written",
      );
    });

    test("shows error when existing mcp.json is malformed JSON", async () => {
      await fs.mkdir(path.dirname(cursorConfigPath), { recursive: true });
      await fs.writeFile(cursorConfigPath, "{not valid json", "utf-8");
      const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");

      await registerInCursorUserConfig(descriptor, cursorConfigPath);

      assert.ok(errorStub.calledOnce, "error message should be shown");
      const errorMsg = errorStub.firstCall.args[0] as string;
      assert.ok(errorMsg.includes("Could not parse"), "error should mention parse failure");
      // file should be unchanged (still malformed)
      const raw = await fs.readFile(cursorConfigPath, "utf-8");
      assert.strictEqual(raw, "{not valid json");
    });
  });

  suite("registerInVscodeWorkspaceConfig()", () => {
    let configUpdateStub: sinon.SinonStub;
    let infoStub: sinon.SinonStub;
    let getStub: sinon.SinonStub;

    setup(() => {
      configUpdateStub = sinon.stub().resolves();
      getStub = sinon.stub().returns({});
      sinon.stub(vscode.workspace, "getConfiguration").returns({
        get: getStub,
        update: configUpdateStub,
      } as unknown as vscode.WorkspaceConfiguration);
      infoStub = sinon.stub(vscode.window, "showInformationMessage");
    });

    test("writes stdio entry when descriptor.stdio is present", async () => {
      const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");

      await registerInVscodeWorkspaceConfig(descriptor);

      assert.ok(configUpdateStub.calledOnce);
      assert.strictEqual(configUpdateStub.firstCall.args[0], "servers");
      const written = configUpdateStub.firstCall.args[1] as Record<string, Record<string, unknown>>;
      assert.deepStrictEqual(written.operator, {
        type: "stdio",
        command: "/usr/local/bin/operator",
        args: ["mcp"],
        cwd: "/Users/dev/work",
      });
      assert.ok(infoStub.notCalled, "workspace register defers toasts to connectMcpServer");
    });

    test("writes sse entry when descriptor.stdio is absent", async () => {
      const descriptor = await loadFixture("mcp-descriptor-response.json");

      await registerInVscodeWorkspaceConfig(descriptor);

      const written = configUpdateStub.firstCall.args[1] as Record<string, Record<string, unknown>>;
      assert.deepStrictEqual(written.operator, {
        type: "sse",
        url: "http://localhost:7008/api/v1/mcp/sse",
      });
    });

    test("preserves existing mcp.servers entries during merge", async () => {
      getStub.returns({
        "other-server": { type: "sse", url: "http://other" },
      });
      const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");

      await registerInVscodeWorkspaceConfig(descriptor);

      const written = configUpdateStub.firstCall.args[1] as Record<string, Record<string, unknown>>;
      assert.ok(written["other-server"]);
      assert.ok(written.operator);
    });
  });

  suite("registerInVscodeWorkspacePortableConfig()", () => {
    let tmpDir: string;
    let configPath: string;

    setup(async () => {
      tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "op-mcp-ws-"));
      configPath = path.join(tmpDir, ".mcp.json");
    });

    teardown(async () => {
      await fs.rm(tmpDir, { recursive: true, force: true });
    });

    test("writes mcpServers.operator from descriptor.stdio", async () => {
      const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");
      const result = await registerInVscodeWorkspacePortableConfig(descriptor, configPath);
      assert.strictEqual(result.ok, true);
      const parsed = JSON.parse(await fs.readFile(configPath, "utf-8")) as {
        mcpServers: { operator: { command: string } };
      };
      assert.strictEqual(parsed.mcpServers.operator.command, "/usr/local/bin/operator");
    });

    test("skips write when descriptor has no stdio", async () => {
      const descriptor = await loadFixture("mcp-descriptor-response.json");
      const result = await registerInVscodeWorkspacePortableConfig(descriptor, configPath);
      assert.strictEqual(result.ok, false);
      if (result.ok) {
        return;
      }
      assert.strictEqual(result.reason, "no-stdio");
      await assert.rejects(() => fs.access(configPath));
    });

    test("skips write when workspace path is missing", async () => {
      sinon.stub(_testable, "workspacePortableMcpPath").returns(undefined);
      const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");
      const result = await registerInVscodeWorkspacePortableConfig(descriptor, undefined);
      assert.strictEqual(result.ok, false);
      if (result.ok) {
        return;
      }
      assert.strictEqual(result.reason, "no-workspace");
    });

    test("uses _testable.workspacePortableMcpPath when configPath is omitted", async () => {
      const pathStub = sinon.stub(_testable, "workspacePortableMcpPath").returns(undefined);
      try {
        const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");
        const result = await registerInVscodeWorkspacePortableConfig(descriptor);
        assert.strictEqual(result.ok, false);
        if (result.ok) {
          return;
        }
        assert.strictEqual(result.reason, "no-workspace");
        assert.ok(pathStub.calledOnce);
      } finally {
        pathStub.restore();
      }
    });
  });

  suite("copilotGlobalConfigPath()", () => {
    test("uses COPILOT_HOME when set", () => {
      const prev = process.env.COPILOT_HOME;
      process.env.COPILOT_HOME = "/tmp/copilot-home-test";
      try {
        assert.strictEqual(
          copilotGlobalConfigPath(),
          path.join("/tmp/copilot-home-test", "mcp-config.json"),
        );
      } finally {
        if (prev === undefined) {
          delete process.env.COPILOT_HOME;
        } else {
          process.env.COPILOT_HOME = prev;
        }
      }
    });

    test("falls back to ~/.copilot/mcp-config.json", () => {
      const prev = process.env.COPILOT_HOME;
      delete process.env.COPILOT_HOME;
      try {
        assert.strictEqual(
          copilotGlobalConfigPath(),
          path.join(os.homedir(), ".copilot", "mcp-config.json"),
        );
      } finally {
        if (prev !== undefined) {
          process.env.COPILOT_HOME = prev;
        }
      }
    });
  });

  suite("registerInCopilotGlobalConfig()", () => {
    let tmpDir: string;
    let configPath: string;

    setup(async () => {
      tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "op-copilot-mcp-"));
      configPath = path.join(tmpDir, "mcp-config.json");
    });

    teardown(async () => {
      await fs.rm(tmpDir, { recursive: true, force: true });
    });

    test("writes mcpServers.operator when stdio is present", async () => {
      const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");
      const result = await registerInCopilotGlobalConfig(descriptor, configPath);
      assert.strictEqual(result.ok, true);
      const parsed = JSON.parse(await fs.readFile(configPath, "utf-8")) as {
        mcpServers: { operator: { cwd: string } };
      };
      assert.strictEqual(parsed.mcpServers.operator.cwd, "/Users/dev/work");
    });

    test("skips write when descriptor has no stdio", async () => {
      const descriptor = await loadFixture("mcp-descriptor-response.json");
      const result = await registerInCopilotGlobalConfig(descriptor, configPath);
      assert.strictEqual(result.ok, false);
      if (result.ok) {
        return;
      }
      assert.strictEqual(result.reason, "no-stdio");
      await assert.rejects(() => fs.access(configPath));
    });
  });

  suite("connectMcpServer() dispatch", () => {
    let detectHostStub: sinon.SinonStub;
    let configUpdateStub: sinon.SinonStub;
    let getStub: sinon.SinonStub;

    setup(() => {
      detectHostStub = sinon.stub(_testable, "rawAppName");
      sinon.stub(apiClient, "discoverApiUrl").resolves("http://localhost:7008");
      configUpdateStub = sinon.stub().resolves();
      getStub = sinon.stub().returns({});
      sinon.stub(vscode.workspace, "getConfiguration").returns({
        get: getStub,
        update: configUpdateStub,
      } as unknown as vscode.WorkspaceConfiguration);
      sinon.stub(vscode.window, "showInformationMessage");
      sinon.stub(vscode.window, "showErrorMessage");
    });

    test("VS Code host writes workspace mcp.servers and .mcp.json when stdio is present", async () => {
      detectHostStub.returns("Visual Studio Code");
      const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "op-connect-ws-"));
      const portablePath = path.join(tmpDir, ".mcp.json");
      sinon.stub(_testable, "workspacePortableMcpPath").returns(portablePath);
      try {
        const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");
        fetchStub.resolves(new Response(JSON.stringify(descriptor), { status: 200 }));

        await connectMcpServer(undefined);

        assert.ok(configUpdateStub.calledOnce, "legacy mcp.servers write");
        const written = configUpdateStub.firstCall.args[1] as Record<
          string,
          Record<string, unknown> | undefined
        >;
        assert.ok(written.operator, "operator entry should be written");
        assert.strictEqual(written.operator.type, "stdio");
        const parsed = JSON.parse(await fs.readFile(portablePath, "utf-8")) as {
          mcpServers: { operator: { command: string } };
        };
        assert.strictEqual(parsed.mcpServers.operator.command, "/usr/local/bin/operator");
      } finally {
        await fs.rm(tmpDir, { recursive: true, force: true });
      }
    });

    test("VS Code host does not write .mcp.json when stdio is absent", async () => {
      detectHostStub.returns("Visual Studio Code");
      const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "op-connect-sse-"));
      const portablePath = path.join(tmpDir, ".mcp.json");
      sinon.stub(_testable, "workspacePortableMcpPath").returns(portablePath);
      try {
        const descriptor = await loadFixture("mcp-descriptor-response.json");
        fetchStub.resolves(new Response(JSON.stringify(descriptor), { status: 200 }));

        await connectMcpServer(undefined);

        assert.ok(configUpdateStub.calledOnce);
        const written = configUpdateStub.firstCall.args[1] as Record<
          string,
          { type: string } | undefined
        >;
        assert.strictEqual(written.operator?.type, "sse");
        await assert.rejects(() => fs.access(portablePath));
      } finally {
        await fs.rm(tmpDir, { recursive: true, force: true });
      }
    });

    test("VS Code host writes Copilot Global when the user accepts the follow-up", async () => {
      detectHostStub.returns("Visual Studio Code");
      const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "op-connect-global-"));
      const portablePath = path.join(tmpDir, ".mcp.json");
      const globalPath = path.join(tmpDir, "mcp-config.json");
      sinon.stub(_testable, "workspacePortableMcpPath").returns(portablePath);
      sinon.stub(_testable, "copilotGlobalConfigPath").returns(globalPath);
      const infoStub = vscode.window.showInformationMessage as unknown as sinon.SinonStub;
      infoStub.resolves(COPILOT_GLOBAL_ACTION);
      try {
        const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");
        fetchStub.resolves(new Response(JSON.stringify(descriptor), { status: 200 }));

        await connectMcpServer(undefined);

        const parsed = JSON.parse(await fs.readFile(globalPath, "utf-8")) as {
          mcpServers: { operator: { command: string } };
        };
        assert.strictEqual(parsed.mcpServers.operator.command, "/usr/local/bin/operator");
      } finally {
        await fs.rm(tmpDir, { recursive: true, force: true });
      }
    });

    test("VS Code host does not write Copilot Global when the user dismisses the toast", async () => {
      detectHostStub.returns("Visual Studio Code");
      const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "op-connect-dismiss-"));
      const portablePath = path.join(tmpDir, ".mcp.json");
      const globalPath = path.join(tmpDir, "mcp-config.json");
      sinon.stub(_testable, "workspacePortableMcpPath").returns(portablePath);
      sinon.stub(_testable, "copilotGlobalConfigPath").returns(globalPath);
      const infoStub = vscode.window.showInformationMessage as unknown as sinon.SinonStub;
      infoStub.resolves(undefined);
      try {
        const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");
        fetchStub.resolves(new Response(JSON.stringify(descriptor), { status: 200 }));

        await connectMcpServer(undefined);

        await assert.rejects(() => fs.access(globalPath));
      } finally {
        await fs.rm(tmpDir, { recursive: true, force: true });
      }
    });

    test("Cursor host does NOT write to workspace mcp.servers", async () => {
      detectHostStub.returns("Cursor");
      // Re-stub cursorMcpConfigPath to a tmp path so the test doesn't
      // mutate the developer's real ~/.cursor/mcp.json.
      const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "op-cursor-dispatch-"));
      const tmpConfigPath = path.join(tmpDir, ".cursor", "mcp.json");
      sinon.stub(_testable, "cursorMcpConfigPath").returns(tmpConfigPath);

      try {
        const descriptor = await loadFixture("mcp-descriptor-response-stdio.json");
        fetchStub.resolves(new Response(JSON.stringify(descriptor), { status: 200 }));

        await connectMcpServer(undefined);

        assert.ok(configUpdateStub.notCalled, "Cursor path should NOT touch workspace mcp.servers");
        // and the cursor config file should exist
        const raw = await fs.readFile(tmpConfigPath, "utf-8");
        const parsed = JSON.parse(raw) as {
          mcpServers: { operator: { command: string } };
        };
        assert.strictEqual(parsed.mcpServers.operator.command, "/usr/local/bin/operator");
      } finally {
        await fs.rm(tmpDir, { recursive: true, force: true });
      }
    });
  });

  suite("inspectMcpRegistration() and mcpStatusDescription()", () => {
    test("mcpStatusDescription vscode workspace-only stdio", () => {
      const { connected, description, tooltip } = mcpStatusDescription(
        {
          vscodeWorkspace: true,
          workspacePortable: false,
          copilotGlobal: false,
          cursor: false,
        },
        "vscode",
        true,
      );
      assert.strictEqual(connected, true);
      assert.strictEqual(description, "editor");
      assert.ok(tooltip.includes("Copilot CLI"));
    });

    test("mcpStatusDescription vscode SSE-only", () => {
      const { description } = mcpStatusDescription(
        {
          vscodeWorkspace: true,
          workspacePortable: false,
          copilotGlobal: false,
          cursor: false,
        },
        "vscode",
        false,
      );
      assert.strictEqual(description, "editor (SSE)");
    });

    test("mcpStatusDescription vscode editor plus portable plus global", () => {
      const { description } = mcpStatusDescription(
        {
          vscodeWorkspace: true,
          workspacePortable: true,
          copilotGlobal: true,
          cursor: false,
        },
        "vscode",
        true,
      );
      assert.strictEqual(description, "editor · .mcp.json · Copilot");
    });

    test("mcpStatusDescription cursor ignores workspace settings", () => {
      const { connected, description } = mcpStatusDescription(
        {
          vscodeWorkspace: true,
          workspacePortable: false,
          copilotGlobal: false,
          cursor: false,
        },
        "cursor",
        true,
      );
      assert.strictEqual(connected, false);
      assert.strictEqual(description, "Connect");
    });

    test("mcpStatusDescription cursor file present", () => {
      const { connected, description } = mcpStatusDescription(
        {
          vscodeWorkspace: false,
          workspacePortable: false,
          copilotGlobal: false,
          cursor: true,
        },
        "cursor",
        true,
      );
      assert.strictEqual(connected, true);
      assert.strictEqual(description, "~/.cursor/mcp.json");
    });

    test("mcpStatusDescription Copilot Global only", () => {
      const { connected, description } = mcpStatusDescription(
        {
          vscodeWorkspace: false,
          workspacePortable: false,
          copilotGlobal: true,
          cursor: false,
        },
        "vscode",
        undefined,
      );
      assert.strictEqual(connected, true);
      assert.strictEqual(description, "Copilot Global");
    });

    test("inspectMcpRegistration reads stubbed portable files", async () => {
      const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), "op-inspect-"));
      const portablePath = path.join(tmpDir, ".mcp.json");
      const globalPath = path.join(tmpDir, "mcp-config.json");
      await fs.writeFile(
        portablePath,
        JSON.stringify({ mcpServers: { operator: { command: "x", args: ["mcp"], cwd: "/" } } }),
        "utf-8",
      );
      sinon.stub(_testable, "workspacePortableMcpPath").returns(portablePath);
      sinon.stub(_testable, "copilotGlobalConfigPath").returns(globalPath);
      sinon.stub(_testable, "cursorMcpConfigPath").returns(path.join(tmpDir, "cursor.json"));
      sinon.stub(vscode.workspace, "getConfiguration").returns({
        get: () => ({ operator: { type: "stdio" } }),
      } as unknown as vscode.WorkspaceConfiguration);
      try {
        const presence = await inspectMcpRegistration();
        assert.strictEqual(presence.vscodeWorkspace, true);
        assert.strictEqual(presence.workspacePortable, true);
        assert.strictEqual(presence.copilotGlobal, false);
        assert.strictEqual(presence.cursor, false);
      } finally {
        await fs.rm(tmpDir, { recursive: true, force: true });
      }
    });
  });
});

// Silence unused-import warning - `mcpConnect` is imported for documentation
// (it shows the public surface tests rely on).
void mcpConnect;
