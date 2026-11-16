import { afterEach, describe, expect, mock, test } from "bun:test";
import { act, cleanup, render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { HostContext, type Host } from "../../host";
import { resetSessionState } from "../../api/adapter";
import { ApiProvider } from "../../api/store";
import { mockFetch, requestPath, restoreFetch } from "../../test-fetch";
import * as webcomponentMocks from "../../test-webcomponents";

mock.module("@operator/webcomponents", () => webcomponentMocks);

const { ProfilesProvider } = await import("../../profiles-context");
const { OnboardingPage } = await import("./OnboardingPage");

const DRAFT_KEY = "operator.onboarding-draft.p1";
const PROFILES = [{ id: "p1", name: "e2e", initialized: false, is_default: true }];
const STATUS = {
  initialized: false,
  admin_configured: true,
  config_path: "operator.toml",
  tickets_path: ".tickets",
  projects_by_tool: {},
  default_acceptance_criteria: "",
};
const STEPS = [{ slug: "welcome", name: "Welcome", description: "", help_text: "", order: 0 }];

const ENTRIES = ["/onboarding?step=welcome"];
const PROVIDER = <ProfilesProvider />;
const PAGE = <OnboardingPage />;

const host: Host = {
  baseUrl: () => "http://operator.test",
  openExternal: () => undefined,
  browseFolder: () => Promise.resolve(null),
  openFile: () => undefined,
};

function settle(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 50);
  });
}

function json(body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: { "Content-Type": "application/json" },
  });
}

afterEach(() => {
  cleanup();
  resetSessionState();
  restoreFetch();
  localStorage.clear();
});

describe("OnboardingPage", () => {
  test("keeps a saved draft when status loads before the steps", async () => {
    localStorage.setItem(
      DRAFT_KEY,
      JSON.stringify({
        version: 1,
        draft: { configurationName: "e2e", executionMode: "remote" },
        currentSlug: "welcome",
      }),
    );
    const steps = Promise.withResolvers<undefined>();
    mockFetch(async (input) => {
      const path = requestPath(input);
      if (path === "/api/v1/profiles") {
        return json(PROFILES);
      }
      if (path.endsWith("/setup/status")) {
        return json(STATUS);
      }
      if (path.endsWith("/setup/steps")) {
        await steps.promise;
        return json(STEPS);
      }
      return json([]);
    });

    render(
      <ApiProvider>
        <HostContext.Provider value={host}>
          <MemoryRouter initialEntries={ENTRIES}>
            <Routes>
              <Route element={PROVIDER}>
                <Route path="onboarding" element={PAGE} />
              </Route>
            </Routes>
          </MemoryRouter>
        </HostContext.Provider>
      </ApiProvider>,
    );

    // Status lands first: the page must not persist its empty pre-hydration draft.
    await act(settle);
    await act(async () => {
      steps.resolve(undefined);
      await settle();
    });

    expect(await screen.findByDisplayValue("e2e")).toBeTruthy();
    const saved = JSON.parse(localStorage.getItem(DRAFT_KEY) ?? "null");
    expect(saved.draft).toMatchObject({ configurationName: "e2e", executionMode: "remote" });
  });
});
