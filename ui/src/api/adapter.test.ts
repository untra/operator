import { afterEach, describe, expect, jest, test } from "bun:test";
import { asyncableEmoji } from "@untra/naiveasync";
import type { Host } from "../host";
import type { OperatorApi } from "../api-client";
import {
  createApiStore,
  lifecycleId,
  mapQueryState,
  mutationController,
  queryController,
  resetSessionState,
  snapshotSecrets,
  type ApiStore,
  type QueryDefinition,
  type MutationDefinition,
} from "./adapter";
import { QUERY_KEYS } from "./queries";

const PASSWORD = "hunter2-secret-password";
const TOKEN = "tok_live_secret_value";

function host(overrides: Partial<Host> = {}): Host {
  return {
    baseUrl: () => "http://operator.test",
    openExternal: () => undefined,
    browseFolder: () => Promise.resolve(null),
    openFile: () => undefined,
    ...overrides,
  };
}

function storeParams(store: ApiStore): unknown[] {
  const slice = store.getState()[asyncableEmoji];
  return Object.values(slice).map((entry) => entry.params);
}

async function flushAsync(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}

afterEach(() => {
  resetSessionState();
});

describe("lifecycleId", () => {
  test("scopes by base URL, profile, query key, and params", () => {
    const a = lifecycleId({ baseUrl: "http://a", profileId: "p1" }, QUERY_KEYS.agent, {
      agentId: "1",
    });
    const b = lifecycleId({ baseUrl: "http://b", profileId: "p1" }, QUERY_KEYS.agent, {
      agentId: "1",
    });
    const c = lifecycleId({ baseUrl: "http://a", profileId: "p2" }, QUERY_KEYS.agent, {
      agentId: "1",
    });
    const d = lifecycleId({ baseUrl: "http://a", profileId: "p1" }, QUERY_KEYS.agent, {
      agentId: "2",
    });
    expect(new Set([a, b, c, d]).size).toBe(4);
  });
});

describe("mapQueryState", () => {
  test("initial query without cached data reports loading when enabled", () => {
    const mapped = mapQueryState(
      { status: "", error: "", params: {}, data: null },
      { enabled: true },
    );
    expect(mapped.isLoading).toBe(true);
    expect(mapped.isFetching).toBe(false);
    expect(mapped.data).toBeNull();
  });

  test("pending with no data is loading and fetching", () => {
    const mapped = mapQueryState(
      { status: "pending", error: "", params: {}, data: null },
      { enabled: true },
    );
    expect(mapped.isLoading).toBe(true);
    expect(mapped.isFetching).toBe(true);
  });

  test("pending with cached data is fetching rather than loading", () => {
    const mapped = mapQueryState(
      { status: "pending", error: "", params: {}, data: { n: 1 } },
      { enabled: true },
    );
    expect(mapped.isLoading).toBe(false);
    expect(mapped.isFetching).toBe(true);
    expect(mapped.data).toEqual({ n: 1 });
  });

  test("error retains cached data", () => {
    const mapped = mapQueryState(
      { status: "error", error: "boom", params: {}, data: { n: 1 } },
      { enabled: true },
    );
    expect(mapped.isLoading).toBe(false);
    expect(mapped.data).toEqual({ n: 1 });
    expect(mapped.error?.message).toBe("boom");
  });

  test("disabled queries are not loading", () => {
    const mapped = mapQueryState(
      { status: "", error: "", params: {}, data: null },
      { enabled: false },
    );
    expect(mapped.isLoading).toBe(false);
    expect(mapped.isFetching).toBe(false);
  });
});

