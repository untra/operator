import { afterEach, describe, expect, jest, test } from "bun:test";
import { act, cleanup, render, renderHook, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { HostContext, type Host } from "../host";
import { resetSessionState, type MutationDefinition, type QueryDefinition } from "./adapter";
import { useApiMutation, useApiQuery } from "./hooks";
import { ApiProvider } from "./store";

function host(profileId?: string): Host {
  return {
    profileId,
    baseUrl: () => "http://operator.test",
    openExternal: () => undefined,
    browseFolder: () => Promise.resolve(null),
    openFile: () => undefined,
  };
}

function Providers({ children, value = host() }: { children: ReactNode; value?: Host }) {
  return (
    <ApiProvider>
      <HostContext.Provider value={value}>{children}</HostContext.Provider>
    </ApiProvider>
  );
}

afterEach(() => {
  cleanup();
  resetSessionState();
  jest.useRealTimers();
});

describe("useApiQuery", () => {
  test("starts when enabled and stops reporting loading when disabled", async () => {
    let calls = 0;
    const definition: QueryDefinition<string> = {
      key: "enabled-transition",
      params: {},
      fetch: () => Promise.resolve(`value-${++calls}`),
    };
    const { result, rerender } = renderHook(({ enabled }) => useApiQuery(definition, { enabled }), {
      initialProps: { enabled: false },
      wrapper: Providers,
    });
    expect(result.current.isLoading).toBe(false);
    expect(calls).toBe(0);
    rerender({ enabled: true });
    await waitFor(() => expect(result.current.data).toBe("value-1"));
    rerender({ enabled: false });
    expect(result.current.isLoading).toBe(false);
  });

  test("switches to a profile-scoped lifecycle", async () => {
    let calls = 0;
    const definition: QueryDefinition<string> = {
      key: "profile-transition",
      params: {},
      fetch: () => Promise.resolve(`value-${++calls}`),
    };
    function Probe({ value }: { value: Host }) {
      return (
        <HostContext.Provider value={value}>
          <Result definition={definition} />
        </HostContext.Provider>
      );
    }
    const view = render(
      <ApiProvider>
        <Probe value={host("one")} />
      </ApiProvider>,
    );
    await screen.findByText("value-1");
    view.rerender(
      <ApiProvider>
        <Probe value={host("two")} />
      </ApiProvider>,
    );
    await screen.findByText("value-2");
  });

  test("keeps cached data visible while refetching", async () => {
    let release!: (value: string) => void;
    let calls = 0;
    const definition: QueryDefinition<string> = {
      key: "cached-refetch",
      params: {},
      fetch: () => {
        calls += 1;
        return calls === 1
          ? Promise.resolve("cached")
          : new Promise<string>((resolve) => {
              release = resolve;
            });
      },
    };
    const { result } = renderHook(() => useApiQuery(definition), { wrapper: Providers });
    await waitFor(() => expect(result.current.data).toBe("cached"));
    let refetch!: Promise<string>;
    act(() => {
      refetch = result.current.refetch();
    });
    expect(result.current.data).toBe("cached");
    expect(result.current.isFetching).toBe(true);
    release("fresh");
    await act(() => refetch);
    expect(result.current.data).toBe("fresh");
  });

  test("cleans up polling on unmount", async () => {
    jest.useFakeTimers();
    let calls = 0;
    const definition: QueryDefinition<number> = {
      key: "poll-cleanup",
      params: {},
      fetch: () => Promise.resolve(++calls),
    };
    const { unmount } = renderHook(() => useApiQuery(definition, { pollIntervalMs: 10 }), {
      wrapper: Providers,
    });
    await act(async () => undefined);
    expect(calls).toBe(1);
    unmount();
    act(() => jest.advanceTimersByTime(50));
    await act(async () => undefined);
    expect(calls).toBe(1);
  });
});

describe("useApiMutation", () => {
  test("reports pending and delivers callbacks", async () => {
    let release!: (value: string) => void;
    const definition: MutationDefinition<string, { value: string }> = {
      key: "pending-callback",
      affected: [],
      run: (_api, variables) =>
        new Promise<string>((resolve) => {
          release = () => resolve(variables.value);
        }),
    };
    const success: string[] = [];
    const { result } = renderHook(() => useApiMutation(definition), { wrapper: Providers });
    act(() =>
      result.current.mutate({ value: "done" }, { onSuccess: (data) => success.push(data) }),
    );
    expect(result.current.isPending).toBe(true);
    await act(async () => release("done"));
    expect(result.current.isPending).toBe(false);
    expect(success).toEqual(["done"]);
  });

  test("rejects duplicate async calls while pending", async () => {
    let release!: () => void;
    const definition: MutationDefinition<void, Record<string, never>> = {
      key: "duplicate-hook-call",
      affected: [],
      run: () =>
        new Promise<void>((resolve) => {
          release = resolve;
        }),
    };
    const { result } = renderHook(() => useApiMutation(definition), { wrapper: Providers });
    let first!: Promise<void>;
    act(() => {
      first = result.current.mutateAsync({});
    });
    await expect(result.current.mutateAsync({})).rejects.toThrow("already pending");
    release();
    await act(() => first);
  });

  test("consumes callback-style rejections after delivering the error", async () => {
    const definition: MutationDefinition<void, Record<string, never>> = {
      key: "callback-hook-error",
      affected: [],
      run: () => Promise.reject(new Error("denied")),
    };
    const errors: string[] = [];
    const { result } = renderHook(() => useApiMutation(definition), { wrapper: Providers });
    act(() => result.current.mutate({}, { onError: (error) => errors.push(error.message) }));
    await waitFor(() => expect(result.current.error?.message).toBe("denied"));
    expect(errors).toEqual(["denied"]);
  });
});

function Result({ definition }: { definition: QueryDefinition<string> }) {
  const result = useApiQuery(definition);
  return <span>{result.data ?? "loading"}</span>;
}
