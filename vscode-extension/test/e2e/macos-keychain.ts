import { execFile } from "node:child_process";
import { randomBytes } from "node:crypto";
import * as fs from "node:fs/promises";
import * as path from "node:path";

export async function createTestKeychain(home: string): Promise<() => Promise<void>> {
  const directory = path.join(home, "Library", "Keychains");
  await Promise.all(
    [directory, path.join(home, "Library", "Preferences")].map((dir) =>
      fs.mkdir(dir, { recursive: true, mode: 0o700 }),
    ),
  );
  const keychain = path.join(directory, "operator-e2e.keychain-db");
  const env = { ...process.env, HOME: home, CFFIXED_USER_HOME: home };
  function security(command: string, ...args: string[]): Promise<string> {
    return new Promise((resolve, reject) => {
      execFile("/usr/bin/security", [command, ...args], { env }, (error, stdout, stderr) => {
        if (error) {
          reject(new Error(`Test keychain ${command}: ${stderr.trim() || error.code}`));
        } else {
          resolve(stdout.trim());
        }
      });
    });
  }
  if (await security("list-keychains", "-d", "user")) {
    throw new Error("Expected an empty keychain search list in the disposable test home");
  }
  const password = randomBytes(32).toString("base64url");
  await security("create-keychain", "-p", password, keychain);
  const dispose = async () => {
    await security("delete-keychain", keychain);
  };
  try {
    await security("unlock-keychain", "-p", password, keychain);
    await security("default-keychain", "-d", "user", "-s", keychain);
    return dispose;
  } catch (error) {
    await dispose();
    throw error;
  }
}
