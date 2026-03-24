/**
 * Tests for mcp-connect.ts
 *
 * Tests MCP descriptor fetching and server registration check.
 */

import * as assert from 'assert';
import * as sinon from 'sinon';
import * as fs from 'fs/promises';
import * as path from 'path';
import {
  fetchMcpDescriptor,
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
});
