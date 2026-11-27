import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { randomBytes, randomUUID } from "node:crypto";
import * as fs from "node:fs/promises";
import { createRequire } from "node:module";
import { networkInterfaces } from "node:os";
import * as path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { promisify } from "node:util";
import type {
  Browser,
  BrowserContext,
  Page,
  chromium as Chromium,
} from "../../../e2e/node_modules/playwright";
import { startHosted, requiredEnv, type Daemon } from "./daemon";

const execute = promisify(execFile);

const accounts = ["admin", "alice", "bob"] as const;
const project = "alpha";
const clientId = "operator";
const grants = [
  "health:read",
  "projects:read",
  "queue:read",
  "tickets:read",
  "agents:read",
  "agents:focus",
  "mcp:connect",
];
const image = "quay.io/keycloak/keycloak:26.1.4";

export async function waitFor<T>(
  read: () => Promise<T | undefined> | T | undefined,
  description: string,
): Promise<T> {
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    const result = await read();
    if (result !== undefined) {
      return result;
    }
    await delay(200);
  }
  throw new Error(`Timed out waiting for ${description}`);
}

async function login(
  page: Page,
  origin: string,
  providerId: string,
  username: string,
  password: string,
) {
  await page.goto(`${origin}/api/v1/auth/oidc/login/${providerId}`);
  await page.locator("#username").fill(username);
  await page.locator("#password").fill(password);
  await Promise.all([
    page.waitForURL((url) => url.origin === origin),
    page.locator("#kc-login").click(),
  ]);
}

