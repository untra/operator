import {
  asyncLifecycle,
  findLifecycleById,
  naiveAsyncMiddleware,
  naiveAsyncReducer,
  type AsyncableSlice,
  type AsyncLifecycle,
  type AsyncState,
  type AnyAction,
} from "@untra/naiveasync";
import { applyMiddleware, legacy_createStore, type Store } from "redux";
import { OperatorApi } from "../api-client";
import type { Host } from "../host";
import type { QueryKey } from "./queries";

const ID_SEP = "\u001f";
const SECRET_REF = "secretRef";

type SecretRef = { readonly [SECRET_REF]: string };

type LifecycleEntry = {
  key: string;
  scope: QueryScope;
  store: ApiStore;
  subscribers: number;
};

const queries = new Map<string, LifecycleEntry>();
const mutations = new Map<string, ApiStore>();
const secrets = new Map<string, unknown>();
const lastErrors = new Map<string, Error>();
const pollRefs = new Map<string, { count: number; ms: number }>();

export type QueryDefinition<Data, Params extends object = Record<string, never>> = {
  key: string;
  params: Params;
  fetch: (api: OperatorApi, params: Params) => Promise<Data>;
};

export type MutationDefinition<Data, Variables extends object> = {
  key: string;
  affected: readonly string[] | "*";
  run: (api: OperatorApi, variables: Variables) => Promise<Data>;
  sensitive?: boolean;
};

export type QuerySnapshot<Data> = {
  data: Data | null;
  error: Error | null;
  isLoading: boolean;
  isFetching: boolean;
};

export type ApiStore = Store<AsyncableSlice, AnyAction>;

export function createApiStore(): ApiStore {
  return legacy_createStore(naiveAsyncReducer, applyMiddleware(naiveAsyncMiddleware));
}

export type QueryScope = {
  baseUrl: string;
  profileId?: string;
};

export function scopeFromHost(host: Host): QueryScope {
  return { baseUrl: host.baseUrl(), profileId: host.profileId };
}

export function lifecycleId(scope: QueryScope, key: string, params: object): string {
  return [scope.baseUrl, scope.profileId ?? "_", key, stableSerialize(params)].join(ID_SEP);
}

function stableSerialize(params: object): string {
  const keys = Object.keys(params).toSorted();
  const sorted: Record<string, unknown> = {};
  for (const key of keys) {
    sorted[key] = (params as Record<string, unknown>)[key];
  }
  return JSON.stringify(sorted);
}

export function mapQueryState<Data>(
  state: AsyncState<Data, object>,
  options: { enabled: boolean },
  id?: string,
): QuerySnapshot<Data> {
  const enabled = options.enabled;
  const data = state.data;
  const isFetching = enabled && state.status === "pending";
  const isLoading = enabled && data == null && (state.status === "pending" || state.status === "");
  const stored = id ? lastErrors.get(id) : undefined;
  const error =
    state.status === "error" ? (stored ?? new Error(state.error || "Request failed")) : null;
  return { data, error, isLoading, isFetching };
}

function asError(error: unknown): Error {
  return error instanceof Error ? error : new Error(String(error));
}

function waitForLifecycle<Data, Params extends object = object>(
  lifecycle: AsyncLifecycle<Data, Params>,
): Promise<Data> {
  const data = lifecycle.awaitResolve();
  const failure = lifecycle.awaitReject().then((error: unknown) => Promise.reject(asError(error)));
  return Promise.race([data, failure]);
}

function registerQuery(id: string, key: string, scope: QueryScope, store: ApiStore): void {
  const current = queries.get(id);
  if (current) {
    current.subscribers += 1;
    return;
  }
  queries.set(id, { key, scope, store, subscribers: 1 });
}

function unregisterQuery(id: string): void {
  const current = queries.get(id);
  if (!current) {
    return;
  }
  current.subscribers -= 1;
  if (current.subscribers === 0) {
    queries.delete(id);
  }
}

