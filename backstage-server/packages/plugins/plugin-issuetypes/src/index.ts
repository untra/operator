/**
 * @operator/plugin-issuetypes
 *
 * Backstage plugin for managing Operator issue types and collections.
 */

export {
  issueTypesPlugin,
  IssueTypesPage,
  IssueTypeDetailPage,
  IssueTypeFormPage,
  CollectionsPage,
  rootRouteRef,
  detailRouteRef,
  formRouteRef,
  collectionsRouteRef,
} from './plugin';

// API exports
export { operatorApiRef } from './api';
export type { OperatorApi } from './api';
export * from './api/types';

// Hooks exports
export * from './hooks';
