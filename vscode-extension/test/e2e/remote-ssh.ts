import { execFile } from "node:child_process";
import { randomUUID } from "node:crypto";
import * as fs from "node:fs/promises";
import * as path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { promisify } from "node:util";
import { resolveCliPathFromVSCodeExecutablePath } from "@vscode/test-electron";

const execute = promisify(execFile);
const image = "operator-vscode-e2e-ssh";
export const remotePath = "/home/agent/work";

export async function prepareRemoteSsh(
  executable: string,
  extensionsDir: string,
  userDataDir: string,
): Promise<void> {
  const extensionPath = path.resolve(__dirname, "../../..");
  const built = process.env.OPERATOR_E2E_VSIX;
  const vsix = built ?? path.join(path.dirname(extensionsDir), "operator-terminals.vsix");
  if (!built) {
    await execute(
      path.join(extensionPath, "node_modules", ".bin", "vsce"),
      ["package", "--out", vsix],
      { cwd: extensionPath, timeout: 180000 },
    );
  }
  await execute(
    resolveCliPathFromVSCodeExecutablePath(executable),
    [
      ...(process.getuid?.() === 0 ? ["--no-sandbox"] : []),
      "--user-data-dir",
      userDataDir,
      "--extensions-dir",
      extensionsDir,
      "--install-extension",
      process.env.OPERATOR_E2E_REMOTE_SSH_VSIX ?? "ms-vscode-remote.remote-ssh",
      "--install-extension",
      vsix,
    ],
    { timeout: 120000 },
  );
}

async function target(host: string, config: string, stopHost: () => Promise<void>) {
  const session = `operator-${randomUUID()}`;
  const marker = `${remotePath}/${session}.attached`;
  const remote = (...args: string[]) =>
    execute(
      "ssh",
      [
        "-F",
        config,
        "-o",
        "BatchMode=yes",
        host,
        args.map((arg) => `'${arg.replaceAll("'", "'\\''")}'`).join(" "),
      ],
      { timeout: 15000 },
    );
  await remote("tmux", "new-session", "-d", "-s", session, "-c", remotePath);
  try {
    await remote(
      "tmux",
      "set-hook",
      "-t",
      session,
      "client-attached",
      `run-shell 'touch ${marker}'`,
    );
  } catch (error) {
    await remote("tmux", "kill-session", "-t", session);
    throw error;
  }
  return {
    host,
    session,
    config,
    stop: async () => {
      try {
        await remote("tmux", "kill-session", "-t", session);
        await remote("rm", "-f", marker);
      } finally {
        await stopHost();
      }
    },
    attached: () =>
      remote("test", "-f", marker).then(
        () => true,
        () => false,
      ),
  };
}

export async function startRemoteSsh(temp: string) {
  const external = process.env.OPERATOR_E2E_SSH_CONFIG;
  if (external) {
    return target("operator-ssh", external, async () => {});
  }
  const host = `operator-${randomUUID()}`;
  const key = path.join(temp, "fixture-key");
  const known = path.join(temp, "known-hosts");
  const config = path.join(temp, "fixture-ssh-config");
  const docker = async (...args: string[]) =>
    (await execute("docker", args, { timeout: 180000 })).stdout.trim();
  const stop = async () => {
    await docker("rm", "--force", host);
  };
  await execute("ssh-keygen", ["-q", "-t", "ed25519", "-N", "", "-f", key]);
  await docker("build", "--tag", image, path.resolve(__dirname, "../../../test/e2e/ssh"));
  await docker("run", "--detach", "--name", host, "--publish", "127.0.0.1::22", image);
  try {
    await docker("cp", `${key}.pub`, `${host}:/home/agent/.ssh/authorized_keys`);
    await docker("exec", host, "chown", "agent:agent", "/home/agent/.ssh/authorized_keys");
    const port = (await docker("port", host, "22/tcp")).split(":").pop();
    const deadline = Date.now() + 10000;
    let publicKey = "";
    while (Date.now() < deadline && !publicKey) {
      publicKey = await docker("exec", host, "cat", "/etc/ssh/ssh_host_ed25519_key.pub").catch(
        () => "",
      );
      if (!publicKey) {
        await delay(100);
      }
    }
    if (!publicKey) {
      throw new Error("Disposable SSH host did not publish its host key");
    }
    await fs.writeFile(known, `[127.0.0.1]:${port} ${publicKey}\n`);
    await fs.writeFile(
      config,
      `Host ${host}\n  HostName 127.0.0.1\n  User agent\n  Port ${port}\n  IdentityFile "${key}"\n  IdentitiesOnly yes\n  UserKnownHostsFile "${known}"\n  StrictHostKeyChecking yes\n`,
      { mode: 0o600 },
    );
    return await target(host, config, stop);
  } catch (error) {
    await stop();
    throw error;
  }
}
