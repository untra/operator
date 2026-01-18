/**
 * Operator binary management for VS Code extension
 *
 * Handles discovery, download, and version checking of the Operator binary.
 * Similar to opr8r.ts but for the main Operator application.
 */

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';
import { createWriteStream } from 'fs';
import { spawn } from 'child_process';
import * as https from 'https';

const GITHUB_REPO = 'untra/operator';

/**
 * Get the extension version from package.json
 */
export function getExtensionVersion(): string {
  const extension = vscode.extensions.getExtension('untra.operator-terminals');
  return extension?.packageJSON.version || '0.2.0';
}

/**
 * Platform-specific binary name mapping
 * Maps Node.js platform/arch to GitHub release artifact names
 *
 * Supported platforms:
 * - darwin + arm64 -> operator-macos-arm64
 * - darwin + x64 -> operator-macos-x86_64
 * - linux + arm64 -> operator-linux-arm64
 * - linux + x64 -> operator-linux-x86_64
 * - win32 + x64 -> operator-windows-x86_64.exe
 * - win32 + arm64 -> operator-windows-arm64.exe
 */
function getArtifactName(): string {
  const platform = process.platform; // 'darwin', 'linux', 'win32'
  const arch = process.arch; // 'arm64', 'x64'

  const platformMap: Record<string, string> = {
    darwin: 'macos',
    linux: 'linux',
    win32: 'windows',
  };

  const archMap: Record<string, string> = {
    arm64: 'arm64',
    x64: 'x86_64',
  };

  const platformName = platformMap[platform] ?? 'linux';
  const archName = archMap[arch] ?? 'x86_64';
  const ext = platform === 'win32' ? '.exe' : '';

  return `operator-${platformName}-${archName}${ext}`;
}

/**
 * Get download URL for current platform
 */
export function getDownloadUrl(version?: string): string {
  const ver = version ?? getExtensionVersion();
  const artifact = getArtifactName();
  return `https://github.com/${GITHUB_REPO}/releases/download/v${ver}/${artifact}`;
}

/**
 * Get storage path for downloaded binary
 */
export function getStoragePath(context: vscode.ExtensionContext): string {
  const binaryName = process.platform === 'win32' ? 'operator.exe' : 'operator';
  return path.join(context.globalStorageUri.fsPath, binaryName);
}

/**
 * Discovers the Operator binary path using priority order:
 * 1. User configuration setting (operator.operatorPath)
 * 2. Downloaded binary in globalStorage
 * 3. System PATH lookup
 *
 * @param context - The VS Code extension context
 * @returns The path to the operator binary, or undefined if not found
 */
export async function getOperatorPath(
  context: vscode.ExtensionContext
): Promise<string | undefined> {
  // 1. Check user configuration
  const config = vscode.workspace.getConfiguration('operator');
  const configPath = config.get<string>('operatorPath');
  if (configPath && (await fileExists(configPath))) {
    return configPath;
  }

  // 2. Check downloaded binary in globalStorage
  const storagePath = getStoragePath(context);
  if (await fileExists(storagePath)) {
    return storagePath;
  }

  // 3. Check system PATH
  const pathBinaryName =
    process.platform === 'win32' ? 'operator.exe' : 'operator';
  const pathResult = await findInPath(pathBinaryName);
  if (pathResult) {
    return pathResult;
  }

  return undefined;
}

/**
 * Check if operator binary is available
 */
export async function isOperatorAvailable(
  context: vscode.ExtensionContext
): Promise<boolean> {
  const operatorPath = await getOperatorPath(context);
  return operatorPath !== undefined;
}

/**
 * Gets the operator version by running `operator --version`
 */
export async function getOperatorVersion(
  operatorPath: string
): Promise<string | undefined> {
  return new Promise((resolve) => {
    const proc = spawn(operatorPath, ['--version']);
    let stdout = '';

    proc.stdout.on('data', (data) => {
      stdout += data;
    });

    proc.on('close', (code) => {
      if (code === 0 && stdout.trim()) {
        // Parse version from "operator 0.1.14" format
        const match = stdout.trim().match(/operator\s+(\S+)/);
        resolve(match ? match[1] : stdout.trim());
      } else {
        resolve(undefined);
      }
    });

    proc.on('error', () => {
      resolve(undefined);
    });
  });
}

