#!/usr/bin/env node

/**
 * Copy generated TypeScript types from ../bindings/ to src/generated/
 *
 * This script is run as part of the prebuild step to ensure the extension
 * uses types generated from Rust source via ts-rs.
 */

const fs = require('fs');
const path = require('path');

// Types to copy from bindings/
const TYPES_TO_COPY = [
  // VSCode webhook API types
  'VsCodeSessionInfo',
  'VsCodeHealthResponse',
  'VsCodeActivityState',
  'VsCodeTerminalState',
  'VsCodeTerminalCreateOptions',
  'VsCodeSendCommandRequest',
  'VsCodeSuccessResponse',
  'VsCodeExistsResponse',
  'VsCodeActivityResponse',
  'VsCodeListResponse',
  'VsCodeErrorResponse',
  // Domain types (VsCodeTicketType removed - now dynamic string)
  'VsCodeTicketStatus',
  'VsCodeTicketInfo',
  'VsCodeModelOption',
  'VsCodeLaunchOptions',
  'VsCodeTicketMetadata',
  // Issue type metadata (for dynamic type styling)
  'IssueTypeSummary',
  // REST API types (for API client)
  'HealthResponse',
  'LaunchTicketRequest',
  'LaunchTicketResponse',
  'LlmProvider',
];

const BINDINGS_DIR = path.resolve(__dirname, '../../bindings');
const GENERATED_DIR = path.resolve(__dirname, '../src/generated');

// Ensure generated directory exists
if (!fs.existsSync(GENERATED_DIR)) {
  fs.mkdirSync(GENERATED_DIR, { recursive: true });
}

console.log('Copying generated TypeScript types...');

const copiedTypes = [];
const missingTypes = [];

for (const typeName of TYPES_TO_COPY) {
  const srcFile = path.join(BINDINGS_DIR, `${typeName}.ts`);
  const destFile = path.join(GENERATED_DIR, `${typeName}.ts`);

  if (fs.existsSync(srcFile)) {
    fs.copyFileSync(srcFile, destFile);
    copiedTypes.push(typeName);
  } else {
    missingTypes.push(typeName);
  }
}

// Generate barrel export (index.ts)
const indexContent = `// AUTO-GENERATED - DO NOT EDIT
// Copied from ../bindings/ by scripts/copy-types.js
// Regenerate with: npm run prebuild

${copiedTypes.map(t => `export * from './${t}';`).join('\n')}
`;

fs.writeFileSync(path.join(GENERATED_DIR, 'index.ts'), indexContent);

console.log(`  Copied ${copiedTypes.length} type files`);
if (missingTypes.length > 0) {
  console.warn(`  Warning: ${missingTypes.length} types not found in bindings:`);
  missingTypes.forEach(t => console.warn(`    - ${t}`));
}
console.log('  Generated: src/generated/index.ts');
console.log('Done!');
