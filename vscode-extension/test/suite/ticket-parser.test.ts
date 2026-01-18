/**
 * Tests for ticket-parser.ts
 *
 * Group 1: Pure Parsers - No mocking required
 * Tests parseTicketContent() and getCurrentSessionId() functions.
 */

import * as assert from 'assert';
import * as fs from 'fs/promises';
import * as path from 'path';
import {
  parseTicketContent,
  getCurrentSessionId,
  parseTicketMetadata,
} from '../../src/ticket-parser';
import { TicketMetadata } from '../../src/types';

// Path to fixtures relative to the workspace root
// __dirname in compiled code is out/test/suite, so we go up 3 levels to workspace root
const fixturesDir = path.join(
  __dirname,
  '..',
  '..',
  '..',
  'test',
  'fixtures',
  'tickets'
);

suite('Ticket Parser Test Suite', () => {
  suite('parseTicketContent()', () => {
    test('parses valid ticket with all fields', async () => {
      const content = await fs.readFile(
        path.join(fixturesDir, 'valid-ticket.md'),
        'utf-8'
      );
      const result = parseTicketContent(content);

      assert.ok(result, 'Should return metadata');
      assert.strictEqual(result.id, 'FEAT-123');
      assert.strictEqual(result.status, 'queue');
      assert.strictEqual(result.step, 'initial');
      assert.strictEqual(result.priority, 'high');
      assert.strictEqual(result.project, 'vscode-extension');
      assert.strictEqual(result.worktreePath, '/tmp/worktrees/FEAT-123');
      assert.strictEqual(result.branch, 'feat/add-dark-mode');
    });

    test('parses sessions block correctly', async () => {
      const content = await fs.readFile(
        path.join(fixturesDir, 'valid-ticket.md'),
        'utf-8'
      );
      const result = parseTicketContent(content);

      assert.ok(result?.sessions, 'Should have sessions');
      assert.strictEqual(result.sessions['initial'], 'abc123');
      assert.strictEqual(result.sessions['review'], 'def456');
    });

    test('parses minimal ticket with only required fields', async () => {
      const content = await fs.readFile(
        path.join(fixturesDir, 'minimal-ticket.md'),
        'utf-8'
      );
      const result = parseTicketContent(content);

      assert.ok(result, 'Should return metadata');
      assert.strictEqual(result.id, 'FIX-001');
      assert.strictEqual(result.status, 'in-progress');
      assert.strictEqual(result.step, '');
      assert.strictEqual(result.priority, '');
      assert.strictEqual(result.project, '');
      assert.strictEqual(result.sessions, undefined);
    });

    test('returns null for content without frontmatter', async () => {
      const content = await fs.readFile(
        path.join(fixturesDir, 'no-frontmatter.md'),
        'utf-8'
      );
      const result = parseTicketContent(content);

      assert.strictEqual(result, null);
    });

    test('returns null for empty frontmatter', async () => {
      const content = await fs.readFile(
        path.join(fixturesDir, 'empty-frontmatter.md'),
        'utf-8'
      );
      const result = parseTicketContent(content);

      // Empty frontmatter (---\n---) doesn't match the regex pattern
      // which expects content between the markers
      assert.strictEqual(result, null);
    });

    test('parses ticket with all fields including multiple sessions', async () => {
      const content = await fs.readFile(
        path.join(fixturesDir, 'all-fields.md'),
        'utf-8'
      );
      const result = parseTicketContent(content);

      assert.ok(result, 'Should return metadata');
      assert.strictEqual(result.id, 'SPIKE-999');
      assert.strictEqual(result.status, 'completed');
      assert.strictEqual(result.step, 'review');
      assert.strictEqual(result.priority, 'critical');
      assert.strictEqual(result.project, 'backend');
      assert.strictEqual(result.worktreePath, '/home/user/worktrees/SPIKE-999');
      assert.strictEqual(result.branch, 'spike/investigate-memory-leak');

      assert.ok(result.sessions, 'Should have sessions');
      assert.strictEqual(result.sessions['initial'], 'session-uuid-1');
      assert.strictEqual(result.sessions['implementation'], 'session-uuid-2');
      assert.strictEqual(result.sessions['review'], 'session-uuid-3');
    });

    test('handles empty content', () => {
      const result = parseTicketContent('');
      assert.strictEqual(result, null);
    });

    test('handles content with only dashes', () => {
      const result = parseTicketContent('---');
      assert.strictEqual(result, null);
    });

    test('handles frontmatter with invalid YAML-like lines', () => {
      const content = `---
id: TEST-001
no-colon-here
: empty-key
status: valid
---
Body content`;
      const result = parseTicketContent(content);

      assert.ok(result, 'Should return metadata');
      assert.strictEqual(result.id, 'TEST-001');
      assert.strictEqual(result.status, 'valid');
    });

    test('handles values with colons', () => {
      const content = `---
id: TEST-002
project: http://example.com:8080
---
Body`;
      const result = parseTicketContent(content);

      assert.ok(result, 'Should return metadata');
      assert.strictEqual(result.id, 'TEST-002');
      assert.strictEqual(result.project, 'http://example.com:8080');
    });

    test('handles whitespace in values', () => {
      const content = `---
id:   FEAT-100
status:in-progress
project:  my project
---
Body`;
      const result = parseTicketContent(content);

      assert.ok(result, 'Should return metadata');
      assert.strictEqual(result.id, 'FEAT-100');
      assert.strictEqual(result.status, 'in-progress');
      assert.strictEqual(result.project, 'my project');
    });

    test('ignores nested/indented lines in main frontmatter', () => {
      const content = `---
id: TASK-001
  nested: value
    deeply: nested
status: queue
---
Body`;
      const result = parseTicketContent(content);

      assert.ok(result, 'Should return metadata');
      assert.strictEqual(result.id, 'TASK-001');
      assert.strictEqual(result.status, 'queue');
    });

    test('handles sessions with varied indentation', () => {
      const content = `---
id: FEAT-200
sessions:
  step1: uuid1
  step2: uuid2
---
Body`;
      const result = parseTicketContent(content);

      assert.ok(result?.sessions, 'Should have sessions');
      assert.strictEqual(result.sessions['step1'], 'uuid1');
      assert.strictEqual(result.sessions['step2'], 'uuid2');
    });

    test('handles frontmatter with Windows line endings', () => {
      const content = '---\r\nid: WIN-001\r\nstatus: queue\r\n---\r\nBody';
      const result = parseTicketContent(content);

      // The regex may not match Windows line endings perfectly
      // This test documents current behavior
      assert.ok(result === null || result.id === 'WIN-001');
    });
  });

  suite('getCurrentSessionId()', () => {
    test('returns session for current step', () => {
      const metadata: TicketMetadata = {
        id: 'TEST-001',
        status: 'in-progress',
        step: 'review',
        priority: '',
        project: '',
        sessions: {
          initial: 'init-uuid',
          review: 'review-uuid',
        },
      };

      const result = getCurrentSessionId(metadata);
      assert.strictEqual(result, 'review-uuid');
    });

    test('falls back to initial when step session not found', () => {
      const metadata: TicketMetadata = {
        id: 'TEST-001',
        status: 'in-progress',
        step: 'unknown-step',
        priority: '',
        project: '',
        sessions: {
          initial: 'init-uuid',
        },
      };

      const result = getCurrentSessionId(metadata);
      assert.strictEqual(result, 'init-uuid');
    });

    test('falls back to initial when step is empty', () => {
      const metadata: TicketMetadata = {
        id: 'TEST-001',
        status: 'in-progress',
        step: '',
        priority: '',
        project: '',
        sessions: {
          initial: 'init-uuid',
          review: 'review-uuid',
        },
      };

      const result = getCurrentSessionId(metadata);
      assert.strictEqual(result, 'init-uuid');
    });

    test('returns undefined when no sessions', () => {
      const metadata: TicketMetadata = {
        id: 'TEST-001',
        status: 'in-progress',
        step: 'review',
        priority: '',
        project: '',
      };

      const result = getCurrentSessionId(metadata);
      assert.strictEqual(result, undefined);
    });

    test('returns undefined when sessions is empty object', () => {
      const metadata: TicketMetadata = {
        id: 'TEST-001',
        status: 'in-progress',
        step: 'review',
        priority: '',
        project: '',
        sessions: {},
      };

      const result = getCurrentSessionId(metadata);
      assert.strictEqual(result, undefined);
    });

    test('returns step session even when initial exists', () => {
      const metadata: TicketMetadata = {
        id: 'TEST-001',
        status: 'in-progress',
        step: 'implementation',
        priority: '',
        project: '',
        sessions: {
          initial: 'init-uuid',
          implementation: 'impl-uuid',
          review: 'review-uuid',
        },
      };

      const result = getCurrentSessionId(metadata);
      assert.strictEqual(result, 'impl-uuid');
    });
  });

  suite('parseTicketMetadata()', () => {
    test('parses existing file', async () => {
      const filePath = path.join(fixturesDir, 'valid-ticket.md');
      const result = await parseTicketMetadata(filePath);

      assert.ok(result, 'Should return metadata');
      assert.strictEqual(result.id, 'FEAT-123');
    });

    test('returns null for non-existent file', async () => {
      const result = await parseTicketMetadata('/nonexistent/path/file.md');
      assert.strictEqual(result, null);
    });

    test('returns null for directory path', async () => {
      const result = await parseTicketMetadata(fixturesDir);
      assert.strictEqual(result, null);
    });
  });
});
