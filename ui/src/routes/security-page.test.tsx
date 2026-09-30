import { afterEach, describe, expect, jest, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ApiProvider } from "../api";
import { resetSessionState } from "../api/adapter";
import { setCsrfToken } from "../api-client";
import { HostContext, type Host } from "../host";

const { SecurityPage } = await import("./SecurityPage");

const TEST_HOST: Host = {
  baseUrl: () => "http://operator.test",
  openExternal: () => undefined,
  browseFolder: () => Promise.resolve(null),
  openFile: () => undefined,
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
  jest.restoreAllMocks();
});

describe("SecurityPage", () => {
  test("creates and revokes credentials through mutation lifecycles", async () => {
    const requests: string[] = [];
    globalThis.fetch = jest.fn((input, init) => {
      const path = new URL(String(input)).pathname;
      const method = init?.method ?? "GET";
      requests.push(`${method} ${path}`);
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
    }) as typeof fetch;
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
    fireEvent.click(screen.getByRole("button", { name: "Create key" }));
    expect(await screen.findByText("operator-secret-once")).toBeTruthy();
    const revokeButtons = screen.getAllByRole("button", { name: "Revoke" });
    fireEvent.click(revokeButtons[0]);
    fireEvent.click(revokeButtons[1]);
    await waitFor(() => {
      expect(requests).toContain("DELETE /api/v1/auth/sessions/session-1");
      expect(requests).toContain("DELETE /api/v1/auth/keys/key-1");
    });
  });
});
