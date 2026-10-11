/**
 * Campaign tree provider tests: paywall without Premium, campaign → wave →
 * ticket children with it, and a context value per lifecycle state.
 */

import * as assert from "node:assert";
import * as sinon from "sinon";
import { CampaignTreeProvider, type CampaignNode } from "../../src/campaign-provider";
import { OperatorApiClient } from "../../src/api-client";
import { launchSummary } from "../../src/campaign-commands";
import type {
  CampaignListResponse,
  CampaignStatus,
  CampaignStatusView,
  LicenseResponse,
} from "../../src/generated";

function license(premium: boolean): LicenseResponse {
  return { status: "valid", profile_id: "default", premium, terms: null, purchase_url: null };
}

function campaigns(...entries: Array<[string, CampaignStatus]>): CampaignListResponse {
  return {
    campaigns: entries.map(([id, status]) => ({
      campaign: { id, title: `${id} title`, status, created: "2026-10-05" },
      members: 2,
    })),
    total: entries.length,
  };
}

function statusView(id: string): CampaignStatusView {
  const member = (ticket: string, depends_on: string[]) => ({
    id: ticket,
    summary: `${ticket} summary`,
    project: "api",
    column: "queue",
    depends_on,
    launches: 0,
  });
  return {
    id,
    status: "draft",
    tickets: [member("FEAT-0001", []), member("FEAT-0002", ["FEAT-0001"])],
    waves: [["FEAT-0001"], ["FEAT-0002"]],
    report: {
      status: "draft",
      launched: [],
      awaiting_operator: [],
      stalled: [],
      blocked: { "FEAT-0002": { kind: "dependencies" } },
      pause_reason: null,
    },
    counters: {
      launched: 0,
      max_launches: null,
      active: 0,
      max_parallel: null,
      retries: 0,
      max_retries: null,
    },
    policy: { name: "campaign", require_confirmation: false },
  };
}

suite("CampaignTreeProvider", () => {
  let sandbox: sinon.SinonSandbox;
  let client: OperatorApiClient;

  setup(() => {
    sandbox = sinon.createSandbox();
    client = new OperatorApiClient("http://localhost:1");
  });

  teardown(() => {
    sandbox.restore();
  });

  async function roots(provider: CampaignTreeProvider): Promise<CampaignNode[]> {
    await provider.refresh();
    return provider.getChildren();
  }

  test("is empty without a server", async () => {
    const provider = new CampaignTreeProvider();
    assert.deepStrictEqual(await roots(provider), []);
  });

  test("shows one paywall row without Premium and lists nothing", async () => {
    sandbox.stub(client, "license").resolves(license(false));
    const list = sandbox.stub(client, "listCampaigns").resolves(campaigns(["CAMP-0001", "draft"]));
    const provider = new CampaignTreeProvider();
    provider.setApiClient(client);

    const nodes = await roots(provider);
    assert.strictEqual(nodes.length, 1);
    assert.strictEqual(nodes[0]?.kind, "paywall");
    assert.strictEqual(nodes[0]?.command?.command, "operator.openLicense");
    assert.ok(list.notCalled, "campaigns must not be fetched without Premium");
  });

  test("lists campaigns with a context value per lifecycle state", async () => {
    sandbox.stub(client, "license").resolves(license(true));
    sandbox
      .stub(client, "listCampaigns")
      .resolves(
        campaigns(["CAMP-0001", "draft"], ["CAMP-0002", "running"], ["CAMP-0003", "completed"]),
      );
    const provider = new CampaignTreeProvider();
    provider.setApiClient(client);

    const nodes = await roots(provider);
    assert.deepStrictEqual(
      nodes.map((n) => [n.label, n.contextValue]),
      [
        ["CAMP-0001 · CAMP-0001 title", "campaign-launchable"],
        ["CAMP-0002 · CAMP-0002 title", "campaign-running"],
        ["CAMP-0003 · CAMP-0003 title", "campaign"],
      ],
    );
  });

  test("expands a campaign into waves, and a wave into its tickets", async () => {
    sandbox.stub(client, "license").resolves(license(true));
    sandbox.stub(client, "listCampaigns").resolves(campaigns(["CAMP-0001", "draft"]));
    sandbox.stub(client, "campaignStatus").resolves(statusView("CAMP-0001"));
    const provider = new CampaignTreeProvider();
    provider.setApiClient(client);

    const [campaign] = await roots(provider);
    assert.ok(campaign);
    const waves = await provider.getChildren(campaign);
    assert.deepStrictEqual(
      waves.map((w) => w.label),
      ["Wave 1", "Wave 2"],
    );
    const second = waves[1];
    assert.ok(second);
    const tickets = await provider.getChildren(second);
    assert.deepStrictEqual(
      tickets.map((t) => [t.label, t.description]),
      [["FEAT-0002", "queue · waiting on dependencies"]],
    );
  });

  test("shows an error row when the server refuses", async () => {
    sandbox.stub(client, "license").rejects(new Error("boom"));
    const provider = new CampaignTreeProvider();
    provider.setApiClient(client);

    const nodes = await roots(provider);
    assert.strictEqual(nodes.length, 1);
    assert.strictEqual(nodes[0]?.kind, "message");
  });
});

suite("launchSummary", () => {
  test("names the waves, the first launches and the policy", () => {
    const summary = launchSummary({
      id: "CAMP-0001",
      dry_run: true,
      status: "draft",
      waves: [["FEAT-0001"], ["FEAT-0002"]],
      policy: { name: "careful", require_confirmation: false },
      violations: [],
      report: {
        status: "draft",
        launched: ["FEAT-0001"],
        awaiting_operator: [],
        stalled: [],
        blocked: {},
        pause_reason: null,
      },
    });
    assert.strictEqual(
      summary,
      "CAMP-0001: 2 waves; the first launches FEAT-0001. Governance: careful.",
    );
  });
});
