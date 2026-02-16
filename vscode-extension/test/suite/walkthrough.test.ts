/**
 * Tests for walkthrough.ts
 *
 * Tests environment variable detection, LLM tool detection, and directory validation.
 */

import * as assert from 'assert';
import * as path from 'path';
import * as os from 'os';
import * as fs from 'fs/promises';
import {
  checkKanbanEnvVars,
  checkLlmToolInPath,
  validateWorkingDirectory,
  detectInstalledLlmTools,
  findEnvVar,
  fetchLinearWorkspace,
  getKanbanWorkspaces,
  KANBAN_ENV_VARS,
  LLM_TOOLS,
  KanbanEnvResult,
} from '../../src/walkthrough';

/** Get all kanban env var keys for setup/teardown */
function getAllKanbanEnvKeys(): string[] {
  return [
    ...KANBAN_ENV_VARS.jira.apiKey,
    ...KANBAN_ENV_VARS.jira.domain,
    ...KANBAN_ENV_VARS.jira.email,
    ...KANBAN_ENV_VARS.linear.apiKey,
  ];
}

suite('Walkthrough Test Suite', () => {
  suite('findEnvVar()', () => {
    const testKeys = ['TEST_VAR_A', 'TEST_VAR_B', 'TEST_VAR_C'];
    const originalEnv: Record<string, string | undefined> = {};

    setup(() => {
      for (const key of testKeys) {
        originalEnv[key] = process.env[key];
        delete process.env[key];
      }
    });

    teardown(() => {
      for (const key of testKeys) {
        if (originalEnv[key] !== undefined) {
          process.env[key] = originalEnv[key];
        } else {
          delete process.env[key];
        }
      }
    });

    test('returns undefined when no keys are set', () => {
      const result = findEnvVar(testKeys);
      assert.strictEqual(result, undefined);
    });

    test('returns first matching env var value', () => {
      process.env['TEST_VAR_B'] = 'value-b';
      process.env['TEST_VAR_C'] = 'value-c';
      const result = findEnvVar(testKeys);
      assert.strictEqual(result, 'value-b');
    });

    test('returns first key when multiple are set', () => {
      process.env['TEST_VAR_A'] = 'value-a';
      process.env['TEST_VAR_B'] = 'value-b';
      const result = findEnvVar(testKeys);
      assert.strictEqual(result, 'value-a');
    });
  });

  suite('checkKanbanEnvVars()', () => {
    // Store original env values
    const originalEnv: Record<string, string | undefined> = {};

    setup(() => {
      // Save original values
      for (const key of getAllKanbanEnvKeys()) {
        originalEnv[key] = process.env[key];
      }
    });

    teardown(() => {
      // Restore original values
      for (const key of getAllKanbanEnvKeys()) {
        if (originalEnv[key] !== undefined) {
          process.env[key] = originalEnv[key];
        } else {
          delete process.env[key];
        }
      }
    });

    test('returns empty workspaces when no env vars set', () => {
      // Clear all kanban env vars
      for (const key of getAllKanbanEnvKeys()) {
        delete process.env[key];
      }

      const result: KanbanEnvResult = checkKanbanEnvVars();

      assert.strictEqual(result.workspaces.length, 0);
      assert.strictEqual(result.anyConfigured, false);
    });

    test('returns jira workspace with domain in URL when both API key and domain set', () => {
      // Clear all kanban env vars first
      for (const key of getAllKanbanEnvKeys()) {
        delete process.env[key];
      }

      process.env['OPERATOR_JIRA_API_KEY'] = 'test-key';
      process.env['OPERATOR_JIRA_DOMAIN'] = 'mycompany.atlassian.net';
      const result: KanbanEnvResult = checkKanbanEnvVars();

      assert.strictEqual(result.workspaces.length, 1);
      assert.strictEqual(result.anyConfigured, true);
      assert.strictEqual(result.workspaces[0].provider, 'jira');
      assert.strictEqual(result.workspaces[0].name, 'mycompany.atlassian.net');
      assert.strictEqual(result.workspaces[0].url, 'https://mycompany.atlassian.net');
      assert.strictEqual(result.workspaces[0].configured, true);
    });

    test('does not return jira workspace when only API key set (no domain)', () => {
      // Clear all kanban env vars first
      for (const key of getAllKanbanEnvKeys()) {
        delete process.env[key];
      }

      process.env['OPERATOR_JIRA_API_KEY'] = 'test-key';
      const result: KanbanEnvResult = checkKanbanEnvVars();

      assert.strictEqual(result.workspaces.length, 0);
      assert.strictEqual(result.anyConfigured, false);
    });

    test('returns linear workspace placeholder when OPERATOR_LINEAR_API_KEY is set', () => {
      // Clear all kanban env vars first
      for (const key of getAllKanbanEnvKeys()) {
        delete process.env[key];
      }

      process.env['OPERATOR_LINEAR_API_KEY'] = 'lin_api_test';
      const result: KanbanEnvResult = checkKanbanEnvVars();

      assert.strictEqual(result.workspaces.length, 1);
      assert.strictEqual(result.anyConfigured, true);
      assert.strictEqual(result.workspaces[0].provider, 'linear');
      assert.strictEqual(result.workspaces[0].name, 'Linear');
      assert.strictEqual(result.workspaces[0].url, 'https://linear.app');
      assert.strictEqual(result.workspaces[0].configured, true);
    });

    test('returns linear workspace placeholder when LINEAR_API_KEY is set', () => {
      // Clear all kanban env vars first
      for (const key of getAllKanbanEnvKeys()) {
        delete process.env[key];
      }

      process.env['LINEAR_API_KEY'] = 'lin_api_test';
      const result: KanbanEnvResult = checkKanbanEnvVars();

      assert.strictEqual(result.workspaces.length, 1);
      assert.strictEqual(result.anyConfigured, true);
      assert.strictEqual(result.workspaces[0].provider, 'linear');
    });

    test('returns both workspaces when both providers configured', () => {
      // Clear all kanban env vars first
      for (const key of getAllKanbanEnvKeys()) {
        delete process.env[key];
      }

      process.env['OPERATOR_JIRA_API_KEY'] = 'jira-key';
      process.env['OPERATOR_JIRA_DOMAIN'] = 'test.atlassian.net';
      process.env['OPERATOR_LINEAR_API_KEY'] = 'linear-key';
      const result: KanbanEnvResult = checkKanbanEnvVars();

      assert.strictEqual(result.workspaces.length, 2);
      assert.strictEqual(result.anyConfigured, true);
      assert.ok(result.workspaces.some((w) => w.provider === 'jira'));
      assert.ok(result.workspaces.some((w) => w.provider === 'linear'));
    });
  });

  suite('fetchLinearWorkspace()', () => {
    test('returns null for invalid API key', async () => {
      // Use an obviously invalid key that will fail authentication
      const result = await fetchLinearWorkspace('invalid-key');
      assert.strictEqual(result, null);
    });
  });

  suite('getKanbanWorkspaces()', () => {
    const originalEnv: Record<string, string | undefined> = {};

    setup(() => {
      for (const key of getAllKanbanEnvKeys()) {
        originalEnv[key] = process.env[key];
      }
    });

    teardown(() => {
      for (const key of getAllKanbanEnvKeys()) {
        if (originalEnv[key] !== undefined) {
          process.env[key] = originalEnv[key];
        } else {
          delete process.env[key];
        }
      }
    });

    test('returns empty array when no providers configured', async () => {
      for (const key of getAllKanbanEnvKeys()) {
        delete process.env[key];
      }

      const result = await getKanbanWorkspaces();
      assert.strictEqual(result.length, 0);
    });

    test('returns jira workspace with correct URL', async () => {
      for (const key of getAllKanbanEnvKeys()) {
        delete process.env[key];
      }

      process.env['OPERATOR_JIRA_API_KEY'] = 'test-key';
      process.env['OPERATOR_JIRA_DOMAIN'] = 'example.atlassian.net';

      const result = await getKanbanWorkspaces();

      assert.strictEqual(result.length, 1);
      assert.strictEqual(result[0].provider, 'jira');
      assert.strictEqual(result[0].url, 'https://example.atlassian.net');
    });
  });

  suite('checkLlmToolInPath()', () => {
    test('returns false for non-existent tool', async () => {
      const result = await checkLlmToolInPath(
        'definitely-not-a-real-tool-12345'
      );
      assert.strictEqual(result, false);
    });

    test('returns true for common system tool (node)', async () => {
      // node should be in PATH since we're running in Node.js
      const result = await checkLlmToolInPath('node');
      assert.strictEqual(result, true);
    });
  });

  suite('validateWorkingDirectory()', () => {
    test('returns false for non-existent path', async () => {
      const result = await validateWorkingDirectory(
        '/definitely/not/a/real/path/12345'
      );
      assert.strictEqual(result, false);
    });

    test('returns true for valid directory', async () => {
      const result = await validateWorkingDirectory(os.tmpdir());
      assert.strictEqual(result, true);
    });

    test('returns true for home directory', async () => {
      const result = await validateWorkingDirectory(os.homedir());
      assert.strictEqual(result, true);
    });

    test('returns false for file path', async () => {
      // Create a temp file
      const tempFile = path.join(os.tmpdir(), `walkthrough-test-${Date.now()}.txt`);
      await fs.writeFile(tempFile, 'test');

      try {
        const result = await validateWorkingDirectory(tempFile);
        assert.strictEqual(result, false);
      } finally {
        await fs.unlink(tempFile);
      }
    });
  });

  suite('detectInstalledLlmTools()', () => {
    test('returns an array', async () => {
      const result = await detectInstalledLlmTools();
      assert.ok(Array.isArray(result));
    });

    test('only returns known LLM tools with required fields', async () => {
      const result = await detectInstalledLlmTools();
      for (const tool of result) {
        assert.ok(
          (LLM_TOOLS as readonly string[]).includes(tool.name),
          `${tool.name} should be in LLM_TOOLS`
        );
        assert.ok(typeof tool.name === 'string', 'tool.name should be a string');
        assert.ok(typeof tool.path === 'string', 'tool.path should be a string');
        assert.ok(typeof tool.version === 'string', 'tool.version should be a string');
        assert.ok(typeof tool.version_ok === 'boolean', 'tool.version_ok should be a boolean');
      }
    });
  });

  suite('KANBAN_ENV_VARS constant', () => {
    test('has jira apiKey keys', () => {
      assert.ok(KANBAN_ENV_VARS.jira.apiKey.length > 0);
      assert.ok(KANBAN_ENV_VARS.jira.apiKey.includes('OPERATOR_JIRA_API_KEY'));
    });

    test('has jira domain keys', () => {
      assert.ok(KANBAN_ENV_VARS.jira.domain.length > 0);
      assert.ok(KANBAN_ENV_VARS.jira.domain.includes('OPERATOR_JIRA_DOMAIN'));
    });

    test('has linear apiKey keys', () => {
      assert.ok(KANBAN_ENV_VARS.linear.apiKey.length > 0);
      assert.ok(KANBAN_ENV_VARS.linear.apiKey.includes('OPERATOR_LINEAR_API_KEY'));
    });
  });

  suite('LLM_TOOLS constant', () => {
    test('includes claude', () => {
      assert.ok((LLM_TOOLS as readonly string[]).includes('claude'));
    });

    test('includes codex', () => {
      assert.ok((LLM_TOOLS as readonly string[]).includes('codex'));
    });

    test('includes gemini', () => {
      assert.ok((LLM_TOOLS as readonly string[]).includes('gemini'));
    });
  });
});
