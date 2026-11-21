import * as assert from "node:assert";
import * as sinon from "sinon";
import * as vscode from "vscode";
import { PICK_HERE, PICK_REMOTE, PICK_RUNNING } from "../../src/connection";
import type { ConnectionPickerDeps } from "../../src/connection-picker";
import { runConnectionPicker } from "../../src/connection-picker";

function deps(overrides: Partial<ConnectionPickerDeps> = {}): ConnectionPickerDeps & {
  saved: string[];
  signedIn: string[];
  ranLocal: number;
} {
  const state = {
    saved: [] as string[],
    signedIn: [] as string[],
    ranLocal: 0,
  };
  return {
    get saved() {
      return state.saved;
    },
    get signedIn() {
      return state.signedIn;
    },
    get ranLocal() {
      return state.ranLocal;
    },
    currentRemoteUrl: () => "",
    discoverLocal: () => Promise.resolve("http://localhost:7008"),
    probe: () => Promise.resolve("ok"),
    saveLocal: () => {
      state.saved.push("local");
      return Promise.resolve();
    },
    saveRemote: (origin: string) => {
      state.saved.push(origin);
      return Promise.resolve();
    },
    markChosen: () => {
      state.saved.push("chosen");
      return Promise.resolve();
    },
    signIn: (url: string) => {
      state.signedIn.push(url);
      return Promise.resolve();
    },
    runLocal: () => {
      state.ranLocal += 1;
      return Promise.resolve();
    },
    ...overrides,
  };
}

suite("runConnectionPicker()", () => {
  teardown(() => {
    sinon.restore();
  });

  test("a reachable local Operator is saved without starting a binary", async () => {
    const info = sinon.stub(vscode.window, "showInformationMessage") as sinon.SinonStub;
    info.resolves(PICK_RUNNING);
    const picker = deps();
    await runConnectionPicker(picker);
    assert.deepStrictEqual(picker.saved, ["local", "chosen"]);
    assert.strictEqual(picker.ranLocal, 0);
    assert.deepStrictEqual(picker.signedIn, []);
  });

  test("a missing local listener offers to run Operator for this folder", async () => {
    const info = sinon.stub(vscode.window, "showInformationMessage") as sinon.SinonStub;
    info.resolves(PICK_RUNNING);
    const warning = sinon.stub(vscode.window, "showWarningMessage") as sinon.SinonStub;
    warning.resolves(PICK_HERE);
    const picker = deps({ probe: () => Promise.resolve("down") });
    await runConnectionPicker(picker);
    assert.deepStrictEqual(picker.saved, ["local", "chosen"]);
    assert.strictEqual(picker.ranLocal, 1);
  });

  test("a remote origin is saved and sign-in runs when the server requires it", async () => {
    const info = sinon.stub(vscode.window, "showInformationMessage") as sinon.SinonStub;
    info.resolves(PICK_REMOTE);
    sinon.stub(vscode.window, "showInputBox").resolves("https://operator.kube.untra.casa/");
    const picker = deps({
      probe: (url: string) => Promise.resolve(url.startsWith("https://") ? "auth" : "ok"),
    });
    await runConnectionPicker(picker);
    assert.deepStrictEqual(picker.saved, ["https://operator.kube.untra.casa", "chosen"]);
    assert.deepStrictEqual(picker.signedIn, ["https://operator.kube.untra.casa"]);
    assert.strictEqual(picker.ranLocal, 0);
  });

  test("an http address is not saved", async () => {
    const info = sinon.stub(vscode.window, "showInformationMessage") as sinon.SinonStub;
    info.resolves(PICK_REMOTE);
    sinon.stub(vscode.window, "showInputBox").resolves("http://operator.kube.untra.casa");
    const error = sinon.stub(vscode.window, "showErrorMessage") as sinon.SinonStub;
    error.resolves(undefined);
    const picker = deps();
    await runConnectionPicker(picker);
    assert.deepStrictEqual(picker.saved, []);
    assert.strictEqual(error.calledOnce, true);
  });
});
