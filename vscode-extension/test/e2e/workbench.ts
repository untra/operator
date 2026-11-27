import { spawn } from "node:child_process";
import { randomUUID } from "node:crypto";
import { once } from "node:events";
import { watchFile, unwatchFile } from "node:fs";
import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { TestOptions } from "@vscode/test-electron";

const extension = path.resolve(__dirname, "../../..");
const driver = path.join(extension, "test/e2e/fixtures/secret-storage");

// VS Code uses in-memory application storage whenever --extensionTestsPath is supplied.
export async function runPersistentWorkbench(options: TestOptions, timeoutMs = 120_000) {
  const env = options.extensionTestsEnv ?? {};
  if (!options.vscodeExecutablePath || !env.OPERATOR_E2E_TEMP_DIR) {
    throw new Error("Incomplete workbench fixture");
  }
  const resultPath = path.join(env.OPERATOR_E2E_TEMP_DIR, `workbench-${randomUUID()}.json`);
  const child = spawn(
    options.vscodeExecutablePath,
    [
      ...(options.launchArgs ?? []),
      ...(process.platform === "linux" && process.getuid?.() === 0
        ? ["--no-sandbox", "--disable-gpu-sandbox"]
        : []),
      "--new-window",
      "--disable-updates",
      `--extensionDevelopmentPath=${extension}`,
      `--extensionDevelopmentPath=${driver}`,
    ],
    {
      env: {
        ...process.env,
        ...env,
        OPERATOR_E2E_MODULE: options.extensionTestsPath,
        OPERATOR_E2E_RESULT: resultPath,
      },
      stdio: "inherit",
    },
  );
  let requestedExit = false;
  let closeTimer: NodeJS.Timeout | undefined;
  const multipleWindows = Boolean(env.OPERATOR_E2E_SSH_HOST);
  if (multipleWindows) {
    watchFile(resultPath, { interval: 250 }, (stat) => {
      if (stat.size && !closeTimer) {
        closeTimer = setTimeout(() => {
          requestedExit = child.kill("SIGTERM");
        }, 3000);
      }
    });
  }
  const timeout = setTimeout(() => child.kill("SIGKILL"), timeoutMs);
  try {
    const [status, signal] = await once(child, "exit");
    if (status !== 0 && !(requestedExit && signal === "SIGTERM")) {
      throw new Error("VS Code workbench exited unsuccessfully");
    }
    const result = JSON.parse(await fs.readFile(resultPath, "utf8")) as {
      passed: boolean;
      error?: string;
    };
    if (!result.passed) {
      throw new Error(result.error ?? "Workbench scenario failed");
    }
  } finally {
    clearTimeout(timeout);
    clearTimeout(closeTimer);
    if (multipleWindows) {
      unwatchFile(resultPath);
    }
  }
}
