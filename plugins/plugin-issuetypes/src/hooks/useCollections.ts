/**
 * Hooks for fetching and managing collections.
 */
import { useApi } from '@backstage/core-plugin-api';
import { useAsync, useAsyncFn } from 'react-use';
import { operatorApiRef } from '../api';
import type { CollectionResponse } from '../api/types';

/** Hook to fetch the list of all collections */
export function useCollections() {
  const api = useApi(operatorApiRef);

  const state = useAsync(async () => {
    return api.listCollections();
  }, [api]);

  return {
    collections: state.value as CollectionResponse[] | undefined,
    loading: state.loading,
    error: state.error,
    retry: () => state.retry?.(),
  };
}

/** Hook to fetch the active collection */
export function useActiveCollection() {
  const api = useApi(operatorApiRef);

  const state = useAsync(async () => {
    try {
      return await api.getActiveCollection();
    } catch (error) {
      // 404 means no active collection, which is valid
      if ((error as { status?: number }).status === 404) {
        return null;
      }
      throw error;
    }
  }, [api]);

  return {
    activeCollection: state.value as CollectionResponse | null | undefined,
    loading: state.loading,
    error: state.error,
    retry: () => state.retry?.(),
  };
}

/** Hook to activate a collection */
export function useActivateCollection() {
  const api = useApi(operatorApiRef);

  const [state, activateCollection] = useAsyncFn(
    async (name: string) => {
      return api.activateCollection(name);
    },
    [api],
  );

  return {
    activateCollection,
    activating: state.loading,
    error: state.error,
    activatedCollection: state.value,
  };
}
