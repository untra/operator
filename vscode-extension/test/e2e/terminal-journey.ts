import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { TestOptions } from "@vscode/test-electron";
import { requiredEnv } from "./daemon";
import { hostedIdentity, waitFor } from "./hosted-identity";
import { remotePath, startRemoteSsh } from "./remote-ssh";
import { runPersistentWorkbench } from "./workbench";

async function main() {
  const options = JSON.parse(process.argv[2]!) as TestOptions;
  const dir = requiredEnv("OPERATOR_E2E_TEMP_DIR");
  const host = await startRemoteSsh(dir);
  try {
    const fixture = await hostedIdentity(
      `\n[[targets]]\nname = "${host.host}"\nkind = "ssh"\nssh_alias = "${host.host}"\nworkdir = "${remotePath}"\n`,
    );
    try {
      const agent = "attach-e2e";
      const now = new Date().toISOString();
      await fs.writeFile(
        path.join(fixture.stateDir, "state.json"),
        JSON.stringify({
          paused: true,
          completed: [],
          agents: [
            {
              id: agent,
              ticket_id: "TASK-alpha",
              ticket_type: "TASK",
              project: fixture.project,
              status: "running",
              started_at: now,
              last_activity: now,
              paired: false,
              session_name: host.session,
              session_wrapper: "tmux",
              target_name: host.host,
              remote_host: host.host,
            },
          ],
        }),
      );

      const userDir = path.join(dir, "profile", "User");
      await fs.mkdir(userDir, { recursive: true });
      await fs.writeFile(
        path.join(userDir, "settings.json"),
        JSON.stringify({
          "remote.SSH.configFile": host.config,
          "remote.SSH.remotePlatform": { [host.host]: "linux" },
          "terminal.integrated.confirmOnExit": "never",
          "window.confirmBeforeClose": "never",
        }),
      );
      const observation = async () => {
        await fixture.approveEditor();
        await waitFor(
          async () => (await host.attached()) || undefined,
          "SSH tmux client attachment",
        );
        await fs.writeFile(path.join(dir, "terminal-attached"), "");
        await waitFor(
          () => fs.stat(path.join(dir, "terminal-revoke")).catch(() => undefined),
          "request to revoke assignment",
        );
        await fixture.revokeAssignment();
        await fs.writeFile(path.join(dir, "terminal-revoked"), "");
      };
      await Promise.all([
        runPersistentWorkbench(
          {
            ...options,
            extensionTestsEnv: {
              ...options.extensionTestsEnv,
              OPERATOR_E2E_ORIGIN: fixture.origin,
              OPERATOR_E2E_SSH_HOST: host.host,
            },
          },
          240_000,
        ),
        observation(),
      ]);
    } finally {
      await fixture.stop();
    }
  } finally {
    await host.stop();
  }
}
void main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
