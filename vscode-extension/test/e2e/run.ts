import { execFile } from "node:child_process";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { promisify } from "node:util";
import { runPersistentWorkbench } from "./workbench";
import { createTestKeychain } from "./macos-keychain";
import { downloadAndUnzipVSCode, runTests } from "@vscode/test-electron";

const execute = promisify(execFile);
const extension = path.resolve(__dirname, "../../..");
const scenarios = path.join(__dirname, "scenarios");

function failureMessage(error: unknown): string {
  const failure = error as Error & { stderr?: string; stdout?: string };
  const text = [failure.message ?? String(error), failure.stderr, failure.stdout]
    .filter(Boolean)
    .join("\n");
  return text
    .replace(/(Bearer\s+)[^\s"',]+/gi, "$1[redacted]")
    .replace(
      /((?:user_code|device_code|access_token|refresh_token|password|secret|code)=)[^&\s]+/gi,
      "$1[redacted]",
    )
    .replace(/(eyJ[A-Za-z0-9_-]+\.){2}[A-Za-z0-9_-]+/g, "[redacted JWT]")
    .replace(/(opk_[a-zA-Z0-9_-]+)/g, "[redacted key]")
    .slice(-12000);
}

async function runScenario(scenario: string, executable: string) {
  const temp = await fs.mkdtemp(path.join(os.tmpdir(), "operator-vscode-e2e-"));
  const workspace = path.join(temp, "workspace");
  const localHome = path.join(temp, "home");
  const profile = path.join(temp, "profile");
  const extensions = path.join(temp, "extensions");
  const tools = path.join(temp, "bin");
  const hosted = ["identity", "hosted-terminal", "hosted"].includes(scenario);
  const host = process.env.OPERATOR_E2E_HOST ?? "operator.localhost";
  const ca = path.join(temp, "ca.pem");
  const caKey = path.join(temp, "ca.key");
  const cert = path.join(temp, "server.pem");
  const key = path.join(temp, "server.key");
  let disposeKeychain: (() => Promise<void>) | undefined;
  try {
    await Promise.all(
      [workspace, localHome, profile, extensions, tools].map((dir) => fs.mkdir(dir)),
    );
    if (process.platform === "darwin" && !hosted) {
      disposeKeychain = await createTestKeychain(localHome);
    }
    if (hosted) {
      if (process.platform !== "linux") {
        throw new Error("Hosted browser E2E requires Linux with NSS and a disposable SSH host");
      }
      const suppliedCert = process.env.OPERATOR_E2E_CA_CERT;
      const suppliedKey = process.env.OPERATOR_E2E_CA_KEY;
      if (suppliedCert || suppliedKey) {
        if (!suppliedCert || !suppliedKey) {
          throw new Error(
            "Provide both CA certificate and CA private key for the disposable identity provider",
          );
        }
        await fs.copyFile(suppliedCert, ca);
        await fs.copyFile(suppliedKey, caKey);
        await fs.chmod(caKey, 0o600);
      } else {
        await execute("openssl", [
          "req",
          "-x509",
          "-nodes",
          "-newkey",
          "rsa:2048",
          "-subj",
          "/CN=Operator test CA",
          "-days",
          "1",
          "-keyout",
          caKey,
          "-out",
          ca,
        ]);
      }
      const csr = path.join(temp, "server.csr");
      await execute("openssl", [
        "req",
        "-nodes",
        "-newkey",
        "rsa:2048",
        "-subj",
        `/CN=${host}`,
        "-keyout",
        key,
        "-out",
        csr,
      ]);
      const ext = path.join(temp, "server.ext");
      await fs.writeFile(ext, `subjectAltName=DNS:${host}\nextendedKeyUsage=serverAuth\n`);
      await execute("openssl", [
        "x509",
        "-req",
        "-in",
        csr,
        "-CA",
        ca,
        "-CAkey",
        caKey,
        "-CAcreateserial",
        "-days",
        "1",
        "-out",
        cert,
        "-extfile",
        ext,
      ]);
      const nss = path.join(localHome, ".pki", "nssdb");
      await fs.mkdir(nss, { recursive: true });
      await execute("certutil", ["-N", "-d", `sql:${nss}`, "--empty-password"]);
      await execute("certutil", [
        "-A",
        "-d",
        `sql:${nss}`,
        "-n",
        "Operator test CA",
        "-t",
        "C,,",
        "-i",
        ca,
      ]);
      await fs.writeFile(
        path.join(tools, "xdg-open"),
        '#!/bin/sh\nprintf "%s\\n" "$1" > "$OPERATOR_E2E_EXTERNAL_URL_FILE"\n',
        { mode: 0o700 },
      );
    }
    if (scenario === "hosted-terminal") {
      const { prepareRemoteSsh } = await import("./remote-ssh.js");
      await prepareRemoteSsh(executable, extensions, profile);
    }
    const options = {
      vscodeExecutablePath: executable,
      extensionDevelopmentPath: extension,
      extensionTestsPath: path.join(scenarios, `${scenario}.js`),
      launchArgs: [
        workspace,
        `--user-data-dir=${profile}`,
        `--extensions-dir=${extensions}`,
        "--disable-workspace-trust",
        "--skip-welcome",
        "--skip-release-notes",
        ...(process.platform === "linux" ? ["--password-store=basic"] : []),
      ],
      extensionTestsEnv: {
        HOME: localHome,
        USERPROFILE: localHome,
        ...(process.platform === "darwin" ? { CFFIXED_USER_HOME: localHome } : {}),
        XDG_CONFIG_HOME: path.join(localHome, ".config"),
        OPERATOR_PROFILE_REGISTRY: path.join(temp, "profiles.sqlite"),
        OPERATOR_E2E_TEMP_DIR: temp,
        OPERATOR_E2E_WORKSPACE: workspace,
        OPERATOR_E2E_HOST: host,
        OPERATOR_E2E_VSCODE_PATH: executable,
        OPERATOR_E2E_SERVER_CERT: cert,
        OPERATOR_E2E_SERVER_KEY: key,
        OPERATOR_E2E_CA_CERT: ca,
        OPERATOR_E2E_CA_KEY: caKey,
        OPERATOR_E2E_EXTERNAL_URL_FILE: path.join(temp, "browser-url"),
        PLAYWRIGHT_BROWSERS_PATH:
          process.env.PLAYWRIGHT_BROWSERS_PATH ??
          path.join(os.homedir(), ".cache", "ms-playwright"),
        ...(hosted ? { NODE_EXTRA_CA_CERTS: ca, SSL_CERT_FILE: ca } : {}),
        PATH: tools + path.delimiter + process.env.PATH,
        OPERATOR_E2E_BINARY:
          process.env.OPERATOR_E2E_BINARY ?? path.resolve(extension, "../target/debug/operator"),
      },
    };
    if (scenario === "secret-storage") {
      for (const phase of ["write", "read"]) {
        await runPersistentWorkbench({
          ...options,
          extensionTestsEnv: { ...options.extensionTestsEnv, OPERATOR_E2E_STORAGE_PHASE: phase },
        });
        if (process.platform === "darwin" && phase === "write") {
          const keychain = path.join(localHome, "Library/Keychains/operator-e2e.keychain-db");
          await execute("/usr/bin/security", ["find-generic-password", keychain], {
            env: { ...process.env, HOME: localHome, CFFIXED_USER_HOME: localHome },
          });
        }
      }
    } else if (scenario === "identity" || scenario === "hosted" || scenario === "hosted-terminal") {
      const driver = {
        identity: "identity-journey.js",
        hosted: "hosted-startup.js",
        "hosted-terminal": "terminal-journey.js",
      }[scenario];
      await execute(process.execPath, [path.join(__dirname, driver), JSON.stringify(options)], {
        env: { ...process.env, ...options.extensionTestsEnv },
        maxBuffer: 8 * 1024 * 1024,
      });
    } else {
      await runTests(options);
    }
  } finally {
    await disposeKeychain?.();
    await fs.rm(temp, { recursive: true, force: true });
  }
}

async function main() {
  const names = (await fs.readdir(scenarios))
    .filter((name) => name.endsWith(".js"))
    .map((name) => name.slice(0, -3))
    .toSorted();
  const requested = process.argv.slice(2);
  const selected = requested.length ? requested : names;
  if (!selected.length || selected.some((name) => !names.includes(name))) {
    throw new Error("Unknown or missing E2E scenario");
  }
  const executable = process.env.OPERATOR_E2E_VSCODE_PATH ?? (await downloadAndUnzipVSCode());
  for (const name of selected) {
    console.log(`Running VS Code E2E: ${name}`);
    await runScenario(name, executable);
  }
}
void main().catch((error: unknown) => {
  console.error(failureMessage(error));
  process.exitCode = 1;
});
