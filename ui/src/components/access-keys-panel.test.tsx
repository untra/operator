import { afterEach, expect, mock, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { ApiProvider, resetSessionState } from "../api";
import { setCsrfToken } from "../api-client";
import { HostContext, type Host } from "../host";
import { mockFetch, requestPath, restoreFetch } from "../test-fetch";
import * as webcomponentMocks from "../test-webcomponents";

mock.module("@operator/webcomponents", () => webcomponentMocks);
const { AccessKeysPanel } = await import("./AccessKeysPanel");

const NOW = Date.parse("2026-10-14T12:00:00Z");
const clock = () => NOW;
const host: Host = {
  baseUrl: () => "https://operator.test",
  openExternal: () => undefined,
  browseFolder: () => Promise.resolve(null),
  openFile: () => undefined,
};
const grants = {
  profile_id: "00000000-0000-0000-0000-000000000001",
  capabilities: ["tickets:read", "tickets:create"],
  projects: ["operator"],
  instance_wide: false,
};
const key = {
  id: "key-1", name: "CI", grants, profile_id: grants.profile_id,
  display_prefix: "opk_v1_key-1_…", created_at: "2026-10-01T00:00:00Z",
  expires_at: "2026-11-01T00:00:00Z", revoked_at: null,
};

function setup(expiresAt = key.expires_at, issuanceExpiresAt = "2026-10-14T12:05:00Z") {
  const writes: { path: string; body: unknown }[] = [];
  mockFetch((input, init) => {
    const path = requestPath(input);
    let body: unknown = {};
    if (path.endsWith("/capabilities")) {
      body = { grants, oidc_user: true, owner_id: "owner", can_issue: true, issuance_expires_at: issuanceExpiresAt };
    } else if ((init?.method ?? "GET") === "GET") {
      body = { keys: [{ ...key, expires_at: expiresAt }] };
    } else {
      writes.push({ path, body: JSON.parse(typeof init?.body === "string" ? init.body : "null") });
      body = { key, secret: "one-time-secret" };
    }
    return Promise.resolve(new Response(JSON.stringify(body), { headers: { "Content-Type": "application/json" } }));
  });
  setCsrfToken("csrf");
  render(<HostContext value={host}><ApiProvider><AccessKeysPanel clock={clock} /></ApiProvider></HostContext>);
  return writes;
}

afterEach(() => {
  cleanup();
  resetSessionState();
  setCsrfToken(null);
  restoreFetch();
});

test("expired keys cannot rotate", async () => {
  const writes = setup("2026-10-14T11:59:59Z");
  fireEvent.click(await screen.findByRole("button", { name: "Rotate" }));
  expect(writes).toEqual([]);
});

test("rotation requires confirmation and can narrow grants", async () => {
  const writes = setup();
  fireEvent.click(await screen.findByRole("button", { name: "Rotate" }));
  expect(writes).toEqual([]);
  const confirmation = within(screen.getByRole("group", { name: "Confirm key action" }));
  fireEvent.click(confirmation.getByLabelText("tickets:create"));
  fireEvent.click(confirmation.getByRole("button", { name: "Confirm rotate" }));
  await waitFor(() => expect(writes).toEqual([{
    path: "/api/v1/auth/keys/key-1/rotate",
    body: { name: "CI", expires_in_days: 90, grants: { ...grants, capabilities: ["tickets:read"] } },
  }]));
});

test("expired reauthentication blocks issuance despite cached eligibility", async () => {
  const writes = setup(key.expires_at, "2026-10-14T11:59:59Z");
  fireEvent.change(await screen.findByRole("textbox", { name: "Name" }), { target: { value: "new" } });
  fireEvent.click(screen.getByLabelText("tickets:read"));
  fireEvent.click(screen.getByRole("button", { name: "Create key" }));
  expect(writes).toEqual([]);
});
