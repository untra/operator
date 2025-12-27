/**
 * Operator API interface definition.
 */
import { createApiRef } from '@backstage/core-plugin-api';
import type {
  IssueTypeSummary,
  IssueTypeResponse,
  CollectionResponse,
  StepResponse,
  StatusResponse,
  CreateIssueTypeRequest,
  UpdateIssueTypeRequest,
  UpdateStepRequest,
} from './types';

/** Interface for the Operator API */
export interface OperatorApi {
  // Health & Status
  getStatus(): Promise<StatusResponse>;

  // Issue Types
  listIssueTypes(): Promise<IssueTypeSummary[]>;
  getIssueType(key: string): Promise<IssueTypeResponse>;
  createIssueType(request: CreateIssueTypeRequest): Promise<IssueTypeResponse>;
  updateIssueType(
    key: string,
    request: UpdateIssueTypeRequest,
  ): Promise<IssueTypeResponse>;
  deleteIssueType(key: string): Promise<void>;

  // Steps
  getSteps(issueTypeKey: string): Promise<StepResponse[]>;
  getStep(issueTypeKey: string, stepName: string): Promise<StepResponse>;
  updateStep(
    issueTypeKey: string,
    stepName: string,
    request: UpdateStepRequest,
  ): Promise<StepResponse>;

  // Collections
  listCollections(): Promise<CollectionResponse[]>;
  getActiveCollection(): Promise<CollectionResponse>;
  getCollection(name: string): Promise<CollectionResponse>;
  activateCollection(name: string): Promise<CollectionResponse>;
}

/** API ref for dependency injection */
export const operatorApiRef = createApiRef<OperatorApi>({
  id: 'plugin.operator.api',
});