describe("queryController", () => {
  test("initial query without cached data reports loading", async () => {
    const store = createApiStore();
    let release!: (value: string) => void;
    const gate = new Promise<string>((resolve) => {
      release = resolve;
    });
    const def: QueryDefinition<string> = {
      key: QUERY_KEYS.health,
      params: {},
      fetch: () => gate,
    };
    const query = queryController(store, host(), def);
    const pending = query.start();
    const loading = query.snapshot();
    expect(loading.isLoading).toBe(true);
    expect(loading.isFetching).toBe(true);
    expect(loading.data).toBeNull();
    release("ok");
    await pending;
    expect(query.snapshot().data).toBe("ok");
    expect(query.snapshot().isLoading).toBe(false);
  });

  test("sync preserves cached data during refresh and reports fetching", async () => {
    const store = createApiStore();
    let n = 0;
    let release!: () => void;
    const def: QueryDefinition<string> = {
      key: QUERY_KEYS.status,
      params: {},
      fetch: async () => {
        n += 1;
        if (n === 1) {
          return "first";
        }
        await new Promise<void>((resolve) => {
          release = resolve;
        });
        return "second";
      },
    };
    const query = queryController(store, host(), def);
    await query.start();
    expect(query.snapshot().data).toBe("first");
    const pending = query.refetch();
    expect(query.snapshot().data).toBe("first");
    expect(query.snapshot().isLoading).toBe(false);
    expect(query.snapshot().isFetching).toBe(true);
    release();
    await pending;
    expect(query.snapshot().data).toBe("second");
    expect(query.snapshot().isFetching).toBe(false);
  });

  test("query errors retain cached data", async () => {
    const store = createApiStore();
    let n = 0;
    const def: QueryDefinition<string> = {
      key: QUERY_KEYS.queueStatus,
      params: {},
      fetch: () => {
        n += 1;
        if (n === 1) {
          return Promise.resolve("cached");
        }
        return Promise.reject(new Error("upstream"));
      },
    };
    const query = queryController(store, host(), def);
    await query.start();
    await expect(query.refetch()).rejects.toBeInstanceOf(Error);
    const snap = query.snapshot();
    expect(snap.data).toBe("cached");
    expect(snap.error?.message).toBe("upstream");
    expect(snap.isLoading).toBe(false);
  });

  test("parameterized and profile-scoped queries cannot overwrite one another", async () => {
    const store = createApiStore();
    const defA: QueryDefinition<string, { agentId: string }> = {
      key: QUERY_KEYS.agent,
      params: { agentId: "a" },
      fetch: (_api, params) => Promise.resolve(`agent-${params.agentId}`),
    };
    const defB: QueryDefinition<string, { agentId: string }> = {
      key: QUERY_KEYS.agent,
      params: { agentId: "b" },
      fetch: (_api, params) => Promise.resolve(`agent-${params.agentId}`),
    };
    const profileHost = host({ profileId: "other" });
    const a = queryController(store, host(), defA);
    const b = queryController(store, host(), defB);
    const c = queryController(store, profileHost, defA);
    await Promise.all([a.start(), b.start(), c.start()]);
    expect(a.snapshot().data).toBe("agent-a");
    expect(b.snapshot().data).toBe("agent-b");
    expect(c.snapshot().data).toBe("agent-a");
    expect(a.snapshot().data).toBe("agent-a");
  });

  test("poll cleanup prevents updates after unmount", async () => {
    jest.useFakeTimers();
    const store = createApiStore();
    let calls = 0;
    const def: QueryDefinition<number> = {
      key: QUERY_KEYS.sections,
      params: {},
      fetch: () => {
        calls += 1;
        return Promise.resolve(calls);
      },
    };
    const query = queryController(store, host(), def, { pollIntervalMs: 20 });
    await query.start();
    const afterStart = calls;
    query.stop();
    jest.advanceTimersByTime(60);
    await flushAsync();
    expect(calls).toBe(afterStart);
    jest.useRealTimers();
  });

  test("poll does not expose stale parameter results", async () => {
    jest.useFakeTimers();
    const store = createApiStore();
    const seen: string[] = [];
    const make = (agentId: string) => {
      const def: QueryDefinition<string, { agentId: string }> = {
        key: QUERY_KEYS.agent,
        params: { agentId },
        fetch: (_api, params) => {
          seen.push(params.agentId);
          return Promise.resolve(params.agentId);
        },
      };
      return queryController(store, host(), def, { pollIntervalMs: 20 });
    };
    const first = make("one");
    await first.start();
    first.stop();
    const second = make("two");
    await second.start();
    jest.advanceTimersByTime(50);
    await flushAsync();
    second.stop();
    expect(second.snapshot().data).toBe("two");
    expect(seen.every((id) => id === "one" || id === "two")).toBe(true);
    expect(seen.filter((id) => id === "two").length).toBeGreaterThan(0);
    jest.useRealTimers();
  });
});

