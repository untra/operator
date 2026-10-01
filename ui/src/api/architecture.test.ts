import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, test } from "bun:test";

const UI_SRC = join(import.meta.dir, "..");
const REPO = join(UI_SRC, "..", "..");

function walk(dir: string): string[] {
  const out: string[] = [];
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) {
      out.push(...walk(path));
    } else {
      out.push(path);
    }
  }
  return out;
}

function isTs(path: string): boolean {
  return path.endsWith(".ts") || path.endsWith(".tsx");
}

describe("API architecture", () => {
  test("components, routes, and contexts do not construct OperatorApi or call fetch", () => {
    const files = walk(UI_SRC).filter(isTs);
    const violations: string[] = [];
    for (const file of files) {
      const rel = relative(UI_SRC, file);
      if (rel.startsWith("api/") || rel === "api-client.ts" || rel.endsWith(".test.ts")) {
        continue;
      }
      const src = readFileSync(file, "utf8");
      if (/\bnew\s+OperatorApi\b/.test(src)) {
        violations.push(`${rel}: constructs OperatorApi`);
      }
      if (/\bfetch\s*\(/.test(src)) {
        violations.push(`${rel}: calls fetch`);
      }
      if (fromNaiveAsync(src)) {
        violations.push(`${rel}: imports NaiveAsync`);
      }
    }
    expect(violations).toEqual([]);
  });

  test("NaiveAsync stays inside ui/src/api", () => {
    const webcomponents = walk(join(REPO, "webcomponents", "src")).filter(isTs);
    const vscode = walk(join(REPO, "vscode-extension", "src")).filter(isTs);
    const hits: string[] = [];
    for (const file of [...webcomponents, ...vscode]) {
      const src = readFileSync(file, "utf8");
      if (fromNaiveAsync(src) || src.includes("react-redux") || src.includes("naiveAsync")) {
        hits.push(relative(REPO, file));
      }
    }
    expect(hits).toEqual([]);
  });

  test("migrated server responses are not mirrored in component state", () => {
    const files = walk(UI_SRC).filter((file) => file.endsWith(".tsx"));
    const mirrored = /\[(?:probes|projects|statuses|newSecret),\s*set\w+\]\s*=\s*useState/;
    const hits = files
      .filter((file) => mirrored.test(readFileSync(file, "utf8")))
      .map((file) => relative(UI_SRC, file));
    expect(hits).toEqual([]);
  });
});

function fromNaiveAsync(src: string): boolean {
  return /from\s+["']@untra\/naiveasync["']/.test(src);
}
