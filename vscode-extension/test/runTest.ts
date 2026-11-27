import { spawn } from "node:child_process";
import { once } from "node:events";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { runTests } from "@vscode/test-electron";
import { createTestKeychain } from "./e2e/macos-keychain";

async function main() {
  const temp = await fs.mkdtemp(path.join(os.tmpdir(), "op-test-"));
  const home = path.join(temp, "home");
  let disposeKeychain: (() => Promise<void>) | undefined;
  try {
    if (process.platform === "darwin") {
      disposeKeychain = await createTestKeychain(home);
    }
    const env = process.platform === "darwin" ? { HOME: home, CFFIXED_USER_HOME: home } : {};
    if (process.argv[2] === "--cli") {
      const cli = path.join(path.dirname(require.resolve("@vscode/test-cli")), "bin.mjs");
      const child = spawn(process.execPath, [cli, ...process.argv.slice(3)], {
        env: { ...process.env, ...env, OPERATOR_TEST_PROFILE_DIR: temp },
        stdio: "inherit",
      });
      const [code] = await once(child, "exit");
      process.exitCode = typeof code === "number" ? code : 1;
      return;
    }
    await runTests({
      vscodeExecutablePath: process.env.OPERATOR_E2E_VSCODE_PATH,
      extensionDevelopmentPath: path.resolve(__dirname, "../../"),
      extensionTestsPath: path.join(__dirname, "suite/index"),
      launchArgs: [
        `--user-data-dir=${path.join(temp, "p")}`,
        `--extensions-dir=${path.join(temp, "e")}`,
      ],
      extensionTestsEnv: env,
    });
  } finally {
    await disposeKeychain?.();
    await fs.rm(temp, { recursive: true, force: true });
  }
}

void main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
