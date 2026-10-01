declare module "@untra/naiveasync" {
  import type { Dispatch, Middleware, Reducer } from "redux";

  export const asyncableEmoji: "🔁";

  export interface AnyAction {
    type: string;
    payload?: any;
  }

  export type AsyncFunction<Data, Params> = (params: Params) => Promise<Data>;
  export type AsyncableStateStatus = "" | "pending" | "error" | "success";
  export type AsyncState<Data, Params> =
    | { status: ""; error: ""; params: {}; data: null }
    | {
        status: "pending";
        error: "" | string;
        params: {} | Params;
        data: null | Data;
      }
    | {
        status: "error";
        error: "" | string;
        params: {} | Params;
        data: null | Data;
      }
    | { status: "success"; error: ""; params: {} | Params; data: Data };

  export interface AsyncableSlice {
    [asyncableEmoji]: Record<string, AsyncState<any, any>>;
  }

  type AsyncPhase =
    | "call"
    | "data"
    | "error"
    | "success"
    | "destroy"
    | "reset"
    | "sync"
    | "assign"
    | "subscribe";

  interface AsyncPostmark {
    name: string;
    phase: AsyncPhase;
    trace?: string;
  }

  export type AsyncActionCreator<Payload> = (payload?: Payload) => {
    readonly type: string;
    readonly postmark: AsyncPostmark;
    readonly match: (action: { type: string; payload: any }) => boolean;
  };

  export interface AsyncableOptions {
    readonly debounce?: number;
    readonly throttle?: number;
    readonly timeout?: number;
    readonly traceDispatch?: boolean;
    readonly dataDepends?: string[];
  }

  export type OnData<Data, Params> =
    | (() => void)
    | ((data: Data) => void)
    | ((data: Data, params: Params) => void)
    | ((data: Data, params: Params, dispatch: Dispatch<AnyAction>) => void);
  export type OnError<Params> =
    | (() => void)
    | ((error: string) => void)
    | ((error: string, params: Params) => void)
    | ((error: string, params: Params, dispatch: Dispatch<AnyAction>) => void);

  export interface AsyncLifecycle<Data, Params> {
    readonly id: string;
    readonly operation: AsyncFunction<Data, Params>;
    readonly selector: (state: AsyncableSlice) => AsyncState<Data, Params>;
    readonly call: AsyncActionCreator<Params>;
    readonly sync: AsyncActionCreator<Params | undefined>;
    readonly destroy: AsyncActionCreator<undefined>;
    readonly data: AsyncActionCreator<Data>;
    readonly error: AsyncActionCreator<string>;
    readonly success: AsyncActionCreator<undefined>;
    readonly reset: AsyncActionCreator<undefined>;
    readonly assign: AsyncActionCreator<AsyncState<Data, Params>>;
    readonly subscribe: AsyncActionCreator<number>;
    readonly memoized: (enabled: boolean) => AsyncLifecycle<Data, Params>;
    readonly throttle: (ms: number) => AsyncLifecycle<Data, Params>;
    readonly debounce: (ms: number) => AsyncLifecycle<Data, Params>;
    readonly timeout: (ms: number) => AsyncLifecycle<Data, Params>;
    readonly retries: (
      retries: number,
      callback?: (error: any, retry?: number) => void,
    ) => AsyncLifecycle<Data, Params>;
    readonly onData: (callback: OnData<Data, Params>) => AsyncLifecycle<Data, Params>;
    readonly onError: (callback: OnError<Params>) => AsyncLifecycle<Data, Params>;
    readonly awaitResolve: () => Promise<Data>;
    readonly awaitReject: () => Promise<Data>;
    readonly dataDepends: (ids: string[]) => AsyncLifecycle<Data, Params>;
    readonly resolveData: () => Promise<Data>;
    readonly rejectError: () => Promise<string>;
    readonly options: (options: AsyncableOptions) => AsyncLifecycle<Data, Params>;
    readonly invalidate: (options?: AsyncableOptions) => AsyncLifecycle<Data, Params>;
    readonly abortController: (controller: AbortController) => AsyncLifecycle<Data, Params>;
  }

  export const naiveAsyncMiddleware: Middleware;
  export const naiveAsyncReducer: Reducer<AsyncableSlice, AnyAction>;
  export function asyncLifecycle<Data, Params extends {}>(
    id: string,
    operation: AsyncFunction<Data, Params>,
    options?: AsyncableOptions,
  ): AsyncLifecycle<Data, Params>;
  export function findLifecycleById(id: string): AsyncLifecycle<any, any> | undefined;
}
