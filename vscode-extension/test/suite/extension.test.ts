import * as assert from 'assert';
import * as vscode from 'vscode';

suite('Extension Test Suite', () => {
  vscode.window.showInformationMessage('Start all tests.');

  test('Extension should be present', () => {
    assert.ok(vscode.extensions.getExtension('untra.operator-terminals'));
  });

  test('Commands should be registered', async () => {
    // Ensure extension is activated before checking commands
    const extension = vscode.extensions.getExtension('untra.operator-terminals');
    assert.ok(extension, 'Extension should be present');

    if (!extension.isActive) {
      await extension.activate();
    }

    const commands = await vscode.commands.getCommands(true);
    assert.ok(commands.includes('operator.startOperatorServer'));
    assert.ok(commands.includes('operator.showStatus'));
  });
});
