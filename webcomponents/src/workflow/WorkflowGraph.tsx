/**
 * Renders an Operator workflow as an interactive graph.
 *
 * Takes the issue type document itself — the native JSON the runtime executes —
 * projects it with `issueTypeToGraph`, and draws it with
 * `@untra/naiveworkflow-react` (React Flow + dagre). Display-only: nothing
 * flows back into operator's domain model.
 *
 * Both the SPA and the docs site mount this same component over the same
 * `<KEY>.json`, so their graphs cannot disagree.
 */

import { useMemo } from 'react';
import { WorkflowFlow } from '@untra/naiveworkflow-react';

import { usePhaseColors, useDocumentTheme } from '../shared/theme';
import { issueTypeToGraph } from './issuetype-to-ir';
import type { IssueType } from '../generated/IssueType';

export interface WorkflowGraphProps {
  issueType: IssueType;
  /** Canvas height. Defaults to 520px, matching the SPA's issue-type pane. */
  height?: number | string;
  /**
   * Lay the graph out top-to-bottom instead of left-to-right.
   *
   * Orientation is presentation, not content — the nodes and edges are
   * identical either way. Narrow columns fit a vertical chain without downscaling the labels into
   * illegibility; wide panes read better left-to-right.
   */
  vertical?: boolean;
  className?: string;
}

export function WorkflowGraph({
  issueType,
  height = 520,
  vertical = false,
  className,
}: WorkflowGraphProps) {
  const theme = useDocumentTheme();
  const phaseColors = usePhaseColors();
  const { nodes, edges } = useMemo(() => issueTypeToGraph(issueType), [issueType]);

  if (!nodes.length) {
    return <div className="operator-workflow-empty">This issue type defines no steps.</div>;
  }

  return (
    <div className={className ?? 'operator-workflow-canvas'} style={{ height }}>
      <WorkflowFlow
        // Remount when the issue type changes
        key={issueType.key}
        nodes={nodes}
        edges={edges}
        theme={theme}
        phaseColors={phaseColors}
        verticalRender={vertical}
        fitView
      />
    </div>
  );
}
