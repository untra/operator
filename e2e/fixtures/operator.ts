import type { APIRequestContext, APIResponse } from "@playwright/test";

const CSRF_HEADER = "x-operator-csrf";

export type LicenseResponse = {
  status: "missing" | "valid" | "expired" | "not_yet_valid" | "invalid";
  profile_id: string;
  premium: boolean;
  terms: { sub: string; jti: string; profile_id: string; tier: string } | null;
};

export type SetupStatus = { initialized: boolean; config_path: string };

async function json<T>(response: Promise<APIResponse>): Promise<T> {
  const resolved = await response;
  if (!resolved.ok()) {
    throw new Error(`${resolved.url()}: HTTP ${resolved.status()} ${await resolved.text()}`);
  }
  return (await resolved.json()) as T;
}

/** Cookie-authenticated client over a page's request context; mutations carry the CSRF token. */
export async function operatorApi(request: APIRequestContext) {
  const csrf = await request.get("/api/v1/auth/csrf");
  if (!csrf.ok()) {
    throw new Error(`csrf token: HTTP ${csrf.status()}`);
  }
  const { csrf_token: token } = (await csrf.json()) as { csrf_token: string };
  const headers = { [CSRF_HEADER]: token };

  return {
    setupStatus: () => json<SetupStatus>(request.get("/api/v1/setup/status")),
    license: () => json<LicenseResponse>(request.get("/api/v1/license")),
    installLicense: (key: string) =>
      request.put("/api/v1/license", { headers, data: { license_key: key } }),
    createTarget: (target: Record<string, unknown>) =>
      request.post("/api/v1/targets", { headers, data: target }),
    removeTarget: (name: string) =>
      request.delete(`/api/v1/targets/${encodeURIComponent(name)}`, { headers }),
  };
}
