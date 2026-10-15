import { afterEach, expect, mock, test } from "bun:test";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { ApiProvider, resetSessionState } from "../api";
import { setCsrfToken } from "../api-client";
import { mockFetch, requestPath, restoreFetch } from "../test-fetch";
import * as webcomponentMocks from "../test-webcomponents";
import { HostContext, type Host } from "../host";
import type { PermissionGrant } from "@operator/bindings/PermissionGrant";

mock.module("@operator/webcomponents", () => webcomponentMocks);
const { IdentityPanel } = await import("./IdentityPanel");

const GRANTS: PermissionGrant = {
  profile_id: "00000000-0000-0000-0000-000000000001",
  instance_wide: false,
  capabilities: ["tickets:read"],
  projects: ["operator"],
};
const HOST: Host = {
  baseUrl: () => "https://operator.test",
  profileId: GRANTS.profile_id,
  openExternal: () => undefined,
  browseFolder: () => Promise.resolve(null),
  openFile: () => undefined,
};

afterEach(() => {
  cleanup();
  resetSessionState();
  setCsrfToken(null);
  restoreFetch();
});

test("an administrator explicitly approves a pending user with selected profile grants", async () => {
  const updates: unknown[] = [];
  mockFetch((input, init) => {
    const path = requestPath(input);
    let body: unknown = [];
    if (path.endsWith("/capabilities")) {
      body = { administrator: true, grants: GRANTS, administration_grants: GRANTS };
    } else if (path.endsWith("/users")) {
      body = [
        {
          id: "user",
          subject: "alice",
          issuer: "https://id.test",
          enabled: false,
          administrator: false,
          grants: [],
        },
      ];
    } else if (path.endsWith("/users/user") && init?.method === "PUT") {
      updates.push(JSON.parse(typeof init.body === "string" ? init.body : "null"));
      body = null;
    }
    return Promise.resolve(
      new Response(JSON.stringify(body), { headers: { "Content-Type": "application/json" } }),
    );
  });
  setCsrfToken("test-csrf");
  render(
    <HostContext value={HOST}>
      <ApiProvider>
        <IdentityPanel />
      </ApiProvider>
    </HostContext>,
  );
  fireEvent.click(await screen.findByLabelText("Enabled"));
  fireEvent.click(screen.getByLabelText("tickets:read"));
  fireEvent.click(screen.getByLabelText("operator"));
  fireEvent.click(screen.getByRole("button", { name: "Save permissions", hidden: true }));
  await waitFor(() =>
    expect(updates).toEqual([{ enabled: true, administrator: false, grants: GRANTS }]),
  );
});
