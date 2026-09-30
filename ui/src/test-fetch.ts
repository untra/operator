import { jest } from "bun:test";

const originalFetch = globalThis.fetch;

export function mockFetch(
  implementation: (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>,
) {
  const mocked = jest.fn(implementation);
  globalThis.fetch = Object.assign(mocked, { preconnect: originalFetch.preconnect });
  return mocked;
}

export function restoreFetch(): void {
  globalThis.fetch = originalFetch;
}
