import { afterEach, describe, expect, jest, mock, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ApiProvider } from "../../api";
import { resetSessionState } from "../../api/adapter";
import { setCsrfToken } from "../../api-client";
import { HostContext, type Host } from "../../host";
import { mockFetch, restoreFetch } from "../../test-fetch";
import * as webcomponentMocks from "../../test-webcomponents";
import type { WizardDraft } from "./types";

mock.module("@operator/webcomponents", () => webcomponentMocks);

const { STEP_COMPONENTS } = await import("./steps");

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

afterEach(() => {
  cleanup();
  resetSessionState();
  setCsrfToken(null);
  restoreFetch();
  jest.restoreAllMocks();
});

describe("onboarding integration steps", () => {
  test("keeps chained mutations separate when the final write fails", async () => {
    const requests: string[] = [];
    mockFetch((input, init) => {
      const path = new URL(String(input)).pathname;
      const method = init?.method ?? "GET";
      requests.push(`${method} ${path}`);
      if (path.endsWith("/kanban/providers")) {
        return Promise.resolve(
          json([
            {
              slug: "github",
              display_name: "GitHub Projects",
              description: "GitHub board",
              setup_url: "https://example.test",
              icon: "github",
              configured: false,
            },
          ]),
        );
      }
      if (path.endsWith("/kanban/validate")) {
        return Promise.resolve(
          json({ valid: true, github: { user_id: "42", user_login: "octocat" } }),
        );
      }
      if (path.endsWith("/kanban/projects")) {
        return Promise.resolve(
          json({ projects: [{ id: "PVT_1", key: "project-1", name: "octocat/#1" }] }),
        );
      }
      if (path.endsWith("/kanban/statuses")) {
        return Promise.resolve(json({ statuses: ["Todo", "Doing", "Done"] }));
      }
      if (path.endsWith("/kanban/session-env")) {
        return Promise.resolve(json({ shell_export_block: "export OPERATOR_GITHUB_TOKEN=..." }));
      }
      if (path.endsWith("/kanban/config")) {
        return Promise.resolve(json({ message: "configuration write failed" }, 500));
      }
      return Promise.resolve(json({ message: `Unexpected request: ${path}` }, 500));
    });
    setCsrfToken("csrf");
    const exports: string[] = [];
    const draft: WizardDraft = {
      configurationName: "default",
      executionMode: "local",
      premium: false,
      preset: "simple",
      taskFields: [],
      wrapper: "tmux",
      executionTarget: { kind: "local" },
      coderParameters: [],
      useWorktrees: false,
      acceptanceCriteria: "",
      modelServers: [],
      hostedCollectionIds: [],
    };
    const KanbanInfo = STEP_COMPONENTS["kanban-info"];
    render(
      <ApiProvider>
        <HostContext.Provider value={TEST_HOST}>
          <KanbanInfo
            status={{
              initialized: false,
              admin_configured: true,
              config_path: "operator.toml",
              tickets_path: ".tickets",
              projects_by_tool: {},
              default_acceptance_criteria: "",
            }}
            integrations={[]}
            collections={[]}
            creating={false}
            draft={draft}
            setDraft={() => undefined}
            exports={exports}
            addExport={(value) => exports.push(value)}
          />
        </HostContext.Provider>
      </ApiProvider>,
    );
    fireEvent.click(await screen.findByRole("button", { name: /GitHub Projects/ }));
    fireEvent.change(screen.getByLabelText("API token"), { target: { value: "github-token" } });
    fireEvent.click(screen.getByRole("button", { name: "Validate and connect" }));
    await screen.findByText("Credentials validated. Choose a project.");
    fireEvent.change(screen.getByLabelText("Project"), { target: { value: "project-1" } });
    await screen.findByLabelText("todo");
    fireEvent.click(screen.getByRole("button", { name: "Save provider" }));
    expect(await screen.findByText("configuration write failed")).toBeTruthy();
    expect(exports).toEqual([]);
    await waitFor(() => {
      expect(requests).toContain("POST /api/v1/kanban/session-env");
      expect(requests).toContain("PUT /api/v1/kanban/config");
    });
  });
});
