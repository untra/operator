import { afterEach, expect, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { ApiProvider } from "../api";
import { resetSessionState } from "../api/adapter";
import { setCsrfToken } from "../api-client";
import { mockFetch, requestBody, requestPath, restoreFetch } from "../test-fetch";
import { HostContext, type Host } from "../host";
import { NewWorkflowPage } from "./NewWorkflowPage";
import { CreateTicketsPage } from "./CreateTicketsPage";

const plannerEntries = ["/tickets/create?workflow=sample%2FREAD"];

const host: Host = {
  baseUrl: () => "http://operator.test",
  openExternal: () => {},
  browseFolder: () => Promise.resolve(null),
  openFile: () => {},
};
function serve() {
  const requests: { path: string; body?: string }[] = [];
  mockFetch((input, init) => {
    const path = requestPath(input);
    requests.push({ path, body: requestBody(init) });
    let result: unknown = {};
    if (path.endsWith("/collections")) {
      result = [{ name: "sample", types: ["READ"] }];
    }
    if (path.includes("/issuetypes/READ")) {
      result = {
        key: "READ",
        collection: "sample",
        name: "Read",
        fields: [{ name: "summary", user_editable: true }],
        project_required: false,
      };
    }
    if (path.endsWith("/workflows/records")) {
      result = [{ summary: "An opinion" }];
    }
    if (path.endsWith("/batch/preview")) {
      result = {
        valid: true,
        confirmation: "reviewed",
        errors: [],
        previously_created: {},
        tickets: [{ ref: "row-1-1", id: "READ-0001", markdown: "# An opinion", errors: [] }],
      };
    }
    if (path.endsWith("/tickets/batch")) {
      result = { tickets: [{ id: "READ-0001" }] };
    }
    if (path.endsWith("/workflows/sample")) {
      result = "Rendered sample";
    }
    if (path.endsWith("/issuetypes")) {
      result = { key: "READ", collection: "sample" };
    }
    return Promise.resolve(Response.json(result));
  });
  setCsrfToken("csrf");
  return requests;
}
afterEach(() => {
  cleanup();
  resetSessionState();
  restoreFetch();
  setCsrfToken(null);
});
test("sampling and saving a workflow only submit its definition and sample", async () => {
  const requests = serve();
  render(
    <ApiProvider>
      <HostContext.Provider value={host}>
        <MemoryRouter>
          <NewWorkflowPage />
        </MemoryRouter>
      </HostContext.Provider>
    </ApiProvider>,
  );
  fireEvent.change(screen.getByLabelText("Name"), { target: { value: "Read" } });
  fireEvent.change(screen.getByLabelText("Outcome"), { target: { value: "Produce a report" } });
  fireEvent.change(screen.getByLabelText("Sample value"), { target: { value: "An opinion" } });
  fireEvent.change(screen.getByLabelText("Instructions and expected output"), {
    target: { value: "Read {{summary}}" },
  });
  fireEvent.click(screen.getByText("Preview sample"));
  await waitFor(() =>
    expect(requests.find((r) => r.path.endsWith("/sample"))?.body).toContain("An opinion"),
  );
  fireEvent.click(screen.getByText("Save workflow"));
  await waitFor(() =>
    expect(requests.find((r) => r.path.endsWith("/issuetypes"))?.body).toContain(
      "Read {{summary}}",
    ),
  );
  expect(requests.some((r) => r.path.includes("/tickets"))).toBe(false);
});
test("pasted records are mapped, previewed, and created only after confirmation", async () => {
  const requests = serve();
  render(
    <ApiProvider>
      <HostContext.Provider value={host}>
        <MemoryRouter initialEntries={plannerEntries}>
          <CreateTicketsPage />
        </MemoryRouter>
      </HostContext.Provider>
    </ApiProvider>,
  );
  fireEvent.change(await screen.findByLabelText("Ticket source"), { target: { value: "rows" } });
  fireEvent.change(screen.getByLabelText("Paste records"), {
    target: { value: "summary\nAn opinion" },
  });
  fireEvent.click(screen.getByText("Read records"));
  await waitFor(() =>
    expect(requests.find((r) => r.path.endsWith("/records"))?.body).toBe(
      JSON.stringify({ text: "summary\nAn opinion" }),
    ),
  );
  fireEvent.change(await screen.findByLabelText("Value from"), { target: { value: "column" } });
  fireEvent.click(screen.getByText("Preview tickets"));
  await screen.findByText("# An opinion");
  expect(requests.some((r) => r.path.endsWith("/batch"))).toBe(false);
  fireEvent.click(screen.getByRole("button", { name: "Create 1 ticket" }));
  await screen.findByText("Created 1 ticket");
  const confirmed = requests.find((r) => r.path.endsWith("/batch"))?.body;
  expect(JSON.parse(confirmed ?? "{}")).toMatchObject({
    confirmation: "reviewed",
    dataset: {
      recipe: { collection: "sample", workflow: "READ", columns: { summary: "summary" } },
    },
  });
});
