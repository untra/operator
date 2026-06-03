/**
 * Tests for mcp-connect.ts
 *
 * Covers:
 * - `fetchMcpDescriptor` HTTP behavior
 * - `detectHostApp` IDE branch detection
 * - `registerInVscodeWorkspaceConfig` (stdio-preferred, SSE fallback)
 * - `registerInCursorUserConfig` (~/.cursor/mcp.json merge semantics)
 * - `connectMcpServer` end-to-end dispatch
 */

import * as assert from 'assert';
import * as sinon from 'sinon';
import * as vscode from 'vscode';
import * as fs from 'fs/promises';
import * as path from 'path';
import * as os from 'os';
import * as mcpConnect from '../../src/mcp-connect';
import * as apiClient from '../../src/api-client';
import {
  fetchMcpDescriptor,
  detectHostApp,
  registerInCursorUserConfig,
  registerInVscodeWorkspaceConfig,
  connectMcpServer,
  _testable,
  McpDescriptorResponse,
} from '../../src/mcp-connect';

const fixturesDir = path.join(
  __dirname,
  '..',
  '..',
  '..',
  'test',
  'fixtures',
  'api'
);

async function loadFixture(name: string): Promise<McpDescriptorResponse> {
  return JSON.parse(
    await fs.readFile(path.join(fixturesDir, name), 'utf-8')
  ) as McpDescriptorResponse;
}

