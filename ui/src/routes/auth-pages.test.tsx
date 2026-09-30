import { afterEach, describe, expect, jest, mock, test } from "bun:test";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { ApiProvider } from "../api";
import { resetSessionState } from "../api/adapter";
import { setCsrfToken } from "../api-client";
import { HostContext, type Host } from "../host";
import { mockFetch, restoreFetch } from "../test-fetch";
import * as webcomponentMocks from "../test-webcomponents";

mock.module("@operator/webcomponents", () => webcomponentMocks);

const { LoginPage } = await import("./LoginPage");
const { SetupPage } = await import("./SetupPage");

const TEST_HOST: Host = {
  baseUrl: () => "http://operator.test",
  openExternal: () => undefined,
  browseFolder: () => Promise.resolve(null),
  openFile: () => undefined,
};

function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

function renderAuth(initialPath: string) {
  return render(
    <ApiProvider>
      <HostContext.Provider value={TEST_HOST}>
        <MemoryRouter initialEntries={[initialPath]}>
          <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route path="/setup" element={<SetupPage />} />
            <Route path="/onboarding" element={<p>Onboarding destination</p>} />
            <Route path="/" element={<p>Workspace destination</p>} />
          </Routes>
        </MemoryRouter>
      </HostContext.Provider>
    </ApiProvider>,
  );
}

afterEach(() => {
  cleanup();
  resetSessionState();
  setCsrfToken(null);
  restoreFetch();
  jest.restoreAllMocks();
});

describe("authentication routes", () => {
  test("login redirects an uninitialized server to setup", async () => {
    mockFetch(() =>
      Promise.resolve(json({ state: "uninitialized", requires_temporary_password: false })),
    );
    renderAuth("/login");
    expect(await screen.findByRole("heading", { name: "Set up Operator" })).toBeTruthy();
  });

  test("login maps rate limiting to an actionable message", async () => {
    mockFetch((input, init) => {
      const path = new URL(String(input)).pathname;
      if (path.endsWith("/bootstrap") && init?.method !== "POST") {
        return Promise.resolve(json({ state: "complete", requires_temporary_password: false }));
      }
      return Promise.resolve(json({ message: "slow down" }, 429));
    });
    renderAuth("/login");
    await screen.findByRole("heading", { name: "Sign in to Operator" });
    fireEvent.change(screen.getByLabelText("Username"), { target: { value: "operator" } });
    fireEvent.change(screen.getByLabelText("Password"), { target: { value: "bad password" } });
    fireEvent.click(screen.getByRole("button", { name: "Sign in" }));
    expect(await screen.findByText("Too many attempts. Wait a moment and try again.")).toBeTruthy();
  });

  test("successful login routes an unfinished workspace to onboarding", async () => {
    mockFetch((input, init) => {
      const path = new URL(String(input)).pathname;
      if (path.endsWith("/bootstrap")) {
        return Promise.resolve(json({ state: "complete", requires_temporary_password: false }));
      }
      if (path.endsWith("/login") && init?.method === "POST") {
        return Promise.resolve(
          json({ scopes: ["admin"], expires_at: "2030-01-01T00:00:00Z", csrf_token: "csrf" }),
        );
      }
      if (path.endsWith("/setup/status")) {
        return Promise.resolve(
          json({
            initialized: false,
            admin_configured: true,
            config_path: "operator.toml",
            tickets_path: ".tickets",
            projects_by_tool: {},
            default_acceptance_criteria: "",
          }),
        );
      }
      return Promise.resolve(json({ message: `Unexpected request: ${path}` }, 500));
    });
    renderAuth("/login");
    await screen.findByRole("heading", { name: "Sign in to Operator" });
    fireEvent.change(screen.getByLabelText("Username"), { target: { value: "operator" } });
    fireEvent.change(screen.getByLabelText("Password"), { target: { value: "valid password" } });
    fireEvent.click(screen.getByRole("button", { name: "Sign in" }));
    expect(await screen.findByText("Onboarding destination")).toBeTruthy();
  });

  test("setup redirects complete servers and maps a rejected temporary password", async () => {
    let complete = true;
    mockFetch((input, init) => {
      const path = new URL(String(input)).pathname;
      if (path.endsWith("/bootstrap") && init?.method !== "POST") {
        return Promise.resolve(
          json({
            state: complete ? "complete" : "awaiting_password",
            requires_temporary_password: !complete,
          }),
        );
      }
      return Promise.resolve(json({ message: "invalid bootstrap password" }, 401));
    });
    const first = renderAuth("/setup");
    expect(await screen.findByRole("heading", { name: "Sign in to Operator" })).toBeTruthy();
    first.unmount();
    resetSessionState();
    complete = false;
    renderAuth("/setup");
    await screen.findByRole("heading", { name: "Set up Operator" });
    fireEvent.change(screen.getByLabelText("Temporary password"), {
      target: { value: "temporary" },
    });
    fireEvent.change(screen.getByLabelText("New admin password"), {
      target: { value: "a sufficiently long password" },
    });
    fireEvent.change(screen.getByLabelText("Confirm password"), {
      target: { value: "a sufficiently long password" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Create admin account" }));
    expect(await screen.findByText("The temporary password is incorrect.")).toBeTruthy();
  });
});
