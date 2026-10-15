import * as assert from "node:assert";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import * as sinon from "sinon";
import * as vscode from "vscode";
import { clearCredentialProvider, setCredentialProvider } from "../../src/auth/credentials";
import * as operatorBinary from "../../src/operator-binary";
import { ConnectionsSection } from "../../src/sections/connections-section";
import type { SectionContext } from "../../src/sections/types";
import { fakeCredentials } from "./helpers/credentials";

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
    sinon.restore();
    clearCredentialProvider();
    await fs.rm(tempDir, { recursive: true, force: true });
  });

  test("local mode still refuses a different binary version and offers download", async () => {
    sinon.stub(vscode.workspace, "getConfiguration").returns(operatorConfig({}));
    fetchStub.callsFake((input: unknown) => {
      const url = String(input);
      if (url.endsWith("/livez")) {
        return Promise.resolve(new Response("ok", { status: 200 }));
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
    assert.strictEqual(api?.command?.command, "operator.startOperatorServer");
  });

  test("remote mode adopts a different version and directory and links to that origin", async () => {
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
    assert.strictEqual(webhook?.description, "Local only");
  });

  test("remote mode keeps the saved URL when the server requires sign-in", async () => {
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
    assert.strictEqual(api?.description, "Sign-in required");
    assert.strictEqual(api?.command?.command, "operator.signIn");
    assert.ok(children.every((item) => item.command?.command !== "operator.downloadOperator"));
    assert.ok(children.every((item) => item.command?.command !== "operator.startOperatorServer"));
  });
});
