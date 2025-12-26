/**
 * Operator API client implementation.
 * Communicates with the Operator REST API via Backstage proxy.
 */
import type { DiscoveryApi, FetchApi } from '@backstage/core-plugin-api';
import type { OperatorApi } from './OperatorApi';
import type {
  IssueTypeSummary,
  IssueTypeResponse,
  CollectionResponse,
  StepResponse,
  StatusResponse,
  CreateIssueTypeRequest,
  UpdateIssueTypeRequest,
  UpdateStepRequest,
  ErrorResponse,
} from './types';

/** Options for creating the Operator API client */
export interface OperatorApiClientOptions {
  discoveryApi: DiscoveryApi;
  fetchApi: FetchApi;
}

/** API client error with status code */
export class OperatorApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
    public readonly errorCode?: string,
  ) {
    super(message);
    this.name = 'OperatorApiError';
  }
}

/** Implementation of the Operator API */
export class OperatorApiClient implements OperatorApi {
  private readonly discoveryApi: DiscoveryApi;
  private readonly fetchApi: FetchApi;

  constructor(options: OperatorApiClientOptions) {
    this.discoveryApi = options.discoveryApi;
    this.fetchApi = options.fetchApi;
  }

  /** Get the base URL for the Operator API via proxy */
  private async getBaseUrl(): Promise<string> {
    const proxyUrl = await this.discoveryApi.getBaseUrl('proxy');
    return `${proxyUrl}/operator`;
  }

  /** Make a request to the Operator API */
  private async request<T>(
    path: string,
    options?: RequestInit,
  ): Promise<T> {
    const baseUrl = await this.getBaseUrl();
    const url = `${baseUrl}${path}`;

    const response = await this.fetchApi.fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    });

    if (!response.ok) {
      let errorMessage = `API error: ${response.status} ${response.statusText}`;
      let errorCode: string | undefined;

      try {
        const errorBody: ErrorResponse = await response.json();
        errorMessage = errorBody.message;
        errorCode = errorBody.error;
      } catch {
        // Use default error message if parsing fails
      }

      throw new OperatorApiError(errorMessage, response.status, errorCode);
    }

    // Handle empty responses (e.g., DELETE)
    const contentType = response.headers.get('content-type');
    if (contentType && contentType.includes('application/json')) {
      return response.json();
    }

    return undefined as T;
  }

  // Health & Status

  async getStatus(): Promise<StatusResponse> {
    return this.request('/api/v1/status');
  }

  // Issue Types

  async listIssueTypes(): Promise<IssueTypeSummary[]> {
    return this.request('/api/v1/issuetypes');
  }

  async getIssueType(key: string): Promise<IssueTypeResponse> {
    return this.request(`/api/v1/issuetypes/${encodeURIComponent(key)}`);
  }

  async createIssueType(
    request: CreateIssueTypeRequest,
  ): Promise<IssueTypeResponse> {
    return this.request('/api/v1/issuetypes', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  async updateIssueType(
    key: string,
    request: UpdateIssueTypeRequest,
  ): Promise<IssueTypeResponse> {
    return this.request(`/api/v1/issuetypes/${encodeURIComponent(key)}`, {
      method: 'PUT',
      body: JSON.stringify(request),
    });
  }

  async deleteIssueType(key: string): Promise<void> {
    await this.request(`/api/v1/issuetypes/${encodeURIComponent(key)}`, {
      method: 'DELETE',
    });
  }

  // Steps

  async getSteps(issueTypeKey: string): Promise<StepResponse[]> {
    return this.request(
      `/api/v1/issuetypes/${encodeURIComponent(issueTypeKey)}/steps`,
    );
  }

  async getStep(
    issueTypeKey: string,
    stepName: string,
  ): Promise<StepResponse> {
    return this.request(
      `/api/v1/issuetypes/${encodeURIComponent(issueTypeKey)}/steps/${encodeURIComponent(stepName)}`,
    );
  }

  async updateStep(
    issueTypeKey: string,
    stepName: string,
    request: UpdateStepRequest,
  ): Promise<StepResponse> {
    return this.request(
      `/api/v1/issuetypes/${encodeURIComponent(issueTypeKey)}/steps/${encodeURIComponent(stepName)}`,
      {
        method: 'PUT',
        body: JSON.stringify(request),
      },
    );
  }

  // Collections

  async listCollections(): Promise<CollectionResponse[]> {
    return this.request('/api/v1/collections');
  }

  async getActiveCollection(): Promise<CollectionResponse> {
    return this.request('/api/v1/collections/active');
  }

  async getCollection(name: string): Promise<CollectionResponse> {
    return this.request(`/api/v1/collections/${encodeURIComponent(name)}`);
  }

  async activateCollection(name: string): Promise<CollectionResponse> {
    return this.request(
      `/api/v1/collections/${encodeURIComponent(name)}/activate`,
      {
        method: 'PUT',
      },
    );
  }
}
