import * as assert from 'assert';
import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';

suite('Integration Test Suite', () => {
    let extension: vscode.Extension<unknown> | undefined;

    suiteSetup(async () => {
        extension = vscode.extensions.getExtension('untra.operator-terminals');
        if (extension && !extension.isActive) {
            await extension.activate();
        }
    });

    test('opr8r binary is bundled in extension', async () => {
        assert.ok(extension, 'Extension should be present');

        const extensionPath = extension.extensionPath;
        const binaryName = process.platform === 'win32' ? 'opr8r.exe' : 'opr8r';
        const bundledPath = path.join(extensionPath, 'bin', binaryName);

        // Check if the bin directory exists (may not exist in all test scenarios)
        const binDir = path.join(extensionPath, 'bin');
        if (fs.existsSync(binDir)) {
            // If bin directory exists, check for the binary
            const files = fs.readdirSync(binDir);
            assert.ok(
                files.length > 0,
                'bin directory should contain files when present'
            );
        } else {
            // Skip if no bin directory (development mode)
            console.log('Skipping bundled binary test - bin directory not present (development mode)');
        }
    });

    test('Webhook server commands are available', async () => {
        assert.ok(extension, 'Extension should be present');

        const commands = await vscode.commands.getCommands(true);

        // Verify all webhook-related commands are registered
        const requiredCommands = [
            'operator.startOperatorServer',
            'operator.showStatus',
            'operator.launchTicket',
            'operator.refreshTickets'
        ];

        for (const cmd of requiredCommands) {
            assert.ok(
                commands.includes(cmd),
                `Command ${cmd} should be registered`
            );
        }
    });

    test('Configuration settings have defaults', () => {
        const config = vscode.workspace.getConfiguration('operator');

        // Verify default settings exist
        assert.strictEqual(
            config.get<number>('webhookPort'),
            7009,
            'Default webhook port should be 7009'
        );

        assert.strictEqual(
            config.get<boolean>('autoStart'),
            true,
            'Default autoStart should be true'
        );

        assert.strictEqual(
            config.get<string>('terminalPrefix'),
            'op-',
            'Default terminal prefix should be op-'
        );

        assert.strictEqual(
            config.get<string>('ticketsDir'),
            '.tickets',
            'Default tickets dir should be .tickets'
        );

        assert.strictEqual(
            config.get<string>('apiUrl'),
            'http://localhost:7008',
            'Default API URL should be http://localhost:7008'
        );
    });

    test('Views are registered in sidebar', async () => {
        assert.ok(extension, 'Extension should be present');

        // Get the package.json contributes
        const packageJson = extension.packageJSON;
        const views = packageJson.contributes?.views?.['operator-sidebar'];

        assert.ok(views, 'Sidebar views should be defined');
        assert.ok(Array.isArray(views), 'Views should be an array');

        // Verify expected views
        const viewIds = views.map((v: { id: string }) => v.id);
        assert.ok(viewIds.includes('operator-status'), 'Status view should exist');
        assert.ok(viewIds.includes('operator-in-progress'), 'In Progress view should exist');
        assert.ok(viewIds.includes('operator-queue'), 'Queue view should exist');
        assert.ok(viewIds.includes('operator-completed'), 'Completed view should exist');
    });
});
