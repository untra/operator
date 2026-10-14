import { afterEach, expect, test } from "bun:test";
import { OperatorApi, setCsrfToken } from "./api-client";
import type { Host } from "./host";
import { mockFetch, requestPath, restoreFetch } from "./test-fetch";

const host: Host = {
  baseUrl: () => "https://operator.test/prefix",
  profileId: "team",
  openExternal: () => undefined,
  browseFolder: () => Promise.resolve(null),
  openFile: () => undefined,
};
afterEach(() => {
  restoreFetch();
  setCsrfToken(null);
});

test("exchanges a raw key and sends only the bearer token to the profile MCP transport", async () => {
  const calls: { path: string; authorization: string | null; body: unknown }[] = [];
  mockFetch((input, init) => {
    const path = requestPath(input);
    calls.push({
      path,
      authorization: new Headers(init?.headers).get("Authorization"),
      body: typeof init?.body === "string" ? JSON.parse(init.body) : null,
    });
    if (path.endsWith("/auth/token")) {
      return Promise.resolve(
        new Response(JSON.stringify({ access_token: "short-lived-token", token_type: "Bearer" })),
      );
    }
    return Promise.resolve(new Response(null, { status: 403 }));
  });
  await expect(
    new OperatorApi(host).checkMcp("raw-key", new AbortController().signal),
  ).rejects.toThrow("Permission denied");
  expect(calls).toEqual([
    {
      path: "/prefix/api/v1/auth/token",
      authorization: null,
      body: { grant_type: "operator:access-key", access_key: "raw-key" },
    },
    {
      path: "/prefix/api/v1/profiles/team/mcp/sse",
      authorization: "Bearer short-lived-token",
      body: null,
    },
  ]);
});

test("a cancelled check cannot proceed to MCP", async () => {
  const controller = new AbortController();
  const calls: string[] = [];
  mockFetch((input) => {
    calls.push(requestPath(input));
    controller.abort();
    return Promise.resolve(
      new Response(JSON.stringify({ access_token: "token", token_type: "Bearer" })),
    );
  });
  await expect(new OperatorApi(host).checkMcp("raw-key", controller.signal)).rejects.toThrow();
  expect(calls).toEqual(["/prefix/api/v1/auth/token"]);
});

test("a malformed token exchange never falls back to the browser session", async () => {
  const calls: string[] = [];
  mockFetch((input) => {
    calls.push(requestPath(input));
    return Promise.resolve(new Response("{}"));
  });
  await expect(
    new OperatorApi(host).checkMcp("raw-key", new AbortController().signal),
  ).rejects.toThrow("Token exchange did not return a bearer token");
  expect(calls).toEqual(["/prefix/api/v1/auth/token"]);
});
