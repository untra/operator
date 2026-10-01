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

export function requestPath(input: RequestInfo | URL): string {
  const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
  return new URL(url).pathname;
}

export function requestBody(init?: RequestInit): string | undefined {
  return typeof init?.body === "string" ? init.body : undefined;
}