export async function hostedIdentity(extraToml = "") {
  const dir = await fs.mkdtemp(path.join(requiredEnv("OPERATOR_E2E_TEMP_DIR"), "keycloak-"));
  const container = `operator-idp-${randomUUID()}`;
  const host = Object.values(networkInterfaces())
    .flat()
    .find((a) => a?.family === "IPv4" && !a.internal)?.address;
  assert(host, "A non-loopback interface is required for the hosted test IdP");
  const external = process.env.OPERATOR_E2E_IDP_BASE;
  const bootstrap = external
    ? (await fs.readFile(requiredEnv("OPERATOR_E2E_IDP_PASSWORD_FILE"), "utf8")).trim()
    : randomBytes(24).toString("hex");
  const realm = `operator-${randomUUID()}`;
  let issuerBase = external ?? "";
  let realmCreated = false;
  const masterToken = async () => {
    const result = await fetch(`${issuerBase}/realms/master/protocol/openid-connect/token`, {
      method: "POST",
      body: new URLSearchParams({
        grant_type: "password",
        client_id: "admin-cli",
        username: "bootstrap",
        password: bootstrap,
      }),
    });
    assert.equal(result.status, 200, "Keycloak fixture administration");
    return ((await result.json()) as { access_token: string }).access_token;
  };
  const users = accounts.map((username) => ({
    id: randomUUID(),
    username,
    enabled: true,
    emailVerified: true,
    email: `${username}@operator.test`,
    firstName: username,
    lastName: "Example",
    credentials: [{ type: "password", value: randomBytes(24).toString("hex"), temporary: false }],
  }));
  const [admin, alice, bob] = users;
  assert(admin && alice && bob);
  let browser: Browser | undefined;
  let daemon: Daemon | undefined;
  const stop = async () => {
    try {
      await browser?.close();
    } finally {
      try {
        await daemon?.stop();
      } finally {
        if (!external) {
          await execute("docker", ["rm", "-f", container]).catch(() => undefined);
        } else if (realmCreated) {
          const deleted = await fetch(`${issuerBase}/admin/realms/${realm}`, {
            method: "DELETE",
            headers: { authorization: `Bearer ${await masterToken()}` },
          });
          assert.equal(deleted.status, 204);
        }
        await fs.rm(dir, { recursive: true, force: true });
      }
    }
  };
  try {
    if (!external) {
      await execute(
        "openssl",
        [
          "req",
          "-new",
          "-newkey",
          "rsa:2048",
          "-nodes",
          "-subj",
          "/CN=Operator test IdP",
          "-keyout",
          "issuer.key",
          "-out",
          "issuer.csr",
        ],
        { cwd: dir },
      );
      await fs.writeFile(
        path.join(dir, "issuer.ext"),
        `subjectAltName=IP:${host}\nextendedKeyUsage=serverAuth\n`,
      );
      await execute(
        "openssl",
        [
          "x509",
          "-req",
          "-days",
          "1",
          "-in",
          "issuer.csr",
          "-CA",
          requiredEnv("OPERATOR_E2E_CA_CERT"),
          "-CAkey",
          requiredEnv("OPERATOR_E2E_CA_KEY"),
          "-set_serial",
          `0x${randomBytes(16).toString("hex")}`,
          "-extfile",
          "issuer.ext",
          "-out",
          "issuer.crt",
        ],
        { cwd: dir },
      );
      await execute(
        "docker",
        [
          "run",
          "--detach",
          "--name",
          container,
          "-p",
          `${host}::8443`,
          "--user",
          "0:0",
          "-v",
          `${dir}:/fixture:ro`,
          "-e",
          "KC_BOOTSTRAP_ADMIN_USERNAME=bootstrap",
          "-e",
          "KC_BOOTSTRAP_ADMIN_PASSWORD",
          image,
          "start-dev",
          "--https-certificate-file=/fixture/issuer.crt",
          "--https-certificate-key-file=/fixture/issuer.key",
          "--hostname-strict=false",
        ],
        { env: { ...process.env, KC_BOOTSTRAP_ADMIN_PASSWORD: bootstrap } },
      );
      const { stdout } = await execute("docker", ["port", container, "8443/tcp"]);
      issuerBase = `https://${stdout.trim()}`;
    }
    await waitFor(async () => {
      try {
        return (
          (await fetch(`${issuerBase}/realms/master/.well-known/openid-configuration`)).ok ||
          undefined
        );
      } catch {
        return undefined;
      }
    }, "Keycloak discovery");

    daemon = await startHosted(
      (origin) => ({
        provider: "keycloak",
        issuer: `${issuerBase}/realms/${realm}`,
        client_id: clientId,
        redirect_uri: `${origin}/api/v1/auth/oidc/callback`,
      }),
      admin.id,
      extraToml,
    );
    const { origin, profileId, workspace, stateDir } = daemon;
    const created = await fetch(`${issuerBase}/admin/realms`, {
      method: "POST",
      headers: {
        authorization: `Bearer ${await masterToken()}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        realm,
        enabled: true,
        users,
        clients: [
          {
            clientId,
            publicClient: true,
            standardFlowEnabled: true,
            redirectUris: [`${origin}/api/v1/auth/oidc/callback`],
          },
        ],
      }),
    });
    assert.equal(created.status, 201, "Creating disposable Keycloak realm");
    realmCreated = true;
    const providers = (await (await fetch(`${origin}/api/v1/auth/oidc/providers`)).json()) as {
      id: string;
    }[];
    assert(providers[0]);
    const provider = providers[0].id;
    const requireE2e = createRequire(path.resolve(__dirname, "../../../../e2e/package.json"));
    const { chromium } = requireE2e("playwright") as { chromium: typeof Chromium };
    browser = await chromium.launch();
    const adminContext = await browser.newContext();
    const adminPage = await adminContext.newPage();
    await login(adminPage, origin, provider, admin.username, admin.credentials[0]!.value);
    const update = async (context: BrowserContext, url: string, data: object) => {
      const csrf = await context.request.get(`${origin}/api/v1/auth/csrf`);
      assert.equal(csrf.status(), 200);
      const { csrf_token } = (await csrf.json()) as { csrf_token: string };
      const result = await context.request.put(origin + url, {
        data,
        headers: { origin, "x-operator-csrf": csrf_token },
      });
      assert.equal(result.status(), 200);
    };
    const setProjects = (id: string, projects: string[], enabled = true) =>
      update(adminContext, `/api/v1/auth/users/${id}`, {
        administrator: false,
        enabled,
        grants: { profile_id: profileId, instance_wide: true, capabilities: grants, projects },
      });
    const userContext = await browser.newContext();
    const userPage = await userContext.newPage();
    let aliceId = "";
    for (const [user, assigned] of [
      [alice, project],
      [bob, "beta"],
    ] as const) {
      const pending = await browser.newContext();
      try {
        await login(
          await pending.newPage(),
          origin,
          provider,
          user.username,
          user.credentials[0]!.value,
        );
        const result = await adminContext.request.get(`${origin}/api/v1/auth/users`);
        assert.equal(result.status(), 200);
        const list = (await result.json()) as { id: string; subject: string }[];
        const account = list.find((a) => a.subject === user.id);
        assert(account, "First OIDC login provisions a pending account");
        await setProjects(account.id, [assigned]);
        if (user === alice) {
          aliceId = account.id;
        }
      } finally {
        await pending.close();
      }
    }
    const captured = requiredEnv("OPERATOR_E2E_EXTERNAL_URL_FILE");
    await fs.rm(captured, { force: true });
    const approveEditor = async () => {
      const approval = await waitFor(async () => {
        const value = await fs.readFile(captured, "utf8").catch(() => "");
        return value.trim().startsWith(`${origin}/#/device?`) ? value.trim() : undefined;
      }, "VS Code browser approval URL");
      const landing = await userPage.goto(approval);
      assert.equal(landing?.status(), 200, "Embedded device approval page");
      await userPage.getByRole("link", { name: /sign in with keycloak/i }).click();
      await userPage.locator("#username").fill(alice.username);
      await userPage.locator("#password").fill(alice.credentials[0]!.value);
      await userPage.locator("#kc-login").click();
      await userPage.getByRole("button", { name: "Approve", exact: true }).click();
      await userPage.getByRole("heading", { name: "Device approved" }).waitFor();
    };
    return {
      origin,
      profileId,
      workspace,
      stateDir,
      project,
      userContext,
      approveEditor,
      subject: aliceId,
      revokeAssignment: () => setProjects(aliceId, []),
      disable: () => setProjects(aliceId, [], false),
      stop,
    };
  } catch (error) {
    await stop();
    throw error;
  }
}