function sameScope(left: QueryScope, right: QueryScope): boolean {
  return left.baseUrl === right.baseUrl && left.profileId === right.profileId;
}

function affectedQueries(
  scope: QueryScope,
  affected: readonly string[] | "*",
): Array<[string, LifecycleEntry]> {
  const matches: Array<[string, LifecycleEntry]> = [];
  for (const entry of queries) {
    if (sameScope(entry[1].scope, scope) && (affected === "*" || affected.includes(entry[1].key))) {
      matches.push(entry);
    }
  }
  return matches;
}

export function invalidateQueries(scope: QueryScope, affected: readonly string[] | "*"): void {
  for (const [id, entry] of affectedQueries(scope, affected)) {
    const lifecycle = findLifecycleById(id);
    if (!lifecycle) {
      continue;
    }
    entry.store.dispatch(lifecycle.sync());
  }
}

export function resetSessionState(): void {
  for (const [id, entry] of queries) {
    const lifecycle = findLifecycleById(id);
    if (lifecycle) {
      entry.store.dispatch(lifecycle.subscribe(0));
      entry.store.dispatch(lifecycle.destroy());
    }
  }
  for (const [id, store] of mutations) {
    const lifecycle = findLifecycleById(id);
    if (lifecycle) {
      store.dispatch(lifecycle.destroy());
    }
  }
  queries.clear();
  mutations.clear();
  secrets.clear();
  lastErrors.clear();
  pollRefs.clear();
}

export function snapshotSecrets(): unknown[] {
  return [...secrets.values()];
}

function putSecret(id: string, value: unknown): void {
  secrets.set(id, value);
}

function peekSecret(id: string): unknown {
  if (!secrets.has(id)) {
    throw new Error("Request credentials expired");
  }
  return secrets.get(id);
}

function clearSecret(id: string): void {
  secrets.delete(id);
}

function addPoll(store: ApiStore, id: string, ms: number): void {
  const lifecycle = findLifecycleById(id);
  if (!lifecycle) {
    return;
  }
  const current = pollRefs.get(id);
  if (current) {
    current.count += 1;
    if (ms < current.ms) {
      current.ms = ms;
      store.dispatch(lifecycle.subscribe(ms));
    }
    return;
  }
  pollRefs.set(id, { count: 1, ms });
  store.dispatch(lifecycle.subscribe(ms));
}

function removePoll(store: ApiStore, id: string): void {
  const current = pollRefs.get(id);
  if (!current) {
    return;
  }
  current.count -= 1;
  if (current.count > 0) {
    return;
  }
  pollRefs.delete(id);
  const lifecycle = findLifecycleById(id);
  if (lifecycle) {
    store.dispatch(lifecycle.subscribe(0));
  }
}

export type QueryOptions = {
  pollIntervalMs?: number;
};

export type QueryController<Data> = {
  id: string;
  selector: (state: AsyncableSlice) => AsyncState<Data, object>;
  start: () => Promise<Data>;
  stop: () => void;
  refetch: () => Promise<Data>;
  snapshot: () => QuerySnapshot<Data>;
};

export function queryController<Data, Params extends object>(
  store: ApiStore,
  host: Host,
  definition: QueryDefinition<Data, Params>,
  options: QueryOptions = {},
): QueryController<Data> {
  const scope = scopeFromHost(host);
  const id = lifecycleId(scope, definition.key, definition.params);
  const lifecycle = asyncLifecycle(id, async (params: Params) => {
    try {
      const data = await definition.fetch(new OperatorApi(host), params);
      lastErrors.delete(id);
      return data;
    } catch (error) {
      lastErrors.set(id, asError(error));
      throw error;
    }
  });

  let started = false;
  return {
    id,
    selector: lifecycle.selector,
    start() {
      if (!started) {
        registerQuery(id, definition.key, scope, store);
        started = true;
      }
      const wait = waitForLifecycle<Data, Params>(lifecycle);
      store.dispatch(lifecycle.sync(definition.params));
      if (options.pollIntervalMs) {
        addPoll(store, id, options.pollIntervalMs);
      }
      return wait;
    },
    stop() {
      if (!started) {
        return;
      }
      if (options.pollIntervalMs) {
        removePoll(store, id);
      }
      unregisterQuery(id);
      started = false;
    },
    refetch() {
      const wait = waitForLifecycle<Data, Params>(lifecycle);
      store.dispatch(lifecycle.sync(definition.params));
      return wait;
    },
    snapshot() {
      return mapQueryState(lifecycle.selector(store.getState()), { enabled: true }, id);
    },
  };
}

