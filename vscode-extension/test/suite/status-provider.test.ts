/**
 * Tests for status-provider.ts
 *
 * Tests webhook status icon rendering and working directory item behavior.
 * Uses real temp directories for file-system-dependent checks and sinon
 * stubs for external dependencies (network, binary discovery, etc.).
 */

import * as assert from 'assert';
import * as sinon from 'sinon';
import * as vscode from 'vscode';
import * as fs from 'fs/promises';
import * as path from 'path';
import * as os from 'os';
import { StatusTreeProvider, StatusItem } from '../../src/status-provider';
import * as configPaths from '../../src/config-paths';
import * as walkthrough from '../../src/walkthrough';
import * as operatorBinary from '../../src/operator-binary';
import * as apiClient from '../../src/api-client';

/**
 * Create a mock ExtensionContext with stubbed globalState
 */
function createMockContext(
  sandbox: sinon.SinonSandbox,
  workingDir?: string
): vscode.ExtensionContext {
  const getStub = sandbox.stub();
  getStub.withArgs('operator.workingDirectory').returns(workingDir ?? '');

  return {
    globalState: {
      get: getStub,
      update: sandbox.stub().resolves(),
      keys: sandbox.stub().returns([]),
      setKeysForSync: sandbox.stub(),
    },
    subscriptions: [],
    extensionPath: '/fake/extension',
    extensionUri: vscode.Uri.file('/fake/extension'),
    globalStorageUri: vscode.Uri.file('/fake/storage'),
    storageUri: vscode.Uri.file('/fake/workspace-storage'),
    logUri: vscode.Uri.file('/fake/log'),
    extensionMode: vscode.ExtensionMode.Test,
    extension: {} as vscode.Extension<unknown>,
    environmentVariableCollection: {} as vscode.GlobalEnvironmentVariableCollection,
    secrets: {} as vscode.SecretStorage,
    storagePath: '/fake/workspace-storage',
    globalStoragePath: '/fake/storage',
    logPath: '/fake/log',
    asAbsolutePath: (p: string) => p,
    languageModelAccessInformation: {} as vscode.LanguageModelAccessInformation,
  } as unknown as vscode.ExtensionContext;
}

/** Helper to find a child item by label */
function findChild(items: StatusItem[], label: string): StatusItem | undefined {
  return items.find((item) => item.label === label);
}

/** Helper to extract section labels from top-level items */
function getSectionLabels(items: StatusItem[]): string[] {
  return items.map((item) => item.label as string);
}