/**
 * Download operator binary with progress
 */
export async function downloadOperator(
  context: vscode.ExtensionContext,
  version?: string
): Promise<string> {
  const url = getDownloadUrl(version);
  const destPath = getStoragePath(context);

  // Ensure storage directory exists
  await fs.mkdir(context.globalStorageUri.fsPath, { recursive: true });

  return vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: 'Downloading Operator',
      cancellable: true,
    },
    async (progress, token) => {
      return downloadWithRedirects(url, destPath, progress, token);
    }
  );
}

/**
 * Download file following redirects (GitHub releases use 302 redirects)
 */
async function downloadWithRedirects(
  url: string,
  destPath: string,
  progress: vscode.Progress<{ increment?: number; message?: string }>,
  token: vscode.CancellationToken,
  redirectCount = 0
): Promise<string> {
  if (redirectCount > 5) {
    throw new Error('Too many redirects');
  }

  return new Promise((resolve, reject) => {
    // Parse URL to determine http vs https
    const urlObj = new URL(url);
    const httpModule = urlObj.protocol === 'https:' ? https : https;

    const request = httpModule.get(url, (response) => {
      // Handle redirects (GitHub releases redirect to CDN)
      if (response.statusCode === 302 || response.statusCode === 301) {
        const redirectUrl = response.headers.location;
        if (redirectUrl) {
          downloadWithRedirects(
            redirectUrl,
            destPath,
            progress,
            token,
            redirectCount + 1
          )
            .then(resolve)
            .catch(reject);
          return;
        }
        reject(new Error('Redirect without location header'));
        return;
      }

      if (response.statusCode !== 200) {
        reject(
          new Error(
            `Download failed: HTTP ${response.statusCode} ${response.statusMessage}`
          )
        );
        return;
      }

      const totalSize = parseInt(response.headers['content-length'] ?? '0', 10);
      let downloadedSize = 0;

      // Create write stream
      const writeStream = createWriteStream(destPath);

      response.on('data', (chunk: Buffer) => {
        writeStream.write(chunk);
        downloadedSize += chunk.length;
        if (totalSize > 0) {
          const percent = Math.round((downloadedSize / totalSize) * 100);
          progress.report({
            increment: (chunk.length / totalSize) * 100,
            message: `${percent}%`,
          });
        }
      });

      response.on('end', async () => {
        writeStream.end();
        // Make executable on Unix
        if (process.platform !== 'win32') {
          try {
            await fs.chmod(destPath, 0o755);
          } catch {
            // Ignore chmod errors
          }
        }
        resolve(destPath);
      });

      response.on('error', (err) => {
        writeStream.end();
        reject(err);
      });

      writeStream.on('error', (err: Error) => {
        reject(err);
      });
    });

    request.on('error', (err) => {
      reject(err);
    });

    token.onCancellationRequested(() => {
      request.destroy();
      reject(new Error('Download cancelled'));
    });
  });
}

/**
 * Checks if a file exists and is executable
 * On Windows, we check for read access since Windows doesn't use Unix-style execute permissions
 */
async function fileExists(filePath: string): Promise<boolean> {
  try {
    // Windows doesn't have X_OK permission model, use R_OK instead
    const accessMode =
      process.platform === 'win32' ? fs.constants.R_OK : fs.constants.X_OK;
    await fs.access(filePath, accessMode);
    return true;
  } catch {
    return false;
  }
}

/**
 * Finds a binary in the system PATH
 */
async function findInPath(binary: string): Promise<string | undefined> {
  return new Promise((resolve) => {
    const cmd = process.platform === 'win32' ? 'where' : 'which';
    const proc = spawn(cmd, [binary]);
    let stdout = '';

    proc.stdout.on('data', (data) => {
      stdout += data;
    });

    proc.on('close', (code) => {
      if (code === 0 && stdout.trim()) {
        // Return the first result (in case of multiple matches)
        resolve(stdout.trim().split('\n')[0]);
      } else {
        resolve(undefined);
      }
    });

    proc.on('error', () => {
      resolve(undefined);
    });
  });
}
