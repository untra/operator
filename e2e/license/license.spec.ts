import { randomUUID } from "node:crypto";
import { expect, test } from "@playwright/test";
import { issuerConfigured, requestTestLicense } from "../fixtures/issuer";
import { operatorApi } from "../fixtures/operator";

const REMOTE_TARGET = {
  name: "e2e-remote",
  kind: "ssh",
  ssh_alias: "e2e-remote",
  workdir: "/srv/e2e",
};
const PAYMENT_REQUIRED = 402;

test.skip(!issuerConfigured, "OPERATOR_E2E_ISSUER_URL and OPERATOR_E2E_LICENSE_TOKEN are unset");

test("a test license applies to this configuration and unlocks premium", async ({ page }) => {
  const operator = await operatorApi(page.request);
  const before = await operator.license();
  expect(before.status).toBe("missing");
  expect((await operator.createTarget(REMOTE_TARGET)).status()).toBe(PAYMENT_REQUIRED);

  const foreign = await requestTestLicense(randomUUID());
  expect((await operator.installLicense(foreign)).ok()).toBe(false);
  expect((await operator.license()).status).toBe("missing");

  const key = await requestTestLicense(before.profile_id);
  await page.goto("/#/settings/license");
  const panel = page.getByRole("region", { name: "Operator license" });
  const status = panel.locator("dt:text-is('Status') + dd");
  await expect(status).toHaveText("Free");
  await panel.getByLabel("License key").fill(key);
  await panel.getByRole("button", { name: "Apply license" }).click();
  await expect(status).toHaveText("Premium");

  const after = await operator.license();
  expect(after).toMatchObject({ status: "valid", premium: true, profile_id: before.profile_id });
  expect(after.terms?.profile_id).toBe(before.profile_id);

  expect((await operator.createTarget(REMOTE_TARGET)).ok()).toBe(true);
  expect((await operator.removeTarget(REMOTE_TARGET.name)).ok()).toBe(true);

  await panel.getByRole("button", { name: "Remove license" }).click();
  await expect(status).toHaveText("Free");
  expect((await operator.license()).status).toBe("missing");
});
