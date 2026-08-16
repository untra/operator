/**
 * Ticket tree provider tests: API-backed refresh with disk fallback.
 *
 * The API-backed tree must yield the same TicketInfo shape the disk parser
 * produces for the same queue, and fall back to disk when no server answers.
 */

import * as assert from 'assert';
import * as sinon from 'sinon';
import * as fs from 'fs/promises';
import * as path from 'path';
import * as os from 'os';
import { TicketTreeProvider } from '../../src/ticket-provider';
import { OperatorApiClient } from '../../src/api-client';
import { IssueTypeService } from '../../src/issuetype-service';
import type { KanbanBoardResponse } from '../../src/generated';

function mockOutputChannel(): import('vscode').OutputChannel {
  return {
    append: () => undefined,
    appendLine: () => undefined,
    clear: () => undefined,
    show: () => undefined,
    hide: () => undefined,
    dispose: () => undefined,
    replace: () => undefined,
    name: 'test',
  } as unknown as import('vscode').OutputChannel;
}

function board(partial: Partial<KanbanBoardResponse>): KanbanBoardResponse {
  return {
    queue: [],
    running: [],
    awaiting: [],
    done: [],
    total_count: 0,
    last_updated: new Date(0).toISOString(),
    ...partial,
  };
}

const QUEUE_CARD = {
  id: 'FEAT-9100',
  step_display_name: null as string | null,
  summary: 'API-backed tree test',
  ticket_type: 'FEAT',
  project: 'operator',
  status: 'queued',
  step: 'plan',
  priority: 'P2-medium',
  timestamp: '20260807-1200',
  filename: '20260807-1200-FEAT-operator-api-tree.md',
};

suite('TicketTreeProvider (API-backed)', () => {
  let sandbox: sinon.SinonSandbox;
  let tmpDir: string;

  setup(async () => {
    sandbox = sinon.createSandbox();
    tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), 'op-tree-'));
    await fs.mkdir(path.join(tmpDir, 'queue'), { recursive: true });
  });

  teardown(async () => {
    sandbox.restore();
    await fs.rm(tmpDir, { recursive: true, force: true });
  });

  test('API cards map onto the same TicketInfo shape as disk parsing', async () => {
    // Disk fixture equivalent to the API card.
    const ticketMd = `---
id: ${QUEUE_CARD.id}
status: queued
step: plan
---

# Feature: ${QUEUE_CARD.summary}
`;
    await fs.writeFile(
      path.join(tmpDir, 'queue', QUEUE_CARD.filename),
      ticketMd
    );

    const service = new IssueTypeService(mockOutputChannel());
    const diskProvider = new TicketTreeProvider('queue', service);
    await diskProvider.setTicketsDir(tmpDir);
    const diskItems = diskProvider.getChildren();

    const apiProvider = new TicketTreeProvider('queue', service);
    const client = new OperatorApiClient('http://localhost:1');
    sandbox
      .stub(client, 'getKanban')
      .resolves(board({ queue: [QUEUE_CARD], total_count: 1 }));
    apiProvider.setApiClient(client);
    await apiProvider.setTicketsDir(tmpDir);
    const apiItems = apiProvider.getChildren();

    assert.strictEqual(apiItems.length, 1, 'API tree shows the card');
    assert.strictEqual(diskItems.length, 1, 'disk tree shows the file');
    const apiTicket = apiItems[0]!.ticket;
    const diskTicket = diskItems[0]!.ticket;
    assert.strictEqual(apiTicket.id, diskTicket.id);
    assert.strictEqual(apiTicket.type, diskTicket.type);
    assert.strictEqual(apiTicket.status, diskTicket.status);
    assert.strictEqual(
      apiTicket.filePath,
      diskTicket.filePath,
      'co-located clients keep a real file path for opening'
    );
  });

  test('a ticket without a frontmatter id falls back to the filename', async () => {
    await fs.writeFile(
      path.join(tmpDir, 'queue', QUEUE_CARD.filename),
      `---\nstatus: queued\n---\n\n# Feature: no id in frontmatter\n`
    );
    const provider = new TicketTreeProvider(
      'queue',
      new IssueTypeService(mockOutputChannel())
    );
    await provider.setTicketsDir(tmpDir);
    const ticket = provider.getChildren()[0]!.ticket;
    assert.strictEqual(ticket.id, 'FEAT-202608071200');
    assert.strictEqual(ticket.type, 'FEAT');
  });

  test('running and awaiting columns both land in the in-progress tree', async () => {
    const service = new IssueTypeService(mockOutputChannel());
    const provider = new TicketTreeProvider('in-progress', service);
    const client = new OperatorApiClient('http://localhost:1');
    sandbox.stub(client, 'getKanban').resolves(
      board({
        running: [{ ...QUEUE_CARD, id: 'FEAT-1', status: 'running' }],
        awaiting: [{ ...QUEUE_CARD, id: 'FEAT-2', status: 'awaiting' }],
        total_count: 2,
      })
    );
    provider.setApiClient(client);
    await provider.refresh();
    const ids = provider.getChildren().map((i) => i.ticket.id);
    assert.deepStrictEqual(ids, ['FEAT-1', 'FEAT-2']);
  });

  test('falls back to disk when the server is unreachable', async () => {
    await fs.writeFile(
      path.join(tmpDir, 'queue', QUEUE_CARD.filename),
      `---\nid: ${QUEUE_CARD.id}\nstatus: queued\n---\n\n# Feature: fallback\n`
    );
    const service = new IssueTypeService(mockOutputChannel());
    const provider = new TicketTreeProvider('queue', service);
    const client = new OperatorApiClient('http://localhost:1');
    sandbox.stub(client, 'getKanban').rejects(new Error('unreachable'));
    provider.setApiClient(client);
    await provider.setTicketsDir(tmpDir);
    const ids = provider.getChildren().map((i) => i.ticket.id);
    assert.deepStrictEqual(ids, [QUEUE_CARD.id], 'disk reads remain the fallback');
  });
});
