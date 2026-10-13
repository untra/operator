import { afterEach, describe, expect, jest, mock, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { ApiProvider } from "../api";
import { resetSessionState } from "../api/adapter";
import { setCsrfToken } from "../api-client";
import { HostContext, type Host } from "../host";
import { mockFetch, requestBody, requestPath, restoreFetch } from "../test-fetch";
import * as webcomponentMocks from "../test-webcomponents";

mock.module("@operator/webcomponents", () => webcomponentMocks);

const { TicketPage } = await import("./TicketPage");

const opened: string[] = [];

const TEST_HOST: Host = {
  baseUrl: () => "http://operator.test",
  openExternal: (url) => {
    opened.push(url);
  },
  browseFolder: () => Promise.resolve(null),
  openFile: () => undefined,
};

const PAGE = <TicketPage />;
const ENTRY = ["/tickets/FEAT-2"];

function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

type Agent = Record<string, unknown>;

function ticket(column: string, over: Record<string, unknown> = {}, agent: Agent | null = null) {
  return {
    id: "FEAT-2",
    summary: "Serve price tiers",
    ticket_type: "FEAT",
    project: "api",
    status: column === "queue" ? "queued" : "running",
    step: "build",
    priority: "P2-medium",
    timestamp: "20260101-0001",
    content: "---\nid: FEAT-2\n---\n# Body",
    body: "# Serve price tiers\n\nDetails.",
    filename: "20260101-0001-FEAT-api-b.md",
    filepath: "/work/.tickets/in-progress/20260101-0001-FEAT-api-b.md",
    sessions: {},
    step_delegators: {},
    column,
    depends_on: ["FEAT-1", "jira:PROJ-9"],
    dependencies: [
      { reference: "FEAT-1", ticket_id: "FEAT-1", column: "completed", satisfied: true },
      { reference: "jira:PROJ-9", satisfied: false },
    ],
    dependents: ["FEAT-3"],
    llm_task: {},
    agent,
    ...over,
  };
}

const ISSUE_TYPE = {
  key: "FEAT",
  name: "Feature",
  steps: [{ name: "plan" }, { name: "build" }, { name: "review" }],
};

type Requested = { path: string; method: string; body?: string };

function serve(detail: unknown, status = 200) {
  const requests: Requested[] = [];
  mockFetch((input, init) => {
    const path = requestPath(input);
    const method = init?.method ?? "GET";
    requests.push({ path, method, body: requestBody(init) });
    if (path.endsWith("/tickets/FEAT-2")) {
      return Promise.resolve(json(detail, status));
    }
    if (path.includes("/issuetypes/")) {
      return Promise.resolve(json(ISSUE_TYPE));
    }
    if (path.endsWith("/status") || path.endsWith("/focus")) {
      return Promise.resolve(json({ ok: true }));
    }
    return Promise.resolve(json({ delegators: [], targets: [], launch: {} }));
  });
  return requests;
}

function mount() {
  setCsrfToken("csrf");
  return render(
    <ApiProvider>
      <HostContext.Provider value={TEST_HOST}>
        <MemoryRouter initialEntries={ENTRY}>
          <Routes>
            <Route path="/tickets/:id" element={PAGE} />
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
  opened.length = 0;
});

describe("TicketPage", () => {
  test("renders the body without its frontmatter", async () => {
    serve(ticket("in-progress"));
    mount();
    const body = await screen.findByTestId("markdown");
    expect(body.textContent).toBe("# Serve price tiers\n\nDetails.");
    expect(screen.getByRole("heading", { level: 1 }).textContent).toContain("FEAT-2");
  });

  test("links out to the kanban issue when it has one", async () => {
    serve(
      ticket("in-progress", {
        external_provider: "jira",
        external_id: "PROJ-12",
        external_url: "https://acme.atlassian.net/browse/PROJ-12",
      }),
    );
    mount();
    fireEvent.click(await screen.findByRole("button", { name: /PROJ-12/ }));
    expect(opened).toEqual(["https://acme.atlassian.net/browse/PROJ-12"]);
  });

  test("opens the ticket file and the worktree in an editor", async () => {
    serve(ticket("in-progress", { worktree_path: "/work/trees/feat-2" }));
    mount();
    const file = await screen.findByRole("group", { name: "Open ticket file" });
    expect(file.textContent).toContain("JetBrains");
    fireEvent.click(file.querySelector("button") as HTMLButtonElement);
    expect(opened).toEqual(["vscode://open-ticket/FEAT-2"]);
    opened.length = 0;
    const tree = screen.getByRole("group", { name: "Open worktree" });
    fireEvent.click(tree.querySelector("button") as HTMLButtonElement);
    expect(opened).toEqual(["vscode://file/work/trees/feat-2"]);
  });

  test("opens a remote worktree over SSH", async () => {
    serve(
      ticket(
        "in-progress",
        {},
        {
          id: "agent-1",
          status: "running",
          paired: false,
          started_at: "2026-01-01T00:00:00Z",
          completed_steps: ["plan"],
          worktree_path: "/srv/feat-2",
          remote_host: "builder",
        },
      ),
    );
    mount();
    const tree = await screen.findByRole("group", { name: "Open worktree" });
    expect(tree.textContent).not.toContain("JetBrains");
    fireEvent.click(tree.querySelector("button") as HTMLButtonElement);
    expect(opened).toEqual(["vscode://ssh/builder/srv/feat-2"]);
  });

  test("offers launch only in the queue", async () => {
    serve(ticket("queue"));
    mount();
    await screen.findByRole("button", { name: "Launch…" });
    cleanup();
    resetSessionState();

    serve(ticket("in-progress"));
    mount();
    await screen.findByTestId("markdown");
    expect(screen.queryByRole("button", { name: "Launch…" })).toBeNull();
  });

  test("moves the ticket to another column", async () => {
    const requests = serve(ticket("in-progress"));
    mount();
    const select = await screen.findByRole("combobox", { name: "Move to" });
    fireEvent.change(select, { target: { value: "completed" } });
    await waitFor(() => {
      expect(requests.find((r) => r.path.endsWith("/tickets/FEAT-2/status"))?.body).toBe(
        JSON.stringify({ status: "completed" }),
      );
    });
  });

  test("offers review only while the agent awaits it", async () => {
    const agent = {
      id: "agent-1",
      status: "running",
      paired: false,
      started_at: "2026-01-01T00:00:00Z",
      completed_steps: [],
    };
    serve(ticket("in-progress", {}, agent));
    mount();
    await screen.findByTestId("markdown");
    expect(screen.queryByRole("button", { name: "Approve" })).toBeNull();
    cleanup();
    resetSessionState();

    serve(
      ticket("in-progress", {}, { ...agent, status: "awaiting_input", review_state: "pending" }),
    );
    mount();
    expect(await screen.findByRole("button", { name: "Approve" })).toBeTruthy();
  });

  test("links dependencies and dependents to their ticket pages", async () => {
    serve(ticket("in-progress"));
    mount();
    const dep = await screen.findByRole("link", { name: "FEAT-1" });
    expect(dep.getAttribute("href")).toBe("/tickets/FEAT-1");
    expect(screen.getByRole("link", { name: "FEAT-3" }).getAttribute("href")).toBe(
      "/tickets/FEAT-3",
    );
    expect(screen.getByText("jira:PROJ-9")).toBeTruthy();
  });

  test("focuses the agent's session", async () => {
    const requests = serve(
      ticket(
        "in-progress",
        {},
        {
          id: "agent-1",
          status: "running",
          paired: false,
          started_at: "2026-01-01T00:00:00Z",
          completed_steps: [],
          session_wrapper: "cmux",
        },
      ),
    );
    mount();
    fireEvent.click(await screen.findByRole("button", { name: "Focus cmux session" }));
    await waitFor(() => {
      expect(requests.some((r) => r.path.endsWith("/agents/agent-1/focus"))).toBe(true);
    });
  });

  test("says when the ticket does not exist", async () => {
    serve({ error: "not_found", message: "Ticket 'FEAT-2' not found" }, 404);
    mount();
    expect(await screen.findByText("Ticket not found")).toBeTruthy();
  });
});
