import { StatusTreeProvider } from "../../src/status-provider";
import * as sinon from "sinon";
import * as vscode from "vscode";
import { OperatorApiClient } from "../../src/api-client";
import * as connection from "../../src/connection";

suite("Preferred LLM tool", () => {
  const origin = "http://127.0.0.1:17008";
  setup(async () => {
    await vscode.extensions.getExtension("untra.operator-terminals")?.activate();
    sinon.stub(connection, "currentConnection").returns({
      origin,
      selection: { kind: "local" },
      signal: new AbortController().signal,
      assessment: { url: origin, connected: true },
    });
    sinon.stub(StatusTreeProvider.prototype, "refresh").resolves();
    sinon.stub(vscode.window, "showInformationMessage").resolves(undefined);
  });

  teardown(() => sinon.restore());

  test("selecting Gemini sends the tool-only preference to the running server", async () => {
    sinon
      .stub(OperatorApiClient.prototype, "getDefaultLlm")
      .resolves({ tool: "claude", model: "opus" });
    const save = sinon.stub(OperatorApiClient.prototype, "setDefaultLlm").resolves();
    await vscode.commands.executeCommand("operator.setDefaultLlm", "gemini");
    sinon.assert.calledOnceWithExactly(save, { tool: "gemini", model: "" });
  });

  test("reselecting a tool preserves its explicit model", async () => {
    const read = sinon
      .stub(OperatorApiClient.prototype, "getDefaultLlm")
      .resolves({ tool: "gemini", model: "pro" });
    const save = sinon.stub(OperatorApiClient.prototype, "setDefaultLlm").resolves();
    await vscode.commands.executeCommand("operator.setDefaultLlm", "gemini");
    sinon.assert.calledOnce(read);
    sinon.assert.notCalled(save);
  });
});
