/**
 * Tests for config-panel.ts kanban provider handling.
 *
 * The canonical list of kanban providers lives in the Rust
 * `KanbanProviderType::ALL` catalog and is projected into the generated
 * `KanbanConfig` TypeScript type (keys: jira, linear, github, …). These tests
 * guard the invariant that the webview config write path supports EVERY
 * provider in that catalog - so a future provider can't be added to the schema
 * without also being wired into `config-panel.ts`.
 */

import * as assert from "node:assert";
import * as path from "node:path";
import { readFileSync } from "node:fs";
import {
  KANBAN_PROVIDERS,
  KANBAN_PROVIDER_SLUGS,
  applyKanbanProviderField,
} from "../../src/config-panel";

// __dirname in compiled code is out/test/suite, so go up 3 levels to the
// extension root, then into the generated types.
const KANBAN_CONFIG_TYPE = path.join(
  __dirname,
  "..",
  "..",
  "..",
  "src",
  "generated",
  "KanbanConfig.ts",
);

/**
 * Extract the provider keys from the generated `KanbanConfig` type. Each
 * provider is rendered by ts-rs as a `slug: { [key in string]?: XConfig }`
 * field, so we scan for those top-level keys.
 */
function generatedProviderSlugs(): string[] {
  const source = readFileSync(KANBAN_CONFIG_TYPE, "utf-8");
  const slugs: string[] = [];
  const re = /^(\w+):\s*\{\s*\[key in string\]/gm;
  let match: RegExpExecArray | null;
  while ((match = re.exec(source)) !== null) {
    slugs.push(match[1]!);
  }
  return slugs;
}

suite("Config Panel Kanban Providers", () => {
  // ---------------------------------------------------------------------
  // Future-provider tripwire: the canonical write table must cover every
  // provider in the generated schema. Add a provider to the Rust catalog,
  // regenerate types, and this fails until config-panel.ts handles it.
  // ---------------------------------------------------------------------
  test("every generated KanbanConfig provider has a write-path entry", () => {
    const generated = generatedProviderSlugs();
    assert.ok(
      generated.length >= 3,
      `Expected to parse provider slugs from KanbanConfig.ts, got [${generated.join(", ")}]`,
    );

    for (const slug of generated) {
      assert.ok(
        Object.prototype.hasOwnProperty.call(KANBAN_PROVIDERS, slug),
        `Kanban provider "${slug}" exists in the generated config schema but ` +
          `is missing from KANBAN_PROVIDERS in config-panel.ts. Add an entry ` +
          `so the webview can read/write its config.`,
      );
    }
  });

  test("KANBAN_PROVIDER_SLUGS matches the generated schema exactly", () => {
    const generated = generatedProviderSlugs().toSorted();
    const known = [...KANBAN_PROVIDER_SLUGS].toSorted();
    assert.deepStrictEqual(known, generated);
  });

  test("canonical catalog includes jira, linear, and github", () => {
    for (const slug of ["jira", "linear", "github"]) {
      assert.ok(
        KANBAN_PROVIDER_SLUGS.includes(slug),
        `Expected "${slug}" in KANBAN_PROVIDER_SLUGS`,
      );
    }
  });

  // ---------------------------------------------------------------------
  // Write-path behavior: every provider must round-trip scalar, instance-key,
  // and project-level field writes into the kanban sub-table.
  // ---------------------------------------------------------------------
  suite("applyKanbanProviderField round-trips for every provider", () => {
    for (const slug of KANBAN_PROVIDER_SLUGS) {
      const meta = KANBAN_PROVIDERS[slug]!;

      test(`${slug}: scalar field writes under the default instance key`, () => {
        const kanban: Record<string, unknown> = {};
        applyKanbanProviderField(kanban, slug, meta.defaultInstanceKey, "enabled", true);
        applyKanbanProviderField(kanban, slug, meta.defaultInstanceKey, "api_key_env", "MY_TOKEN");

        const providerMap = kanban[slug] as Record<string, unknown>;
        assert.ok(providerMap, `Expected kanban.${slug} table to exist`);
        const instance = providerMap[meta.defaultInstanceKey] as Record<string, unknown>;
        assert.ok(instance, `Expected default instance "${meta.defaultInstanceKey}"`);
        assert.strictEqual(instance.enabled, true);
        assert.strictEqual(instance.api_key_env, "MY_TOKEN");
      });

      test(`${slug}: instance-key field renames the provider map key`, () => {
        const kanban: Record<string, unknown> = {};
        applyKanbanProviderField(kanban, slug, meta.defaultInstanceKey, "enabled", true);
        applyKanbanProviderField(
          kanban,
          slug,
          meta.defaultInstanceKey,
          meta.instanceKeyField,
          "renamed-instance",
        );

        const providerMap = kanban[slug] as Record<string, unknown>;
        assert.ok(
          Object.prototype.hasOwnProperty.call(providerMap, "renamed-instance"),
          `Expected ${slug} instance key to be renamed to "renamed-instance"`,
        );
        assert.ok(
          !Object.prototype.hasOwnProperty.call(providerMap, meta.defaultInstanceKey),
          `Expected old instance key "${meta.defaultInstanceKey}" to be gone`,
        );
      });

      test(`${slug}: project-scoped field writes into a project sub-table`, () => {
        const kanban: Record<string, unknown> = {};
        applyKanbanProviderField(
          kanban,
          slug,
          meta.defaultInstanceKey,
          "projects.PROJ.sync_user_id",
          "user-123",
        );

        const providerMap = kanban[slug] as Record<string, unknown>;
        const instance = providerMap[meta.defaultInstanceKey] as Record<string, unknown>;
        const projects = instance.projects as Record<string, unknown>;
        const proj = projects.PROJ as Record<string, unknown>;
        assert.ok(proj, `Expected project "PROJ" sub-table for ${slug}`);
        assert.strictEqual(proj.sync_user_id, "user-123");
      });

      test(`${slug}: status_mapping object round-trips via projects path and shorthand`, () => {
        const mapping = { todo: "To Do", doing: "In Progress", done: "Done" };

        // Explicit projects.<key>.status_mapping path (ProjectRow write path)
        const kanban: Record<string, unknown> = {};
        applyKanbanProviderField(
          kanban,
          slug,
          meta.defaultInstanceKey,
          "projects.PROJ.status_mapping",
          mapping,
        );
        let providerMap = kanban[slug] as Record<string, unknown>;
        let instance = providerMap[meta.defaultInstanceKey] as Record<string, unknown>;
        let proj = (instance.projects as Record<string, unknown>).PROJ as Record<string, unknown>;
        assert.deepStrictEqual(proj.status_mapping, mapping);

        // Shorthand project-level key (single-project config forms)
        const kanban2: Record<string, unknown> = {};
        applyKanbanProviderField(kanban2, slug, meta.defaultInstanceKey, "status_mapping", mapping);
        providerMap = kanban2[slug] as Record<string, unknown>;
        instance = providerMap[meta.defaultInstanceKey] as Record<string, unknown>;
        const projects = instance.projects as Record<string, unknown>;
        const firstKey = Object.keys(projects)[0]!;
        proj = projects[firstKey] as Record<string, unknown>;
        assert.deepStrictEqual(proj.status_mapping, mapping);
      });

      test(`${slug}: updates and renames only the selected instance`, () => {
        const secondInstanceKey = "second-instance";
        const renamedInstanceKey = "renamed-second-instance";
        const firstInstance = { enabled: true, projects: {} };
        const secondInstance = { enabled: false, projects: {} };
        const kanban: Record<string, unknown> = {
          [slug]: {
            [meta.defaultInstanceKey]: firstInstance,
            [secondInstanceKey]: secondInstance,
          },
        };

        applyKanbanProviderField(kanban, slug, secondInstanceKey, "enabled", true);
        applyKanbanProviderField(
          kanban,
          slug,
          secondInstanceKey,
          "projects.PROJ.sync_user_id",
          "user-456",
        );
        applyKanbanProviderField(
          kanban,
          slug,
          secondInstanceKey,
          meta.instanceKeyField,
          renamedInstanceKey,
        );

        const providerMap = kanban[slug] as Record<string, unknown>;
        assert.deepStrictEqual(providerMap[meta.defaultInstanceKey], firstInstance);
        assert.ok(!Object.prototype.hasOwnProperty.call(providerMap, secondInstanceKey));
        const renamed = providerMap[renamedInstanceKey] as Record<string, unknown>;
        assert.strictEqual(renamed.enabled, true);
        const projects = renamed.projects as Record<string, unknown>;
        const project = projects.PROJ as Record<string, unknown>;
        assert.strictEqual(project.sync_user_id, "user-456");
      });
    }
  });

  test("unknown provider slug throws rather than silently dropping the write", () => {
    assert.throws(
      () => applyKanbanProviderField({}, "notaprovider", "instance", "enabled", true),
      /Unknown kanban provider/,
    );
  });

  test("missing instance identifier throws rather than selecting the first instance", () => {
    assert.throws(
      () => applyKanbanProviderField({}, "jira", undefined, "enabled", true),
      /instance identifier/,
    );
  });

  test("missing instance target throws rather than creating a sibling entry", () => {
    const kanban = {
      jira: {
        "first.atlassian.net": { enabled: true, projects: {} },
      },
    };
    assert.throws(
      () => applyKanbanProviderField(kanban, "jira", "missing.atlassian.net", "enabled", false),
      /was not found/,
    );
    assert.deepStrictEqual(Object.keys(kanban.jira), ["first.atlassian.net"]);
  });

  test("instance rename collision throws without overwriting either instance", () => {
    const first = { enabled: true, projects: {} };
    const second = { enabled: false, projects: {} };
    const kanban = {
      jira: {
        "first.atlassian.net": first,
        "second.atlassian.net": second,
      },
    };
    assert.throws(
      () =>
        applyKanbanProviderField(
          kanban,
          "jira",
          "second.atlassian.net",
          "domain",
          "first.atlassian.net",
        ),
      /already exists/,
    );
    assert.deepStrictEqual(kanban.jira, {
      "first.atlassian.net": first,
      "second.atlassian.net": second,
    });
  });
});
