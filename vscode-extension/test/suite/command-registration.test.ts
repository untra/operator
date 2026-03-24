import * as assert from 'assert';
import * as vscode from 'vscode';
import * as fs from 'fs';
import * as path from 'path';

/**
 * Tests that verify command registration works correctly in the extension.
 *
 * These tests enforce:
 * 1. Every command in package.json is actually registered at runtime
 * 2. Every registered operator.* command has a package.json entry
 * 3. Activation events include onView and onCommand triggers (not just onStartupFinished)
 * 4. Commands are available immediately after activation
 */
suite('Command Registration Tests', () => {
  let extension: vscode.Extension<unknown> | undefined;
  let packageJson: {
    activationEvents?: string[];
    contributes?: {
      commands?: Array<{ command: string }>;
      views?: {
        'operator-sidebar'?: Array<{ id: string }>;
      };
    };
  };

  suiteSetup(async () => {
    extension = vscode.extensions.getExtension('untra.operator-terminals');
    assert.ok(extension, 'Extension must be present');
    packageJson = extension.packageJSON as typeof packageJson;
    if (!extension.isActive) {
      await extension.activate();
    }
  });

  // -----------------------------------------------------------------------
  // Manifest parity: every contributed command must be registered at runtime
  // -----------------------------------------------------------------------

  test('All package.json commands are registered at runtime', async () => {
    const manifestCommands = (packageJson.contributes?.commands ?? []).map(c => c.command);
    assert.ok(manifestCommands.length > 0, 'package.json should contribute at least one command');

    const registeredCommands = await vscode.commands.getCommands(true);

    const missing: string[] = [];
    for (const cmd of manifestCommands) {
      if (!registeredCommands.includes(cmd)) {
        missing.push(cmd);
      }
    }

    assert.strictEqual(
      missing.length,
      0,
      `Commands declared in package.json but NOT registered at runtime:\n  ${missing.join('\n  ')}`
    );
  });

  // -----------------------------------------------------------------------
  // Reverse parity: every registered operator.* command should be in manifest
  // -----------------------------------------------------------------------

  test('All registered operator.* commands are declared in package.json', async () => {
    const manifestCommands = new Set(
      (packageJson.contributes?.commands ?? []).map(c => c.command)
    );

    const registeredCommands = await vscode.commands.getCommands(true);
    const operatorCommands = registeredCommands.filter(c => c.startsWith('operator.'));

    const undeclared: string[] = [];
    for (const cmd of operatorCommands) {
      if (!manifestCommands.has(cmd)) {
        undeclared.push(cmd);
      }
    }

    assert.strictEqual(
      undeclared.length,
      0,
      `Commands registered at runtime but NOT in package.json:\n  ${undeclared.join('\n  ')}`
    );
  });

  // -----------------------------------------------------------------------
  // Activation events: extension must activate on view open AND commands
  // -----------------------------------------------------------------------

  test('activationEvents includes onView triggers for sidebar views', () => {
    const activationEvents = packageJson.activationEvents ?? [];
    const viewIds = (packageJson.contributes?.views?.['operator-sidebar'] ?? []).map(v => v.id);

    assert.ok(viewIds.length > 0, 'Should have sidebar views defined');

    const missingViews: string[] = [];
    for (const viewId of viewIds) {
      if (!activationEvents.includes(`onView:${viewId}`)) {
        missingViews.push(viewId);
      }
    }

    assert.strictEqual(
      missingViews.length,
      0,
      `activationEvents missing onView triggers for:\n  ${missingViews.join('\n  ')}`
    );
  });

  test('activationEvents includes onCommand triggers for key commands', () => {
    const activationEvents = packageJson.activationEvents ?? [];

    // These are commands users invoke from command palette or keybindings —
    // the extension MUST activate when they fire.
    const criticalCommands = [
      'operator.showStatus',
      'operator.startOperatorServer',
      'operator.launchTicket',
      'operator.openSettings',
      'operator.openWalkthrough',
      'operator.selectWorkingDirectory',
    ];

    const missing: string[] = [];
    for (const cmd of criticalCommands) {
      if (!activationEvents.includes(`onCommand:${cmd}`)) {
        missing.push(cmd);
      }
    }

    assert.strictEqual(
      missing.length,
      0,
      `activationEvents missing onCommand triggers for:\n  ${missing.join('\n  ')}`
    );
  });

  // -----------------------------------------------------------------------
  // Key commands must be available immediately after activation
  // -----------------------------------------------------------------------

  test('Critical commands are available after activation', async () => {
    const commands = await vscode.commands.getCommands(true);

    const critical = [
      'operator.showStatus',
      'operator.startOperatorServer',
      'operator.launchTicket',
      'operator.startWebhookServer',
      'operator.refreshTickets',
      'operator.openSettings',
      'operator.selectWorkingDirectory',
      'operator.detectLlmTools',
    ];

    const missing: string[] = [];
    for (const cmd of critical) {
      if (!commands.includes(cmd)) {
        missing.push(cmd);
      }
    }

    assert.strictEqual(
      missing.length,
      0,
      `Critical commands not registered:\n  ${missing.join('\n  ')}`
    );
  });
});
