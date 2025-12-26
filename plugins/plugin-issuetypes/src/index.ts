/**
 * @operator/plugin-issuetypes
 *
 * Backstage plugin for managing Operator issue types and collections.
 */

// Plugin and extensions
export {
  issueTypesPlugin,
  IssueTypesPage,
  IssueTypeDetailPage,
  IssueTypeFormPage,
  CollectionsPage,
} from './plugin';

// API
export { operatorApiRef } from './api';
export type { OperatorApi } from './api';

// Types
export type {
  IssueTypeSummary,
  IssueTypeResponse,
  StepResponse,
  FieldResponse,
  CollectionResponse,
  CreateIssueTypeRequest,
  UpdateIssueTypeRequest,
  CreateStepRequest,
  CreateFieldRequest,
} from './api/types';

// Routes
export {
  rootRouteRef,
  detailRouteRef,
  createRouteRef,
  editRouteRef,
  collectionsRouteRef,
} from './routes';
