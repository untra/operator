// Local stand-in for the Coder registry's `~test` helper module.
//
// `main.test.ts` is written to the upstream coder/registry conventions so it
// can be published there unchanged, but that repo's shared helper is not
// available here. This provides the same four exports against the local
// module directory, so `bun test` runs standalone and in CI.
//
// Resolved via the `paths` mapping in tsconfig.json.

import { readFile, rm } from "node:fs/promises";
import { spawn } from "node:child_process";
import * as path from "node:path";
import { expect, it } from "bun:test";

/// OpenTofu is a drop-in for the commands used here; prefer whichever exists
/// so contributors aren't forced to install a specific CLI.
const tfBin = (): string =>
  process.env.TF_CLI ?? (Bun.which("terraform") ? "terraform" : "tofu");

interface ExecResult {
  code: number;
  stdout: string;
  stderr: string;
}

const exec = (
  args: string[],
  cwd: string,
  env: NodeJS.ProcessEnv = {},
): Promise<ExecResult> =>
  new Promise((resolve, reject) => {
    const child = spawn(tfBin(), args, {
      cwd,
      env: { ...process.env, ...env },
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (d) => (stdout += d));
    child.stderr.on("data", (d) => (stderr += d));
    child.on("error", reject);
    child.on("close", (code) => resolve({ code: code ?? 0, stdout, stderr }));
  });

export const runTerraformInit = async (dir: string): Promise<void> => {
  const { code, stderr } = await exec(["init", "-input=false", "-no-color"], dir);
  if (code !== 0) throw new Error(`terraform init failed:\n${stderr}`);
};

export interface TerraformStateResource {
  type: string;
  name: string;
  instances: { attributes: Record<string, unknown> }[];
}

export interface TerraformState {
  resources: TerraformStateResource[];
  outputs: Record<string, { value: unknown }>;
}

/// Applies the module with `vars` as TF_VAR_* and returns the resulting state.
/// Throws with terraform's stderr so tests can assert on validation messages.
export const runTerraformApply = async (
  dir: string,
  vars: Record<string, string>,
): Promise<TerraformState> => {
  const env: NodeJS.ProcessEnv = {};
  for (const [k, v] of Object.entries(vars)) env[`TF_VAR_${k}`] = v;

  // Each apply is independent; a stale state file would mask a failed apply.
  const statePath = path.join(dir, "terraform.tfstate");
  await rm(statePath, { force: true });

  const { code, stderr } = await exec(
    ["apply", "-auto-approve", "-input=false", "-no-color"],
    dir,
    env,
  );
  if (code !== 0) throw new Error(stderr);

  return JSON.parse(await readFile(statePath, "utf8")) as TerraformState;
};

/// Returns the single instance's attributes for a resource type (and optional
/// name), mirroring the upstream helper's signature.
export const findResourceInstance = (
  state: TerraformState,
  type: string,
  name?: string,
): Record<string, any> => {
  const resource = state.resources.find(
    (r) => r.type === type && (name === undefined || r.name === name),
  );
  if (!resource) {
    throw new Error(
      `Resource ${type}${name ? `.${name}` : ""} not found in state`,
    );
  }
  if (resource.instances.length !== 1) {
    throw new Error(
      `Expected 1 instance of ${type}, got ${resource.instances.length}`,
    );
  }
  return resource.instances[0].attributes;
};

/// Registers a test per required variable asserting the apply fails when it is
/// omitted, plus one asserting the module applies with all of them supplied.
export const testRequiredVariables = (
  dir: string,
  vars: Record<string, string>,
): void => {
  it("applies with all required variables", async () => {
    await runTerraformApply(dir, vars);
  });

  for (const varName of Object.keys(vars)) {
    it(`fails without required variable: ${varName}`, async () => {
      const withoutVar = { ...vars };
      delete withoutVar[varName];
      await expect(runTerraformApply(dir, withoutVar)).rejects.toThrow(
        `No value for required variable`,
      );
    });
  }
};