suite('MCP Connect Test Suite', () => {
  let fetchStub: sinon.SinonStub;

  setup(() => {
    fetchStub = sinon.stub(global, 'fetch');
  });

  teardown(() => {
    sinon.restore();
  });

  suite('fetchMcpDescriptor()', () => {
    test('fetches descriptor from correct URL', async () => {
      const descriptorResponse = await loadFixture(
        'mcp-descriptor-response.json'
      );

      fetchStub.resolves(
        new Response(JSON.stringify(descriptorResponse), { status: 200 })
      );

      const result = await fetchMcpDescriptor('http://localhost:7008');

      assert.ok(fetchStub.calledOnce);
      assert.strictEqual(
        fetchStub.firstCall.args[0],
        'http://localhost:7008/api/v1/mcp/descriptor'
      );
      assert.strictEqual(result.server_name, 'operator');
      assert.strictEqual(result.server_id, 'operator-mcp');
      assert.strictEqual(result.version, '0.1.26');
      assert.strictEqual(
        result.transport_url,
        'http://localhost:7008/api/v1/mcp/sse'
      );
    });

    test('throws on network failure', async () => {
      fetchStub.rejects(new Error('Connection refused'));

      await assert.rejects(
        () => fetchMcpDescriptor('http://localhost:7008'),
        /Operator API is not running/
      );
    });

    test('throws on HTTP 404', async () => {
      fetchStub.resolves(new Response('Not Found', { status: 404 }));

      await assert.rejects(
        () => fetchMcpDescriptor('http://localhost:7008'),
        /MCP descriptor unavailable/
      );
    });

    test('throws on HTTP 500', async () => {
      fetchStub.resolves(
        new Response('Internal Server Error', { status: 500 })
      );

      await assert.rejects(
        () => fetchMcpDescriptor('http://localhost:7008'),
        /MCP descriptor unavailable/
      );
    });

    test('uses custom API URL', async () => {
      const descriptorResponse: McpDescriptorResponse = {
        server_name: 'operator',
        server_id: 'operator-mcp',
        version: '0.1.26',
        transport_url: 'http://localhost:9999/api/v1/mcp/sse',
        label: 'Operator MCP Server',
        openapi_url: null,
      };

      fetchStub.resolves(
        new Response(JSON.stringify(descriptorResponse), { status: 200 })
      );

      await fetchMcpDescriptor('http://localhost:9999');

      assert.strictEqual(
        fetchStub.firstCall.args[0],
        'http://localhost:9999/api/v1/mcp/descriptor'
      );
    });

    test('parses stdio field when present', async () => {
      const descriptorResponse = await loadFixture(
        'mcp-descriptor-response-stdio.json'
      );

      fetchStub.resolves(
        new Response(JSON.stringify(descriptorResponse), { status: 200 })
      );

      const result = await fetchMcpDescriptor('http://localhost:7008');

      assert.ok(result.stdio, 'stdio field should be populated');
      assert.strictEqual(result.stdio?.command, '/usr/local/bin/operator');
      assert.deepStrictEqual(result.stdio?.args, ['mcp']);
      assert.strictEqual(result.stdio?.cwd, '/Users/dev/work');
    });
  });

  suite('detectHostApp()', () => {
    let rawAppNameStub: sinon.SinonStub;

    setup(() => {
      rawAppNameStub = sinon.stub(_testable, 'rawAppName');
    });

    test("returns 'cursor' for exact 'Cursor'", () => {
      rawAppNameStub.returns('Cursor');
      assert.strictEqual(detectHostApp(), 'cursor');
    });

    test("returns 'cursor' for 'Cursor (Anysphere)'", () => {
      rawAppNameStub.returns('Cursor (Anysphere)');
      assert.strictEqual(detectHostApp(), 'cursor');
    });

    test("returns 'vscode' for 'Visual Studio Code'", () => {
      rawAppNameStub.returns('Visual Studio Code');
      assert.strictEqual(detectHostApp(), 'vscode');
    });

    test("returns 'vscode' for 'Visual Studio Code - Insiders'", () => {
      rawAppNameStub.returns('Visual Studio Code - Insiders');
      assert.strictEqual(detectHostApp(), 'vscode');
    });

    test("returns 'other' for empty string", () => {
      rawAppNameStub.returns('');
      assert.strictEqual(detectHostApp(), 'other');
    });

    test("returns 'other' for unknown host (e.g. 'Theia IDE')", () => {
      rawAppNameStub.returns('Theia IDE');
      assert.strictEqual(detectHostApp(), 'other');
    });
  });

  suite('registerInCursorUserConfig()', () => {
    let tmpDir: string;
    let cursorConfigPath: string;
    let infoStub: sinon.SinonStub;
    let errorStub: sinon.SinonStub;

    setup(async () => {
      tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), 'op-cursor-test-'));
      cursorConfigPath = path.join(tmpDir, '.cursor', 'mcp.json');
      infoStub = sinon.stub(vscode.window, 'showInformationMessage');
      errorStub = sinon.stub(vscode.window, 'showErrorMessage');
    });

    teardown(async () => {
      await fs.rm(tmpDir, { recursive: true, force: true });
    });

    test('creates ~/.cursor directory when missing', async () => {
      const descriptor = await loadFixture(
        'mcp-descriptor-response-stdio.json'
      );

      await registerInCursorUserConfig(descriptor, cursorConfigPath);

      const stat = await fs.stat(path.dirname(cursorConfigPath));
      assert.ok(stat.isDirectory(), '~/.cursor should exist');
    });

    test('writes mcpServers.operator with stdio shape', async () => {
      const descriptor = await loadFixture(
        'mcp-descriptor-response-stdio.json'
      );

      await registerInCursorUserConfig(descriptor, cursorConfigPath);

      const raw = await fs.readFile(cursorConfigPath, 'utf-8');
      const parsed = JSON.parse(raw) as {
        mcpServers: Record<string, { command: string; args: string[]; cwd: string }>;
      };
      assert.deepStrictEqual(parsed.mcpServers.operator, {
        command: '/usr/local/bin/operator',
        args: ['mcp'],
        cwd: '/Users/dev/work',
      });
      assert.ok(
        infoStub.calledOnce,
        'showInformationMessage should be called'
      );
    });

    test('preserves existing mcpServers.* entries during merge', async () => {
      await fs.mkdir(path.dirname(cursorConfigPath), { recursive: true });
      await fs.writeFile(
        cursorConfigPath,
        JSON.stringify({
          mcpServers: {
            'other-server': { command: '/usr/bin/other', args: [] },
          },
        }),
        'utf-8'
      );
      const descriptor = await loadFixture(
        'mcp-descriptor-response-stdio.json'
      );

      await registerInCursorUserConfig(descriptor, cursorConfigPath);

      const parsed = JSON.parse(
        await fs.readFile(cursorConfigPath, 'utf-8')
      ) as { mcpServers: Record<string, unknown> };
      assert.ok(
        parsed.mcpServers['other-server'],
        'existing other-server should survive'
      );
      assert.ok(
        parsed.mcpServers['operator'],
        'new operator entry should be written'
      );
    });

    test('preserves other top-level keys during merge', async () => {
      await fs.mkdir(path.dirname(cursorConfigPath), { recursive: true });
      await fs.writeFile(
        cursorConfigPath,
        JSON.stringify({
          customKey: { foo: 'bar' },
          mcpServers: {},
        }),
        'utf-8'
      );
      const descriptor = await loadFixture(
        'mcp-descriptor-response-stdio.json'
      );

      await registerInCursorUserConfig(descriptor, cursorConfigPath);

      const parsed = JSON.parse(
        await fs.readFile(cursorConfigPath, 'utf-8')
      ) as { customKey: { foo: string }; mcpServers: Record<string, unknown> };
      assert.deepStrictEqual(parsed.customKey, { foo: 'bar' });
      assert.ok(parsed.mcpServers['operator']);
    });

    test('shows error and skips write when descriptor lacks stdio', async () => {
      const descriptor: McpDescriptorResponse = {
        server_name: 'operator',
        server_id: 'operator-mcp',
        version: '0.1.32',
        transport_url: 'http://localhost:7008/api/v1/mcp/sse',
        label: 'Operator MCP Server',
        openapi_url: null,
        // no stdio
      };

      await registerInCursorUserConfig(descriptor, cursorConfigPath);

      assert.ok(errorStub.calledOnce, 'error message should be shown');
      const errorMsg = errorStub.firstCall.args[0] as string;
      assert.ok(
        errorMsg.includes('stdio_advertised'),
        'error should name the config knob'
      );
      await assert.rejects(
        () => fs.access(cursorConfigPath),
        'mcp.json should NOT have been written'
      );
    });

    test('shows error when existing mcp.json is malformed JSON', async () => {
      await fs.mkdir(path.dirname(cursorConfigPath), { recursive: true });
      await fs.writeFile(cursorConfigPath, '{not valid json', 'utf-8');
      const descriptor = await loadFixture(
        'mcp-descriptor-response-stdio.json'
      );

      await registerInCursorUserConfig(descriptor, cursorConfigPath);

      assert.ok(errorStub.calledOnce, 'error message should be shown');
      const errorMsg = errorStub.firstCall.args[0] as string;
      assert.ok(
        errorMsg.includes('Could not parse'),
        'error should mention parse failure'
      );
      // file should be unchanged (still malformed)
      const raw = await fs.readFile(cursorConfigPath, 'utf-8');
      assert.strictEqual(raw, '{not valid json');
    });
  });

  suite('registerInVscodeWorkspaceConfig()', () => {
    let configUpdateStub: sinon.SinonStub;
    let infoStub: sinon.SinonStub;
    let getStub: sinon.SinonStub;

    setup(() => {
      configUpdateStub = sinon.stub().resolves();
      getStub = sinon.stub().returns({});
      sinon.stub(vscode.workspace, 'getConfiguration').returns({
        get: getStub,
        update: configUpdateStub,
      } as unknown as vscode.WorkspaceConfiguration);
      infoStub = sinon.stub(vscode.window, 'showInformationMessage');
    });

    test('writes stdio entry when descriptor.stdio is present', async () => {
      const descriptor = await loadFixture(
        'mcp-descriptor-response-stdio.json'
      );

      await registerInVscodeWorkspaceConfig(descriptor);

      assert.ok(configUpdateStub.calledOnce);
      assert.strictEqual(configUpdateStub.firstCall.args[0], 'servers');
      const written = configUpdateStub.firstCall.args[1] as Record<
        string,
        Record<string, unknown>
      >;
      assert.deepStrictEqual(written.operator, {
        type: 'stdio',
        command: '/usr/local/bin/operator',
        args: ['mcp'],
        cwd: '/Users/dev/work',
      });
      assert.ok(infoStub.calledOnce);
    });

    test('writes sse entry when descriptor.stdio is absent', async () => {
      const descriptor = await loadFixture(
        'mcp-descriptor-response.json'
      );

      await registerInVscodeWorkspaceConfig(descriptor);

      const written = configUpdateStub.firstCall.args[1] as Record<
        string,
        Record<string, unknown>
      >;
      assert.deepStrictEqual(written.operator, {
        type: 'sse',
        url: 'http://localhost:7008/api/v1/mcp/sse',
      });
    });

    test('preserves existing mcp.servers entries during merge', async () => {
      getStub.returns({
        'other-server': { type: 'sse', url: 'http://other' },
      });
      const descriptor = await loadFixture(
        'mcp-descriptor-response-stdio.json'
      );

      await registerInVscodeWorkspaceConfig(descriptor);

      const written = configUpdateStub.firstCall.args[1] as Record<
        string,
        Record<string, unknown>
      >;
      assert.ok(written['other-server']);
      assert.ok(written.operator);
    });
  });

  suite('connectMcpServer() dispatch', () => {
    let detectHostStub: sinon.SinonStub;
    let discoverApiUrlStub: sinon.SinonStub;
    let configUpdateStub: sinon.SinonStub;
    let getStub: sinon.SinonStub;

    setup(() => {
      detectHostStub = sinon.stub(_testable, 'rawAppName');
      discoverApiUrlStub = sinon
        .stub(apiClient, 'discoverApiUrl')
        .resolves('http://localhost:7008');
      configUpdateStub = sinon.stub().resolves();
      getStub = sinon.stub().returns({});
      sinon.stub(vscode.workspace, 'getConfiguration').returns({
        get: getStub,
        update: configUpdateStub,
      } as unknown as vscode.WorkspaceConfiguration);
      sinon.stub(vscode.window, 'showInformationMessage');
      sinon.stub(vscode.window, 'showErrorMessage');
    });

    test('VS Code host writes to workspace mcp.servers', async () => {
      detectHostStub.returns('Visual Studio Code');
      const descriptor = await loadFixture(
        'mcp-descriptor-response-stdio.json'
      );
      fetchStub.resolves(
        new Response(JSON.stringify(descriptor), { status: 200 })
      );

      await connectMcpServer(undefined);

      assert.ok(discoverApiUrlStub.calledOnce);
      assert.ok(
        configUpdateStub.calledOnce,
        'VS Code path should write workspace config'
      );
      const written = configUpdateStub.firstCall.args[1] as Record<
        string,
        Record<string, unknown> | undefined
      >;
      assert.ok(written.operator, 'operator entry should be written');
      assert.strictEqual(written.operator.type, 'stdio');
    });

    test('Cursor host does NOT write to workspace mcp.servers', async () => {
      detectHostStub.returns('Cursor');
      // Re-stub cursorMcpConfigPath to a tmp path so the test doesn't
      // mutate the developer's real ~/.cursor/mcp.json.
      const tmpDir = await fs.mkdtemp(
        path.join(os.tmpdir(), 'op-cursor-dispatch-')
      );
      const tmpConfigPath = path.join(tmpDir, '.cursor', 'mcp.json');
      sinon.stub(_testable, 'cursorMcpConfigPath').returns(tmpConfigPath);

      try {
        const descriptor = await loadFixture(
          'mcp-descriptor-response-stdio.json'
        );
        fetchStub.resolves(
          new Response(JSON.stringify(descriptor), { status: 200 })
        );

        await connectMcpServer(undefined);

        assert.ok(
          configUpdateStub.notCalled,
          'Cursor path should NOT touch workspace mcp.servers'
        );
        // and the cursor config file should exist
        const raw = await fs.readFile(tmpConfigPath, 'utf-8');
        const parsed = JSON.parse(raw) as {
          mcpServers: { operator: { command: string } };
        };
        assert.strictEqual(
          parsed.mcpServers.operator.command,
          '/usr/local/bin/operator'
        );
      } finally {
        await fs.rm(tmpDir, { recursive: true, force: true });
      }
    });
  });
});

// Silence unused-import warning — `mcpConnect` is imported for documentation
// (it shows the public surface tests rely on).
void mcpConnect;
