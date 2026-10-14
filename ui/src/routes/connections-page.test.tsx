import { afterEach, expect, mock, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ApiProvider, resetSessionState } from "../api";
import { HostContext, type Host } from "../host";
import { mockFetch, requestPath, restoreFetch } from "../test-fetch";
import { setCsrfToken } from "../api-client";
import * as webcomponentMocks from "../test-webcomponents";

mock.module("@operator/webcomponents", () => webcomponentMocks);
const { ConnectionsPage } = await import("./ConnectionsPage");
const HOST: Host = {
  baseUrl: () => "https://operator.test",
  profileId: "team",
  openExternal: () => undefined,
  browseFolder: () => Promise.resolve(null),
  openFile: () => undefined,
};
function json(body: unknown) {
  return new Response(JSON.stringify(body), { headers: { "Content-Type": "application/json" } });
}

afterEach(() => {
  cleanup();
  resetSessionState();
  setCsrfToken(null);
  restoreFetch();
});

test("creates an MCP key with read and execute scopes and profile-scopes connections", async () => {
  const paths: string[] = [];
  const creations: unknown[] = [];
  mockFetch((input, init) => {
    const path = requestPath(input);
    paths.push(path);
    if (path.endsWith("/auth/session")) {
      return Promise.resolve(json({ scopes: ["read", "admin", "execute"] }));
    }
    if (path.endsWith("/connections")) {
      return Promise.resolve(
        json({
          version: "test",
          profile_name: "team",
          observed_at: "2026-10-13T00:00:00Z",
          public_url: null,
          api_path: "/api/v1/profiles/team",
          mcp_path: "/api/v1/profiles/team/mcp/sse",
          mcp_http_mounted: true,
          mcp_connected_clients: 0,
        }),
      );
    }
    if (path.endsWith("/auth/keys")) {
      if (init?.method === "POST") {
        creations.push(JSON.parse(typeof init.body === "string" ? init.body : "null"));
        return Promise.resolve(json({ key: { id: "new" }, secret: "test-secret" }));
      }
      return Promise.resolve(json({ keys: [] }));
    }
    return Promise.resolve(json([]));
  });
  setCsrfToken("test-csrf");
  render(
    <HostContext value={HOST}>
      <ApiProvider>
        <ConnectionsPage />
      </ApiProvider>
    </HostContext>,
  );
  fireEvent.change(await screen.findByRole("textbox", { name: "Name" }), {
    target: { value: "claude" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Create key" }));
  await waitFor(() =>
    expect(creations).toEqual([
      { name: "claude", scopes: ["read", "execute"], expires_in_days: 90 },
    ]),
  );
  expect(paths).toContain("/api/v1/profiles/team/connections");
  expect(paths).toContain("/api/v1/profiles/team/connections/mcp/clients");
  expect(paths).not.toContain("/api/v1/profiles/team/auth/keys");
});

test("read-only users never request administrative client or key data", async () => {
  const paths: string[] = [];
  mockFetch((input) => {
    const path = requestPath(input);
    paths.push(path);
    if (path.endsWith("/auth/session")) {
      return Promise.resolve(json({ scopes: ["read"] }));
    }
    if (path.endsWith("/connections")) {
      return Promise.resolve(
        json({
          version: "test",
          profile_name: "team",
          observed_at: "2026-10-13T00:00:00Z",
          public_url: null,
          api_path: "/api/v1/profiles/team",
          mcp_path: "/api/v1/profiles/team/mcp/sse",
          mcp_http_mounted: false,
          mcp_connected_clients: 0,
        }),
      );
    }
    return Promise.resolve(json([]));
  });
  render(
    <HostContext value={HOST}>
      <ApiProvider>
        <ConnectionsPage />
      </ApiProvider>
    </HostContext>,
  );
  await waitFor(() => expect(paths).toContain("/api/v1/profiles/team/connections"));
  expect(paths.some((path) => path.endsWith("/clients") || path.endsWith("/keys"))).toBe(false);
});
