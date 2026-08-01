/**
 * Custom-elements entry point — consumed by the Jekyll docs site.
 *
 * Bundles React (the docs site has no module infrastructure) and registers the
 * shared components as plain HTML tags, so generated markdown can write
 * `<operator-workflow-explorer base="…">` with no build step of its own.
 *
 * Registration is idempotent: re-loading the bundle on a page that already
 * defined these tags is a no-op rather than a `NotSupportedError`.
 */

import '@xyflow/react/dist/style.css';
import '@untra/naiveworkflow-react/styles.css';
import './elements.css';

import {
  OperatorCollectionSearch,
  OPERATOR_COLLECTION_SEARCH_TAG,
} from './elements/operator-collection-search';
import {
  OperatorWorkflowExplorer,
  OPERATOR_WORKFLOW_EXPLORER_TAG,
} from './elements/operator-workflow-explorer';

function define(tag: string, ctor: CustomElementConstructor) {
  if (!customElements.get(tag)) customElements.define(tag, ctor);
}

define(OPERATOR_WORKFLOW_EXPLORER_TAG, OperatorWorkflowExplorer);
define(OPERATOR_COLLECTION_SEARCH_TAG, OperatorCollectionSearch);

export { OperatorCollectionSearch, OperatorWorkflowExplorer };
