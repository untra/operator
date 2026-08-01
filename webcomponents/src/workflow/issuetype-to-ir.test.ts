/**
 * Mapper tests run against the real collection fixtures under
 * `src/collections/`, so a change to a shipped Operator workflow that the
 * renderer cannot express fails here rather than silently drawing a wrong graph.
 */

import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

import { issueTypeToGraph, orderedSteps } from './issuetype-to-ir';
import type { IssueType } from '../generated/IssueType';
import type { StepSchema } from '../generated/StepSchema';

const REPO_ROOT = join(import.meta.dir, '../../..');

/**
 * Hand-authored collection files omit every serde-defaulted key, so reading one
 * is the same unchecked cast the docs explorer performs at its fetch boundary.
 */
function fixture(collection: string, key: string): IssueType {
  const path = join(REPO_ROOT, 'src/collections', collection, `${key}.json`);
  return JSON.parse(readFileSync(path, 'utf8')) as IssueType;
}

/**
 * A minimal valid step. The generated type marks the serde-defaulted fields
 * required because the API always serializes them, so the helper supplies the
 * same defaults Rust would.
 */
function step(partial: Partial<StepSchema> & { name: string }): StepSchema {
  return {
    prompt: '',
    outputs: [],
    type: 'task',
    review_type: 'none',
    allowed_tools: [],
    permission_mode: 'default',
    artifact_patterns: [],
    ...partial,
  };
}

function doc(steps: StepSchema[]): IssueType {
  return {
    key: 'T',
    name: 'T',
    description: '',
    mode: 'autonomous',
    glyph: '>',
    project_required: true,
    fields: [],
    steps,
    source: 'builtin',
  };
}

describe('orderedSteps', () => {
  test('follows the next_step chain', () => {
    const steps = [
      step({ name: 'a', next_step: 'c' }),
      step({ name: 'b' }),
      step({ name: 'c', next_step: 'b' }),
    ];
    expect(orderedSteps(steps).map((s) => s.name)).toEqual(['a', 'c', 'b']);
  });

  test('falls back to declaration order when next_step is absent', () => {
    const steps = [step({ name: 'a' }), step({ name: 'b' }), step({ name: 'c' })];
    expect(orderedSteps(steps).map((s) => s.name)).toEqual(['a', 'b', 'c']);
  });

  test('breaks next_step cycles instead of hanging', () => {
    const steps = [step({ name: 'a', next_step: 'b' }), step({ name: 'b', next_step: 'a' })];
    expect(orderedSteps(steps).map((s) => s.name)).toEqual(['a', 'b']);
  });
});

