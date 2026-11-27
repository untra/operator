import * as assert from "node:assert";
import * as sinon from "sinon";
import * as vscode from "vscode";
import { PICK_LOCAL, PICK_REMOTE } from "../../src/connection";
import { runConnectionPicker } from "../../src/connection-picker";

suite("runConnectionPicker()", () => {
  const notices: {
    showInformationMessage(message: string, ...options: unknown[]): Thenable<string | undefined>;
  } = vscode.window;
  const origin = "https://operator.example.com";
  const saveLocal = sinon.stub<[], Promise<void>>().resolves();
  const saveRemote = sinon.stub<[string, boolean], Promise<void>>().resolves();

  teardown(() => {
    sinon.restore();
    saveLocal.resetHistory();
    saveRemote.resetHistory();
  });

  test("offers the two connection choices before any setup", async () => {
    const show = sinon.stub(notices, "showInformationMessage").resolves(PICK_LOCAL);
    await runConnectionPicker({ saveLocal, saveRemote });
    assert.deepStrictEqual(show.firstCall.args.slice(1), [PICK_LOCAL, PICK_REMOTE]);
    sinon.assert.calledOnce(saveLocal);
    sinon.assert.notCalled(saveRemote);
  });

  test("saves a normalized hosted origin", async () => {
    sinon.stub(vscode.window, "showInputBox").resolves(`${origin}/`);
    await runConnectionPicker({ saveLocal, saveRemote }, PICK_REMOTE);
    sinon.assert.calledOnceWithExactly(saveRemote, origin, false);
  });

  test("requires confirmation before saving a repository suggestion", async () => {
    const show = sinon.stub(notices, "showInformationMessage").resolves(undefined);
    await runConnectionPicker({ saveLocal, saveRemote, suggestion: origin }, PICK_REMOTE);
    sinon.assert.notCalled(saveRemote);

    show.resolves("Use This Server");
    await runConnectionPicker({ saveLocal, saveRemote, suggestion: origin }, PICK_REMOTE);
    sinon.assert.calledOnceWithExactly(saveRemote, origin, true);
  });

  test("refuses a malformed hosted origin", async () => {
    sinon.stub(vscode.window, "showInputBox").resolves("http://operator.example.com");
    await runConnectionPicker({ saveLocal, saveRemote }, PICK_REMOTE);
    sinon.assert.notCalled(saveRemote);
  });
});
