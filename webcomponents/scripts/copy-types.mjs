#!/usr/bin/env node

/**
 * Copy the ts-rs generated types from ../bindings/ into src/generated/.
 *
 * The components are typed against the Rust domain types, not against a
 * hand-written mirror. `bindings/` is produced by `cargo test` (ts-rs emits an
 * `export_bindings_*` test per exported type), so the ordering is:
 *
 *     cargo test  ->  bindings/  ->  this script  ->  tsc / vite
 *
 * Every compile script in package.json runs this first, so there is no way to
 * typecheck, test, or build against missing or stale types.
 *
 * The copy is necessary rather than a path alias: tsconfig.build.json sets
 * `rootDir: "src"` for declaration emit, and a file imported from outside
 * rootDir breaks that emit (TS6059). Mirrors
 * vscode-extension/scripts/copy-types.js, which solves the same problem.
 *
 * src/generated/ is gitignored — bindings/ is the committed artifact.
 */

import { copyFileSync, existsSync, mkdirSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const BINDINGS_DIR = resolve(HERE, '../../bindings');
const GENERATED_DIR = resolve(HERE, '../src/generated');

/** Recursively copy every `.ts` file, preserving subdirectories (serde_json/). */
function copyTsFiles(srcDir, destDir, relativeBase = '') {
  const copied = [];
  mkdirSync(destDir, { recursive: true });

  for (const entry of readdirSync(srcDir, { withFileTypes: true })) {
    const srcPath = join(srcDir, entry.name);
    const destPath = join(destDir, entry.name);
    const relativePath = relativeBase ? `${relativeBase}/${entry.name}` : entry.name;

    if (entry.isDirectory()) {
      copied.push(...copyTsFiles(srcPath, destPath, relativePath));
    } else if (entry.isFile() && entry.name.endsWith('.ts')) {
      copyFileSync(srcPath, destPath);
      copied.push(relativePath);
    }
  }
  return copied;
}

if (!existsSync(BINDINGS_DIR)) {
  console.error(
    `No bindings at ${BINDINGS_DIR}.\n` +
      'Generate them first: cargo test --locked export_bindings_  (or `make bindings`).'
  );
  process.exit(1);
}

rmSync(GENERATED_DIR, { recursive: true, force: true });
const copied = copyTsFiles(BINDINGS_DIR, GENERATED_DIR);

const topLevel = copied.filter((f) => !f.includes('/')).map((f) => f.replace(/\.ts$/, ''));
writeFileSync(
  join(GENERATED_DIR, 'index.ts'),
  `// AUTO-GENERATED - DO NOT EDIT\n` +
    `// Copied from ../../bindings/ by scripts/copy-types.mjs\n` +
    `// Regenerate with: cargo test --locked export_bindings_ && npm run copy-types\n\n` +
    `${topLevel.map((t) => `export * from './${t}';`).join('\n')}\n\n` +
    `// Subdirectory re-exports\n` +
    `export * from './serde_json/JsonValue';\n`
);

console.log(`copy-types: ${copied.length} generated types -> src/generated/`);
