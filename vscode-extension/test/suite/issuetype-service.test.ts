/**
 * Tests for issuetype-service.ts
 *
 * Group 2: Service Logic - Requires fetch mock
 * Tests IssueTypeService class methods for icon/color lookup and type extraction.
 */

import * as assert from 'assert';
import * as vscode from 'vscode';
import * as sinon from 'sinon';
import * as fs from 'fs/promises';
import * as path from 'path';
import { IssueTypeService } from '../../src/issuetype-service';
import { IssueTypeSummary } from '../../src/generated';

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

suite('IssueType Service Test Suite', () => {
  let outputChannel: vscode.OutputChannel;
  let service: IssueTypeService;
  let fetchStub: sinon.SinonStub;

  setup(() => {
    // Create a mock output channel
    outputChannel = {
      name: 'test',
      append: sinon.stub(),
      appendLine: sinon.stub(),
      clear: sinon.stub(),
      show: sinon.stub(),
      hide: sinon.stub(),
      dispose: sinon.stub(),
      replace: sinon.stub(),
    } as unknown as vscode.OutputChannel;

    // Stub global fetch
    fetchStub = sinon.stub(global, 'fetch');
  });

  teardown(() => {
    sinon.restore();
  });

  suite('constructor and defaults', () => {
    test('initializes with default issue types', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      // Should have default types loaded
      assert.ok(service.isKnownType('FEAT'));
      assert.ok(service.isKnownType('FIX'));
      assert.ok(service.isKnownType('TASK'));
      assert.ok(service.isKnownType('SPIKE'));
      assert.ok(service.isKnownType('INV'));
    });

    test('uses provided baseUrl', () => {
      service = new IssueTypeService(outputChannel, 'http://custom:9000');

      // We can verify by checking that refresh would use the custom URL
      const customUrl = 'http://custom:9000';
      fetchStub.resolves(new Response(JSON.stringify([]), { status: 200 }));

      service.refresh();

      assert.ok(
        fetchStub.calledWith(`${customUrl}/api/v1/issuetypes`),
        'Should use custom URL'
      );
    });
  });

  suite('getType()', () => {
    test('returns type metadata for known type', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const feat = service.getType('FEAT');
      assert.ok(feat, 'Should return FEAT type');
      assert.strictEqual(feat.key, 'FEAT');
      assert.strictEqual(feat.name, 'Feature');
      assert.strictEqual(feat.glyph, '*');
    });

    test('returns type metadata case-insensitively', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const feat = service.getType('feat');
      assert.ok(feat, 'Should return FEAT type');
      assert.strictEqual(feat.key, 'FEAT');

      const fix = service.getType('Fix');
      assert.ok(fix, 'Should return FIX type');
      assert.strictEqual(fix.key, 'FIX');
    });

    test('returns undefined for unknown type', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const unknown = service.getType('UNKNOWN');
      assert.strictEqual(unknown, undefined);
    });
  });

  suite('getKnownKeys()', () => {
    test('returns all known type keys', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const keys = service.getKnownKeys();
      assert.ok(keys.includes('FEAT'));
      assert.ok(keys.includes('FIX'));
      assert.ok(keys.includes('TASK'));
      assert.ok(keys.includes('SPIKE'));
      assert.ok(keys.includes('INV'));
    });
  });

  suite('isKnownType()', () => {
    test('returns true for known types', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      assert.strictEqual(service.isKnownType('FEAT'), true);
      assert.strictEqual(service.isKnownType('feat'), true);
      assert.strictEqual(service.isKnownType('FIX'), true);
    });

    test('returns false for unknown types', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      assert.strictEqual(service.isKnownType('UNKNOWN'), false);
      assert.strictEqual(service.isKnownType(''), false);
    });
  });

  suite('getIcon()', () => {
    test('returns ThemeIcon with correct icon name', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const featIcon = service.getIcon('FEAT');
      assert.ok(featIcon instanceof vscode.ThemeIcon);
      assert.strictEqual(featIcon.id, 'sparkle'); // * -> sparkle

      const fixIcon = service.getIcon('FIX');
      assert.strictEqual(fixIcon.id, 'wrench'); // # -> wrench

      const spikeIcon = service.getIcon('SPIKE');
      assert.strictEqual(spikeIcon.id, 'beaker'); // ? -> beaker

      const invIcon = service.getIcon('INV');
      assert.strictEqual(invIcon.id, 'search'); // ! -> search
    });

    test('returns beaker icon for unknown type', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const unknownIcon = service.getIcon('UNKNOWN');
      assert.ok(unknownIcon instanceof vscode.ThemeIcon);
      // Unknown type defaults to '?' glyph which maps to 'beaker'
      assert.strictEqual(unknownIcon.id, 'beaker');
    });
  });

  suite('getIconName()', () => {
    test('returns icon name string for known type', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      assert.strictEqual(service.getIconName('FEAT'), 'sparkle');
      assert.strictEqual(service.getIconName('FIX'), 'wrench');
      assert.strictEqual(service.getIconName('TASK'), 'tasklist');
      assert.strictEqual(service.getIconName('SPIKE'), 'beaker');
      assert.strictEqual(service.getIconName('INV'), 'search');
    });

    test('returns beaker icon name for unknown type', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      // Unknown type defaults to '?' glyph which maps to 'beaker'
      assert.strictEqual(service.getIconName('UNKNOWN'), 'beaker');
    });
  });

  suite('getColor()', () => {
    test('returns ThemeColor for known type', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const featColor = service.getColor('FEAT');
      assert.ok(featColor instanceof vscode.ThemeColor);
      // FEAT has cyan color -> terminal.ansiCyan

      const fixColor = service.getColor('FIX');
      assert.ok(fixColor instanceof vscode.ThemeColor);
      // FIX has red color -> terminal.ansiRed
    });

    test('returns undefined for unknown type', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const color = service.getColor('UNKNOWN');
      assert.strictEqual(color, undefined);
    });
  });

  suite('extractTypeFromId()', () => {
    test('extracts type from standard ticket ID', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      assert.strictEqual(service.extractTypeFromId('FEAT-123'), 'FEAT');
      assert.strictEqual(service.extractTypeFromId('FIX-456'), 'FIX');
      assert.strictEqual(service.extractTypeFromId('TASK-789'), 'TASK');
      assert.strictEqual(service.extractTypeFromId('SPIKE-001'), 'SPIKE');
      assert.strictEqual(service.extractTypeFromId('INV-999'), 'INV');
    });

    test('handles lowercase ticket IDs', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      assert.strictEqual(service.extractTypeFromId('feat-123'), 'FEAT');
      assert.strictEqual(service.extractTypeFromId('fix-456'), 'FIX');
    });

    test('handles mixed case ticket IDs', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      assert.strictEqual(service.extractTypeFromId('Feat-123'), 'FEAT');
      assert.strictEqual(service.extractTypeFromId('FiX-456'), 'FIX');
    });

    test('handles custom type prefixes', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      assert.strictEqual(service.extractTypeFromId('CUSTOM-123'), 'CUSTOM');
      assert.strictEqual(service.extractTypeFromId('BUG-001'), 'BUG');
      assert.strictEqual(service.extractTypeFromId('EPIC-999'), 'EPIC');
    });

    test('returns TASK for invalid ticket ID formats', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      assert.strictEqual(service.extractTypeFromId('invalid'), 'TASK');
      assert.strictEqual(service.extractTypeFromId('123-FEAT'), 'TASK');
      assert.strictEqual(service.extractTypeFromId(''), 'TASK');
      // Note: 'no-number' extracts 'NO' because 'no' becomes 'NO' (uppercase)
      // and passes the [A-Z]+ check
      assert.strictEqual(service.extractTypeFromId('no-number'), 'NO');
    });

    test('handles ticket IDs with extra segments', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      assert.strictEqual(
        service.extractTypeFromId('FEAT-123-title-here'),
        'FEAT'
      );
      assert.strictEqual(
        service.extractTypeFromId('FIX-456-bug-description'),
        'FIX'
      );
    });
  });

  suite('parseTicketFilename()', () => {
    test('parses standard ticket filename', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const result1 = service.parseTicketFilename('FEAT-123.md');
      assert.strictEqual(result1.id, 'FEAT-123');
      assert.strictEqual(result1.type, 'FEAT');

      const result2 = service.parseTicketFilename('FIX-456.md');
      assert.strictEqual(result2.id, 'FIX-456');
      assert.strictEqual(result2.type, 'FIX');
    });

    test('parses filename with title suffix', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const result = service.parseTicketFilename('FEAT-123-add-dark-mode.md');
      assert.strictEqual(result.id, 'FEAT-123');
      assert.strictEqual(result.type, 'FEAT');
    });

    test('handles lowercase filenames', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const result = service.parseTicketFilename('feat-123.md');
      assert.strictEqual(result.id, 'FEAT-123');
      assert.strictEqual(result.type, 'FEAT');
    });

    test('handles non-standard filenames', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const result = service.parseTicketFilename('random.md');
      assert.strictEqual(result.id, 'random');
      assert.strictEqual(result.type, 'TASK');
    });

    test('handles filename without .md extension', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const result = service.parseTicketFilename('FEAT-123');
      assert.strictEqual(result.id, 'FEAT-123');
      assert.strictEqual(result.type, 'FEAT');
    });

    test('handles edge cases', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const result1 = service.parseTicketFilename('.md');
      assert.strictEqual(result1.id, '');
      assert.strictEqual(result1.type, 'TASK');

      const result2 = service.parseTicketFilename('');
      assert.strictEqual(result2.id, '');
      assert.strictEqual(result2.type, 'TASK');
    });
  });

  suite('getIconForTerminal()', () => {
    test('extracts type from terminal name and returns icon', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const featIcon = service.getIconForTerminal('op-FEAT-123');
      assert.strictEqual(featIcon.id, 'sparkle');

      const fixIcon = service.getIconForTerminal('op-FIX-456');
      assert.strictEqual(fixIcon.id, 'wrench');
    });

    test('returns terminal icon for non-matching names', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const icon = service.getIconForTerminal('some-other-terminal');
      assert.strictEqual(icon.id, 'terminal');
    });

    test('handles case-insensitive terminal names', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const icon = service.getIconForTerminal('op-feat-123');
      assert.strictEqual(icon.id, 'sparkle');
    });
  });

  suite('getColorForTerminal()', () => {
    test('extracts type from terminal name and returns color', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const featColor = service.getColorForTerminal('op-FEAT-123');
      assert.ok(featColor instanceof vscode.ThemeColor);

      const fixColor = service.getColorForTerminal('op-FIX-456');
      assert.ok(fixColor instanceof vscode.ThemeColor);
    });

    test('returns white for non-matching names', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const color = service.getColorForTerminal('some-other-terminal');
      assert.ok(color instanceof vscode.ThemeColor);
      // Should be terminal.ansiWhite (default)
    });
  });

  suite('setBaseUrl()', () => {
    test('updates the base URL', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      service.setBaseUrl('http://newurl:9000');

      // Verify by checking fetch calls
      fetchStub.resolves(new Response(JSON.stringify([]), { status: 200 }));
      service.refresh();

      assert.ok(
        fetchStub.calledWith('http://newurl:9000/api/v1/issuetypes'),
        'Should use new URL'
      );
    });
  });

  suite('refresh()', () => {
    test('fetches issue types from API and updates types', async () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const apiResponse: IssueTypeSummary[] = await fs
        .readFile(path.join(fixturesDir, 'issuetypes-response.json'), 'utf-8')
        .then(JSON.parse);

      fetchStub.resolves(
        new Response(JSON.stringify(apiResponse), { status: 200 })
      );

      await service.refresh();

      // Should now have CUSTOM type from API
      assert.ok(service.isKnownType('CUSTOM'));
      const custom = service.getType('CUSTOM');
      assert.ok(custom, 'Should have CUSTOM type');
      assert.strictEqual(custom.glyph, '+');
      assert.strictEqual(custom.color, 'blue');
      assert.strictEqual(custom.source, 'api');
    });

    test('keeps defaults when API is unavailable', async () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      fetchStub.rejects(new Error('Network error'));

      await service.refresh();

      // Should still have default types
      assert.ok(service.isKnownType('FEAT'));
      assert.ok(service.isKnownType('FIX'));

      // Output channel should log the error
      assert.ok(
        (outputChannel.appendLine as sinon.SinonStub).calledWith(
          sinon.match(/API unavailable/)
        )
      );
    });

    test('keeps defaults when API returns error status', async () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      fetchStub.resolves(
        new Response('Internal Server Error', { status: 500 })
      );

      await service.refresh();

      // Should still have default types
      assert.ok(service.isKnownType('FEAT'));
      assert.ok(service.isKnownType('FIX'));

      // Output channel should log the failure
      assert.ok(
        (outputChannel.appendLine as sinon.SinonStub).calledWith(
          sinon.match(/Failed to fetch issue types: 500/)
        )
      );
    });

    test('clears existing types before loading from API', async () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      // Custom API response with only one type
      const apiResponse: IssueTypeSummary[] = [
        {
          key: 'ONLY',
          name: 'Only Type',
          description: 'The only type',
          mode: 'autonomous',
          glyph: '~',
          color: 'white',
          source: 'api',
          stepCount: 1,
        },
      ];

      fetchStub.resolves(
        new Response(JSON.stringify(apiResponse), { status: 200 })
      );

      await service.refresh();

      // Should only have the API type
      assert.ok(service.isKnownType('ONLY'));
      assert.strictEqual(service.isKnownType('FEAT'), false);
      assert.strictEqual(service.isKnownType('FIX'), false);
    });

    test('logs success message with count', async () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      const apiResponse: IssueTypeSummary[] = [
        {
          key: 'TYPE1',
          name: 'Type 1',
          description: 'First type',
          mode: 'autonomous',
          glyph: '*',
          source: 'api',
          stepCount: 1,
        },
        {
          key: 'TYPE2',
          name: 'Type 2',
          description: 'Second type',
          mode: 'paired',
          glyph: '#',
          source: 'api',
          stepCount: 2,
        },
      ];

      fetchStub.resolves(
        new Response(JSON.stringify(apiResponse), { status: 200 })
      );

      await service.refresh();

      assert.ok(
        (outputChannel.appendLine as sinon.SinonStub).calledWith(
          sinon.match(/Loaded 2 issue types from API/)
        )
      );
    });
  });

  suite('glyph to icon mapping', () => {
    test('maps all default glyphs to icons', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      // Test each default glyph
      const glyphMap: Record<string, string> = {
        FEAT: 'sparkle', // *
        FIX: 'wrench', // #
        TASK: 'tasklist', // >
        SPIKE: 'beaker', // ?
        INV: 'search', // !
      };

      for (const [type, expectedIcon] of Object.entries(glyphMap)) {
        const icon = service.getIcon(type);
        assert.strictEqual(
          icon.id,
          expectedIcon,
          `${type} should map to ${expectedIcon}`
        );
      }
    });
  });

  suite('color to theme mapping', () => {
    test('maps colors to theme color IDs', () => {
      service = new IssueTypeService(outputChannel, 'http://localhost:7008');

      // All default types have colors
      const featColor = service.getColor('FEAT');
      const fixColor = service.getColor('FIX');
      const taskColor = service.getColor('TASK');
      const spikeColor = service.getColor('SPIKE');
      const invColor = service.getColor('INV');

      assert.ok(featColor, 'FEAT should have color');
      assert.ok(fixColor, 'FIX should have color');
      assert.ok(taskColor, 'TASK should have color');
      assert.ok(spikeColor, 'SPIKE should have color');
      assert.ok(invColor, 'INV should have color');
    });
  });
});
