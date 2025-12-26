import { describe, test, expect } from 'bun:test';

describe('plugin-issuetypes smoke tests', () => {
  test('plugin exports are defined', async () => {
    const plugin = await import('../index');
    expect(plugin).toBeDefined();
  });

  test('Chip component renders', async () => {
    const { Chip } = await import('../components/ui');
    expect(Chip).toBeDefined();
    expect(typeof Chip).toBe('function');
  });
});
