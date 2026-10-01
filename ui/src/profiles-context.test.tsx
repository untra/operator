import { afterEach, describe, expect, mock, test } from "bun:test";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useCallback } from "react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { HostContext, type Host } from "./host";
import { resetSessionState } from "./api/adapter";
import { ApiProvider } from "./api/store";
import { mockFetch, restoreFetch } from "./test-fetch";
import * as webcomponentMocks from "./test-webcomponents";

mock.module("@operator/webcomponents", () => webcomponentMocks);

const { ProfileSelector, ProfilesProvider, useProfiles } = await import("./profiles-context");

const PROFILES = [{ id: "p1", name: "Primary", initialized: true, is_default: true }];

const host: Host = {
  baseUrl: () => "http://operator.test",
  openExternal: () => undefined,
  browseFolder: () => Promise.resolve(null),
  openFile: () => undefined,
};

function json(body: unknown, status = 200): Promise<Response> {
  return Promise.resolve(
    new Response(JSON.stringify(body), {
      status,
      headers: { "Content-Type": "application/json" },
    }),
  );
}

function Editor() {
  const { refresh } = useProfiles();
  const onRefresh = useCallback(() => {
    void refresh().catch(() => undefined);
  }, [refresh]);
  return (
    <>
      <ProfileSelector />
      <input aria-label="draft" />
      <button type="button" onClick={onRefresh}>
        refresh
      </button>
    </>
  );
}

function renderProvider() {
  return render(
    <ApiProvider>
      <HostContext.Provider value={host}>
        <MemoryRouter>
          <Routes>
            <Route element={<ProfilesProvider />}>
              <Route index element={<Editor />} />
            </Route>
          </Routes>
        </MemoryRouter>
      </HostContext.Provider>
    </ApiProvider>,
  );
}

afterEach(() => {
  cleanup();
  resetSessionState();
  restoreFetch();
});

describe("ProfilesProvider", () => {
  test("gates the app when the initial load fails", async () => {
    mockFetch(() => json({ error: "boom" }, 500));
    const view = renderProvider();
    await waitFor(() =>
      expect(view.container.querySelector('[data-status="error"]')).not.toBeNull(),
    );
    expect(screen.queryByLabelText("draft")).toBeNull();
  });

  test("keeps routes mounted when a background refetch fails", async () => {
    let calls = 0;
    mockFetch(() => (++calls === 1 ? json(PROFILES) : json({ error: "boom" }, 500)));
    renderProvider();

    const draft = await screen.findByLabelText<HTMLInputElement>("draft");
    fireEvent.change(draft, { target: { value: "unsaved edit" } });

    await act(async () => {
      fireEvent.click(screen.getByText("refresh"));
      await Promise.resolve();
    });

    expect(await screen.findByText(/Couldn't refresh configurations/)).toBeTruthy();
    const current = screen.getByLabelText<HTMLInputElement>("draft");
    expect(current).toBe(draft);
    expect(current.value).toBe("unsaved edit");
    expect(calls).toBe(2);
  });
});
