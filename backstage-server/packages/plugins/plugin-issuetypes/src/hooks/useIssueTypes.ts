/**
 * Hooks for fetching and managing issue types.
 */
import { useState, useEffect, useCallback } from 'react';
import { useApi } from '@backstage/core-plugin-api';
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
  const [issueTypes, setIssueTypes] = useState<IssueTypeSummary[]>();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error>();

  const load = useCallback(async () => {
    setLoading(true);
    setError(undefined);
    try {
      const data = await api.listIssueTypes();
      setIssueTypes(data);
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)));
    } finally {
      setLoading(false);
    }
  }, [api]);

  useEffect(() => {
    load();
  }, [load]);

  return {
    issueTypes,
    loading,
    error,
    retry: load,
  };
}

/** Hook to fetch a single issue type by key */
export function useIssueType(key: string) {
  const api = useApi(operatorApiRef);
  const [issueType, setIssueType] = useState<IssueTypeResponse>();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error>();

  const load = useCallback(async () => {
    setLoading(true);
    setError(undefined);
    try {
      const data = await api.getIssueType(key);
      setIssueType(data);
    } catch (err) {
      setError(err instanceof Error ? err : new Error(String(err)));
    } finally {
      setLoading(false);
    }
  }, [api, key]);

  useEffect(() => {
    load();
  }, [load]);

  return {
    issueType,
    loading,
    error,
    retry: load,
  };
}

/** Hook to create a new issue type */
export function useCreateIssueType() {
  const api = useApi(operatorApiRef);
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<Error>();
  const [createdIssueType, setCreatedIssueType] = useState<IssueTypeResponse>();

  const createIssueType = useCallback(
    async (request: CreateIssueTypeRequest) => {
      setCreating(true);
      setError(undefined);
      try {
        const result = await api.createIssueType(request);
        setCreatedIssueType(result);
        return result;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        throw error;
      } finally {
        setCreating(false);
      }
    },
    [api],
  );

  return {
    createIssueType,
    creating,
    error,
    createdIssueType,
  };
}

/** Hook to update an existing issue type */
export function useUpdateIssueType() {
  const api = useApi(operatorApiRef);
  const [updating, setUpdating] = useState(false);
  const [error, setError] = useState<Error>();
  const [updatedIssueType, setUpdatedIssueType] = useState<IssueTypeResponse>();

  const updateIssueType = useCallback(
    async (key: string, request: UpdateIssueTypeRequest) => {
      setUpdating(true);
      setError(undefined);
      try {
        const result = await api.updateIssueType(key, request);
        setUpdatedIssueType(result);
        return result;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        throw error;
      } finally {
        setUpdating(false);
      }
    },
    [api],
  );

  return {
    updateIssueType,
    updating,
    error,
    updatedIssueType,
  };
}

/** Hook to delete an issue type */
export function useDeleteIssueType() {
  const api = useApi(operatorApiRef);
  const [deleting, setDeleting] = useState(false);
  const [error, setError] = useState<Error>();
  const [deletedKey, setDeletedKey] = useState<string>();

  const deleteIssueType = useCallback(
    async (key: string) => {
      setDeleting(true);
      setError(undefined);
      try {
        await api.deleteIssueType(key);
        setDeletedKey(key);
        return key;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        throw error;
      } finally {
        setDeleting(false);
      }
    },
    [api],
  );

  return {
    deleteIssueType,
    deleting,
    error,
    deletedKey,
  };
}