suite('Status Provider Test Suite', () => {
  let sandbox: sinon.SinonSandbox;
  let tempDir: string;

  setup(async () => {
    sandbox = sinon.createSandbox();

    // Create temp directory for session files
    tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'status-provider-test-'));

    // Stub external dependencies that make network calls or spawn processes
    sandbox.stub(walkthrough, 'detectInstalledLlmTools').resolves([]);
    sandbox.stub(walkthrough, 'getKanbanWorkspaces').resolves([]);
    sandbox.stub(operatorBinary, 'getOperatorPath').resolves(undefined);
    sandbox.stub(operatorBinary, 'getOperatorVersion').resolves(undefined);
    sandbox.stub(apiClient, 'discoverApiUrl').resolves('http://localhost:7008');
    sandbox.stub(global, 'fetch').rejects(new Error('no network in tests'));
  });

  teardown(async () => {
    sandbox.restore();
    try {
      await fs.rm(tempDir, { recursive: true });
    } catch {
      // Ignore cleanup errors
    }
  });

  suite('webhook status rendering', () => {
    test('shows pass icon and Running description when webhook is running', async () => {
      const mockContext = createMockContext(sandbox, '/fake/working-dir');
      sandbox.stub(configPaths, 'configFileExists').resolves(true);
      sandbox.stub(configPaths, 'getResolvedConfigPath').returns('');
      sandbox.stub(configPaths, 'resolveWorkingDirectory').returns('/fake/working-dir');

      // Write a real session file in the temp directory
      const operatorDir = path.join(tempDir, 'operator');
      await fs.mkdir(operatorDir, { recursive: true });
      await fs.writeFile(
        path.join(operatorDir, 'vscode-session.json'),
        JSON.stringify({
          wrapper: 'vscode',
          port: 7009,
          pid: 12345,
          version: '0.1.26',
          startedAt: '2024-01-01T00:00:00Z',
          workspace: '/fake/workspace',
        })
      );

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const sections = provider.getChildren();
      const connections = findChild(sections, 'Connections');
      assert.ok(connections, 'Should have Connections section');

      const children = provider.getChildren(connections);
      const webhook = findChild(children, 'Webhook');
      assert.ok(webhook, 'Should have Webhook item');

      const icon = webhook.iconPath as vscode.ThemeIcon;
      assert.strictEqual(icon.id, 'pass', 'Webhook icon should be pass when running');
      assert.ok(
        (webhook.description as string).includes('Running'),
        `Description "${webhook.description}" should contain "Running"`
      );
      assert.ok(
        (webhook.description as string).includes(':7009'),
        `Description "${webhook.description}" should contain port ":7009"`
      );
    });

    test('shows circle-slash icon and Stopped when webhook is not running', async () => {
      const mockContext = createMockContext(sandbox, '/fake/working-dir');
      sandbox.stub(configPaths, 'configFileExists').resolves(true);
      sandbox.stub(configPaths, 'getResolvedConfigPath').returns('');
      sandbox.stub(configPaths, 'resolveWorkingDirectory').returns('/fake/working-dir');

      // No session file — webhook not running
      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const sections = provider.getChildren();
      const connections = findChild(sections, 'Connections');
      assert.ok(connections, 'Should have Connections section');

      const children = provider.getChildren(connections);
      const webhook = findChild(children, 'Webhook');
      assert.ok(webhook, 'Should have Webhook item');

      const icon = webhook.iconPath as vscode.ThemeIcon;
      assert.strictEqual(icon.id, 'circle-slash', 'Webhook icon should be circle-slash when stopped');
      assert.strictEqual(webhook.description, 'Stopped', 'Description should be "Stopped"');
    });
  });

  suite('working directory item', () => {
    test('has contextValue and no command when working directory is set', async () => {
      const mockContext = createMockContext(sandbox, '/fake/working-dir');
      sandbox.stub(configPaths, 'configFileExists').resolves(true);
      sandbox.stub(configPaths, 'getResolvedConfigPath').returns('');
      sandbox.stub(configPaths, 'resolveWorkingDirectory').returns('/fake/working-dir');

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const sections = provider.getChildren();
      const config = findChild(sections, 'Configuration');
      assert.ok(config, 'Should have Configuration section');

      const children = provider.getChildren(config);
      const workDir = findChild(children, 'Working Directory');
      assert.ok(workDir, 'Should have Working Directory item');

      assert.strictEqual(
        workDir.contextValue,
        'workingDirConfigured',
        'Should have contextValue "workingDirConfigured"'
      );
      assert.strictEqual(
        workDir.command,
        undefined,
        'Should not have a click command when directory is set'
      );
      assert.strictEqual(workDir.description, '/fake/working-dir');
    });

    test('has click command and no contextValue when working directory is not set', async () => {
      const mockContext = createMockContext(sandbox);
      sandbox.stub(configPaths, 'configFileExists').resolves(false);
      sandbox.stub(configPaths, 'getResolvedConfigPath').returns('');
      sandbox.stub(configPaths, 'resolveWorkingDirectory').returns('');

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const sections = provider.getChildren();
      const config = findChild(sections, 'Configuration');
      assert.ok(config, 'Should have Configuration section');

      const children = provider.getChildren(config);
      const workDir = findChild(children, 'Working Directory');
      assert.ok(workDir, 'Should have Working Directory item');

      assert.ok(workDir.command, 'Should have a click command when directory is not set');
      assert.strictEqual(
        workDir.command?.command,
        'operator.selectWorkingDirectory',
        'Command should be selectWorkingDirectory'
      );
      assert.strictEqual(workDir.contextValue, undefined, 'Should not have contextValue');
      assert.strictEqual(workDir.description, 'Not set');
    });
  });

  suite('session wrapper item', () => {
    test('shows pass icon with VS Code Terminal when wrapper defaults to vscode and webhook running', async () => {
      const mockContext = createMockContext(sandbox, '/fake/working-dir');
      sandbox.stub(configPaths, 'configFileExists').resolves(true);
      sandbox.stub(configPaths, 'getResolvedConfigPath').returns('');
      sandbox.stub(configPaths, 'resolveWorkingDirectory').returns('/fake/working-dir');

      // Write webhook session file so webhook shows as running
      const operatorDir = path.join(tempDir, 'operator');
      await fs.mkdir(operatorDir, { recursive: true });
      await fs.writeFile(
        path.join(operatorDir, 'vscode-session.json'),
        JSON.stringify({
          wrapper: 'vscode',
          port: 7009,
          pid: 12345,
          version: '0.1.26',
          startedAt: '2024-01-01T00:00:00Z',
          workspace: '/fake/workspace',
        })
      );

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const sections = provider.getChildren();
      const connections = findChild(sections, 'Connections');
      assert.ok(connections, 'Should have Connections section');

      const children = provider.getChildren(connections);
      const wrapper = findChild(children, 'Session Wrapper');
      assert.ok(wrapper, 'Should have Session Wrapper item');

      const icon = wrapper.iconPath as vscode.ThemeIcon;
      assert.strictEqual(icon.id, 'pass', 'Should show pass icon when vscode wrapper and webhook running');
      assert.strictEqual(wrapper.description, 'VS Code Terminal');
    });

    test('shows warning icon when wrapper is not vscode', async () => {
      const mockContext = createMockContext(sandbox, '/fake/working-dir');
      sandbox.stub(configPaths, 'configFileExists').resolves(true);
      sandbox.stub(configPaths, 'resolveWorkingDirectory').returns('/fake/working-dir');

      // Write a config.toml with sessions.wrapper = "tmux"
      const configPath = path.join(tempDir, 'config.toml');
      await fs.writeFile(configPath, '[sessions]\nwrapper = "tmux"\n');
      sandbox.stub(configPaths, 'getResolvedConfigPath').returns(configPath);

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const sections = provider.getChildren();
      const connections = findChild(sections, 'Connections');
      assert.ok(connections, 'Should have Connections section');

      const children = provider.getChildren(connections);
      const wrapper = findChild(children, 'Session Wrapper');
      assert.ok(wrapper, 'Should have Session Wrapper item');

      const icon = wrapper.iconPath as vscode.ThemeIcon;
      assert.strictEqual(icon.id, 'warning', 'Should show warning icon for non-vscode wrapper');
      assert.strictEqual(wrapper.description, 'tmux');
    });
  });

  suite('progressive disclosure', () => {
    test('tier 0: only Configuration when config not ready', async () => {
      const mockContext = createMockContext(sandbox);
      sandbox.stub(configPaths, 'configFileExists').resolves(false);
      sandbox.stub(configPaths, 'getResolvedConfigPath').returns('');
      sandbox.stub(configPaths, 'resolveWorkingDirectory').returns('');

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.deepStrictEqual(labels, ['Configuration']);
    });

    test('tier 1: Configuration + Connections when config ready but no connections', async () => {
      const mockContext = createMockContext(sandbox, '/fake/working-dir');
      sandbox.stub(configPaths, 'configFileExists').resolves(true);
      sandbox.stub(configPaths, 'getResolvedConfigPath').returns('');
      sandbox.stub(configPaths, 'resolveWorkingDirectory').returns('/fake/working-dir');

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.deepStrictEqual(labels, ['Configuration', 'Connections']);
    });

    test('tier 2: adds Kanban, LLM Tools, Git when connections ready', async () => {
      const mockContext = createMockContext(sandbox, '/fake/working-dir');
      sandbox.stub(configPaths, 'configFileExists').resolves(true);
      sandbox.stub(configPaths, 'getResolvedConfigPath').returns('');
      sandbox.stub(configPaths, 'resolveWorkingDirectory').returns('/fake/working-dir');

      // Write webhook session file so connections are ready
      const operatorDir = path.join(tempDir, 'operator');
      await fs.mkdir(operatorDir, { recursive: true });
      await fs.writeFile(
        path.join(operatorDir, 'vscode-session.json'),
        JSON.stringify({
          wrapper: 'vscode',
          port: 7009,
          pid: 12345,
          version: '0.1.26',
          startedAt: '2024-01-01T00:00:00Z',
          workspace: '/fake/workspace',
        })
      );

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.deepStrictEqual(
        labels,
        ['Configuration', 'Connections', 'Kanban', 'LLM Tools', 'Git']
      );
    });

    test('tier 3: Issue Types appears when kanban configured', async () => {
      const mockContext = createMockContext(sandbox, '/fake/working-dir');
      sandbox.stub(configPaths, 'configFileExists').resolves(true);
      sandbox.stub(configPaths, 'resolveWorkingDirectory').returns('/fake/working-dir');

      // Write config.toml with kanban section
      const configPath = path.join(tempDir, 'config.toml');
      await fs.writeFile(configPath, '[kanban.jira."test.atlassian.net"]\nenabled = true\n');
      sandbox.stub(configPaths, 'getResolvedConfigPath').returns(configPath);

      // Write webhook session so connections are ready
      const operatorDir = path.join(tempDir, 'operator');
      await fs.mkdir(operatorDir, { recursive: true });
      await fs.writeFile(
        path.join(operatorDir, 'vscode-session.json'),
        JSON.stringify({
          wrapper: 'vscode',
          port: 7009,
          pid: 12345,
          version: '0.1.26',
          startedAt: '2024-01-01T00:00:00Z',
          workspace: '/fake/workspace',
        })
      );

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.ok(labels.includes('Issue Types'), 'Should include Issue Types when kanban configured');
      assert.ok(!labels.includes('Managed Projects'), 'Should not include Managed Projects when git not configured');
    });

    test('tier 3: Managed Projects appears when git configured', async () => {
      const mockContext = createMockContext(sandbox, '/fake/working-dir');
      sandbox.stub(configPaths, 'configFileExists').resolves(true);
      sandbox.stub(configPaths, 'resolveWorkingDirectory').returns('/fake/working-dir');

      // Write config.toml with git section
      const configPath = path.join(tempDir, 'config.toml');
      await fs.writeFile(configPath, '[git]\nprovider = "github"\n');
      sandbox.stub(configPaths, 'getResolvedConfigPath').returns(configPath);

      // Write webhook session so connections are ready
      const operatorDir = path.join(tempDir, 'operator');
      await fs.mkdir(operatorDir, { recursive: true });
      await fs.writeFile(
        path.join(operatorDir, 'vscode-session.json'),
        JSON.stringify({
          wrapper: 'vscode',
          port: 7009,
          pid: 12345,
          version: '0.1.26',
          startedAt: '2024-01-01T00:00:00Z',
          workspace: '/fake/workspace',
        })
      );

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.ok(labels.includes('Managed Projects'), 'Should include Managed Projects when git configured');
      assert.ok(!labels.includes('Issue Types'), 'Should not include Issue Types when kanban not configured');
    });

    test('all tiers: all sections visible when fully configured', async () => {
      const mockContext = createMockContext(sandbox, '/fake/working-dir');
      sandbox.stub(configPaths, 'configFileExists').resolves(true);
      sandbox.stub(configPaths, 'resolveWorkingDirectory').returns('/fake/working-dir');

      // Write config.toml with kanban + git sections
      const configPath = path.join(tempDir, 'config.toml');
      await fs.writeFile(
        configPath,
        '[kanban.jira."test.atlassian.net"]\nenabled = true\n\n[git]\nprovider = "github"\n'
      );
      sandbox.stub(configPaths, 'getResolvedConfigPath').returns(configPath);

      // Write webhook session so connections are ready
      const operatorDir = path.join(tempDir, 'operator');
      await fs.mkdir(operatorDir, { recursive: true });
      await fs.writeFile(
        path.join(operatorDir, 'vscode-session.json'),
        JSON.stringify({
          wrapper: 'vscode',
          port: 7009,
          pid: 12345,
          version: '0.1.26',
          startedAt: '2024-01-01T00:00:00Z',
          workspace: '/fake/workspace',
        })
      );

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.deepStrictEqual(
        labels,
        ['Configuration', 'Connections', 'Kanban', 'LLM Tools', 'Git', 'Issue Types', 'Managed Projects']
      );
    });

    test('tier 3 not visible when connections disconnected even if kanban/git configured', async () => {
      const mockContext = createMockContext(sandbox, '/fake/working-dir');
      sandbox.stub(configPaths, 'configFileExists').resolves(true);
      sandbox.stub(configPaths, 'resolveWorkingDirectory').returns('/fake/working-dir');

      // Write config.toml with kanban + git — but NO webhook session
      const configPath = path.join(tempDir, 'config.toml');
      await fs.writeFile(
        configPath,
        '[kanban.jira."test.atlassian.net"]\nenabled = true\n\n[git]\nprovider = "github"\n'
      );
      sandbox.stub(configPaths, 'getResolvedConfigPath').returns(configPath);

      const provider = new StatusTreeProvider(mockContext);
      await provider.setTicketsDir(tempDir);

      const labels = getSectionLabels(provider.getChildren());
      assert.deepStrictEqual(
        labels,
        ['Configuration', 'Connections'],
        'Should only show tier 0+1 when connections not ready'
      );
    });
  });
});
