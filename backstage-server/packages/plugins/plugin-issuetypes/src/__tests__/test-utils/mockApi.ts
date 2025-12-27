/**
 * Mock Operator API for testing.
 */
import type { OperatorApi } from '../../api';
import type {
  IssueTypeSummary,
  IssueTypeResponse,
  CollectionResponse,
  StepResponse,
  StatusResponse,
  FieldResponse,
} from '../../api/types';

/** Mock issue type summaries */
export const mockIssueTypeSummaries: IssueTypeSummary[] = [
  {
    key: 'FEAT',
    name: 'Feature',
    description: 'New feature implementation',
    mode: 'autonomous',
    glyph: '*',
    source: 'builtin',
    step_count: 5,
  },
  {
    key: 'FIX',
    name: 'Fix',
    description: 'Bug fix',
    mode: 'autonomous',
    glyph: '#',
    source: 'builtin',
    step_count: 3,
  },
  {
    key: 'TASK',
    name: 'Task',
    description: 'Simple task',
    mode: 'autonomous',
    glyph: '>',
    source: 'builtin',
    step_count: 1,
  },
  {
    key: 'SPIKE',
    name: 'Spike',
    description: 'Research spike',
    mode: 'paired',
    glyph: '?',
    source: 'builtin',
    step_count: 3,
  },
];

/** Mock fields for an issue type */
export const mockFields: FieldResponse[] = [
  {
    name: 'id',
    description: 'Unique ticket ID',
    field_type: 'string',
    required: true,
    options: [],
    user_editable: false,
  },
  {
    name: 'summary',
    description: 'Brief summary of the task',
    field_type: 'string',
    required: true,
    placeholder: 'Enter a brief summary',
    max_length: 120,
    options: [],
    user_editable: true,
  },
  {
    name: 'priority',
    description: 'Task priority',
    field_type: 'enum',
    required: true,
    default: 'P2-medium',
    options: ['P0-critical', 'P1-high', 'P2-medium', 'P3-low'],
    user_editable: true,
  },
];

/** Mock steps for an issue type */
export const mockSteps: StepResponse[] = [
  {
    name: 'plan',
    display_name: 'Plan',
    prompt: 'Create a plan for implementing the feature',
    outputs: ['plan'],
    allowed_tools: ['Read', 'Glob', 'Grep'],
    requires_review: true,
    next_step: 'build',
    permission_mode: 'plan',
  },
  {
    name: 'build',
    display_name: 'Build',
    prompt: 'Implement the feature according to the plan',
    outputs: ['code'],
    allowed_tools: ['Read', 'Write', 'Edit', 'Glob', 'Grep', 'Bash'],
    requires_review: false,
    next_step: 'test',
    permission_mode: 'default',
  },
  {
    name: 'test',
    display_name: 'Test',
    prompt: 'Write and run tests for the implementation',
    outputs: ['test', 'code'],
    allowed_tools: ['Read', 'Write', 'Edit', 'Glob', 'Grep', 'Bash'],
    requires_review: false,
    permission_mode: 'default',
  },
];

/** Mock full issue type response */
export const mockIssueTypeResponse: IssueTypeResponse = {
  key: 'FEAT',
  name: 'Feature',
  description: 'New feature implementation',
  mode: 'autonomous',
  glyph: '*',
  color: 'green',
  project_required: true,
  source: 'builtin',
  fields: mockFields,
  steps: mockSteps,
};

/** Mock collections */
export const mockCollections: CollectionResponse[] = [
  {
    name: 'default',
    description: 'Default collection with all issue types',
    types: ['FEAT', 'FIX', 'TASK', 'SPIKE', 'INV'],
    is_active: true,
  },
  {
    name: 'minimal',
    description: 'Minimal collection for simple tasks',
    types: ['TASK', 'FIX'],
    is_active: false,
  },
];

/** Mock status response */
export const mockStatus: StatusResponse = {
  status: 'ok',
  version: '0.1.0',
  issuetype_count: 4,
  collection_count: 2,
  active_collection: 'default',
};

