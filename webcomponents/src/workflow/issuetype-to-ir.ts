/**
 * Project an Operator workflow (an issue type's `steps[]`) onto the flat
 * node/edge graph `@untra/naiveworkflow-react` renders.
 *
 * This is the *native* projection: it reads the same JSON the runtime executes,
 * so the docs site and the SPA draw identical graphs from identical bytes. It
 * deliberately does not go through a Claude/AGNT export — those are lossy
 * targets derived from this same source.
 *
 * Ordering mirrors `ordered_steps` in `src/workflow_gen/export.rs`: follow the
 * `next_step` chain from the first step, then append any steps that chain never
 * reached, in declaration order. Consecutive pairs in that order are the
 * sequential edges, which makes the `next_step` chain and the
 * collections that omit `next_step` entirely fall out of one rule.
 */

import type { FlatEdge, FlatNode } from '@untra/naiveworkflow-react';

import type { IssueType } from '../generated/IssueType';
import type { StepSchema } from '../generated/StepSchema';
import type { StepTypeTag } from '../generated/StepTypeTag';

export interface OperatorWorkflowGraph {
  nodes: FlatNode[];
  edges: FlatEdge[];
}

/** Step types that run a model. Everything else gets a distinct node kind. */
const AGENT_STEP_TYPES: StepTypeTag[] = [
  'task',
  'classifier',
  'rag',
  'delegator',
  'mcp',
  'multi_model',
  'multi_prompt',
  'matrixed',
  'pipeline',
];

/**
 * A step's type, defaulting the way serde does.
 *
 * The generated type marks `type` required because the API always serializes
 * it, but hand-authored `<KEY>.json` files in a collection bundle omit it and
 * rely on the Rust-side `#[serde(default)]`. This mirrors that default for the
 * file-read path.
 */
function stepType(step: StepSchema): StepTypeTag {
  return step.type ?? 'task';
}

function label(step: StepSchema): string {
  return step.display_name?.trim() || step.name;
}

/**
 * Steps in execution order. Mirrors `ordered_steps` in the Rust exporter,
 * including its cycle guard: a `next_step` loop stops the chain walk rather
 * than hanging.
 */
export function orderedSteps(steps: StepSchema[]): StepSchema[] {
  const byName = new Map(steps.map((s) => [s.name, s]));
  const order: StepSchema[] = [];
  const seen = new Set<string>();

  let current: StepSchema | undefined = steps[0];
  while (current) {
    if (seen.has(current.name)) break;
    seen.add(current.name);
    order.push(current);
    current = current.next_step ? byName.get(current.next_step) : undefined;
  }
  for (const step of steps) {
    if (!seen.has(step.name)) {
      seen.add(step.name);
      order.push(step);
    }
  }
  return order;
}

/** `plan review`, `pr review` — the gate annotation shown on a node. */
function reviewBadge(step: StepSchema): string | undefined {
  const review = step.review_type;
  return review && review !== 'none' ? `${review} review` : undefined;
}

/**
 * Sub-nodes contributed by a fan-out or pipeline step, plus the edges that
 * attach them. Returns the ids downstream sequencing should continue from —
 * for fan-outs that is the aggregate node, so the graph reconverges.
 */
