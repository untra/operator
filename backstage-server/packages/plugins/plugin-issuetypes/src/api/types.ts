/**
 * TypeScript types for the Operator REST API.
 * These mirror the Rust DTOs in src/rest/dto.rs
 */

/** Execution mode for issue types */
export type ExecutionMode = 'autonomous' | 'paired';

/** Permission mode for steps */
export type PermissionMode = 'default' | 'plan' | 'acceptEdits' | 'delegate';

/** Field types for issue type fields */
export type FieldType = 'string' | 'enum' | 'bool' | 'date' | 'text';

/** Step output types */
export type StepOutput =
  | 'plan'
  | 'code'
  | 'test'
  | 'pr'
  | 'ticket'
  | 'review'
  | 'report'
  | 'documentation';

/** All available step output types */
export const STEP_OUTPUTS: StepOutput[] = [
  'plan',
  'code',
  'test',
  'pr',
  'ticket',
  'review',
  'report',
  'documentation',
];

/** Common allowed tools for Claude Code */
export const ALLOWED_TOOLS = [
  'Read',
  'Write',
  'Edit',
  'Glob',
  'Grep',
  'Bash',
  'Task',
  'WebFetch',
  'WebSearch',
  'LSP',
  'NotebookEdit',
  'TodoWrite',
] as const;

/** Summary response for listing issue types */
export interface IssueTypeSummary {
  key: string;
  name: string;
  description: string;
  mode: ExecutionMode;
  glyph: string;
  source: string;
  step_count: number;
}

/** Full response for a single issue type */
export interface IssueTypeResponse {
  key: string;
  name: string;
  description: string;
  mode: ExecutionMode;
  glyph: string;
  color?: string;
  project_required: boolean;
  source: string;
  fields: FieldResponse[];
  steps: StepResponse[];
}

/** Response for a field within an issue type */
export interface FieldResponse {
  name: string;
  description: string;
  field_type: FieldType;
  required: boolean;
  default?: string;
  options: string[];
  placeholder?: string;
  max_length?: number;
  user_editable: boolean;
}

/** Response for a step within an issue type */
export interface StepResponse {
  name: string;
  display_name?: string;
  prompt: string;
  outputs: StepOutput[];
  allowed_tools: string[];
  requires_review: boolean;
  next_step?: string;
  on_reject?: OnRejectConfig;
  permission_mode: PermissionMode;
}

/** On reject configuration for review steps */
export interface OnRejectConfig {
  goto_step: string;
  prompt?: string;
}

/** Response for a collection of issue types */
export interface CollectionResponse {
  name: string;
  description: string;
  types: string[];
  is_active: boolean;
}

/** Request to create a new issue type */
export interface CreateIssueTypeRequest {
  key: string;
  name: string;
  description: string;
  mode?: ExecutionMode;
  glyph: string;
  color?: string;
  project_required?: boolean;
  fields?: CreateFieldRequest[];
  steps: CreateStepRequest[];
}

/** Request to update an existing issue type */
export interface UpdateIssueTypeRequest {
  name?: string;
  description?: string;
  mode?: ExecutionMode;
  glyph?: string;
  color?: string;
  project_required?: boolean;
  fields?: CreateFieldRequest[];
  steps?: CreateStepRequest[];
}

/** Request to create a field */
export interface CreateFieldRequest {
  name: string;
  description: string;
  field_type?: FieldType;
  required?: boolean;
  default?: string;
  options?: string[];
  placeholder?: string;
  max_length?: number;
  user_editable?: boolean;
}

/** Request to create a step */
export interface CreateStepRequest {
  name: string;
  display_name?: string;
  prompt: string;
  outputs?: StepOutput[];
  allowed_tools?: string[];
  requires_review?: boolean;
  next_step?: string;
  on_reject?: OnRejectConfig;
  permission_mode?: PermissionMode;
}

/** Request to update a step */
export interface UpdateStepRequest {
  display_name?: string;
  prompt?: string;
  outputs?: StepOutput[];
  allowed_tools?: string[];
  requires_review?: boolean;
  next_step?: string;
  on_reject?: OnRejectConfig;
  permission_mode?: PermissionMode;
}

/** Health check response */
export interface HealthResponse {
  status: string;
  version: string;
}

/** Status response with registry info */
export interface StatusResponse {
  status: string;
  version: string;
  issuetype_count: number;
  collection_count: number;
  active_collection: string;
}

/** Error response from the API */
export interface ErrorResponse {
  error: string;
  message: string;
}
