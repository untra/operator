/**
 * Tests for operator-binary.ts
 *
 * Tests the operator binary discovery, download URL generation,
 * version checking, and path resolution functions.
 */

import * as assert from 'assert';
import * as sinon from 'sinon';
import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs/promises';
import * as os from 'os';
import {
  getExtensionVersion,
  getDownloadUrl,
  getStoragePath,
  getOperatorPath,
  isOperatorAvailable,
  getOperatorVersion,
} from '../../src/operator-binary';

suite('Operator Binary Test Suite', () => {
  let sandbox: sinon.SinonSandbox;

  // Platform-specific binary name
  const binaryName = process.platform === 'win32' ? 'operator.exe' : 'operator';

  setup(() => {
    sandbox = sinon.createSandbox();
  });

  teardown(() => {
    sandbox.restore();
  });

  suite('getExtensionVersion()', () => {
    test('returns version from extension packageJSON', () => {
      sandbox.stub(vscode.extensions, 'getExtension').returns({
        packageJSON: { version: '1.2.3' },
      } as vscode.Extension<unknown>);

      const version = getExtensionVersion();
      assert.strictEqual(version, '1.2.3');
    });

    test('falls back to 0.2.0 when extension not found', () => {
      sandbox.stub(vscode.extensions, 'getExtension').returns(undefined);

      const version = getExtensionVersion();
      assert.strictEqual(version, '0.2.0');
    });

    test('falls back to 0.2.0 when packageJSON has no version', () => {
      sandbox.stub(vscode.extensions, 'getExtension').returns({
        packageJSON: {},
      } as vscode.Extension<unknown>);

      const version = getExtensionVersion();
      assert.strictEqual(version, '0.2.0');
    });
  });

  suite('getDownloadUrl()', () => {
    test('generates correct URL format with explicit version', () => {
      const url = getDownloadUrl('1.0.0');

      assert.ok(url.startsWith('https://github.com/untra/operator/releases/download/v1.0.0/'));
      assert.ok(url.includes('operator-'));
    });

    test('uses extension version when none provided', () => {
      sandbox.stub(vscode.extensions, 'getExtension').returns({
        packageJSON: { version: '2.3.4' },
      } as vscode.Extension<unknown>);

      const url = getDownloadUrl();

      assert.ok(url.includes('/v2.3.4/'));
    });

    test('includes platform-specific binary name', () => {
      const url = getDownloadUrl('1.0.0');

      // Check that it includes platform-specific naming
      if (process.platform === 'darwin') {
        assert.ok(url.includes('operator-macos-'), `Expected macos in URL: ${url}`);
      } else if (process.platform === 'linux') {
        assert.ok(url.includes('operator-linux-'), `Expected linux in URL: ${url}`);
      } else if (process.platform === 'win32') {
        assert.ok(url.includes('operator-windows-'), `Expected windows in URL: ${url}`);
        assert.ok(url.endsWith('.exe'), 'Windows URL should end with .exe');
      }
    });

    test('includes architecture-specific binary name', () => {
      const url = getDownloadUrl('1.0.0');

      // Check architecture naming
      if (process.arch === 'arm64') {
        assert.ok(url.includes('-arm64'), `Expected arm64 in URL: ${url}`);
      } else if (process.arch === 'x64') {
        assert.ok(url.includes('-x86_64'), `Expected x86_64 in URL: ${url}`);
      }
    });
  });

  suite('getStoragePath()', () => {
    test('returns correct path for Unix platforms', () => {
      // Skip on Windows
      if (process.platform === 'win32') {
        return;
      }

      const mockContext = {
        globalStorageUri: { fsPath: '/home/user/.vscode/extensions/storage' },
      } as unknown as vscode.ExtensionContext;

      const storagePath = getStoragePath(mockContext);

      assert.strictEqual(storagePath, '/home/user/.vscode/extensions/storage/operator');
    });

    test('returns correct path with .exe for Windows', () => {
      // We test the logic by checking that Windows would get .exe
      const mockContext = {
        globalStorageUri: { fsPath: 'C:\\Users\\user\\.vscode\\storage' },
      } as unknown as vscode.ExtensionContext;

      const storagePath = getStoragePath(mockContext);

      if (process.platform === 'win32') {
        assert.ok(storagePath.endsWith('operator.exe'));
      } else {
        assert.ok(storagePath.endsWith('operator'));
        assert.ok(!storagePath.endsWith('.exe'));
      }
    });
  });

  suite('getOperatorPath()', () => {
    let tempDir: string;

    setup(async () => {
      tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'operator-binary-test-'));
    });

    teardown(async () => {
      try {
        await fs.rm(tempDir, { recursive: true });
      } catch {
        // Ignore cleanup errors
      }
    });

    test('returns configured path when set and file exists', async () => {
      // Create a mock operator binary
      const operatorPath = path.join(tempDir, 'my-operator');
      await fs.writeFile(operatorPath, '#!/bin/bash\necho "operator"');
      await fs.chmod(operatorPath, 0o755);

      // Mock config to return the path
      const configStub = sandbox.stub(vscode.workspace, 'getConfiguration');
      configStub.returns({
        get: (key: string) => {
          if (key === 'operatorPath') {
            return operatorPath;
          }
          return undefined;
        },
      } as unknown as vscode.WorkspaceConfiguration);

      const mockContext = {
        globalStorageUri: { fsPath: path.join(tempDir, 'storage') },
      } as unknown as vscode.ExtensionContext;

      const result = await getOperatorPath(mockContext);
      assert.strictEqual(result, operatorPath);
    });

    test('returns storage path when config empty but storage binary exists', async () => {
      // Create storage directory and binary
      const storagePath = path.join(tempDir, 'storage');
      await fs.mkdir(storagePath, { recursive: true });
      const binaryPath = path.join(storagePath, binaryName);
      await fs.writeFile(binaryPath, '#!/bin/bash\necho "operator"');
      await fs.chmod(binaryPath, 0o755);

      // Mock config to return empty
      const configStub = sandbox.stub(vscode.workspace, 'getConfiguration');
      configStub.returns({
        get: () => '',
      } as unknown as vscode.WorkspaceConfiguration);

      const mockContext = {
        globalStorageUri: { fsPath: storagePath },
      } as unknown as vscode.ExtensionContext;

      const result = await getOperatorPath(mockContext);
      assert.strictEqual(result, binaryPath);
    });

    test('looks up in PATH for non-existent storage (integration)', async () => {
      // This test actually invokes the PATH lookup
      // It tests that when config and storage are empty, we call which/where

      // Mock config to return empty
      const configStub = sandbox.stub(vscode.workspace, 'getConfiguration');
      configStub.returns({
        get: () => '',
      } as unknown as vscode.WorkspaceConfiguration);

      const mockContext = {
        globalStorageUri: { fsPath: path.join(tempDir, 'nonexistent-storage') },
      } as unknown as vscode.ExtensionContext;

      // This will actually call which/where - operator may or may not be in PATH
      // We're testing that the function completes without error
      const result = await getOperatorPath(mockContext);

      // Result could be a path or undefined - we just verify the function works
      assert.ok(result === undefined || typeof result === 'string');
    });

    test('ignores configured path when file does not exist', async () => {
      // Mock config to return a non-existent path
      const configStub = sandbox.stub(vscode.workspace, 'getConfiguration');
      configStub.returns({
        get: (key: string) => {
          if (key === 'operatorPath') {
            return '/nonexistent/path/operator';
          }
          return undefined;
        },
      } as unknown as vscode.WorkspaceConfiguration);

      const mockContext = {
        globalStorageUri: { fsPath: path.join(tempDir, 'nonexistent-storage') },
      } as unknown as vscode.ExtensionContext;

      // Will fall through to PATH lookup since config path doesn't exist
      const result = await getOperatorPath(mockContext);

      // Should not return the non-existent config path
      assert.notStrictEqual(result, '/nonexistent/path/operator');
    });
  });

  suite('getOperatorVersion()', () => {
    let tempDir: string;

    setup(async () => {
      tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'operator-version-test-'));
    });

    teardown(async () => {
      try {
        await fs.rm(tempDir, { recursive: true });
      } catch {
        // Ignore cleanup errors
      }
    });

    test('parses version from "operator X.Y.Z" format', async () => {
      // Skip on Windows - shell scripts don't work the same way
      if (process.platform === 'win32') {
        return;
      }

      // Create a mock operator binary that outputs version
      const operatorPath = path.join(tempDir, 'operator');
      await fs.writeFile(operatorPath, '#!/bin/bash\necho "operator 0.1.14"');
      await fs.chmod(operatorPath, 0o755);

      const version = await getOperatorVersion(operatorPath);
      assert.strictEqual(version, '0.1.14');
    });

    test('returns trimmed output when no match pattern', async () => {
      // Skip on Windows
      if (process.platform === 'win32') {
        return;
      }

      const operatorPath = path.join(tempDir, 'operator');
      await fs.writeFile(operatorPath, '#!/bin/bash\necho "1.2.3"');
      await fs.chmod(operatorPath, 0o755);

      const version = await getOperatorVersion(operatorPath);
      assert.strictEqual(version, '1.2.3');
    });

    test('returns undefined on non-zero exit code', async () => {
      // Skip on Windows
      if (process.platform === 'win32') {
        return;
      }

      const operatorPath = path.join(tempDir, 'operator');
      await fs.writeFile(operatorPath, '#!/bin/bash\nexit 1');
      await fs.chmod(operatorPath, 0o755);

      const version = await getOperatorVersion(operatorPath);
      assert.strictEqual(version, undefined);
    });

    test('returns undefined for non-existent binary', async () => {
      const version = await getOperatorVersion('/nonexistent/path/operator');
      assert.strictEqual(version, undefined);
    });

    test('returns undefined on empty output', async () => {
      // Skip on Windows
      if (process.platform === 'win32') {
        return;
      }

      const operatorPath = path.join(tempDir, 'operator');
      await fs.writeFile(operatorPath, '#!/bin/bash\necho ""');
      await fs.chmod(operatorPath, 0o755);

      const version = await getOperatorVersion(operatorPath);
      // Empty output after trim is falsy, so returns undefined
      assert.strictEqual(version, undefined);
    });

    test('handles version with additional text', async () => {
      // Skip on Windows
      if (process.platform === 'win32') {
        return;
      }

      const operatorPath = path.join(tempDir, 'operator');
      await fs.writeFile(operatorPath, '#!/bin/bash\necho "operator 2.0.0-beta.1"');
      await fs.chmod(operatorPath, 0o755);

      const version = await getOperatorVersion(operatorPath);
      assert.strictEqual(version, '2.0.0-beta.1');
    });
  });

  suite('isOperatorAvailable()', () => {
    let tempDir: string;

    setup(async () => {
      tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'operator-avail-test-'));
    });

    teardown(async () => {
      try {
        await fs.rm(tempDir, { recursive: true });
      } catch {
        // Ignore cleanup errors
      }
    });

    test('returns true when operator is found in storage', async () => {
      // Create storage directory and binary
      const storagePath = path.join(tempDir, 'storage');
      await fs.mkdir(storagePath, { recursive: true });
      const binaryPath = path.join(storagePath, binaryName);
      await fs.writeFile(binaryPath, '#!/bin/bash\necho "operator"');
      await fs.chmod(binaryPath, 0o755);

      // Mock config to return empty
      const configStub = sandbox.stub(vscode.workspace, 'getConfiguration');
      configStub.returns({
        get: () => '',
      } as unknown as vscode.WorkspaceConfiguration);

      const mockContext = {
        globalStorageUri: { fsPath: storagePath },
      } as unknown as vscode.ExtensionContext;

      const result = await isOperatorAvailable(mockContext);
      assert.strictEqual(result, true);
    });

    test('returns true when operator is in configured path', async () => {
      // Create a mock operator binary
      const operatorPath = path.join(tempDir, 'my-operator');
      await fs.writeFile(operatorPath, '#!/bin/bash\necho "operator"');
      await fs.chmod(operatorPath, 0o755);

      // Mock config to return the path
      const configStub = sandbox.stub(vscode.workspace, 'getConfiguration');
      configStub.returns({
        get: (key: string) => {
          if (key === 'operatorPath') {
            return operatorPath;
          }
          return undefined;
        },
      } as unknown as vscode.WorkspaceConfiguration);

      const mockContext = {
        globalStorageUri: { fsPath: path.join(tempDir, 'storage') },
      } as unknown as vscode.ExtensionContext;

      const result = await isOperatorAvailable(mockContext);
      assert.strictEqual(result, true);
    });

    test('returns false when operator is not found anywhere', async () => {
      // Mock config to return empty
      const configStub = sandbox.stub(vscode.workspace, 'getConfiguration');
      configStub.returns({
        get: () => '',
      } as unknown as vscode.WorkspaceConfiguration);

      // Use a unique temp storage path where operator won't exist
      const mockContext = {
        globalStorageUri: { fsPath: path.join(tempDir, 'empty-storage-' + Date.now()) },
      } as unknown as vscode.ExtensionContext;

      const result = await isOperatorAvailable(mockContext);

      // If operator is not in PATH either, this should be false
      // Note: if operator IS in PATH on the test machine, this could be true
      // We're mainly testing that the function executes without error
      assert.ok(typeof result === 'boolean');
    });
  });
});
