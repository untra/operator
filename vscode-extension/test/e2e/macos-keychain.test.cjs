const { test } = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs/promises");
const os = require("node:os");
const path = require("node:path");
const { execFile } = require("node:child_process");
const exec = require("node:util").promisify(execFile);
const { createTestKeychain } = require("../../out/test/e2e/macos-keychain");

void test(
  "a disposable macOS test home has its own writable default keychain",
  { skip: process.platform !== "darwin" },
  async () => {
    const home = await fs.mkdtemp(path.join(os.tmpdir(), "operator-keychain-test-"));
    const env = { ...process.env, HOME: home, CFFIXED_USER_HOME: home };
    const security = async (...args) =>
      (await exec("/usr/bin/security", args, { env })).stdout.trim();
    try {
      const dispose = await createTestKeychain(home);
      try {
        const defaultPath = (await security("default-keychain", "-d", "user")).replaceAll('"', "");
        assert.equal(
          await fs.realpath(defaultPath),
          await fs.realpath(path.join(home, "Library/Keychains/operator-e2e.keychain-db")),
        );
        await security(
          "add-generic-password",
          "-a",
          "e2e",
          "-s",
          "operator-e2e-test",
          "-w",
          "fixture-value",
        );
        assert.equal(
          await security(
            "find-generic-password",
            "-a",
            "e2e",
            "-s",
            "operator-e2e-test",
            "-w",
            defaultPath,
          ),
          "fixture-value",
        );
      } finally {
        await dispose();
      }
      assert.equal(await security("list-keychains", "-d", "user"), "");
    } finally {
      await fs.rm(home, { recursive: true, force: true });
    }
  },
);
