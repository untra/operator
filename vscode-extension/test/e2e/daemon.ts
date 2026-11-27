import { execFile, spawn } from "node:child_process";
import { once } from "node:events";
import * as fs from "node:fs/promises";
import * as http from "node:http";
import * as https from "node:https";
import type { AddressInfo } from "node:net";
import * as path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { promisify } from "node:util";

const execute = promisify(execFile);
const tickets = ".tickets";

export interface Daemon {
  origin: string;
  profileId: string;
  workspace: string;
  stateDir: string;
  stop(): Promise<void>;
  restart(): Promise<void>;
}

type Connection = (origin: string) => Record<string, unknown>;

async function start(
  workspace: string,
  connection?: Connection,
  subject?: string,
  extraToml = "",
): Promise<Daemon> {
  const binary = process.env.OPERATOR_E2E_BINARY;
  if (!binary) {
    throw new Error("OPERATOR_E2E_BINARY is required");
  }
  const stateDir = path.join(workspace, tickets, "operator");
  const sessionFile = path.join(stateDir, "api-session.json");
  const config = path.join(stateDir, "config.toml");
  let profileId = "";
  await fs.mkdir(stateDir, { recursive: true });
  const queue = path.join(workspace, tickets, "queue");
  await fs.mkdir(queue, { recursive: true });
  for (const project of ["alpha", "beta"]) {
    await fs.mkdir(path.join(workspace, project), { recursive: true });
    await fs.writeFile(path.join(workspace, project, "AGENTS.md"), project);
    const id = `TASK-${project}`;
    await fs.writeFile(
      path.join(queue, `20260101-0900-TASK-${project}-fixture.md`),
      `---\nid: ${id}\nstatus: queued\npriority: normal\nstep: plan\n---\n\n# Task: ${project}\n`,
    );
  }

  let upstream = "";
  let origin = "";
  let proxy: https.Server | undefined;
  if (connection) {
    proxy = https.createServer(
      {
        key: await fs.readFile(requiredEnv("OPERATOR_E2E_SERVER_KEY")),
        cert: await fs.readFile(requiredEnv("OPERATOR_E2E_SERVER_CERT")),
      },
      (request, response) => {
        if (!upstream) {
          response.writeHead(503).end();
          return;
        }
        const forwarded = http.request(
          upstream + request.url,
          { method: request.method, headers: request.headers },
          (res) => {
            response.writeHead(res.statusCode ?? 502, res.headers);
            res.pipe(response);
          },
        );
        forwarded.on("error", () => response.destroy());
        response.on("close", () => forwarded.destroy());
        request.pipe(forwarded);
      },
    );
    proxy.listen(0, "127.0.0.1");
    await once(proxy, "listening");
    origin = `https://${requiredEnv("OPERATOR_E2E_HOST")}:${(proxy.address() as AddressInfo).port}`;
  }
  let child: ReturnType<typeof spawn> | undefined;
  let exited: Promise<unknown> | undefined;
  async function stopProcess() {
    if (child?.exitCode === null && child.signalCode === null) {
      child.kill("SIGTERM");
      const force = setTimeout(() => child?.kill("SIGKILL"), 5000);
      try {
        await exited;
      } finally {
        clearTimeout(force);
      }
    }
  }
  async function launch() {
    child = spawn(binary!, ["--config", config, "api", "--port", "0"], {
      cwd: workspace,
      stdio: "ignore",
    });
    exited = once(child, "exit");
    exited.catch(() => {});
    for (let attempt = 0; attempt < 300; attempt++) {
      if (child.exitCode !== null || child.signalCode !== null) {
        throw new Error(`Operator fixture exited (${child.exitCode ?? child.signalCode})`);
      }
      try {
        const session = JSON.parse(await fs.readFile(sessionFile, "utf8")) as {
          port: number;
          pid: number;
        };
        if (session.pid === child.pid) {
          upstream = `http://127.0.0.1:${session.port}`;
          if ((await fetch(`${upstream}/livez`)).ok) {
            return;
          }
        }
      } catch {
        /* Wait for the daemon's first registration. */
      }
      await delay(100);
    }
    throw new Error("Operator did not become ready");
  }
  try {
    await fs.writeFile(
      config,
      `projects = ["alpha", "beta"]\n[paths]\ntickets = ${JSON.stringify(path.join(workspace, tickets))}\nstate = ${JSON.stringify(stateDir)}\nprojects = ${JSON.stringify(workspace)}\n[rest_api]\ndeployment_mode = "${connection ? "hosted" : "local"}"\nhost = "127.0.0.1"\nport = 0\n${connection ? `public_url = "${origin}"\n` : ""}${extraToml}`,
    );
    await execute(binary, ["--config", config, "auth", "status"], { cwd: workspace });
    const registered = await fs.readFile(config, "utf8");
    const profile = /^\[profile\]\s*\nid = "([0-9a-f-]{36})"/m.exec(registered);
    if (!profile?.[1]) {
      throw new Error("Operator failed to register the E2E profile");
    }
    profileId = profile[1];
    if (connection && subject) {
      const fixture = path.resolve(__dirname, "../../../test/e2e/license-fixture.mjs");
      await execute("node", [fixture, "issue", profileId, stateDir]);
      const connectionFile = path.join(stateDir, "connection.json");
      await fs.writeFile(connectionFile, JSON.stringify(connection(origin)), { mode: 0o600 });
      await execute(
        binary,
        [
          "--config",
          config,
          "auth",
          "provision-hosted-admin",
          "--connection",
          connectionFile,
          "--subject",
          subject,
        ],
        { cwd: workspace },
      );
    }
    await launch();
    return {
      get origin() {
        return origin || upstream;
      },
      profileId,
      workspace,
      stateDir,
      restart: async () => {
        await stopProcess();
        await launch();
      },
      stop: async () => {
        proxy?.closeAllConnections();
        proxy?.close();
        await stopProcess();
      },
    };
  } catch (error) {
    proxy?.closeAllConnections();
    proxy?.close();
    await stopProcess();
    throw error;
  }
}

export function requiredEnv(key: string): string {
  const value = process.env[key];
  if (!value) {
    throw new Error(`Missing ${key}`);
  }
  return value;
}

export function startLocal(workspace = requiredEnv("OPERATOR_E2E_WORKSPACE")): Promise<Daemon> {
  return start(workspace);
}

export function startHosted(
  connection: Connection,
  subject: string,
  extraToml = "",
): Promise<Daemon> {
  return start(
    path.join(requiredEnv("OPERATOR_E2E_TEMP_DIR"), "hosted"),
    connection,
    subject,
    extraToml,
  );
}
