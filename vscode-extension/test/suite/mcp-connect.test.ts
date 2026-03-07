/**
 * Tests for mcp-connect.ts
 *
 * Tests MCP descriptor fetching, deep link building, and the
 * connectMcpServer flow.
 */

import * as assert from 'assert';
import * as sinon from 'sinon';
import * as fs from 'fs/promises';
import * as path from 'path';
import {
  fetchMcpDescriptor,
  buildMcpDeepLink,
  McpDescriptorResponse,
} from '../../src/mcp-connect';

// Path to fixtures
const fixturesDir = path.join(
  __dirname,
  '..',
  '..',
  '..',
  'test',
  'fixtures',
  'api'
);

/** Shape of the decoded MCP config embedded in deep link URIs */
interface McpDeepLinkConfig {
  name: string;
  type: string;
  url: string;
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
      const descriptorResponse: McpDescriptorResponse = JSON.parse(
        await fs.readFile(
          path.join(fixturesDir, 'mcp-descriptor-response.json'),
          'utf-8'
        )
      ) as McpDescriptorResponse;

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
  });

  suite('buildMcpDeepLink()', () => {
    test('builds correct vscode:// URI', () => {
      const descriptor: McpDescriptorResponse = {
        server_name: 'operator',
        server_id: 'operator-mcp',
        version: '0.1.26',
        transport_url: 'http://localhost:7008/api/v1/mcp/sse',
        label: 'Operator MCP Server',
        openapi_url: 'http://localhost:7008/api-docs/openapi.json',
      };

      const uri = buildMcpDeepLink(descriptor);

      assert.strictEqual(uri.scheme, 'vscode');
      assert.strictEqual(uri.authority, 'modelcontextprotocol.mcp');
      assert.strictEqual(uri.path, '/connect');
    });

    test('encodes correct config in base64', () => {
      const descriptor: McpDescriptorResponse = {
        server_name: 'operator',
        server_id: 'operator-mcp',
        version: '0.1.26',
        transport_url: 'http://localhost:7008/api/v1/mcp/sse',
        label: 'Operator MCP Server',
        openapi_url: null,
      };

      const uri = buildMcpDeepLink(descriptor);
      const query = uri.query;

      // Extract base64 config from query
      assert.ok(query.startsWith('config='), 'Query should start with config=');
      const base64 = query.replace('config=', '');
      const decoded = JSON.parse(Buffer.from(base64, 'base64').toString()) as McpDeepLinkConfig;

      assert.strictEqual(decoded.name, 'operator');
      assert.strictEqual(decoded.type, 'sse');
      assert.strictEqual(
        decoded.url,
        'http://localhost:7008/api/v1/mcp/sse'
      );
    });

    test('uses server_name from descriptor', () => {
      const descriptor: McpDescriptorResponse = {
        server_name: 'custom-operator',
        server_id: 'custom-mcp',
        version: '1.0.0',
        transport_url: 'http://localhost:9999/api/v1/mcp/sse',
        label: 'Custom MCP',
        openapi_url: null,
      };

      const uri = buildMcpDeepLink(descriptor);
      const base64 = uri.query.replace('config=', '');
      const decoded = JSON.parse(Buffer.from(base64, 'base64').toString()) as McpDeepLinkConfig;

      assert.strictEqual(decoded.name, 'custom-operator');
      assert.strictEqual(
        decoded.url,
        'http://localhost:9999/api/v1/mcp/sse'
      );
    });
  });
});
