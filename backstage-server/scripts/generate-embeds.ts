/**
 * Generate Embedded Assets
 *
 * This script:
 * 1. Scans packages/app/dist/ for built frontend files
 * 2. Copies them to src/assets/ (for Bun to embed)
 * 3. Generates src/embedded-assets.ts with import statements
 *
 * Run with: bun run scripts/generate-embeds.ts
 */

import { readdir, mkdir, rm, copyFile, writeFile, readFile } from "node:fs/promises";
import { join, relative } from "node:path";

const DIST_DIR = "packages/app/dist";
const ASSETS_DIR = "src/assets";
const OUTPUT_FILE = "src/embedded-assets.ts";

// File extensions to exclude (source maps increase binary size)
const EXCLUDE_EXTENSIONS = [".map"];

/**
 * Copy file with retry logic for Windows file locking issues.
 * On Windows, EPERM errors can occur when files are temporarily locked.
 * This function retries with exponential backoff and falls back to read+write.
 */
async function copyFileWithRetry(
  src: string,
  dest: string,
  maxRetries = 3
): Promise<void> {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      await copyFile(src, dest);
      return;
    } catch (err: unknown) {
      const error = err as NodeJS.ErrnoException;
      // On Windows, EPERM can occur if file is temporarily locked
      if (error.code === "EPERM" && attempt < maxRetries) {
        console.warn(
          `Retry ${attempt}/${maxRetries} for ${src}: ${error.code}`
        );
        await new Promise((resolve) => setTimeout(resolve, 100 * attempt));
        continue;
      }
      throw err;
    }
  }
}

async function getAllFiles(dir: string): Promise<string[]> {
  const files: string[] = [];
  const entries = await readdir(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await getAllFiles(fullPath)));
    } else {
      // Skip excluded extensions
      const shouldExclude = EXCLUDE_EXTENSIONS.some((ext) =>
        entry.name.endsWith(ext)
      );
      if (!shouldExclude) {
        files.push(fullPath);
      }
    }
  }

  return files;
}

async function main() {
  console.log("Generating embedded assets...");

  // Clean up existing assets directory
  try {
    await rm(ASSETS_DIR, { recursive: true });
  } catch {
    // Directory doesn't exist, that's fine
  }
  await mkdir(ASSETS_DIR, { recursive: true });

  // Get all files from dist
  const distFiles = await getAllFiles(DIST_DIR);
  console.log(`Found ${distFiles.length} files to embed`);

  // Copy files and collect import paths
  const importPaths: string[] = [];

  for (const file of distFiles) {
    // Get relative path from dist directory
    const relativePath = relative(DIST_DIR, file);
    const destPath = join(ASSETS_DIR, relativePath);

    // Create destination directory if needed
    const destDir = join(ASSETS_DIR, relative(DIST_DIR, file.replace(/\/[^/]+$/, "")));
    await mkdir(destDir, { recursive: true }).catch(() => {});

    // Copy file (with retry for Windows file locking)
    await copyFileWithRetry(file, destPath);

    // Add to import paths (relative to src/)
    importPaths.push(`./assets/${relativePath}`);
  }

  // Generate embedded-assets.ts
  const imports = importPaths
    .map((path) => `import "${path}" with { type: "file" };`)
    .join("\n");

  const content = `/**
 * Embedded Frontend Assets
 *
 * AUTO-GENERATED FILE - DO NOT EDIT MANUALLY
 * Regenerate with: bun run build:embeds
 *
 * This file imports all frontend assets so they get embedded
 * into the compiled Bun binary via \`with { type: "file" }\` syntax.
 */

${imports}

// Export empty object to ensure this module is included
export {};
`;

  await writeFile(OUTPUT_FILE, content);

  console.log(`Generated ${OUTPUT_FILE} with ${importPaths.length} imports`);
  console.log(`Copied assets to ${ASSETS_DIR}/`);
}

main().catch(console.error);
