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

const { CampaignsPage } = await import("./CampaignsPage");
const { CampaignDetailPage } = await import("./CampaignDetailPage");

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

function license(premium: boolean) {
  return { status: "valid", profile_id: "default", premium, terms: null, purchase_url: null };
}

const CAMPAIGN = {
  id: "CAMP-0001",
  title: "Pricing overhaul",
  status: "draft",
  created: "2026-10-05",
};

const POLICY = { name: "campaign", require_confirmation: false };

function member(id: string, column: string, depends_on: string[] = []) {
  return { id, summary: `${id} summary`, project: "api", column, depends_on, launches: 0 };
}

const MEMBERS = [member("FEAT-0001", "queue"), member("FEAT-0002", "queue", ["FEAT-0001"])];

const WAVES = [["FEAT-0001"], ["FEAT-0002"]];

function report(status: string) {
  return {
    status,
    launched: [],
    awaiting_operator: [],
    stalled: [],
    blocked: { "FEAT-0002": { kind: "dependencies" } },
    pause_reason: null,
  };
}

function statusView(status: string) {
  return {
    id: CAMPAIGN.id,
    status,
    tickets: MEMBERS,
    waves: WAVES,
    report: report(status),
    counters: {
      launched: 0,
      max_launches: 5,
      active: 0,
      max_parallel: 2,
      retries: 0,
      max_retries: null,
    },
    policy: POLICY,
  };
}

type Requested = { path: string; method: string; body?: string };

function serve(premium: boolean, status = "draft") {
  const requests: Requested[] = [];
  mockFetch((input, init) => {
    const path = requestPath(input);
    const method = init?.method ?? "GET";
    requests.push({ path, method, body: requestBody(init) });
    if (path.endsWith("/license")) {
      return Promise.resolve(json(license(premium)));
    }
    if (path.endsWith("/agents/active")) {
      return Promise.resolve(json({ agents: [], count: 0 }));
    }
    if (path.endsWith("/campaigns")) {
      return Promise.resolve(
        json({ campaigns: [{ campaign: { ...CAMPAIGN, status }, members: 2 }], total: 1 }),
      );
    }
    if (path.endsWith("/status")) {
      return Promise.resolve(json(statusView(status)));
    }
    if (path.endsWith("/launch")) {
      const dryRun = (JSON.parse(requestBody(init) ?? "{}") as { dry_run: boolean }).dry_run;
      return Promise.resolve(
        json({
          id: CAMPAIGN.id,
          dry_run: dryRun,
          status: dryRun ? status : "running",
          waves: WAVES,
          policy: POLICY,
          violations: [],
          report: report(dryRun ? status : "running"),
        }),
      );
    }
    if (path.endsWith(`/campaigns/${CAMPAIGN.id}`)) {
      return Promise.resolve(
        json({
          campaign: { ...CAMPAIGN, status },
          members: MEMBERS,
          waves: WAVES,
          gates: {},
          unsynced: {},
        }),
      );
    }
    return Promise.resolve(json({}));
  });
  return requests;
}

const LIST_PAGE = <CampaignsPage />;
const DETAIL_PAGE = <CampaignDetailPage />;
const LIST_ENTRY = ["/campaigns"];
const DETAIL_ENTRY = ["/campaigns/CAMP-0001"];

function mount(entry: string[]) {
  setCsrfToken("csrf");
  return render(
    <ApiProvider>
      <HostContext.Provider value={TEST_HOST}>
        <MemoryRouter initialEntries={entry}>
          <Routes>
            <Route path="/campaigns" element={LIST_PAGE} />
            <Route path="/campaigns/:id" element={DETAIL_PAGE} />
          </Routes>
        </MemoryRouter>
      </HostContext.Provider>
    </ApiProvider>,
  );
}

function campaignCalls(requests: Requested[]): Requested[] {
  return requests.filter((r) => r.path.includes("/campaigns"));
}

afterEach(() => {
  cleanup();
  resetSessionState();
  setCsrfToken(null);
  restoreFetch();
  jest.restoreAllMocks();
});

describe("CampaignsPage", () => {
  test("shows the paywall without Premium and never fetches campaigns", async () => {
    const requests = serve(false);
    mount(LIST_ENTRY);
    await screen.findByText("Premium required");
    expect(campaignCalls(requests)).toEqual([]);
  });

  test("lists campaigns with Premium", async () => {
    serve(true);
    mount(LIST_ENTRY);
    const link = await screen.findByRole("link", { name: /CAMP-0001/ });
    expect(link.getAttribute("href")).toBe("/campaigns/CAMP-0001");
    expect(screen.getByText("Pricing overhaul")).toBeTruthy();
  });
});

describe("CampaignDetailPage", () => {
  test("shows the paywall without Premium and never fetches the campaign", async () => {
    const requests = serve(false);
    mount(DETAIL_ENTRY);
    await screen.findByText("Premium required");
    expect(campaignCalls(requests)).toEqual([]);
  });

  test("renders members wave by wave", async () => {
    serve(true);
    mount(DETAIL_ENTRY);
    const waves = await screen.findAllByRole("region", { name: /^Wave / });
    expect(waves.map((w) => w.getAttribute("aria-label"))).toEqual(["Wave 1", "Wave 2"]);
    expect(waves[0]?.textContent).toContain("FEAT-0001");
    expect(waves[1]?.textContent).toContain("FEAT-0002");
    expect(waves[1]?.textContent).toContain("waiting on dependencies");
    const link = screen.getByRole("link", { name: "FEAT-0001" });
    expect(link.getAttribute("href")).toBe("/tickets/FEAT-0001");
  });

  test("previews with a dry run before launching", async () => {
    const requests = serve(true);
    mount(DETAIL_ENTRY);
    fireEvent.click(await screen.findByRole("button", { name: "Launch…" }));
    fireEvent.click(await screen.findByRole("button", { name: "Confirm launch" }));
    await waitFor(() => {
      expect(requests.filter((r) => r.path.endsWith("/launch")).length).toBe(2);
    });
    const launches = requests.filter((r) => r.path.endsWith("/launch")).map((r) => r.body);
    expect(launches).toEqual([
      JSON.stringify({ dry_run: true }),
      JSON.stringify({ dry_run: false }),
    ]);
  });

  test("offers pause only while running", async () => {
    serve(true, "draft");
    mount(DETAIL_ENTRY);
    await screen.findByRole("button", { name: "Launch…" });
    expect(screen.queryByRole("button", { name: "Pause" })).toBeNull();
    cleanup();
    resetSessionState();

    const requests = serve(true, "running");
    mount(DETAIL_ENTRY);
    fireEvent.click(await screen.findByRole("button", { name: "Pause" }));
    await waitFor(() => {
      expect(requests.some((r) => r.path.endsWith("/pause") && r.method === "POST")).toBe(true);
    });
  });
});
