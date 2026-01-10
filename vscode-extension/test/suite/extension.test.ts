import * as assert from 'assert';
import * as vscode from 'vscode';

suite('Extension Test Suite', () => {
  vscode.window.showInformationMessage('Start all tests.');

  test('Extension should be present', () => {
    assert.ok(vscode.extensions.getExtension('untra.operator-terminals'));
  });

  test('Commands should be registered', async () => {
    const commands = await vscode.commands.getCommands(true);
    assert.ok(commands.includes('operator.startServer'));
    assert.ok(commands.includes('operator.stopServer'));
    assert.ok(commands.includes('operator.showStatus'));
  });
});
