/**
 * Tests for config-panel.ts kanban provider handling.
 *
 * The canonical list of kanban providers lives in the Rust
 * `KanbanProviderType::ALL` catalog and is projected into the generated
 * `KanbanConfig` TypeScript type (keys: jira, linear, github, …). These tests
 * guard the invariant that the webview config write path supports EVERY
 * provider in that catalog — so a future provider can't be added to the schema
 * without also being wired into `config-panel.ts`.
 */

import * as assert from 'assert';
import * as path from 'path';
import { readFileSync } from 'fs';
import {
  KANBAN_PROVIDERS,
  KANBAN_PROVIDER_SLUGS,
  applyKanbanProviderField,
} from '../../src/config-panel';

// __dirname in compiled code is out/test/suite, so go up 3 levels to the
// extension root, then into the generated types.
const KANBAN_CONFIG_TYPE = path.join(
  __dirname,
  '..',
  '..',
  '..',
  'src',
  'generated',
  'KanbanConfig.ts'
);

/**
 * Extract the provider keys from the generated `KanbanConfig` type. Each
 * provider is rendered by ts-rs as a `slug: { [key in string]?: XConfig }`
 * field, so we scan for those top-level keys.
 */
function generatedProviderSlugs(): string[] {
  const source = readFileSync(KANBAN_CONFIG_TYPE, 'utf-8');
  const slugs: string[] = [];
  const re = /^(\w+):\s*\{\s*\[key in string\]/gm;
  let match: RegExpExecArray | null;
  while ((match = re.exec(source)) !== null) {
    slugs.push(match[1]!);
  }
  return slugs;
}

suite('Config Panel Kanban Providers', () => {
  // ---------------------------------------------------------------------
  // Future-provider tripwire: the canonical write table must cover every
  // provider in the generated schema. Add a provider to the Rust catalog,
  // regenerate types, and this fails until config-panel.ts handles it.
  // ---------------------------------------------------------------------
  test('every generated KanbanConfig provider has a write-path entry', () => {
    const generated = generatedProviderSlugs();
    assert.ok(
      generated.length >= 3,
      `Expected to parse provider slugs from KanbanConfig.ts, got [${generated.join(', ')}]`
    );

    for (const slug of generated) {
      assert.ok(
        Object.prototype.hasOwnProperty.call(KANBAN_PROVIDERS, slug),
        `Kanban provider "${slug}" exists in the generated config schema but ` +
          `is missing from KANBAN_PROVIDERS in config-panel.ts. Add an entry ` +
          `so the webview can read/write its config.`
      );
    }
  });

  test('KANBAN_PROVIDER_SLUGS matches the generated schema exactly', () => {
    const generated = generatedProviderSlugs().sort();
    const known = [...KANBAN_PROVIDER_SLUGS].sort();
    assert.deepStrictEqual(known, generated);
  });

  test('canonical catalog includes jira, linear, and github', () => {
    for (const slug of ['jira', 'linear', 'github']) {
      assert.ok(
        KANBAN_PROVIDER_SLUGS.includes(slug),
        `Expected "${slug}" in KANBAN_PROVIDER_SLUGS`
      );
    }
  });

  // ---------------------------------------------------------------------
  // Write-path behavior: every provider must round-trip scalar, instance-key,
  // and project-level field writes into the kanban sub-table.
  // ---------------------------------------------------------------------
  suite('applyKanbanProviderField round-trips for every provider', () => {
    for (const slug of KANBAN_PROVIDER_SLUGS) {
      const meta = KANBAN_PROVIDERS[slug]!;

      test(`${slug}: scalar field writes under the default instance key`, () => {
        const kanban: Record<string, unknown> = {};
        applyKanbanProviderField(kanban, slug, 'enabled', true);
        applyKanbanProviderField(kanban, slug, 'api_key_env', 'MY_TOKEN');

        const providerMap = kanban[slug] as Record<string, unknown>;
        assert.ok(providerMap, `Expected kanban.${slug} table to exist`);
        const instance = providerMap[meta.defaultInstanceKey] as Record<string, unknown>;
        assert.ok(instance, `Expected default instance "${meta.defaultInstanceKey}"`);
        assert.strictEqual(instance.enabled, true);
        assert.strictEqual(instance.api_key_env, 'MY_TOKEN');
      });

      test(`${slug}: instance-key field renames the provider map key`, () => {
        const kanban: Record<string, unknown> = {};
        applyKanbanProviderField(kanban, slug, 'enabled', true);
        applyKanbanProviderField(kanban, slug, meta.instanceKeyField, 'renamed-instance');

        const providerMap = kanban[slug] as Record<string, unknown>;
        assert.ok(
          Object.prototype.hasOwnProperty.call(providerMap, 'renamed-instance'),
          `Expected ${slug} instance key to be renamed to "renamed-instance"`
        );
        assert.ok(
          !Object.prototype.hasOwnProperty.call(providerMap, meta.defaultInstanceKey),
          `Expected old instance key "${meta.defaultInstanceKey}" to be gone`
        );
      });

      test(`${slug}: project-scoped field writes into a project sub-table`, () => {
        const kanban: Record<string, unknown> = {};
        applyKanbanProviderField(kanban, slug, 'projects.PROJ.sync_user_id', 'user-123');

        const providerMap = kanban[slug] as Record<string, unknown>;
        const instance = providerMap[meta.defaultInstanceKey] as Record<string, unknown>;
        const projects = instance.projects as Record<string, unknown>;
        const proj = projects.PROJ as Record<string, unknown>;
        assert.ok(proj, `Expected project "PROJ" sub-table for ${slug}`);
        assert.strictEqual(proj.sync_user_id, 'user-123');
      });
    }
  });

  test('unknown provider slug throws rather than silently dropping the write', () => {
    assert.throws(
      () => applyKanbanProviderField({}, 'notaprovider', 'enabled', true),
      /Unknown kanban provider/
    );
  });
});
