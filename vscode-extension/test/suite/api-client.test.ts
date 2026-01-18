/**
 * Tests for api-client.ts
 *
 * Group 2: Service Logic - Requires fetch mock
 * Tests OperatorApiClient class methods and discoverApiUrl function.
 */

import * as assert from 'assert';
import * as sinon from 'sinon';
import * as fs from 'fs/promises';
import * as path from 'path';
import * as os from 'os';
import {
  OperatorApiClient,
  discoverApiUrl,
  QueueControlResponse,
  KanbanSyncResponse,
  ReviewResponse,
} from '../../src/api-client';
import {
  HealthResponse,
  LaunchTicketRequest,
  LaunchTicketResponse,
} from '../../src/generated';

// Path to fixtures relative to the workspace root
// __dirname in compiled code is out/test/suite, so we go up 3 levels to workspace root
const fixturesDir = path.join(
  __dirname,
  '..',
  '..',
  '..',
  'test',
  'fixtures',
  'api'
);

suite('API Client Test Suite', () => {
  let fetchStub: sinon.SinonStub;

  setup(() => {
    // Stub global fetch
    fetchStub = sinon.stub(global, 'fetch');
  });

  teardown(() => {
    sinon.restore();
  });

  suite('discoverApiUrl()', () => {
    let tempDir: string;

    setup(async () => {
      tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'api-client-test-'));
    });

    teardown(async () => {
      try {
        await fs.rm(tempDir, { recursive: true });
      } catch {
        // Ignore cleanup errors
      }
    });

    test('reads port from api-session.json when available', async () => {
      // Create the operator directory and session file
      const operatorDir = path.join(tempDir, 'operator');
      await fs.mkdir(operatorDir, { recursive: true });
      await fs.writeFile(
        path.join(operatorDir, 'api-session.json'),
        JSON.stringify({
          port: 9999,
          pid: 12345,
          started_at: '2024-01-01T00:00:00Z',
          version: '0.1.14',
        })
      );

      const url = await discoverApiUrl(tempDir);
      assert.strictEqual(url, 'http://localhost:9999');
    });

    test('falls back to configured URL when session file missing', async () => {
      // No session file exists
      const url = await discoverApiUrl(tempDir);
      // Falls back to default from vscode config (mocked to http://localhost:7008)
      assert.strictEqual(url, 'http://localhost:7008');
    });

    test('falls back to configured URL when session file is invalid JSON', async () => {
      const operatorDir = path.join(tempDir, 'operator');
      await fs.mkdir(operatorDir, { recursive: true });
      await fs.writeFile(
        path.join(operatorDir, 'api-session.json'),
        'not valid json'
      );

      const url = await discoverApiUrl(tempDir);
      assert.strictEqual(url, 'http://localhost:7008');
    });

    test('falls back to configured URL when ticketsDir is undefined', async () => {
      const url = await discoverApiUrl(undefined);
      assert.strictEqual(url, 'http://localhost:7008');
    });
  });

  suite('OperatorApiClient constructor', () => {
    test('uses provided baseUrl', () => {
      const client = new OperatorApiClient('http://custom:9000');

      // Verify by making a request
      fetchStub.resolves(
        new Response(JSON.stringify({ status: 'healthy', version: '1.0.0' }), {
          status: 200,
        })
      );

      client.health();

      assert.ok(
        fetchStub.calledWith('http://custom:9000/api/v1/health'),
        'Should use custom URL'
      );
    });

    test('uses default URL when none provided', () => {
      const client = new OperatorApiClient();

      fetchStub.resolves(
        new Response(JSON.stringify({ status: 'healthy', version: '1.0.0' }), {
          status: 200,
        })
      );

      client.health();

      // Default is http://localhost:7008 from vscode config
      assert.ok(
        fetchStub.calledWith('http://localhost:7008/api/v1/health'),
        'Should use default URL'
      );
    });
  });

  suite('health()', () => {
    test('returns health response on success', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      const healthResponse: HealthResponse = await fs
        .readFile(path.join(fixturesDir, 'health-response.json'), 'utf-8')
        .then(JSON.parse);

      fetchStub.resolves(
        new Response(JSON.stringify(healthResponse), { status: 200 })
      );

      const result = await client.health();

      assert.strictEqual(result.status, 'healthy');
      assert.strictEqual(result.version, '0.1.14');
    });

    test('throws error when API not available', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response('Service Unavailable', { status: 503 })
      );

      await assert.rejects(
        () => client.health(),
        /Operator API not available/
      );
    });

    test('throws error on network failure', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.rejects(new Error('Network error'));

      await assert.rejects(() => client.health(), /Network error/);
    });
  });

  suite('launchTicket()', () => {
    test('sends POST request with correct body', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      const launchResponse: LaunchTicketResponse = await fs
        .readFile(path.join(fixturesDir, 'launch-response.json'), 'utf-8')
        .then(JSON.parse);

      fetchStub.resolves(
        new Response(JSON.stringify(launchResponse), { status: 200 })
      );

      const options: LaunchTicketRequest = {
        provider: 'claude',
        model: 'sonnet',
        yolo_mode: true,
        wrapper: 'vscode',
        retry_reason: null,
        resume_session_id: null,
      };

      const result = await client.launchTicket('FEAT-123', options);

      // Verify the fetch call
      assert.ok(fetchStub.calledOnce);
      const [url, init] = fetchStub.firstCall.args;
      assert.strictEqual(
        url,
        'http://localhost:7008/api/v1/tickets/FEAT-123/launch'
      );
      assert.strictEqual(init.method, 'POST');
      assert.strictEqual(init.headers['Content-Type'], 'application/json');

      const body = JSON.parse(init.body);
      assert.strictEqual(body.provider, 'claude');
      assert.strictEqual(body.model, 'sonnet');
      assert.strictEqual(body.yolo_mode, true);
      assert.strictEqual(body.wrapper, 'vscode');

      // Verify response
      assert.strictEqual(result.ticket_id, 'FEAT-123');
      assert.strictEqual(result.terminal_name, 'op-FEAT-123');
      assert.strictEqual(result.worktree_created, true);
    });

    test('URL-encodes ticket ID', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response(
          JSON.stringify({
            agent_id: 'agent-1',
            ticket_id: 'FEAT-123/sub',
            working_directory: '/path',
            command: 'cmd',
            terminal_name: 'term',
            tmux_session_name: 'tmux',
            session_id: 'sess',
            worktree_created: false,
            branch: null,
          }),
          { status: 200 }
        )
      );

      const options: LaunchTicketRequest = {
        provider: null,
        model: null,
        yolo_mode: false,
        wrapper: null,
        retry_reason: null,
        resume_session_id: null,
      };

      await client.launchTicket('FEAT-123/sub', options);

      const [url] = fetchStub.firstCall.args;
      assert.ok(url.includes('FEAT-123%2Fsub'), 'Should URL-encode slash');
    });

    test('throws error with message on HTTP error', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response(
          JSON.stringify({
            error: 'not_found',
            message: 'Ticket FEAT-999 not found',
          }),
          { status: 404 }
        )
      );

      const options: LaunchTicketRequest = {
        provider: null,
        model: null,
        yolo_mode: false,
        wrapper: null,
        retry_reason: null,
        resume_session_id: null,
      };

      await assert.rejects(
        () => client.launchTicket('FEAT-999', options),
        /Ticket FEAT-999 not found/
      );
    });

    test('handles non-JSON error response', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response('Internal Server Error', { status: 500 })
      );

      const options: LaunchTicketRequest = {
        provider: null,
        model: null,
        yolo_mode: false,
        wrapper: null,
        retry_reason: null,
        resume_session_id: null,
      };

      await assert.rejects(
        () => client.launchTicket('FEAT-123', options),
        /HTTP 500/
      );
    });

    test('defaults yolo_mode to false', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response(
          JSON.stringify({
            agent_id: 'agent-1',
            ticket_id: 'FEAT-123',
            working_directory: '/path',
            command: 'cmd',
            terminal_name: 'term',
            tmux_session_name: 'tmux',
            session_id: 'sess',
            worktree_created: false,
            branch: null,
          }),
          { status: 200 }
        )
      );

      // Note: yolo_mode is undefined in options
      const options: Partial<LaunchTicketRequest> = {
        provider: 'claude',
        model: 'sonnet',
        wrapper: null,
      };

      await client.launchTicket('FEAT-123', options as LaunchTicketRequest);

      const body = JSON.parse(fetchStub.firstCall.args[1].body);
      assert.strictEqual(body.yolo_mode, false);
    });
  });

  suite('pauseQueue()', () => {
    test('sends POST request and returns response', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      const pauseResponse: QueueControlResponse = await fs
        .readFile(path.join(fixturesDir, 'queue-paused-response.json'), 'utf-8')
        .then(JSON.parse);

      fetchStub.resolves(
        new Response(JSON.stringify(pauseResponse), { status: 200 })
      );

      const result = await client.pauseQueue();

      assert.ok(fetchStub.calledOnce);
      const [url, init] = fetchStub.firstCall.args;
      assert.strictEqual(url, 'http://localhost:7008/api/v1/queue/pause');
      assert.strictEqual(init.method, 'POST');

      assert.strictEqual(result.paused, true);
      assert.strictEqual(result.message, 'Queue processing paused');
    });

    test('throws error on failure', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response(
          JSON.stringify({
            error: 'failed',
            message: 'Cannot pause queue',
          }),
          { status: 500 }
        )
      );

      await assert.rejects(() => client.pauseQueue(), /Cannot pause queue/);
    });
  });

  suite('resumeQueue()', () => {
    test('sends POST request and returns response', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      const resumeResponse: QueueControlResponse = await fs
        .readFile(
          path.join(fixturesDir, 'queue-resumed-response.json'),
          'utf-8'
        )
        .then(JSON.parse);

      fetchStub.resolves(
        new Response(JSON.stringify(resumeResponse), { status: 200 })
      );

      const result = await client.resumeQueue();

      assert.ok(fetchStub.calledOnce);
      const [url, init] = fetchStub.firstCall.args;
      assert.strictEqual(url, 'http://localhost:7008/api/v1/queue/resume');
      assert.strictEqual(init.method, 'POST');

      assert.strictEqual(result.paused, false);
      assert.strictEqual(result.message, 'Queue processing resumed');
    });

    test('throws error on failure', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response(
          JSON.stringify({
            error: 'failed',
            message: 'Cannot resume queue',
          }),
          { status: 500 }
        )
      );

      await assert.rejects(() => client.resumeQueue(), /Cannot resume queue/);
    });
  });

  suite('syncKanban()', () => {
    test('sends POST request and returns sync response', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      const syncResponse: KanbanSyncResponse = await fs
        .readFile(path.join(fixturesDir, 'sync-response.json'), 'utf-8')
        .then(JSON.parse);

      fetchStub.resolves(
        new Response(JSON.stringify(syncResponse), { status: 200 })
      );

      const result = await client.syncKanban();

      assert.ok(fetchStub.calledOnce);
      const [url, init] = fetchStub.firstCall.args;
      assert.strictEqual(url, 'http://localhost:7008/api/v1/queue/sync');
      assert.strictEqual(init.method, 'POST');

      assert.deepStrictEqual(result.created, ['FEAT-201', 'FIX-202']);
      assert.deepStrictEqual(result.skipped, ['FEAT-100']);
      assert.deepStrictEqual(result.errors, []);
      assert.strictEqual(result.total_processed, 3);
    });

    test('throws error on failure', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response(
          JSON.stringify({
            error: 'sync_failed',
            message: 'Kanban sync failed: connection error',
          }),
          { status: 500 }
        )
      );

      await assert.rejects(
        () => client.syncKanban(),
        /Kanban sync failed: connection error/
      );
    });
  });

  suite('approveReview()', () => {
    test('sends POST request with agent ID', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      const approveResponse: ReviewResponse = await fs
        .readFile(
          path.join(fixturesDir, 'review-approved-response.json'),
          'utf-8'
        )
        .then(JSON.parse);

      fetchStub.resolves(
        new Response(JSON.stringify(approveResponse), { status: 200 })
      );

      const result = await client.approveReview('agent-abc123');

      assert.ok(fetchStub.calledOnce);
      const [url, init] = fetchStub.firstCall.args;
      assert.strictEqual(
        url,
        'http://localhost:7008/api/v1/agents/agent-abc123/approve'
      );
      assert.strictEqual(init.method, 'POST');

      assert.strictEqual(result.agent_id, 'agent-abc123');
      assert.strictEqual(result.status, 'approved');
    });

    test('URL-encodes agent ID', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response(
          JSON.stringify({
            agent_id: 'agent/special',
            status: 'approved',
            message: 'ok',
          }),
          { status: 200 }
        )
      );

      await client.approveReview('agent/special');

      const [url] = fetchStub.firstCall.args;
      assert.ok(url.includes('agent%2Fspecial'), 'Should URL-encode slash');
    });

    test('throws error on failure', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response(
          JSON.stringify({
            error: 'not_found',
            message: 'Agent not found',
          }),
          { status: 404 }
        )
      );

      await assert.rejects(
        () => client.approveReview('nonexistent'),
        /Agent not found/
      );
    });
  });

  suite('rejectReview()', () => {
    test('sends POST request with agent ID and reason', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      const rejectResponse: ReviewResponse = await fs
        .readFile(
          path.join(fixturesDir, 'review-rejected-response.json'),
          'utf-8'
        )
        .then(JSON.parse);

      fetchStub.resolves(
        new Response(JSON.stringify(rejectResponse), { status: 200 })
      );

      const result = await client.rejectReview(
        'agent-abc123',
        'Tests are failing'
      );

      assert.ok(fetchStub.calledOnce);
      const [url, init] = fetchStub.firstCall.args;
      assert.strictEqual(
        url,
        'http://localhost:7008/api/v1/agents/agent-abc123/reject'
      );
      assert.strictEqual(init.method, 'POST');
      assert.strictEqual(init.headers['Content-Type'], 'application/json');

      const body = JSON.parse(init.body);
      assert.strictEqual(body.reason, 'Tests are failing');

      assert.strictEqual(result.agent_id, 'agent-abc123');
      assert.strictEqual(result.status, 'rejected');
    });

    test('handles empty reason', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response(
          JSON.stringify({
            agent_id: 'agent-abc123',
            status: 'rejected',
            message: 'Review rejected',
          }),
          { status: 200 }
        )
      );

      await client.rejectReview('agent-abc123', '');

      const body = JSON.parse(fetchStub.firstCall.args[1].body);
      assert.strictEqual(body.reason, '');
    });

    test('throws error on failure', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response(
          JSON.stringify({
            error: 'not_found',
            message: 'Agent not found',
          }),
          { status: 404 }
        )
      );

      await assert.rejects(
        () => client.rejectReview('nonexistent', 'reason'),
        /Agent not found/
      );
    });
  });

  suite('error handling edge cases', () => {
    test('handles network timeout', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      const timeoutError = new Error('timeout');
      timeoutError.name = 'TimeoutError';
      fetchStub.rejects(timeoutError);

      await assert.rejects(() => client.health(), /timeout/);
    });

    test('handles response.json() failure', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      // Mock response where json() throws
      const mockResponse = {
        ok: true,
        json: () => Promise.reject(new Error('Invalid JSON')),
      };
      fetchStub.resolves(mockResponse as unknown as Response);

      await assert.rejects(() => client.health(), /Invalid JSON/);
    });

    test('handles HTTP 401 Unauthorized', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response(
          JSON.stringify({
            error: 'unauthorized',
            message: 'Authentication required',
          }),
          { status: 401 }
        )
      );

      await assert.rejects(
        () => client.pauseQueue(),
        /Authentication required/
      );
    });

    test('handles HTTP 403 Forbidden', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response(
          JSON.stringify({
            error: 'forbidden',
            message: 'Permission denied',
          }),
          { status: 403 }
        )
      );

      await assert.rejects(() => client.resumeQueue(), /Permission denied/);
    });

    test('handles HTTP 429 Rate Limited', async () => {
      const client = new OperatorApiClient('http://localhost:7008');

      fetchStub.resolves(
        new Response(
          JSON.stringify({
            error: 'rate_limited',
            message: 'Too many requests',
          }),
          { status: 429 }
        )
      );

      await assert.rejects(() => client.syncKanban(), /Too many requests/);
    });
  });
});
