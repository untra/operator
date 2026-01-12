import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';
import { spawn } from 'child_process';

/**
 * Discovers the opr8r binary path using priority order:
 * 1. User configuration setting (operator.opr8rPath)
 * 2. Bundled binary in extension (bin/opr8r)
 * 3. System PATH lookup
 *
 * @param context - The VS Code extension context
 * @returns The path to the opr8r binary, or undefined if not found
 */
export async function getOpr8rPath(context: vscode.ExtensionContext): Promise<string | undefined> {
    // 1. Check user configuration
    const config = vscode.workspace.getConfiguration('operator');
    const configPath = config.get<string>('opr8rPath');
    if (configPath && await fileExists(configPath)) {
        return configPath;
    }

    // 2. Check bundled binary in extension
    const binaryName = process.platform === 'win32' ? 'opr8r.exe' : 'opr8r';
    const bundledPath = path.join(context.extensionPath, 'bin', binaryName);
    if (await fileExists(bundledPath)) {
        return bundledPath;
    }

    // 3. Check system PATH
    const pathBinaryName = process.platform === 'win32' ? 'opr8r.exe' : 'opr8r';
    const pathResult = await findInPath(pathBinaryName);
    if (pathResult) {
        return pathResult;
    }

    return undefined;
}

/**
 * Checks if a file exists and is executable
 * On Windows, we check for read access since Windows doesn't use Unix-style execute permissions
 */
async function fileExists(filePath: string): Promise<boolean> {
    try {
        // Windows doesn't have X_OK permission model, use R_OK instead
        const accessMode = process.platform === 'win32' ? fs.constants.R_OK : fs.constants.X_OK;
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

/**
 * Gets the opr8r version by running `opr8r --version`
 */
export async function getOpr8rVersion(opr8rPath: string): Promise<string | undefined> {
    return new Promise((resolve) => {
        const proc = spawn(opr8rPath, ['--version']);
        let stdout = '';

        proc.stdout.on('data', (data) => {
            stdout += data;
        });

        proc.on('close', (code) => {
            if (code === 0 && stdout.trim()) {
                // Parse version from "opr8r 0.1.14" format
                const match = stdout.trim().match(/opr8r\s+(\S+)/);
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