function expandStep(step: StepSchema): {
  nodes: FlatNode[];
  edges: FlatEdge[];
  exit: string;
} {
  const nodes: FlatNode[] = [];
  const edges: FlatEdge[] = [];
  const type = stepType(step);

  const fanOut = (variants: string[], kind: FlatEdge['kind'], aggregateLabel: string) => {
    for (const [i, variant] of variants.entries()) {
      const id = `${step.name}::${i}`;
      nodes.push({ id, kind: 'agent', label: variant, phase: step.name });
      edges.push({ id: `${step.name}->${id}`, source: step.name, target: id, kind });
    }
    const aggregate = `${step.name}::aggregate`;
    nodes.push({
      id: aggregate,
      kind: 'workflow',
      label: aggregateLabel,
      phase: step.name,
      badge: `×${variants.length}`,
    });
    for (const [i] of variants.entries()) {
      const id = `${step.name}::${i}`;
      edges.push({ id: `${id}->${aggregate}`, source: id, target: aggregate, kind: 'seq' });
    }
    return aggregate;
  };

  switch (type) {
    case 'multi_model': {
      const delegators = step.multi_model_config?.delegators ?? [];
      if (!delegators.length) break;
      const strategy = step.multi_model_config?.voting_strategy ?? 'vote';
      return { nodes, edges, exit: fanOut(delegators, 'parallel', `vote: ${strategy}`) };
    }
    case 'multi_prompt': {
      const variations = step.multi_prompt_config?.prompt_variations ?? [];
      if (!variations.length) break;
      const strategy = step.multi_prompt_config?.selection_strategy ?? 'select';
      const labels = variations.map((_, i) => `variation ${i + 1}`);
      return { nodes, edges, exit: fanOut(labels, 'parallel', `select: ${strategy}`) };
    }
    case 'matrixed': {
      const delegators = step.matrixed_config?.delegators ?? [];
      const variations = step.matrixed_config?.prompt_variations ?? [];
      if (!delegators.length || !variations.length) break;
      const cells = delegators.flatMap((d) =>
        variations.map((_, i) => `${d} · variation ${i + 1}`)
      );
      return { nodes, edges, exit: fanOut(cells, 'parallel', 'matrix aggregate') };
    }
    case 'pipeline': {
      const stages = step.pipeline_config?.stages ?? [];
      if (!stages.length) break;
      let previous = step.name;
      for (const [i, stage] of stages.entries()) {
        const id = `${step.name}::stage${i}`;
        nodes.push({
          id,
          kind: 'agent',
          label: stage.label?.trim() || `stage ${i + 1}`,
          phase: step.name,
          model: stage.model ?? undefined,
          agentType: stage.agent ?? undefined,
        });
        edges.push({ id: `${previous}->${id}`, source: previous, target: id, kind: 'pipeline' });
        previous = id;
      }
      return { nodes, edges, exit: previous };
    }
    // Single-node step types: they run one session and contribute no sub-nodes.
    // Listed explicitly rather than swept up by `default` so the `never` check
    // below turns a new StepTypeTag variant in Rust into a compile error here,
    // forcing a decision about how it should be drawn.
    case 'task':
    case 'classifier':
    case 'rag':
    case 'delegator':
    case 'mcp':
      break;
    default: {
      const unhandled: never = type;
      throw new Error(`unhandled step type: ${String(unhandled)}`);
    }
  }

  return { nodes, edges, exit: step.name };
}

/**
 * Build the flat graph for one issue type's Operator workflow.
 *
 * Feed the result straight to `<WorkflowFlow nodes edges />`.
 */
export function issueTypeToGraph(issueType: IssueType): OperatorWorkflowGraph {
  const steps = orderedSteps(issueType.steps ?? []);
  const nodes: FlatNode[] = [];
  const edges: FlatEdge[] = [];
  const exits = new Map<string, string>();

  for (const step of steps) {
    const type = stepType(step);
    nodes.push({
      id: step.name,
      kind: AGENT_STEP_TYPES.includes(type) ? 'agent' : 'note',
      label: label(step),
      phase: step.name,
      prompt: step.prompt ?? undefined,
      agentType: step.agent ?? step.delegator_config?.delegator ?? undefined,
      badge: reviewBadge(step),
      source: type,
    });

    const expanded = expandStep(step);
    nodes.push(...expanded.nodes);
    edges.push(...expanded.edges);
    exits.set(step.name, expanded.exit);
  }

  // Sequential edges: consecutive pairs in execution order. Because
  // `orderedSteps` places `next_step` targets consecutively, this yields the
  // declared chain for wired workflows and declaration order for unwired ones.
  for (let i = 0; i < steps.length - 1; i += 1) {
    const from = exits.get(steps[i].name) ?? steps[i].name;
    const to = steps[i + 1].name;
    edges.push({ id: `${from}->${to}`, source: from, target: to, kind: 'seq' });
  }

  // Reject edges run backwards from the gated step to its retry target.
  const known = new Set(steps.map((s) => s.name));
  for (const step of steps) {
    const target = step.on_reject?.goto_step;
    if (!target || !known.has(target)) continue;
    edges.push({
      id: `${step.name}~reject~${target}`,
      source: step.name,
      target,
      kind: 'loop',
    });
  }

  return { nodes, edges };
}