/** Create a mock OperatorApi */
export function createMockOperatorApi(
  overrides: Partial<OperatorApi> = {},
): OperatorApi {
  return {
    getStatus: async () => mockStatus,
    listIssueTypes: async () => mockIssueTypeSummaries,
    getIssueType: async (key: string) => {
      const found = mockIssueTypeSummaries.find((t) => t.key === key);
      if (!found) {
        throw new Error(`Issue type not found: ${key}`);
      }
      return {
        ...mockIssueTypeResponse,
        key: found.key,
        name: found.name,
        description: found.description,
        mode: found.mode,
        glyph: found.glyph,
        source: found.source,
      };
    },
    createIssueType: async (request) => ({
      key: request.key,
      name: request.name,
      description: request.description,
      mode: request.mode || 'autonomous',
      glyph: request.glyph,
      color: request.color,
      project_required: request.project_required ?? true,
      source: 'user',
      fields: (request.fields || []).map((f) => ({
        name: f.name,
        description: f.description,
        field_type: f.field_type || 'string',
        required: f.required || false,
        default: f.default,
        options: f.options || [],
        placeholder: f.placeholder,
        max_length: f.max_length,
        user_editable: f.user_editable ?? true,
      })),
      steps: request.steps.map((s) => ({
        name: s.name,
        display_name: s.display_name,
        prompt: s.prompt,
        outputs: s.outputs || [],
        allowed_tools: s.allowed_tools || [],
        requires_review: s.requires_review || false,
        next_step: s.next_step,
        on_reject: s.on_reject,
        permission_mode: s.permission_mode || 'default',
      })),
    }),
    updateIssueType: async (key, request) => ({
      ...mockIssueTypeResponse,
      key,
      name: request.name ?? mockIssueTypeResponse.name,
      description: request.description ?? mockIssueTypeResponse.description,
      mode: request.mode ?? mockIssueTypeResponse.mode,
      glyph: request.glyph ?? mockIssueTypeResponse.glyph,
      color: request.color,
      project_required: request.project_required ?? mockIssueTypeResponse.project_required,
      fields: request.fields
        ? request.fields.map((f) => ({
            name: f.name,
            description: f.description,
            field_type: f.field_type || 'string',
            required: f.required || false,
            default: f.default,
            options: f.options || [],
            placeholder: f.placeholder,
            max_length: f.max_length,
            user_editable: f.user_editable ?? true,
          }))
        : mockIssueTypeResponse.fields,
      steps: request.steps
        ? request.steps.map((s) => ({
            name: s.name,
            display_name: s.display_name,
            prompt: s.prompt,
            outputs: s.outputs || [],
            allowed_tools: s.allowed_tools || [],
            requires_review: s.requires_review || false,
            next_step: s.next_step,
            on_reject: s.on_reject,
            permission_mode: s.permission_mode || 'default',
          }))
        : mockIssueTypeResponse.steps,
    }),
    deleteIssueType: async () => {},
    getSteps: async () => mockSteps,
    getStep: async (_key, stepName) => {
      const found = mockSteps.find((s) => s.name === stepName);
      if (!found) {
        throw new Error(`Step not found: ${stepName}`);
      }
      return found;
    },
    updateStep: async (_key, stepName, request) => {
      const found = mockSteps.find((s) => s.name === stepName);
      if (!found) {
        throw new Error(`Step not found: ${stepName}`);
      }
      return { ...found, ...request };
    },
    listCollections: async () => mockCollections,
    getActiveCollection: async () =>
      mockCollections.find((c) => c.is_active) || mockCollections[0],
    getCollection: async (name) => {
      const found = mockCollections.find((c) => c.name === name);
      if (!found) {
        throw new Error(`Collection not found: ${name}`);
      }
      return found;
    },
    activateCollection: async (name) => {
      const found = mockCollections.find((c) => c.name === name);
      if (!found) {
        throw new Error(`Collection not found: ${name}`);
      }
      return { ...found, is_active: true };
    },
    ...overrides,
  };
}
