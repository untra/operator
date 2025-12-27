/**
 * Hooks for fetching and managing collections.
 */
import { useState, useEffect, useCallback } from 'react';
import { useApi } from '@backstage/core-plugin-api';
import { operatorApiRef } from '../api';
import type { CollectionResponse } from '../api/types';

/** Hook to fetch the list of all collections */
export function useCollections() {
  const api = useApi(operatorApiRef);
  const [collections, setCollections] = useState<CollectionResponse[]>();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error>();

  const load = useCallback(async () => {
    setLoading(true);
    setError(undefined);
    try {
      const data = await api.listCollections();
      setCollections(data);
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
    collections,
    loading,
    error,
    retry: load,
  };
}

/** Hook to fetch the active collection */
export function useActiveCollection() {
  const api = useApi(operatorApiRef);
  const [collection, setCollection] = useState<CollectionResponse>();
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error>();

  const load = useCallback(async () => {
    setLoading(true);
    setError(undefined);
    try {
      const data = await api.getActiveCollection();
      setCollection(data);
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
    collection,
    loading,
    error,
    retry: load,
  };
}

/** Hook to activate a collection */
export function useActivateCollection() {
  const api = useApi(operatorApiRef);
  const [activating, setActivating] = useState(false);
  const [error, setError] = useState<Error>();
  const [activatedCollection, setActivatedCollection] =
    useState<CollectionResponse>();

  const activateCollection = useCallback(
    async (name: string) => {
      setActivating(true);
      setError(undefined);
      try {
        const result = await api.activateCollection(name);
        setActivatedCollection(result);
        return result;
      } catch (err) {
        const error = err instanceof Error ? err : new Error(String(err));
        setError(error);
        throw error;
      } finally {
        setActivating(false);
      }
    },
    [api],
  );

  return {
    activateCollection,
    activating,
    error,
    activatedCollection,
  };
}
