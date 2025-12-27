import { describe, test, expect } from 'bun:test';

describe('plugin-issuetypes smoke tests', () => {
  test('plugin exports are defined', async () => {
    const plugin = await import('../index');
    expect(plugin).toBeDefined();
    expect(plugin.issueTypesPlugin).toBeDefined();
    expect(plugin.operatorApiRef).toBeDefined();
  });

  test('Chip component renders', async () => {
    const { Chip } = await import('../components/ui');
    expect(Chip).toBeDefined();
    expect(typeof Chip).toBe('function');
  });

  test('API types are exported', async () => {
    const types = await import('../api/types');
    expect(types.STEP_OUTPUTS).toBeDefined();
    expect(types.ALLOWED_TOOLS).toBeDefined();
  });

  test('hooks are exported', async () => {
    const hooks = await import('../hooks');
    expect(hooks.useIssueTypes).toBeDefined();
    expect(hooks.useIssueType).toBeDefined();
    expect(hooks.useCreateIssueType).toBeDefined();
    expect(hooks.useCollections).toBeDefined();
    expect(hooks.useSteps).toBeDefined();
  });

  test('mock API works', async () => {
    const { createMockOperatorApi, mockIssueTypeSummaries } = await import(
      './test-utils'
    );
    const api = createMockOperatorApi();

    const issueTypes = await api.listIssueTypes();
    expect(issueTypes).toEqual(mockIssueTypeSummaries);
    expect(issueTypes.length).toBe(4);

    const feat = await api.getIssueType('FEAT');
    expect(feat.key).toBe('FEAT');
    expect(feat.mode).toBe('autonomous');

    const collections = await api.listCollections();
    expect(collections.length).toBe(2);
  });
});
