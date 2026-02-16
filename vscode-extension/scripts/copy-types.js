#!/usr/bin/env node

/**
 * Copy generated TypeScript types from ../bindings/ to src/generated/
 *
 * This script is run as part of the prebuild step to ensure the extension
 * uses types generated from Rust source via ts-rs.
 *
 * Copies ALL .ts files (including subdirectories like serde_json/)
 * and generates a barrel export (index.ts).
 */

const fs = require('fs');
const path = require('path');

const BINDINGS_DIR = path.resolve(__dirname, '../../bindings');
const GENERATED_DIR = path.resolve(__dirname, '../src/generated');

/**
 * Recursively copy all .ts files from src to dest, preserving directory structure.
 * Returns an array of relative paths (from dest root) that were copied.
 */
function copyTsFiles(srcDir, destDir, relativeBase = '') {
  const copied = [];

  if (!fs.existsSync(destDir)) {
    fs.mkdirSync(destDir, { recursive: true });
  }

  const entries = fs.readdirSync(srcDir, { withFileTypes: true });
  for (const entry of entries) {
    const srcPath = path.join(srcDir, entry.name);
    const destPath = path.join(destDir, entry.name);
    const relativePath = relativeBase ? `${relativeBase}/${entry.name}` : entry.name;

    if (entry.isDirectory()) {
      const subCopied = copyTsFiles(srcPath, destPath, relativePath);
      copied.push(...subCopied);
    } else if (entry.isFile() && entry.name.endsWith('.ts')) {
      fs.copyFileSync(srcPath, destPath);
      copied.push(relativePath);
    }
  }

  return copied;
}

// Clean generated directory
if (fs.existsSync(GENERATED_DIR)) {
  fs.rmSync(GENERATED_DIR, { recursive: true });
}
fs.mkdirSync(GENERATED_DIR, { recursive: true });

console.log('Copying generated TypeScript types...');

const copiedFiles = copyTsFiles(BINDINGS_DIR, GENERATED_DIR);

// Generate barrel export (index.ts) for top-level .ts files only
const topLevelTypes = copiedFiles
  .filter(f => !f.includes('/')) // exclude subdirectory files
  .map(f => f.replace('.ts', ''));

const indexContent = `// AUTO-GENERATED - DO NOT EDIT
// Copied from ../bindings/ by scripts/copy-types.js
// Regenerate with: npm run copy-types

${topLevelTypes.map(t => `export * from './${t}';`).join('\n')}

// Subdirectory re-exports
export * from './serde_json/JsonValue';
`;

fs.writeFileSync(path.join(GENERATED_DIR, 'index.ts'), indexContent);

console.log(`  Copied ${copiedFiles.length} type files`);
console.log('  Generated: src/generated/index.ts');
console.log('Done!');
