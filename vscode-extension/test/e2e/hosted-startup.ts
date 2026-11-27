import * as assert from "node:assert/strict";
import * as fs from "node:fs/promises";
import * as path from "node:path";
import { runTests, type TestOptions } from "@vscode/test-electron";
import { requiredEnv, startHosted } from "./daemon";

async function main() {
  const options = JSON.parse(process.argv[2] ?? "{}") as TestOptions;
  const root = requiredEnv("OPERATOR_E2E_TEMP_DIR");
  const workspace = requiredEnv("OPERATOR_E2E_WORKSPACE");
  const coverage = path.join(root, "startup-coverage");
  const daemon = await startHosted(
    (origin) => ({
      provider: "keycloak",
      issuer: "https://idp.example.test/realms/operator",
      client_id: "operator-vscode",
      redirect_uri: `${origin}/api/v1/auth/oidc/callback`,
    }),
    "first-administrator",
  );
  try {
    await assert.rejects(fs.access(path.join(daemon.stateDir, "local-token")));
    const inactive = await fetch(`${daemon.origin}/api/v1/auth/bootstrap`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ new_password: "test-only-password" }),
    });
    assert.equal(inactive.status, 403);
    const anonymous = await fetch(`${daemon.origin}/api/v1/health`, {
      headers: { authorization: "Bearer test-local-unlock" },
    });
    assert.equal(anonymous.status, 401);
    await fs.mkdir(path.join(workspace, ".tickets", "operator"), { recursive: true });
    await fs.writeFile(
      path.join(workspace, ".tickets", "operator", "config.toml"),
      "[rest_api]\nenabled = true\n",
    );
    const sentinel = path.join(root, "local-operator");
    await fs.writeFile(
      sentinel,
      '#!/bin/sh\ntouch "$OPERATOR_E2E_TEMP_DIR/local-process-started"\nexit 1\n',
      { mode: 0o700 },
    );
    const settings = path.join(root, "profile", "User");
    await fs.mkdir(settings, { recursive: true });
    await fs.writeFile(
      path.join(settings, "settings.json"),
      JSON.stringify({ "operator.apiUrl": daemon.origin, "operator.operatorPath": sentinel }),
    );
    await fs.mkdir(coverage);
    await runTests({
      ...options,
      extensionTestsEnv: {
        ...options.extensionTestsEnv,
        NODE_V8_COVERAGE: coverage,
        OPERATOR_E2E_ORIGIN: daemon.origin,
      },
    });
    await assert.rejects(fs.access(path.join(root, "local-process-started")));
    await assert.rejects(
      fs.access(path.join(workspace, ".tickets", "operator", "api-session.json")),
    );
  } finally {
    await daemon.stop();
  }
}
void main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
