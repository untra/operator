import { useCallback, useEffect, useMemo } from "react";
import { useSelector, useStore } from "react-redux";
import type { AnyAction, AsyncableSlice } from "@untra/naiveasync";
import { useHost } from "../host";
import {
  mapQueryState,
  mutationController,
  queryController,
  resetSessionState,
  type MutateCallbacks,
  type MutationDefinition,
  type QueryDefinition,
  type QuerySnapshot,
} from "./adapter";

export type ApiQueryResult<Data> = QuerySnapshot<Data> & {
  refetch: () => Promise<Data>;
};

export type ApiMutationResult<Data, Variables extends object> = {
  data: Data | null;
  error: Error | null;
  isPending: boolean;
  mutate: (variables: Variables, callbacks?: MutateCallbacks<Data>) => void;
  mutateAsync: (variables: Variables) => Promise<Data>;
  reset: () => void;
};

export function useApiQuery<Data, Params extends object = Record<string, never>>(
  definition: QueryDefinition<Data, Params>,
  options?: { enabled?: boolean; pollIntervalMs?: number },
): ApiQueryResult<Data> {
  const store = useStore<AsyncableSlice, AnyAction>();
  const host = useHost();
  const enabled = options?.enabled ?? true;
  const pollIntervalMs = options?.pollIntervalMs;
  const controller = useMemo(
    () => queryController(store, host, definition, { pollIntervalMs }),
    [definition, host, pollIntervalMs, store],
  );

  useEffect(() => {
    if (!enabled) {
      return undefined;
    }
    void controller.start().catch(() => undefined);
    return () => {
      controller.stop();
    };
  }, [controller, enabled]);

  const state = useSelector((slice: AsyncableSlice) => controller.selector(slice));
  return {
    ...mapQueryState(state, { enabled }, controller.id),
    refetch: controller.refetch,
  };
}

export function useApiMutation<Data, Variables extends object>(
  definition: MutationDefinition<Data, Variables>,
): ApiMutationResult<Data, Variables> {
  const store = useStore<AsyncableSlice, AnyAction>();
  const host = useHost();
  const controller = useMemo(
    () => mutationController<Data, Variables>(store, host, definition),
    [definition, host, store],
  );
  const state = useSelector((slice: AsyncableSlice) => controller.selector(slice));
  const mutate = useCallback(
    (variables: Variables, callbacks?: MutateCallbacks<Data>) => {
      controller.mutate(variables, callbacks);
    },
    [controller],
  );
  const mutateAsync = useCallback(
    (variables: Variables) => controller.mutateAsync(variables),
    [controller],
  );
  const reset = useCallback(() => {
    controller.reset();
  }, [controller]);
  return {
    data: state.data,
    error:
      state.status === "error"
        ? (controller.snapshot().error ?? new Error(state.error || "Request failed"))
        : null,
    isPending: state.status === "pending",
    mutate,
    mutateAsync,
    reset,
  };
}

export function useResetSession(): () => void {
  return resetSessionState;
}
