import * as assert from "node:assert";
import * as sinon from "sinon";
import * as vscode from "vscode";
import { openApprovalPage } from "../../src/auth/sign-in";
import {
  approvalTarget,
  connectionModeFromInspect,
  downloadsApply,
  effectiveMode,
  gateRemoteMcp,
  localAdoptMismatch,
  normalizeFailureMessage,
  normalizeRemoteOrigin,
  shouldStartWebhook,
  startupAction,
  swaggerUrl,
} from "../../src/connection";

suite("connection", () => {
  teardown(() => {
    sinon.restore();
  });

  suite("normalizeRemoteOrigin()", () => {
    test("accepts a public https origin and drops a trailing slash", () => {
      const result = normalizeRemoteOrigin("https://operator.kube.untra.casa/");
      assert.deepStrictEqual(result, { ok: true, origin: "https://operator.kube.untra.casa" });
    });

    test("adds https when the scheme is omitted", () => {
      const result = normalizeRemoteOrigin("operator.kube.untra.casa");
      assert.deepStrictEqual(result, { ok: true, origin: "https://operator.kube.untra.casa" });
    });

    test("keeps a non-default port", () => {
      const result = normalizeRemoteOrigin("https://operator.example.com:8443");
      assert.deepStrictEqual(result, {
        ok: true,
        origin: "https://operator.example.com:8443",
      });
    });

    test("rejects http, userinfo, a path, and loopback", () => {
      const http = normalizeRemoteOrigin("http://operator.example.com");
      const userinfo = normalizeRemoteOrigin("https://user:secret@operator.example.com");
      const path = normalizeRemoteOrigin("https://operator.example.com/api");
      const localhost = normalizeRemoteOrigin("https://localhost:7008");
      const loopback = normalizeRemoteOrigin("https://127.0.0.1:7008");
      assert.strictEqual(http.ok, false);
      assert.strictEqual(userinfo.ok, false);
      assert.strictEqual(path.ok, false);
      assert.strictEqual(localhost.ok, false);
      assert.strictEqual(loopback.ok, false);
      if (!http.ok) {
        assert.strictEqual(http.reason, "http");
      }
    });

    test("failure copy names the rule that failed", () => {
      assert.match(normalizeFailureMessage("http"), /https/);
      assert.match(normalizeFailureMessage("userinfo"), /password/);
      assert.match(normalizeFailureMessage("loopback"), /local mode/);
    });
  });

  suite("mode", () => {
    test("an unset mode is local", () => {
      assert.strictEqual(connectionModeFromInspect(undefined), "local");
      assert.strictEqual(connectionModeFromInspect({ globalValue: "local" }), "local");
    });

    test("workspace remote wins over a global local value", () => {
      assert.strictEqual(
        connectionModeFromInspect({ globalValue: "local", workspaceValue: "remote" }),
        "remote",
      );
    });

    test("a non-loopback URL is remote even when the mode setting is local", () => {
      assert.strictEqual(effectiveMode("local", "https://operator.kube.untra.casa"), "remote");
      assert.strictEqual(effectiveMode("local", "http://localhost:7008"), "local");
      assert.strictEqual(effectiveMode("remote", "http://localhost:7008"), "remote");
    });

    test("downloads apply only to a local connection", () => {
      assert.strictEqual(
        downloadsApply({ mode: "local", explicitApiUrl: "http://127.0.0.1:7008" }),
        true,
      );
      assert.strictEqual(
        downloadsApply({ mode: "remote", explicitApiUrl: "https://operator.kube.untra.casa" }),
        false,
      );
      assert.strictEqual(
        downloadsApply({ mode: "local", explicitApiUrl: "https://operator.kube.untra.casa" }),
        false,
      );
    });
  });

  suite("startupAction()", () => {
    test("a reachable authenticated server attaches", () => {
      assert.strictEqual(
        startupAction({
          mode: "remote",
          apiUrl: "https://operator.kube.untra.casa",
          reachable: true,
          authenticated: true,
          autoStart: true,
          hasConfig: true,
        }),
        "attach",
      );
    });

    test("remote auth failure does not start a local server", () => {
      assert.strictEqual(
        startupAction({
          mode: "remote",
          apiUrl: "https://operator.kube.untra.casa",
          reachable: true,
          authenticated: false,
          autoStart: true,
          hasConfig: true,
        }),
        "sign-in",
      );
    });

    test("an unreachable remote host is reported and autoStart does not run", () => {
      assert.strictEqual(
        startupAction({
          mode: "remote",
          apiUrl: "https://operator.kube.untra.casa",
          reachable: false,
          authenticated: false,
          autoStart: true,
          hasConfig: true,
        }),
        "report-unreachable",
      );
    });

    test("local autoStart still starts a server when nothing is listening", () => {
      assert.strictEqual(
        startupAction({
          mode: "local",
          apiUrl: "http://localhost:7008",
          reachable: false,
          authenticated: false,
          autoStart: true,
          hasConfig: true,
        }),
        "start-local",
      );
    });

    test("a non-loopback URL stored while mode is local does not auto-start", () => {
      assert.strictEqual(
        startupAction({
          mode: "local",
          apiUrl: "https://operator.kube.untra.casa",
          reachable: false,
          authenticated: false,
          autoStart: true,
          hasConfig: true,
        }),
        "report-unreachable",
      );
    });

    test("webhook starts only for a local connection that has config", () => {
      assert.strictEqual(shouldStartWebhook("local", "http://localhost:7008", true), true);
      assert.strictEqual(
        shouldStartWebhook("remote", "https://operator.kube.untra.casa", true),
        false,
      );
      assert.strictEqual(shouldStartWebhook("local", "http://localhost:7008", false), false);
    });
  });

  suite("adopt and links", () => {
    test("local mode still refuses a different binary version", () => {
      const mismatch = localAdoptMismatch({
        remote: false,
        expectedVersion: "0.2.16",
        foundVersion: "0.2.0",
      });
      assert.strictEqual(mismatch?.kind, "version");
    });

    test("remote mode accepts a different version and directory name", () => {
      assert.strictEqual(
        localAdoptMismatch({
          remote: true,
          expectedVersion: "0.0.1",
          foundVersion: "0.2.16",
          localDirectoryName: "laptop",
          foundDirectoryName: "casa",
        }),
        undefined,
      );
    });

    test("swagger follows the saved origin", () => {
      assert.strictEqual(
        swaggerUrl("https://operator.kube.untra.casa"),
        "https://operator.kube.untra.casa/swagger-ui",
      );
    });
  });

  suite("approval and MCP", () => {
    test("a matching remote sign-in link is opened as itself", () => {
      const target = approvalTarget(
        "https://operator.kube.untra.casa",
        "https://operator.kube.untra.casa/#/device?user_code=ABCD",
      );
      assert.strictEqual(target.ok, true);
      if (target.ok) {
        assert.strictEqual(target.portForward, false);
        assert.strictEqual(target.host, "operator.kube.untra.casa");
      }
    });

    test("loopback sign-in still uses port forwarding", () => {
      const target = approvalTarget(
        "http://localhost:7008",
        "http://localhost:7008/#/device?user_code=ABCD",
      );
      assert.strictEqual(target.ok, true);
      if (target.ok) {
        assert.strictEqual(target.portForward, true);
      }
    });

    test("a sign-in link on a different origin is refused", () => {
      const target = approvalTarget(
        "https://operator.kube.untra.casa",
        "http://10.0.0.5:7008/#/device?user_code=ABCD",
      );
      assert.strictEqual(target.ok, false);
      if (!target.ok) {
        assert.match(target.message, /public_url/);
      }
    });

    test("remote MCP requires the saved https origin and ignores a server-local path", () => {
      const refused = gateRemoteMcp(
        "https://operator.kube.untra.casa",
        "http://127.0.0.1:7008/api/v1/mcp/sse",
      );
      assert.strictEqual(refused.ok, false);

      const accepted = gateRemoteMcp(
        "https://operator.kube.untra.casa",
        "https://operator.kube.untra.casa/api/v1/mcp/sse",
      );
      assert.deepStrictEqual(accepted, {
        ok: true,
        transportUrl: "https://operator.kube.untra.casa/api/v1/mcp/sse",
      });
    });

    test("openApprovalPage leaves a public origin unchanged", async () => {
      const asExternal = sinon.stub(vscode.env, "asExternalUri");
      const open = sinon.stub(vscode.env, "openExternal").resolves(true);
      await openApprovalPage(
        "https://operator.kube.untra.casa",
        "https://operator.kube.untra.casa/#/device?user_code=ABCD",
      );
      assert.strictEqual(asExternal.called, false);
      assert.strictEqual(open.calledOnce, true);
      const opened = open.firstCall.args[0];
      assert.ok(opened);
      assert.strictEqual(opened.authority, "operator.kube.untra.casa");
    });

    test("openApprovalPage refuses a sign-in link on a different origin", async () => {
      const open = sinon.stub(vscode.env, "openExternal").resolves(true);
      await assert.rejects(() =>
        openApprovalPage(
          "https://operator.kube.untra.casa",
          "http://10.0.0.5:7008/#/device?user_code=ABCD",
        ),
      );
      assert.strictEqual(open.called, false);
    });
  });
});