describe('issueTypeToGraph', () => {
  test('draws the FEAT chain with its review gates and reject edges', () => {
    const { nodes, edges } = issueTypeToGraph(fixture('dev_kanban', 'FEAT'));

    expect(nodes.map((n) => n.id)).toEqual(['plan', 'build', 'code', 'test', 'deploy']);

    const seq = edges.filter((e) => e.kind === 'seq').map((e) => `${e.source}->${e.target}`);
    expect(seq).toEqual(['plan->build', 'build->code', 'code->test', 'test->deploy']);

    // on_reject is a backward edge, not a sequential one.
    const loops = edges.filter((e) => e.kind === 'loop').map((e) => `${e.source}->${e.target}`);
    expect(loops).toEqual(['plan->plan', 'deploy->code']);

    // Review gates surface as node badges.
    expect(nodes.find((n) => n.id === 'plan')?.badge).toBe('plan review');
    expect(nodes.find((n) => n.id === 'deploy')?.badge).toBe('pr review');
    expect(nodes.find((n) => n.id === 'build')?.badge).toBeUndefined();
  });

  test('fans multi_model steps out to a voting aggregate', () => {
    const { nodes, edges } = issueTypeToGraph(
      doc([
        step({
          name: 'review',
          type: 'multi_model',
          multi_model_config: {
            delegators: ['claude', 'codex'],
            voting_strategy: 'majority',
            share_answers: false,
            voting_mode: 'single_judge',
          },
        }),
        step({ name: 'land' }),
      ])
    );

    expect(nodes.map((n) => n.id)).toEqual([
      'review',
      'review::0',
      'review::1',
      'review::aggregate',
      'land',
    ]);
    expect(edges.filter((e) => e.kind === 'parallel')).toHaveLength(2);
    expect(nodes.find((n) => n.id === 'review::aggregate')?.label).toBe('vote: majority');
    // Sequencing continues from the aggregate, so the graph reconverges.
    expect(edges).toContainEqual({
      id: 'review::aggregate->land',
      source: 'review::aggregate',
      target: 'land',
      kind: 'seq',
    });
  });

  test('chains pipeline stages and continues from the last one', () => {
    const { nodes, edges } = issueTypeToGraph(
      doc([
        step({
          name: 'sweep',
          type: 'pipeline',
          pipeline_config: {
            item_source: { type: 'projects' },
            stages: [
              { label: 'find', prompt: '' },
              { label: 'fix', prompt: '' },
            ],
          },
        }),
        step({ name: 'report' }),
      ])
    );

    expect(nodes.map((n) => n.id)).toEqual([
      'sweep',
      'sweep::stage0',
      'sweep::stage1',
      'report',
    ]);
    expect(edges.filter((e) => e.kind === 'pipeline').map((e) => e.id)).toEqual([
      'sweep->sweep::stage0',
      'sweep::stage0->sweep::stage1',
    ]);
    expect(edges).toContainEqual({
      id: 'sweep::stage1->report',
      source: 'sweep::stage1',
      target: 'report',
      kind: 'seq',
    });
  });

  test('a fan-out step with no configured variants stays a single node', () => {
    const { nodes, edges } = issueTypeToGraph(
      doc([
        step({
          name: 'solo',
          type: 'multi_model',
          // Configured but with no delegators: the step stays a single node.
          multi_model_config: {
            delegators: [],
            voting_strategy: 'majority',
            share_answers: false,
            voting_mode: 'single_judge',
          },
        }),
      ])
    );
    expect(nodes.map((n) => n.id)).toEqual(['solo']);
    expect(edges).toEqual([]);
  });

  test('ignores on_reject targets that are not real steps', () => {
    const { edges } = issueTypeToGraph(
      doc([step({ name: 'only', on_reject: { goto_step: 'ghost', prompt: '' } })])
    );
    expect(edges).toEqual([]);
  });

  test('every shipped issue type produces a connected, well-formed graph', () => {
    const collections: Record<string, string[]> = {
      dev_kanban: ['TASK', 'FEAT', 'FIX'],
      devops_kanban: ['TASK', 'FEAT', 'FIX', 'SPIKE', 'INV'],
      operator: ['ASSESS', 'SYNC', 'INIT', 'AGENT_SETUP', 'PROJECT_INIT'],
      simple: ['TASK'],
      ralph_loop: ['PRD', 'STORY', 'RLOOP'],
      jr_orchestration: ['JRPLAN', 'JRFEAT', 'JRTASK', 'JRREV', 'JRREBASE'],
      elves_overnight: ['ELVSTAGE', 'ELVBATCH', 'LANDPR', 'ELVRPT'],
      coder: ['FEATURE', 'IMPROVEMENT', 'BUG'],
    };

    for (const [collection, keys] of Object.entries(collections)) {
      for (const key of keys) {
        const { nodes, edges } = issueTypeToGraph(fixture(collection, key));
        const ids = new Set(nodes.map((n) => n.id));

        expect(nodes.length).toBeGreaterThan(0);
        expect(ids.size).toBe(nodes.length);
        for (const edge of edges) {
          expect(ids.has(edge.source)).toBe(true);
          expect(ids.has(edge.target)).toBe(true);
        }
        expect(new Set(edges.map((e) => e.id)).size).toBe(edges.length);
      }
    }
  });
});
