import { request } from "@playwright/test";
import { ISSUER_URL, LICENSE_TOKEN } from "./env";

/** Both the issuer URL and its token are required; absent config skips the suite. */
export const issuerConfigured = Boolean(ISSUER_URL && LICENSE_TOKEN);

/**
 * A short-lived test-mode license bound to `profileId`, from the platform
 * webhook's E2E route. Its own empty context: operator cookies never leave home.
 */
export async function requestTestLicense(profileId: string): Promise<string> {
  const issuer = await request.newContext({ storageState: { cookies: [], origins: [] } });
  try {
    const response = await issuer.get(new URL("/e2e/license", ISSUER_URL).toString(), {
      params: { profile_id: profileId },
      headers: { authorization: `Bearer ${LICENSE_TOKEN}`, "x-operator-e2e": "test-mode" },
    });
    if (!response.ok()) {
      throw new Error(`test license issuer: HTTP ${response.status()}`);
    }
    return (await response.text()).trim();
  } finally {
    await issuer.dispose();
  }
}
