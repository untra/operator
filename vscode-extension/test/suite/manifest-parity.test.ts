import * as assert from 'assert';
import * as vscode from 'vscode';

/**
 * Tests that validate the extension manifest (package.json) is consistent
 * with runtime behavior and VS Code API requirements.
 */
suite('Manifest Parity Tests', () => {
  let extension: vscode.Extension<unknown> | undefined;
  let packageJson: {
    engines?: { vscode?: string };
    activationEvents?: string[];
    contributes?: {
      commands?: Array<{ command: string }>;
      views?: {
        'operator-sidebar'?: Array<{ id: string }>;
      };
    };
  };

  suiteSetup(() => {
    extension = vscode.extensions.getExtension('untra.operator-terminals');
    assert.ok(extension, 'Extension must be present');
    packageJson = extension.packageJSON as typeof packageJson;
  });

  // -----------------------------------------------------------------------
  // engines.vscode must be >= 1.93 for terminal shell execution APIs
  // -----------------------------------------------------------------------

  test('engines.vscode floor is at least 1.93 for shell execution APIs', () => {
    const enginesVscode = packageJson.engines?.vscode;
    assert.ok(enginesVscode, 'engines.vscode must be defined');

    // Extract the minimum version number from the semver range (e.g. "^1.93.0" -> "1.93.0")
    const match = enginesVscode.match(/(\d+)\.(\d+)/);
    assert.ok(match, `Could not parse version from engines.vscode: ${enginesVscode}`);

    const major = parseInt(match[1]!, 10);
    const minor = parseInt(match[2]!, 10);

    // onDidStartTerminalShellExecution was added in 1.93
    const meetsMinimum = major > 1 || (major === 1 && minor >= 93);
    assert.ok(
      meetsMinimum,
      `engines.vscode "${enginesVscode}" is below 1.93 — TerminalManager uses ` +
      `onDidStartTerminalShellExecution/onDidEndTerminalShellExecution which require VS Code 1.93+`
    );
  });

  // -----------------------------------------------------------------------
  // activationEvents must not be empty/too narrow
  // -----------------------------------------------------------------------

  test('activationEvents should include more than just onStartupFinished', () => {
    const events = packageJson.activationEvents ?? [];

    // onStartupFinished alone is too narrow — commands and views should also trigger activation
    const hasViewOrCommandTrigger = events.some(
      e => e.startsWith('onView:') || e.startsWith('onCommand:')
    );

    assert.ok(
      hasViewOrCommandTrigger,
      `activationEvents only contains [${events.join(', ')}] — ` +
      'should include onView: or onCommand: triggers for reliable activation'
    );
  });

  // -----------------------------------------------------------------------
  // Command count sanity
  // -----------------------------------------------------------------------

  test('Extension contributes a reasonable number of commands', () => {
    const commands = packageJson.contributes?.commands ?? [];
    assert.ok(
      commands.length >= 10,
      `Expected at least 10 contributed commands, got ${commands.length}`
    );
  });
});
