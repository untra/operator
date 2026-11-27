import * as assert from "node:assert/strict";
import * as fs from "node:fs/promises";
import * as path from "node:path";
import * as v8 from "node:v8";
import * as vscode from "vscode";
import { currentConnection } from "../../../src/connection";
import { requiredEnv } from "../daemon";

interface ScriptCoverage {
  url: string;
  functions: { functionName: string; ranges: { count: number }[] }[];
}

export async function run() {
  await vscode.extensions.getExtension("untra.operator-terminals")!.activate();
  assert.equal(currentConnection().selection.kind, "hosted");
  assert.equal(currentConnection().origin, requiredEnv("OPERATOR_E2E_ORIGIN"));
  await vscode.commands.executeCommand("operator.refreshTickets");
  const registration = path.join(
    requiredEnv("OPERATOR_E2E_WORKSPACE"),
    ".tickets",
    "operator",
    "vscode-session.json",
  );
  await assert.rejects(fs.access(registration));
  const registry = path.join(requiredEnv("HOME"), ".operator", "vscode-sessions");
  assert.deepEqual(
    await fs.readdir(registry).catch((error: NodeJS.ErrnoException) => {
      if (error.code !== "ENOENT") {
        throw error;
      }
      return [];
    }),
    [],
  );
  v8.takeCoverage();
  const coverageDir = requiredEnv("NODE_V8_COVERAGE");
  const profiles = await fs.readdir(coverageDir);
  const records: ScriptCoverage[] = [];
  for (const file of profiles.filter((name) => name.startsWith(`coverage-${process.pid}-`))) {
    const coverage = JSON.parse(await fs.readFile(path.join(coverageDir, file), "utf8")) as {
      result: ScriptCoverage[];
    };
    records.push(
      ...coverage.result.filter((script) => script.url.endsWith("/out/src/operator-binary.js")),
    );
  }
  const downloads = records
    .flatMap((record) => record.functions)
    .filter((fn) => fn.functionName === "downloadOperator");
  assert.ok(
    downloads.length,
    "Extension host must collect coverage for the actual binary downloader",
  );
  assert.ok(
    downloads.every((fn) => fn.ranges[0]?.count === 0),
    "Hosted startup attempted to download a local daemon",
  );
}
