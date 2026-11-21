import { expect, test } from "@playwright/test";
import { ADMIN_STATE } from "../fixtures/env";
import { operatorApi } from "../fixtures/operator";
import { continueUntil, createAdmin } from "../fixtures/setup";

test("a fresh server walks admin setup and onboarding to an initialized workspace", async ({
  page,
}) => {
  await createAdmin(page);

  const initialize = page.getByRole("button", { name: "Initialize workspace" });
  expect(await continueUntil(page, () => initialize.isVisible())).toBe(true);
  await initialize.click();

  await expect(page).toHaveURL(/#\/$/);
  const status = await (await operatorApi(page.request)).setupStatus();
  expect(status.initialized).toBe(true);

  await page.context().storageState({ path: ADMIN_STATE });
});
