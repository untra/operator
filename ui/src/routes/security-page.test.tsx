import { afterEach, describe, expect, jest, mock, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ApiProvider } from "../api";
import { resetSessionState } from "../api/adapter";
import { setCsrfToken } from "../api-client";
import { HostContext, type Host } from "../host";
import { mockFetch, requestPath, restoreFetch } from "../test-fetch";

import * as webcomponentMocks from "../test-webcomponents";
mock.module("@operator/webcomponents", () => webcomponentMocks);

const { SecurityPage } = await import("./SecurityPage");

const TEST_HOST: Host = {
  baseUrl: () => "http://operator.test",
  openExternal: () => undefined,
  browseFolder: () => Promise.resolve(null),
  openFile: () => undefined,
};

const GRANTS = {
  profile_id: "00000000-0000-0000-0000-000000000001",
  capabilities: ["tickets:read"],
  projects: ["operator"],
  instance_wide: false,
};

function json(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

afterEach(() => {
  cleanup();
  resetSessionState();
  setCsrfToken(null);
  restoreFetch();
  jest.restoreAllMocks();
});

describe("SecurityPage", () => {
  test("creates and revokes credentials through mutation lifecycles", async () => {
    const requests: string[] = [];
    mockFetch((input, init) => {
      const path = requestPath(input);
      const method = init?.method ?? "GET";
      requests.push(`${method} ${path}`);
      if (path.endsWith("/auth/capabilities")) {
        return Promise.resolve(
          json({
            grants: GRANTS,
            oidc_user: true,
            administrator: false,
            can_issue: true,
            connection_id: null,
            administration_grants: null,
          }),
        );
      }
      if (path.endsWith("/auth/sessions") && method === "GET") {
        return Promise.resolve(
          json({
            sessions: [
              {
                id: "session-1",
                created_at: "2026-09-30T00:00:00Z",
                expires_at: "2026-10-01T00:00:00Z",
                current: false,
              },
            ],
            devices: [],
          }),
        );
      }
      if (path.endsWith("/auth/keys") && method === "GET") {
        return Promise.resolve(
          json({
            keys: [
              {
                id: "key-1",
                name: "existing",
                grants: GRANTS,
                display_prefix: "opk_v1_key-1_",
                scopes: ["read"],
                created_at: "2026-09-30T00:00:00Z",
                expires_at: "2026-10-01T00:00:00Z",
              },
            ],
          }),
        );
      }
      if (path.endsWith("/auth/keys") && method === "POST") {
        return Promise.resolve(
          json({
            key: {
              id: "key-2",
              name: "ci",
              grants: GRANTS,
              display_prefix: "opk_v1_key-2_",
              scopes: ["read"],
              created_at: "2026-09-30T00:00:00Z",
              expires_at: "2026-10-01T00:00:00Z",
            },
            secret: "operator-secret-once",
          }),
        );
      }
      if (method === "DELETE" && path.includes("/auth/sessions/")) {
        return Promise.resolve(json({ message: "logged out" }));
      }
      if (method === "DELETE" && path.includes("/auth/keys/")) {
        return Promise.resolve(json({ id: "key-1", revoked_at: "2026-09-30T01:00:00Z" }));
      }
      return Promise.resolve(json({ message: `Unexpected request: ${path}` }));
    });
    setCsrfToken("csrf");
    render(
      <ApiProvider>
        <HostContext.Provider value={TEST_HOST}>
          <SecurityPage />
        </HostContext.Provider>
      </ApiProvider>,
    );
    await screen.findByText("existing");
    fireEvent.change(screen.getByLabelText("Name"), { target: { value: "ci" } });
    fireEvent.click(screen.getByLabelText("tickets:read"));
    fireEvent.click(screen.getByRole("button", { name: "Create key" }));
    expect(await screen.findByText("operator-secret-once")).toBeTruthy();
    const revokeButtons = screen.getAllByRole("button", { name: "Revoke" });
    fireEvent.click(revokeButtons[0]);
    fireEvent.click(revokeButtons[1]);
    expect(requests).not.toContain("DELETE /api/v1/auth/keys/key-1");
    fireEvent.click(await screen.findByRole("button", { name: "Confirm revoke" }));
    await waitFor(() => {
      expect(requests).toContain("DELETE /api/v1/auth/sessions/session-1");
      expect(requests).toContain("DELETE /api/v1/auth/keys/key-1");
    });
  });
});
