/**
 * React entry point — consumed by the operator SPA (`ui/`).
 *
 * Exports components and this package's own graph types only. Domain types
 * (`IssueType`, `StepSchema`, …) are generated from Rust into `bindings/` and
 * imported from `@operator/bindings/*` directly — this package is not a second
 * source of truth for them.
 *
 * React is a peer dependency here so the SPA keeps a single React instance.
 * The docs site uses the sibling `elements` entry instead, which bundles React
 * and exposes the same components as custom elements.
 */

export { WorkflowGraph } from './workflow/WorkflowGraph';
export type { WorkflowGraphProps } from './workflow/WorkflowGraph';

export { issueTypeToGraph, orderedSteps } from './workflow/issuetype-to-ir';
export type { OperatorWorkflowGraph } from './workflow/issuetype-to-ir';

export { useDocumentTheme, usePhaseColors } from './shared/theme';
export type { Theme } from './shared/theme';