describe("mutationController", () => {
  test("success and failure callbacks fire", async () => {
    const store = createApiStore();
    let n = 0;
    const def: MutationDefinition<string, { n: number }> = {
      key: "echo",
      affected: [],
      run: (_api, variables) => {
        n += 1;
        if (variables.n === 0) {
          return Promise.reject(new Error("nope"));
        }
        return Promise.resolve(`ok-${variables.n}`);
      },
    };
    const mutation = mutationController(store, host(), def);
    const ok: string[] = [];
    const err: string[] = [];
    mutation.mutate(
      { n: 1 },
      { onSuccess: (data) => ok.push(data), onError: (e) => err.push(e.message) },
    );
    await mutation.wait();
    mutation.mutate(
      { n: 0 },
      { onSuccess: (data) => ok.push(data), onError: (e) => err.push(e.message) },
    );
    await mutation.wait().catch(() => undefined);
    expect(ok).toEqual(["ok-1"]);
    expect(err).toEqual(["nope"]);
    expect(n).toBe(2);
  });

  test("invalidates every affected query on success and on error", async () => {
    const store = createApiStore();
    let reads = 0;
    const queryDef: QueryDefinition<number> = {
      key: QUERY_KEYS.kanban,
      params: {},
      fetch: () => {
        reads += 1;
        return Promise.resolve(reads);
      },
    };
    const query = queryController(store, host(), queryDef);
    await query.start();
    const loaded = query.snapshot().data;
    expect(loaded).toBeGreaterThan(0);
    const mutation: MutationDefinition<string, { fail: boolean }> = {
      key: "write-ticket",
      affected: [QUERY_KEYS.kanban],
      run: (_api, variables) => {
        if (variables.fail) {
          return Promise.reject(new Error("partial write"));
        }
        return Promise.resolve("saved");
      },
    };
    const write = mutationController(store, host(), mutation);
    await write.mutateAsync({ fail: false });
    await flushAsync();
    if (loaded === null) {
      throw new Error("expected cached query data");
    }
    expect(query.snapshot().data).toBe(loaded + 1);
    await expect(write.mutateAsync({ fail: true })).rejects.toBeInstanceOf(Error);
    await flushAsync();
    expect(query.snapshot().data).toBe(loaded + 2);
  });

  test("invalidates after failure of the second operation in a chain", async () => {
    const store = createApiStore();
    let reads = 0;
    const queryDef: QueryDefinition<number> = {
      key: QUERY_KEYS.collections,
      params: {},
      fetch: () => {
        reads += 1;
        return Promise.resolve(reads);
      },
    };
    const query = queryController(store, host(), queryDef);
    await query.start();
    const loaded = query.snapshot().data;
    expect(loaded).toBeGreaterThan(0);
    const first: MutationDefinition<string, Record<string, never>> = {
      key: "first-write",
      affected: [QUERY_KEYS.collections],
      run: () => Promise.resolve("one"),
    };
    const second: MutationDefinition<string, Record<string, never>> = {
      key: "second-write",
      affected: [QUERY_KEYS.collections],
      run: () => Promise.reject(new Error("second failed")),
    };
    await mutationController(store, host(), first).mutateAsync({});
    await flushAsync();
    if (loaded === null) {
      throw new Error("expected cached query data");
    }
    expect(query.snapshot().data).toBe(loaded + 1);
    await expect(mutationController(store, host(), second).mutateAsync({})).rejects.toBeInstanceOf(
      Error,
    );
    await flushAsync();
    expect(query.snapshot().data).toBe(loaded + 2);
  });

  test("sensitive variables never appear in Redux state or dispatched lifecycle parameters", async () => {
    const store = createApiStore();
    const def: MutationDefinition<string, { password: string; token: string }> = {
      key: "login",
      affected: [],
      sensitive: true,
      run: (_api: OperatorApi, variables) => {
        expect(variables.password).toBe(PASSWORD);
        expect(variables.token).toBe(TOKEN);
        return Promise.resolve("session");
      },
    };
    const mutation = mutationController(store, host(), def);
    await mutation.mutateAsync({ password: PASSWORD, token: TOKEN });
    const encoded = JSON.stringify(store.getState());
    expect(encoded).not.toContain(PASSWORD);
    expect(encoded).not.toContain(TOKEN);
    for (const params of storeParams(store)) {
      expect(JSON.stringify(params)).not.toContain(PASSWORD);
      expect(JSON.stringify(params)).not.toContain(TOKEN);
    }
    expect(snapshotSecrets()).toEqual([]);
  });

  test("clears sensitive references after a failed request", async () => {
    const store = createApiStore();
    const def: MutationDefinition<string, { password: string }> = {
      key: "login-failure",
      affected: [],
      sensitive: true,
      run: () => Promise.reject(new Error("denied")),
    };
    const mutation = mutationController(store, host(), def);
    await expect(mutation.mutateAsync({ password: PASSWORD })).rejects.toBeInstanceOf(Error);
    expect(snapshotSecrets()).toEqual([]);
    expect(JSON.stringify(store.getState())).not.toContain(PASSWORD);
  });

  test("invalidates only queries in the mutation scope", async () => {
    const store = createApiStore();
    let firstReads = 0;
    let secondReads = 0;
    const first = queryController(store, host({ profileId: "first" }), {
      key: QUERY_KEYS.sessions,
      params: {},
      fetch: () => Promise.resolve(++firstReads),
    });
    const second = queryController(store, host({ profileId: "second" }), {
      key: QUERY_KEYS.sessions,
      params: {},
      fetch: () => Promise.resolve(++secondReads),
    });
    await Promise.all([first.start(), second.start()]);
    const mutation = mutationController(store, host({ profileId: "first" }), {
      key: "scoped-write",
      affected: [QUERY_KEYS.sessions],
      run: () => Promise.resolve("ok"),
    });
    await mutation.mutateAsync({});
    await flushAsync();
    expect(firstReads).toBe(2);
    expect(secondReads).toBe(1);
  });

  test("wildcard invalidation never replays mutations", async () => {
    const store = createApiStore();
    let writes = 0;
    const first = mutationController(store, host(), {
      key: "first-write",
      affected: [],
      run: () => Promise.resolve(++writes),
    });
    await first.mutateAsync({});
    const wildcard = mutationController(store, host(), {
      key: "wildcard-write",
      affected: "*",
      run: () => Promise.resolve("ok"),
    });
    await wildcard.mutateAsync({});
    await flushAsync();
    expect(writes).toBe(1);
  });

  test("mutation settlement does not wait for affected query refetches", async () => {
    const store = createApiStore();
    let release!: (value: string) => void;
    let reads = 0;
    const query = queryController(store, host(), {
      key: QUERY_KEYS.health,
      params: {},
      fetch: () => {
        reads += 1;
        return reads === 1
          ? Promise.resolve("initial")
          : new Promise<string>((resolve) => {
              release = resolve;
            });
      },
    });
    await query.start();
    const mutation = mutationController(store, host(), {
      key: "write-health",
      affected: [QUERY_KEYS.health],
      run: () => Promise.resolve("saved"),
    });
    await expect(mutation.mutateAsync({})).resolves.toBe("saved");
    expect(query.snapshot().isFetching).toBe(true);
    release("refreshed");
    await flushAsync();
  });

  test("rejects a duplicate call while the mutation is pending", async () => {
    const store = createApiStore();
    let release!: (value: string) => void;
    const mutation = mutationController(store, host(), {
      key: "single-flight",
      affected: [],
      run: () =>
        new Promise<string>((resolve) => {
          release = resolve;
        }),
    });
    const first = mutation.mutateAsync({});
    await expect(mutation.mutateAsync({})).rejects.toThrow("already pending");
    release("done");
    await expect(first).resolves.toBe("done");
  });

  test("callback-style failures are consumed after onError", async () => {
    const store = createApiStore();
    const errors: string[] = [];
    const mutation = mutationController(store, host(), {
      key: "callback-failure",
      affected: [],
      run: () => Promise.reject(new Error("expected")),
    });
    mutation.mutate({}, { onError: (error) => errors.push(error.message) });
    await expect(mutation.wait()).resolves.toBeUndefined();
    expect(errors).toEqual(["expected"]);
  });

  test("stopped queries are not invalidated", async () => {
    const store = createApiStore();
    let reads = 0;
    const query = queryController(store, host(), {
      key: QUERY_KEYS.status,
      params: {},
      fetch: () => Promise.resolve(++reads),
    });
    await query.start();
    query.stop();
    const mutation = mutationController(store, host(), {
      key: "write-status",
      affected: [QUERY_KEYS.status],
      run: () => Promise.resolve("ok"),
    });
    await mutation.mutateAsync({});
    await flushAsync();
    expect(reads).toBe(1);
  });
});
