import * as assert from "node:assert";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import * as sinon from "sinon";
import * as vscode from "vscode";
import { clearCredentialProvider, setCredentialProvider } from "../../src/auth/credentials";
import { discoverLocalApiUrl } from "../../src/api-client";
import * as mcpConnect from "../../src/mcp-connect";
import * as operatorBinary from "../../src/operator-binary";
import { ConnectionsSection } from "../../src/sections/connections-section";
import type { SectionContext } from "../../src/sections/types";
import { fakeCredentials } from "./helpers/credentials";

import {
  resetConnection,
  setConnectionAssessment,
  setConnectionOrigin,
} from "../../src/connection";

function operatorConfig(values: Record<string, string>): vscode.WorkspaceConfiguration {
  return {
    inspect: (key: string) => {
      const value = values[key];
      return value === undefined ? {} : { globalValue: value };
    },
    get: (key: string, fallback?: unknown) => values[key] ?? fallback,
  } as unknown as vscode.WorkspaceConfiguration;
}

function context(ticketsDir: string | undefined, configReady: boolean): SectionContext {
  const blank: Record<string, never> = {};
  const extensionContext = blank as unknown as vscode.ExtensionContext;
  return {
    extensionContext,
    ticketsDir,
    readConfigToml: () => Promise.resolve({}),
    configReady,
    connectionsReady: false,
    kanbanConfigured: false,
    llmConfigured: false,
    gitConfigured: false,
  };
}