export type MutateCallbacks<Data> = {
  onSuccess?: (data: Data) => void;
  onError?: (error: Error) => void;
};

export type MutationController<Data, Variables extends object> = {
  selector: (state: AsyncableSlice) => AsyncState<Data, object>;
  mutate: (variables: Variables, callbacks?: MutateCallbacks<Data>) => void;
  mutateAsync: (variables: Variables) => Promise<Data>;
  wait: () => Promise<unknown>;
  reset: () => void;
  snapshot: () => { data: Data | null; error: Error | null; isPending: boolean };
};

let mutationSeq = 0;

export function mutationController<Data, Variables extends object>(
  store: ApiStore,
  host: Host,
  definition: MutationDefinition<Data, Variables>,
): MutationController<Data, Variables> {
  mutationSeq += 1;
  const scope = scopeFromHost(host);
  const id = [scope.baseUrl, scope.profileId ?? "_", definition.key, String(mutationSeq)].join(
    ID_SEP,
  );
  mutations.set(id, store);
  const lifecycle = asyncLifecycle<Data, Variables | SecretRef>(
    id,
    async (params: Variables | SecretRef) => {
      const ref = SECRET_REF in params ? params[SECRET_REF] : undefined;
      const variables = (ref === undefined ? params : peekSecret(ref)) as Variables;
      try {
        const data = await definition.run(new OperatorApi(host), variables);
        lastErrors.delete(id);
        return data;
      } catch (error) {
        lastErrors.set(id, asError(error));
        throw error;
      } finally {
        if (ref !== undefined) {
          clearSecret(ref);
        }
      }
    },
  );

  let inflight: Promise<unknown> = Promise.resolve();
  let pending = false;

  async function mutateAsync(variables: Variables): Promise<Data> {
    if (pending) {
      throw new Error(`Mutation "${definition.key}" is already pending`);
    }
    pending = true;
    const wait = waitForLifecycle<Data, Variables | SecretRef>(lifecycle);
    if (definition.sensitive) {
      const ref = crypto.randomUUID();
      putSecret(ref, variables);
      const payload: SecretRef = { [SECRET_REF]: ref };
      store.dispatch(lifecycle.call(payload));
    } else {
      store.dispatch(lifecycle.call(variables));
    }
    try {
      return await wait;
    } finally {
      pending = false;
      invalidateQueries(scope, definition.affected);
    }
  }

  return {
    selector: lifecycle.selector,
    mutate(variables, callbacks) {
      inflight = mutateAsync(variables)
        .then(
          (data) => {
            callbacks?.onSuccess?.(data);
            return data;
          },
          (error: unknown) => {
            const err = asError(error);
            callbacks?.onError?.(err);
          },
        )
        .catch(() => undefined);
    },
    mutateAsync,
    wait() {
      return inflight;
    },
    reset() {
      store.dispatch(lifecycle.reset());
      lastErrors.delete(id);
    },
    snapshot() {
      const state = lifecycle.selector(store.getState());
      return {
        data: state.data,
        error: state.status === "error" ? (lastErrors.get(id) ?? new Error(state.error)) : null,
        isPending: pending || state.status === "pending",
      };
    },
  };
}

export type { QueryKey };
