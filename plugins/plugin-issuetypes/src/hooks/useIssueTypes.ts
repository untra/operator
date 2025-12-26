/**
 * Hooks for fetching and managing issue types.
 */
import { useApi } from '@backstage/core-plugin-api';
import { useAsync, useAsyncFn } from 'react-use';
import { operatorApiRef } from '../api';
import type {
  IssueTypeSummary,
  IssueTypeResponse,
  CreateIssueTypeRequest,
  UpdateIssueTypeRequest,
} from '../api/types';

/** Hook to fetch the list of all issue types */
export function useIssueTypes() {
  const api = useApi(operatorApiRef);

  const state = useAsync(async () => {
    return api.listIssueTypes();
  }, [api]);

  return {
    issueTypes: state.value as IssueTypeSummary[] | undefined,
    loading: state.loading,
    error: state.error,
    retry: () => state.retry?.(),
  };
}

/** Hook to fetch a single issue type by key */
export function useIssueType(key: string) {
  const api = useApi(operatorApiRef);

  const state = useAsync(async () => {
    return api.getIssueType(key);
  }, [api, key]);

  return {
    issueType: state.value as IssueTypeResponse | undefined,
    loading: state.loading,
    error: state.error,
    retry: () => state.retry?.(),
  };
}

/** Hook to create a new issue type */
export function useCreateIssueType() {
  const api = useApi(operatorApiRef);

  const [state, createIssueType] = useAsyncFn(
    async (request: CreateIssueTypeRequest) => {
      return api.createIssueType(request);
    },
    [api],
  );

  return {
    createIssueType,
    creating: state.loading,
    error: state.error,
    createdIssueType: state.value,
  };
}

/** Hook to update an existing issue type */
export function useUpdateIssueType() {
  const api = useApi(operatorApiRef);

  const [state, updateIssueType] = useAsyncFn(
    async (key: string, request: UpdateIssueTypeRequest) => {
      return api.updateIssueType(key, request);
    },
    [api],
  );

  return {
    updateIssueType,
    updating: state.loading,
    error: state.error,
    updatedIssueType: state.value,
  };
}

/** Hook to delete an issue type */
export function useDeleteIssueType() {
  const api = useApi(operatorApiRef);

  const [state, deleteIssueType] = useAsyncFn(
    async (key: string) => {
      await api.deleteIssueType(key);
      return key;
    },
    [api],
  );

  return {
    deleteIssueType,
    deleting: state.loading,
    error: state.error,
    deletedKey: state.value,
  };
}
