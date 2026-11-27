import * as assert from "node:assert/strict";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import * as sinon from "sinon";
import * as vscode from "vscode";
import { writeConfigField } from "../../src/config-panel";
import { getResolvedConfigPath } from "../../src/config-paths";
import { findParentTicketsDir } from "../../src/tickets-dir";

suite("Workspace configuration targets", () => {
  let root: string;
  let settings: Record<string, string>;
  setup(async () => {
    root = await fs.mkdtemp(path.join(os.tmpdir(), "operator-settings-"));
    settings = {};
    const config = vscode.workspace.getConfiguration("operator");
    const get = sinon.stub().callsFake((key: string) => settings[key]);
    const workspaceConfig: vscode.WorkspaceConfiguration = {
      ...config,
      get,
      update: (key: string, value: string) => {
        settings[key] = value;
        return Promise.resolve();
      },
    };
    sinon.stub(vscode.workspace, "getConfiguration").returns(workspaceConfig);
  });
  teardown(async () => {
    sinon.restore();
    await fs.rm(root, { recursive: true, force: true });
  });

  test("typed first directory works without an open folder or existing config", async () => {
    sinon.stub(vscode.workspace, "workspaceFolders").value(undefined);
    await writeConfigField("primary", "working_directory", root);
    assert.equal(settings.workingDirectory, root);
    await assert.rejects(writeConfigField("agents", "max_parallel", 4), { code: "ENOENT" });
    assert.deepEqual(await fs.readdir(root), []);
  });

  test("explicit tickets path wins over the parent and sparse TOML preserves adjacent fields", async () => {
    const workspace = path.join(root, "repo");
    const custom = path.join(workspace, "board");
    await fs.mkdir(path.join(root, ".tickets"));
    await fs.mkdir(path.join(custom, "operator"), { recursive: true });
    sinon.stub(vscode.workspace, "workspaceFolders").value([{ uri: vscode.Uri.file(workspace) }]);
    settings.ticketsDir = "board";
    const file = path.join(custom, "operator", "config.toml");
    assert.equal(await findParentTicketsDir(), custom);
    assert.equal(getResolvedConfigPath(), file);
    await fs.writeFile(file, '[git]\nprovider="gitlab"\nbranch_format="custom"\n');
    await writeConfigField("git", "provider", null);
    await writeConfigField("git.gitea", "token_env", "TEAM_TOKEN");
    const text = await fs.readFile(file, "utf8");
    assert.doesNotMatch(text, /provider\s*=/);
    assert.match(text, /branch_format\s*=\s*"custom"/);
    assert.match(text, /token_env\s*=\s*"TEAM_TOKEN"/);
  });
});
