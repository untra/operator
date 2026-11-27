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
  const saveOff = sinon.stub<[], Promise<void>>().resolves();
  const saveLocal = sinon.stub<[], Promise<void>>().resolves();
  const saveRemote = sinon.stub<[string, boolean], Promise<void>>().resolves();

  teardown(() => {
    sinon.restore();
    saveOff.resetHistory();
    saveLocal.resetHistory();
    saveRemote.resetHistory();
  });

  test("offers Local, Remote, and Off in the workspace picker", async () => {
    const picker = sinon.stub(vscode.window, "showQuickPick").resolves({ label: PICK_LOCAL });
    await runConnectionPicker({ saveOff, saveLocal, saveRemote });
    const items = picker.firstCall.args[0];
    assert.ok(!("then" in items));
    assert.deepStrictEqual(
      items.map((item) => item.label),
      ["Local", "Remote", "Off"],
    );
    sinon.assert.calledOnce(saveLocal);
    picker.resolves({ label: "Off" });
    await runConnectionPicker({ saveOff, saveLocal, saveRemote });
    sinon.assert.calledOnce(saveOff);
    picker.resolves(undefined);
    await runConnectionPicker({ saveOff, saveLocal, saveRemote });
    sinon.assert.calledOnce(saveLocal);
    sinon.assert.calledOnce(saveOff);
    sinon.assert.notCalled(saveRemote);
  });

  test("saves a normalized hosted origin", async () => {
    sinon.stub(vscode.window, "showInputBox").resolves(`${origin}/`);
    await runConnectionPicker({ saveOff, saveLocal, saveRemote }, PICK_REMOTE);
    sinon.assert.calledOnceWithExactly(saveRemote, origin, false);
  });

  test("requires confirmation before saving a repository suggestion", async () => {
    const show = sinon.stub(notices, "showInformationMessage").resolves(undefined);
    await runConnectionPicker({ saveOff, saveLocal, saveRemote, suggestion: origin }, PICK_REMOTE);
    sinon.assert.notCalled(saveRemote);

    show.resolves("Use This Server");
    await runConnectionPicker({ saveOff, saveLocal, saveRemote, suggestion: origin }, PICK_REMOTE);
    sinon.assert.calledOnceWithExactly(saveRemote, origin, true);
  });

  test("refuses a malformed hosted origin", async () => {
    sinon.stub(vscode.window, "showInputBox").resolves("http://operator.example.com");
    await runConnectionPicker({ saveOff, saveLocal, saveRemote }, PICK_REMOTE);
    sinon.assert.notCalled(saveRemote);
  });
});
