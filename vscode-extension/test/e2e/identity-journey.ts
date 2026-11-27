import assert from "node:assert/strict";
import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { runTests } from "@vscode/test-electron";
import { runPersistentWorkbench } from "./workbench";
import { hostedIdentity, waitFor } from "./hosted-identity";
import { requiredEnv } from "./daemon";

async function main() {
  const fixture = await hostedIdentity();
  try {
    const options = JSON.parse(process.argv[2]!) as Parameters<typeof runTests>[0];
    const { origin, subject, userContext, profileId, stateDir } = fixture;
    const launch = (phase: string) =>
      runPersistentWorkbench({
        ...options,
        extensionTestsEnv: {
          ...options.extensionTestsEnv,
          OPERATOR_E2E_IDENTITY_PHASE: phase,
          OPERATOR_E2E_ORIGIN: origin,
          OPERATOR_E2E_SUBJECT: subject,
        },
      });
    await Promise.all([launch("sign-in"), fixture.approveEditor()]);
    const dir = requiredEnv("OPERATOR_E2E_TEMP_DIR");
    const keyLifecycle = async () => {
      await waitFor(
        async () => fs.stat(path.join(dir, "identity-refreshed")).catch(() => undefined),
        "VS Code profile reload and refresh",
      );
      const csrf = await userContext.request.get(`${origin}/api/v1/auth/csrf`);
      assert.equal(csrf.status(), 200);
      const mutation = {
        origin,
        "x-operator-csrf": ((await csrf.json()) as { csrf_token: string }).csrf_token,
      };
      const issue = async () => {
        const created = await userContext.request.post(`${origin}/api/v1/auth/keys`, {
          headers: mutation,
          data: {
            name: "VS Code E2E automation",
            expires_in_days: 1,
            grants: {
              profile_id: profileId,
              instance_wide: true,
              capabilities: ["tickets:read", "mcp:connect"],
              projects: ["alpha"],
            },
          },
        });
        assert.equal(created.status(), 200);
        const { secret, key } = (await created.json()) as { secret: string; key: { id: string } };
        const exchange = () =>
          fetch(`${origin}/api/v1/auth/token`, {
            method: "POST",
            headers: { "content-type": "application/json" },
            body: JSON.stringify({ grant_type: "operator:access-key", access_key: secret }),
          });
        const exchanged = await exchange();
        assert.equal(exchanged.status, 200);
        const { access_token } = (await exchanged.json()) as { access_token: string };
        const headers = { authorization: `Bearer ${access_token}` };
        const read = () => fetch(`${origin}/api/v1/tickets/TASK-alpha`, { headers });
        assert.equal((await read()).status, 200);
        return { key, exchange, read, headers };
      };
      const first = await issue();
      const revoked = await userContext.request.delete(
        `${origin}/api/v1/auth/keys/${first.key.id}`,
        { headers: mutation },
      );
      assert.equal(revoked.status(), 200);
      assert.equal((await first.read()).ok, false);
      assert.equal((await first.exchange()).ok, false);
      const second = await issue();
      await fixture.revokeAssignment();
      assert.equal((await second.read()).status, 404);
      await fs.writeFile(path.join(dir, "identity-unassigned"), "");
      const sse = await fetch(`${origin}/api/v1/mcp/sse`, {
        headers: second.headers,
        signal: AbortSignal.timeout(30_000),
      });
      assert.equal(sse.status, 200);
      const stream = sse.body!.getReader();
      assert.equal((await stream.read()).done, false);
      const license = path.join(stateDir, "license.key");
      const withheld = path.join(stateDir, "license.withheld");
      await fs.rename(license, withheld);
      assert.equal((await second.exchange()).ok, false);
      assert.equal((await second.read()).ok, false);
      while (!(await stream.read()).done) {
        /* consume keepalives until the server closes the stream */
      }
      await fs.rename(withheld, license);
      assert.equal((await second.exchange()).status, 200);
      await fixture.disable();
      assert.equal((await second.exchange()).ok, false);
      await fs.writeFile(path.join(dir, "identity-disabled"), "");
    };
    await Promise.all([launch("reload"), keyLifecycle()]);
  } finally {
    await fixture.stop();
  }
}
void main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
