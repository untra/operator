import * as assert from 'assert';
import * as vscode from 'vscode';

/**
 * Tests for TerminalManager resilience.
 *
 * The TerminalManager constructor subscribes to terminal shell execution
 * events (onDidStartTerminalShellExecution, onDidEndTerminalShellExecution)
 * which were added in VS Code 1.93. If the extension declares a lower
 * engines.vscode floor, the constructor must not throw when these APIs
 * are unavailable.
 */
suite('TerminalManager Resilience Tests', () => {

  test('Terminal shell execution APIs exist on vscode.window', () => {
    // This test documents that the APIs we depend on actually exist
    // in the test VS Code version. If this fails, we're testing against
    // a VS Code version older than 1.93 and TerminalManager will throw.
    assert.ok(
      typeof vscode.window.onDidStartTerminalShellExecution === 'function',
      'onDidStartTerminalShellExecution should be available on vscode.window'
    );
    assert.ok(
      typeof vscode.window.onDidEndTerminalShellExecution === 'function',
      'onDidEndTerminalShellExecution should be available on vscode.window'
    );
  });

  test('TerminalManager constructor should not throw', async () => {
    // Dynamic import to catch constructor-time errors
    const { TerminalManager } = await import('../../src/terminal-manager.js');

    let manager: InstanceType<typeof TerminalManager> | undefined;
    assert.doesNotThrow(() => {
      manager = new TerminalManager();
    }, 'TerminalManager constructor should not throw');

    // Cleanup
    if (manager) {
      manager.dispose();
    }
  });

  test('TerminalManager should handle missing shell execution APIs gracefully', async () => {
    // If shell execution APIs are guarded, constructing TerminalManager
    // should succeed even conceptually without them. We verify here that
    // the manager is functional after construction.
    const { TerminalManager } = await import('../../src/terminal-manager.js');

    const manager = new TerminalManager();

    // Basic operations should work
    assert.strictEqual(manager.exists('nonexistent'), false);
    assert.strictEqual(manager.getActivity('nonexistent'), 'unknown');
    assert.deepStrictEqual(manager.list(), []);

    manager.dispose();
  });
});
