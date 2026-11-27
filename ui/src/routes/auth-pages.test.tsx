import { afterEach, describe, expect, jest, mock, test } from "bun:test";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { ApiProvider } from "../api";
import { resetSessionState } from "../api/adapter";
import { setCsrfToken } from "../api-client";
import { HostContext, type Host } from "../host";
import { mockFetch as mockRequest, requestPath, restoreFetch } from "../test-fetch";
import * as webcomponentMocks from "../test-webcomponents";

await mock.module("@operator/webcomponents", () => webcomponentMocks);

const { DevicePage } = await import("./DevicePage");
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

function mockFetch(implementation: Parameters<typeof mockRequest>[0]) {
  return mockRequest((input, init) =>
    requestPath(input) === "/api/v1/auth/oidc/providers"
      ? Promise.resolve(json([]))
      : implementation(input, init),
  );
}

function renderAuth(initialPath: string) {
  return render(
    <ApiProvider>
      <HostContext.Provider value={TEST_HOST}>
        <MemoryRouter initialEntries={[initialPath]}>
          <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route path="/setup" element={<SetupPage />} />
            <Route path="/device" element={<DevicePage />} />
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
  test("hosted sign-in carries the device approval code to the configured identity provider", async () => {
    mockRequest((input) =>
      Promise.resolve(
        requestPath(input).endsWith("/providers")
          ? json([{ id: "organization", provider: "keycloak" }])
          : json({ state: "complete", requires_temporary_password: false, oidc_only: true }),
      ),
    );
    renderAuth("/login?user_code=ABCD-EFGH");
    const link = await screen.findByRole("link", { name: "Sign in with keycloak" });
    expect(link.getAttribute("href")).toBe(
      "http://operator.test/api/v1/auth/oidc/login/organization?user_code=ABCD-EFGH",
    );
    expect(screen.queryByLabelText("Password")).toBeNull();
  });

  test("login redirects an uninitialized server to setup", async () => {
    mockFetch(() =>
      Promise.resolve(json({ state: "uninitialized", requires_temporary_password: false })),
    );
    renderAuth("/login");
    expect(await screen.findByRole("heading", { name: "Set up Operator" })).toBeTruthy();
  });

  test("login maps rate limiting to an actionable message", async () => {
    mockFetch((input, init) => {
      const path = requestPath(input);
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

  test.each([
    ["/login", "Onboarding destination"],
    ["/login?user_code=ABCD-EFGH", "Approve"],
  ])("successful login resumes %s", async (route, destination) => {
    mockFetch((input, init) => {
      const path = requestPath(input);
      if (path.endsWith("/bootstrap")) {
        return Promise.resolve(json({ state: "complete", requires_temporary_password: false }));
      }
      if (path.endsWith("/login") && init?.method === "POST") {
        return Promise.resolve(
          json({ scopes: ["admin"], expires_at: "2030-01-01T00:00:00Z", csrf_token: "csrf" }),
        );
      }
      if (path.endsWith("/auth/session")) {
        return Promise.resolve(json({ subject: "operator" }));
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
    renderAuth(route);
    await screen.findByRole("heading", { name: "Sign in to Operator" });
    fireEvent.change(screen.getByLabelText("Username"), { target: { value: "operator" } });
    fireEvent.change(screen.getByLabelText("Password"), { target: { value: "valid password" } });
    fireEvent.click(screen.getByRole("button", { name: "Sign in" }));
    expect(await screen.findByText(destination)).toBeTruthy();
  });

  test("manual verification submits the displayed code only after consent", async () => {
    const approved: string[] = [];
    mockFetch((input, init) => {
      const path = requestPath(input);
      if (path.endsWith("/approve")) {
        if (typeof init?.body === "string") {
          approved.push(init.body);
        }
        return Promise.resolve(json({ client_id: "vscode" }));
      }
      return Promise.resolve(json({ subject: "operator", csrf_token: "csrf" }));
    });
    renderAuth("/device");
    fireEvent.change(screen.getByLabelText("Code"), { target: { value: "abcd-efgh" } });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));
    const confirm = await screen.findByRole("button", { name: "Approve" });
    expect(approved).toEqual([]);
    fireEvent.click(confirm);
    await screen.findByText("Device approved");
    expect(approved).toEqual([JSON.stringify({ user_code: "ABCD-EFGH" })]);
  });

  test("first-time setup returns to the same code for explicit consent", async () => {
    mockFetch((input, init) => {
      const path = requestPath(input);
      if (path.endsWith("/bootstrap")) {
        return Promise.resolve(
          json(init?.method === "POST" ? { username: "operator" } : { state: "uninitialized" }),
        );
      }
      return Promise.resolve(json({ subject: "operator", csrf_token: "csrf" }));
    });
    renderAuth("/login?user_code=ABCD-EFGH");
    await screen.findByLabelText("New admin password");
    for (const label of ["New admin password", "Confirm password"]) {
      fireEvent.change(screen.getByLabelText(label), {
        target: { value: "a sufficiently long password" },
      });
    }
    fireEvent.click(screen.getByRole("button", { name: "Create admin account" }));
    await screen.findByRole("button", { name: "Approve" });
    expect(screen.getByDisplayValue("ABCD-EFGH")).toBeTruthy();
  });

  test("setup redirects complete servers and maps a rejected temporary password", async () => {
    let complete = true;
    mockFetch((input, init) => {
      const path = requestPath(input);
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
