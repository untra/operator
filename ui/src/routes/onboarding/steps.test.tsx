import { afterEach, describe, expect, jest, mock, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ApiProvider } from "../../api";
import { resetSessionState } from "../../api/adapter";
import { setCsrfToken } from "../../api-client";
import { HostContext, type Host } from "../../host";
import { mockFetch, requestBody, requestPath, restoreFetch } from "../../test-fetch";
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
const EMPTY_VALUES: never[] = [];
const SETUP_STATUS = {
  initialized: false,
  admin_configured: true,
  config_path: "operator.toml",
  tickets_path: ".tickets",
  projects_by_tool: {},
  default_acceptance_criteria: "",
};
let recordedExports: string[] = [];

function ignoreDraftUpdate(): void {}

function recordExport(value: string): void {
  recordedExports.push(value);
}

function json(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

const DRAFT: WizardDraft = {
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

const GITHUB_PROVIDER = {
  slug: "github",
  display_name: "GitHub Projects",
  description: "GitHub board",
  setup_url: "https://example.test",
  icon: "github",
  configured: false,
};

function renderKanbanInfo() {
  const KanbanInfo = STEP_COMPONENTS["kanban-info"];
  return render(
    <ApiProvider>
      <HostContext.Provider value={TEST_HOST}>
        <KanbanInfo
          status={SETUP_STATUS}
          integrations={EMPTY_VALUES}
          collections={EMPTY_VALUES}
          creating={false}
          draft={DRAFT}
          setDraft={ignoreDraftUpdate}
          exports={recordedExports}
          addExport={recordExport}
        />
      </HostContext.Provider>
    </ApiProvider>,
  );
}

afterEach(() => {
  cleanup();
  resetSessionState();
  setCsrfToken(null);
  recordedExports = [];
  restoreFetch();
  jest.restoreAllMocks();
});

describe("onboarding integration steps", () => {
  test("keeps chained mutations separate when the final write fails", async () => {
    const requests: string[] = [];
    mockFetch((input, init) => {
      const path = requestPath(input);
      const method = init?.method ?? "GET";
      requests.push(`${method} ${path}`);
      if (path.endsWith("/kanban/providers")) {
        return Promise.resolve(json([GITHUB_PROVIDER]));
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
    renderKanbanInfo();
    fireEvent.click(await screen.findByRole("button", { name: /GitHub Projects/ }));
    fireEvent.change(screen.getByLabelText("API token"), { target: { value: "github-token" } });
    fireEvent.click(screen.getByRole("button", { name: "Validate and connect" }));
    await screen.findByText("Credentials validated. Choose a project.");
    fireEvent.change(screen.getByLabelText("Project"), { target: { value: "project-1" } });
    await screen.findByLabelText("todo");
    fireEvent.click(screen.getByRole("button", { name: "Save provider" }));
    expect(await screen.findByText("configuration write failed")).toBeTruthy();
    expect(recordedExports).toEqual([]);
    await waitFor(() => {
      expect(requests).toContain("POST /api/v1/kanban/session-env");
      expect(requests).toContain("PUT /api/v1/kanban/config");
    });
  });
  test("maps columns from the selected project's statuses, not an earlier pick", async () => {
    const releases = new Map<string, () => void>();
    const configBodies: string[] = [];
    mockFetch((input, init) => {
      const path = requestPath(input);
      if (path.endsWith("/kanban/providers")) {
        return Promise.resolve(json([GITHUB_PROVIDER]));
      }
      if (path.endsWith("/kanban/validate")) {
        return Promise.resolve(
          json({ valid: true, github: { user_id: "42", user_login: "octocat" } }),
        );
      }
      if (path.endsWith("/kanban/projects")) {
        return Promise.resolve(
          json({
            projects: [
              { id: "PVT_A", key: "project-a", name: "octocat/#1" },
              { id: "PVT_B", key: "project-b", name: "octocat/#2" },
            ],
          }),
        );
      }
      if (path.endsWith("/kanban/statuses")) {
        const projectKey = (JSON.parse(requestBody(init) ?? "{}") as { project_key: string })
          .project_key;
        const statuses =
          projectKey === "project-a"
            ? ["A-todo", "A-doing", "A-done"]
            : ["B-todo", "B-doing", "B-done"];
        return new Promise<Response>((resolve) => {
          releases.set(projectKey, () => resolve(json({ statuses })));
        });
      }
      if (path.endsWith("/kanban/session-env")) {
        return Promise.resolve(json({ shell_export_block: "export OPERATOR_GITHUB_TOKEN=..." }));
      }
      if (path.endsWith("/kanban/config")) {
        configBodies.push(requestBody(init) ?? "");
        return Promise.resolve(json({ message: "ok" }));
      }
      return Promise.resolve(json({ message: `Unexpected request: ${path}` }, 500));
    });
    setCsrfToken("csrf");
    renderKanbanInfo();
    fireEvent.click(await screen.findByRole("button", { name: /GitHub Projects/ }));
    fireEvent.change(screen.getByLabelText("API token"), { target: { value: "github-token" } });
    fireEvent.click(screen.getByRole("button", { name: "Validate and connect" }));
    await screen.findByText("Credentials validated. Choose a project.");

    const project = screen.getByLabelText<HTMLSelectElement>("Project");
    fireEvent.change(project, { target: { value: "project-a" } });
    await waitFor(() => expect(releases.has("project-a")).toBe(true));
    expect(project.disabled).toBe(true);

    releases.get("project-a")?.();
    await waitFor(() => expect(project.disabled).toBe(false));
    fireEvent.change(project, { target: { value: "project-b" } });
    expect(screen.queryByLabelText("todo")).toBeNull();
    await waitFor(() => expect(releases.has("project-b")).toBe(true));
    releases.get("project-b")?.();
    await waitFor(() =>
      expect(screen.getByLabelText<HTMLSelectElement>("todo").value).toBe("B-todo"),
    );

    fireEvent.click(screen.getByRole("button", { name: "Save provider" }));
    await waitFor(() => expect(configBodies).toHaveLength(1));
    const saved = JSON.parse(configBodies[0]) as {
      github: { project_key: string; status_mapping: { todo: string } };
    };
    expect(saved.github.project_key).toBe("PVT_B");
    expect(saved.github.status_mapping.todo).toBe("B-todo");
  });
});

test("Gitea onboarding saves the selected repository board with separate credentials", async () => {
  const writes: string[] = [];
  const projectKey = "https://forge.test/team/repo/projects/1";
  mockFetch((input, init) => {
    const path = requestPath(input);
    if (path.endsWith("/kanban/providers")) {
      return Promise.resolve(
        json([{ ...GITHUB_PROVIDER, slug: "gitea", display_name: "Gitea Projects" }]),
      );
    }
    if (path.endsWith("/kanban/validate")) {
      return Promise.resolve(
        json({
          valid: true,
          gitea: { user_id: "4", user_name: "Agent", users: [{ id: "4", name: "Agent" }] },
        }),
      );
    }
    if (path.endsWith("/kanban/projects")) {
      return Promise.resolve(
        json({ projects: [{ id: projectKey, key: projectKey, name: "Work" }] }),
      );
    }
    if (path.endsWith("/kanban/statuses")) {
      return Promise.resolve(json({ statuses: ["Todo", "Done"] }));
    }
    if (path.endsWith("/kanban/session-env")) {
      return Promise.resolve(json({ shell_export_block: "export OPERATOR_GITEA_TOKEN=..." }));
    }
    if (path.endsWith("/kanban/config")) {
      writes.push(String(requestBody(init)));
      return Promise.resolve(json({ written_path: "config.toml", section_header: "gitea" }));
    }
    return Promise.resolve(json({ message: path }, 500));
  });
  setCsrfToken("csrf");
  renderKanbanInfo();
  fireEvent.click(await screen.findByRole("button", { name: /Gitea Projects/ }));
  fireEvent.change(screen.getByLabelText("Instance URL"), {
    target: { value: "https://forge.test" },
  });
  fireEvent.change(screen.getByLabelText("Repository"), { target: { value: "team/repo" } });
  fireEvent.change(screen.getByLabelText("API token"), { target: { value: "board-token" } });
  fireEvent.click(screen.getByRole("button", { name: "Validate and connect" }));
  await screen.findByText("Credentials validated. Choose a project.");
  fireEvent.change(screen.getByLabelText("Project"), { target: { value: projectKey } });
  await screen.findByLabelText("todo");
  fireEvent.click(screen.getByRole("button", { name: "Save provider" }));
  await waitFor(() => expect(writes).toHaveLength(1));
  expect(JSON.parse(writes[0] ?? "{}")).toMatchObject({
    provider: "gitea",
    gitea: {
      base_url: "https://forge.test",
      repositories: ["team/repo"],
      project_key: projectKey,
      api_key_env: "OPERATOR_GITEA_TOKEN",
      sync_user_id: "4",
    },
  });
  expect(writes[0]).not.toContain("board-token");
});