suite("ConnectionsSection", () => {
  let fetchStub: sinon.SinonStub;
  let tempDir: string;

  setup(async () => {
    fetchStub = sinon.stub(global, "fetch");
    setCredentialProvider(fakeCredentials("token"));
    sinon.stub(operatorBinary, "getOperatorPath").resolves("/usr/local/bin/operator");
    sinon.stub(operatorBinary, "getOperatorVersion").resolves("0.2.16");
    tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "op-connections-"));
  });

  teardown(async () => {
    resetConnection({ kind: "undecided" });
    sinon.restore();
    clearCredentialProvider();
    await fs.rm(tempDir, { recursive: true, force: true });
  });

  test("a failed MCP probe clears the previous registration", async () => {
    const origin = "https://operator.example.com";
    resetConnection({ kind: "hosted", origin });
    setConnectionOrigin(origin);
    setConnectionAssessment({ connected: true, url: origin });
    sinon.stub(mcpConnect, "detectHostApp").returns("vscode");
    sinon.stub(mcpConnect, "fetchMcpDescriptor").resolves({
      server_name: "operator",
      server_id: "operator",
      version: "1",
      label: "Operator",
      transport_url: origin,
      openapi_url: null,
    });
    const registration = sinon.stub(mcpConnect, "inspectMcpRegistration");
    registration.onFirstCall().resolves({
      vscodeWorkspace: true,
      workspacePortable: false,
      copilotGlobal: false,
      cursor: false,
    });
    registration.onSecondCall().rejects(new Error("registration unavailable"));

    const section = new ConnectionsSection();
    const ctx = context(undefined, true);
    await section.check(ctx);
    assert.strictEqual(
      section.getChildren(ctx).find((item) => item.label === "MCP")?.command?.title,
      "Reconnect MCP Server",
    );
    await section.check(ctx);
    const rows = section.getChildren(ctx);
    const mcp = rows.find((item) => item.label === "MCP");
    assert.strictEqual(mcp?.command?.title, "Connect MCP Server");
    assert.strictEqual((mcp.iconPath as vscode.ThemeIcon).id, "plug");
    assert.strictEqual(rows.find((item) => item.label === "API")?.description, origin);
  });

  test("renders the checked selection until the next check completes", async () => {
    const origin = "https://operator.example.com";
    resetConnection({ kind: "hosted", origin });
    setConnectionOrigin(origin);
    setConnectionAssessment({ connected: true, url: origin });
    sinon.stub(mcpConnect, "fetchMcpDescriptor").rejects(new Error("descriptor unavailable"));
    const ctx = context(undefined, true);
    const section = new ConnectionsSection();
    await section.check(ctx);
    resetConnection({ kind: "undecided" });
    assert.strictEqual(
      section.getChildren(ctx).find((item) => item.label === "API")?.description,
      origin,
    );
    assert.strictEqual(section.health(), "Green");
    await section.check(ctx);
    assert.deepStrictEqual(
      section.getChildren(ctx).map((item) => item.label),
      ["Connect Operator to This Workspace"],
    );
    assert.strictEqual(section.health(), "Gray");
  });

  test("checks the selected configuration on a server hosting multiple profiles", async () => {
    const dir = path.join(tempDir, "Documents", ".tickets");
    const url = "http://127.0.0.1:17008";
    await fs.mkdir(path.join(dir, "operator"), { recursive: true });
    await fs.writeFile(
      path.join(dir, "operator", "api-session.json"),
      JSON.stringify({ origin: url, port: 17008, profile_id: "documents" }),
    );
    resetConnection({ kind: "local" });
    setConnectionOrigin(await discoverLocalApiUrl(dir));
    fetchStub.callsFake((input: string) =>
      Promise.resolve(
        Response.json({
          version: "0.2.16",
          directory_name: input.endsWith("/profiles/documents/status") ? "Documents" : "iac",
        }),
      ),
    );
    const ctx = context(dir, true);
    const section = new ConnectionsSection();
    await section.check(ctx);
    assert.equal(section.isApiConnected, true);
    assert.ok(fetchStub.calledWith(`${url}/api/v1/profiles/documents/status`));
  });

  test("local mode still refuses a different binary version and offers download", async () => {
    resetConnection({ kind: "local" });
    setConnectionOrigin("http://localhost:7008");
    sinon.stub(vscode.workspace, "getConfiguration").returns(operatorConfig({}));
    fetchStub.callsFake((input: unknown) => {
      const url = String(input);
      if (url.endsWith("/livez")) {
        return Promise.resolve(new Response("ok", { status: 200 }));
      }
      if (url.endsWith("/auth/session")) {
        return Promise.resolve(new Response("{}"));
      }
      if (url.includes("/api/v1/health")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              status: "ok",
              version: "9.9.9",
              directory_name: "other",
              directory_id: "abc",
            }),
            { status: 200 },
          ),
        );
      }
      return Promise.resolve(new Response("no", { status: 404 }));
    });

    const section = new ConnectionsSection();
    const ticketsDir = path.join(tempDir, "proj", ".tickets");
    await fs.mkdir(ticketsDir, { recursive: true });
    await section.check(context(ticketsDir, true));
    const children = section.getChildren(context(ticketsDir, true));
    const version = children.find((item) => item.label === "Operator Version");
    const api = children.find((item) => item.label === "API");
    assert.strictEqual(version?.command?.command, "operator.downloadOperator");
    assert.strictEqual(api?.description, "Version mismatch");
    assert.strictEqual(api?.command?.command, "operator.chooseConnection");
  });

  test("remote mode adopts a different version and directory and links to that origin", async () => {
    const origin = "https://operator.kube.untra.casa";
    resetConnection({ kind: "hosted", origin });
    setConnectionOrigin(origin);
    sinon
      .stub(vscode.workspace, "getConfiguration")
      .returns(
        operatorConfig({ connectionMode: "remote", apiUrl: "https://operator.kube.untra.casa" }),
      );
    fetchStub.callsFake((input: unknown) => {
      const url = String(input);
      if (url.endsWith("/livez")) {
        return Promise.resolve(new Response("ok", { status: 200 }));
      }
      if (url.endsWith("/auth/session")) {
        return Promise.resolve(new Response("{}"));
      }
      if (url.includes("/api/v1/health")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              status: "ok",
              version: "0.2.16",
              directory_name: "casa",
              directory_id: "abc",
            }),
            { status: 200 },
          ),
        );
      }
      return Promise.resolve(
        new Response("<html></html>", {
          status: 200,
          headers: { "content-type": "text/html" },
        }),
      );
    });

    const section = new ConnectionsSection();
    const ticketsDir = path.join(tempDir, "laptop", ".tickets");
    await fs.mkdir(ticketsDir, { recursive: true });
    await section.check(context(ticketsDir, false));
    const children = section.getChildren(context(ticketsDir, false));
    assert.ok(
      children.every((item) => item.command?.command !== "operator.downloadOperator"),
      "remote mode does not offer a download",
    );
    const version = children.find((item) => item.label === "Operator");
    const swagger = version?.command?.arguments?.[0] as vscode.Uri | undefined;
    assert.strictEqual(swagger?.toString(), "https://operator.kube.untra.casa/swagger-ui");
    const web = children.find((item) => item.label === "Web UI");
    assert.strictEqual(web?.description, "https://operator.kube.untra.casa");
    const api = children.find((item) => item.label === "API");
    assert.strictEqual(api?.description, "https://operator.kube.untra.casa");
    const tooltip = typeof api?.tooltip === "string" ? api.tooltip : "";
    assert.match(tooltip, /serves 'casa'/);
    const webhook = children.find((item) => item.label === "Webhook");
    assert.strictEqual(webhook, undefined);
    assert.strictEqual(section.health(), "Green");
  });

  test("a hosted session without health permission remains connected", async () => {
    const origin = "https://operator.example.com";
    resetConnection({ kind: "hosted", origin });
    setConnectionOrigin(origin);
    fetchStub.callsFake((url: string) =>
      Promise.resolve(
        url.endsWith("/api/v1/health") ? new Response("{}", { status: 403 }) : new Response("{}"),
      ),
    );
    const section = new ConnectionsSection();
    await section.check(context(undefined, true));
    const api = section.getChildren(context(undefined, true)).find((item) => item.label === "API");
    assert.equal(api?.description, origin);
    assert.ok(fetchStub.calledWith(`${origin}/api/v1/auth/session`));
  });

  test("a rejected local registration offers configuration recovery", async () => {
    const url = "http://localhost:7008";
    resetConnection({ kind: "local" });
    setConnectionOrigin(url);
    setConnectionAssessment({
      connected: false,
      url,
      mismatch: { kind: "auth", detail: "requires sign-in" },
    });
    const section = new ConnectionsSection();
    const ctx = context(tempDir, true);
    await section.check(ctx);
    const items = section.getChildren(ctx);
    assert.equal(
      items.find((item) => item.label === "API")?.command?.command,
      "operator.retryConnection",
    );
    assert.equal(
      items.find((item) => item.label === "Choose Configuration")?.command?.command,
      "operator.selectWorkingDirectory",
    );
    assert.ok(items.every((item) => item.command?.command !== "operator.signIn"));
  });

  test("remote mode keeps the saved URL when the server requires sign-in", async () => {
    const origin = "https://operator.kube.untra.casa";
    resetConnection({ kind: "hosted", origin });
    setConnectionOrigin(origin);
    sinon
      .stub(vscode.workspace, "getConfiguration")
      .returns(
        operatorConfig({ connectionMode: "remote", apiUrl: "https://operator.kube.untra.casa" }),
      );
    fetchStub.callsFake((input: unknown) => {
      const url = String(input);
      if (url.endsWith("/livez")) {
        return Promise.resolve(new Response("ok", { status: 200 }));
      }
      return Promise.resolve(new Response("{}", { status: 401 }));
    });

    const section = new ConnectionsSection();
    await section.check(context(undefined, false));
    const children = section.getChildren(context(undefined, false));
    const api = children.find((item) => item.label === "API");
    assert.strictEqual(api?.description, `Sign in to ${origin}`);
    assert.strictEqual(api?.command?.command, "operator.signIn");
    assert.ok(children.every((item) => item.command?.command !== "operator.downloadOperator"));
    assert.ok(children.every((item) => item.command?.command !== "operator.startOperatorServer"));
  });
});
