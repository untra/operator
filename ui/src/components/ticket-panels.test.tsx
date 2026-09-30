import { afterEach, describe, expect, jest, mock, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { MemoryRouter } from "react-router-dom";
import { ApiProvider } from "../api";
import { resetSessionState } from "../api/adapter";
import { setCsrfToken } from "../api-client";
import { HostContext, type Host } from "../host";
import { RightPanelProvider } from "../right-panel";

mock.module("@operator/webcomponents", () => ({
  TicketCreateForm: ({
    onChange,
    onSubmit,
  }: {
    onChange: (value: { issueType: string; project: string; summary: string }) => void;
    onSubmit: () => void;
  }) => (
    <>
      <button
        type="button"
        onClick={() => onChange({ issueType: "TASK", project: "operator", summary: "Test" })}
      >
        Fill ticket
      </button>
      <button type="button" onClick={onSubmit}>
        Submit ticket
      </button>
    </>
  ),
  TicketDetailView: ({
    launchControls,
    launchActions,
  }: {
    launchControls?: ReactNode;
    launchActions?: ReactNode;
  }) => (
    <div>
      {launchControls}
      {launchActions}
    </div>
  ),
  LaunchForm: ({ onSubmit }: { onSubmit: () => void }) => (
    <button type="button" onClick={onSubmit}>
      Launch ticket
    </button>
  ),
}));

const { TicketCreatePanel } = await import("./TicketCreatePanel");
const { TicketDetailPanel } = await import("./TicketDetailPanel");

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

function renderPanel(content: ReactNode) {
  return render(
    <ApiProvider>
      <HostContext.Provider value={TEST_HOST}>
        <MemoryRouter>
          <RightPanelProvider>{content}</RightPanelProvider>
        </MemoryRouter>
      </HostContext.Provider>
    </ApiProvider>,
  );
}

afterEach(() => {
  cleanup();
  resetSessionState();
  setCsrfToken(null);
  jest.restoreAllMocks();
});

describe("ticket panels", () => {
  test("creates a ticket with the selected type, project, and summary", async () => {
    const requests: Array<{ path: string; method: string; body?: string }> = [];
    globalThis.fetch = jest.fn((input, init) => {
      const path = new URL(String(input)).pathname;
      requests.push({ path, method: init?.method ?? "GET", body: init?.body?.toString() });
      if (path.endsWith("/issuetypes")) {
        return Promise.resolve(json([{ key: "TASK", name: "Task" }]));
      }
      if (path.endsWith("/projects")) {
        return Promise.resolve(json([{ name: "operator", path: "/operator", exists: true }]));
      }
      if (path.endsWith("/tickets")) {
        return Promise.resolve(json({ id: "TASK-1", filename: "TASK-1.md", path: "/ticket" }));
      }
      return Promise.resolve(json({ message: `Unexpected request: ${path}` }, 500));
    }) as typeof fetch;
    setCsrfToken("csrf");
    const created: string[] = [];
    renderPanel(<TicketCreatePanel onCreated={() => created.push("created")} />);
    await screen.findByRole("button", { name: "Fill ticket" });
    fireEvent.click(screen.getByRole("button", { name: "Fill ticket" }));
    fireEvent.click(screen.getByRole("button", { name: "Submit ticket" }));
    await waitFor(() => expect(created).toEqual(["created"]));
    const request = requests.find((item) => item.path.endsWith("/tickets"));
    expect(request?.method).toBe("POST");
    expect(JSON.parse(request?.body ?? "{}")).toEqual({
      template: "TASK",
      project: "operator",
      summary: "Test",
      values: {},
    });
  });

  test("launches a ticket and focuses its cmux session", async () => {
    const requests: string[] = [];
    globalThis.fetch = jest.fn((input, init) => {
      const path = new URL(String(input)).pathname;
      requests.push(`${init?.method ?? "GET"} ${path}`);
      if (path.endsWith("/configuration")) {
        return Promise.resolve(json({ launch: { session_wrapper: "cmux" } }));
      }
      if (path.endsWith("/delegators")) {
        return Promise.resolve(json({ delegators: [], total: 0 }));
      }
      if (path.endsWith("/execution-targets")) {
        return Promise.resolve(json({ targets: [], total: 0 }));
      }
      if (path.endsWith("/issuetypes/TASK/document")) {
        return Promise.resolve(json({ key: "TASK", name: "Task", steps: [] }));
      }
      if (path.endsWith("/tickets/TASK-1/launch")) {
        return Promise.resolve(
          json({
            executed_server_side: true,
            agent_id: "agent-1",
            ticket_id: "TASK-1",
            working_directory: "/operator",
            command: "",
            terminal_name: "op-task-1",
            tmux_session_name: "op-task-1",
            session_wrapper: "cmux",
            session_window_ref: null,
            session_context_ref: null,
            session_id: "session-1",
            worktree_created: false,
            branch: null,
          }),
        );
      }
      if (path.endsWith("/agents/agent-1/focus")) {
        return Promise.resolve(new Response(null, { status: 204 }));
      }
      return Promise.resolve(json({ message: `Unexpected request: ${path}` }, 500));
    }) as typeof fetch;
    setCsrfToken("csrf");
    renderPanel(
      <TicketDetailPanel
        ticket={{
          id: "TASK-1",
          summary: "Test",
          ticket_type: "TASK",
          project: "operator",
          status: "queued",
          step: "implement",
          step_display_name: "Implement",
          priority: "P2-medium",
          timestamp: "20260930-1200",
          filename: "TASK-1.md",
        }}
      />,
    );
    fireEvent.click(await screen.findByRole("button", { name: "Launch ticket" }));
    fireEvent.click(await screen.findByRole("button", { name: "Focus cmux session" }));
    await screen.findByRole("button", { name: /Focus cmux session/ });
    expect(requests).toContain("POST /api/v1/tickets/TASK-1/launch");
    expect(requests).toContain("POST /api/v1/agents/agent-1/focus");
  });
});
