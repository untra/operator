import { randomUUID } from "node:crypto";
import { expect, test } from "@playwright/test";
import { ADMIN_STATE, BOOTSTRAP_PASSWORD } from "../fixtures/env";
import { operatorApi } from "../fixtures/operator";

const MAX_STEPS = 25;

test("a fresh server walks admin setup and onboarding to an initialized workspace", async ({
  page,
}) => {
  const password = `e2e-${randomUUID()}`;

  await page.goto("/");
  await expect(page).toHaveURL(/#\/setup$/);
  if (BOOTSTRAP_PASSWORD) {
    await page.getByLabel("Temporary password").fill(BOOTSTRAP_PASSWORD);
  }
  await page.getByLabel("New admin password").fill(password);
  await page.getByLabel("Confirm password").fill(password);
  await page.getByRole("button", { name: "Create admin account" }).click();

  await expect(page).toHaveURL(/#\/onboarding/);
  await page.getByLabel("Configuration name").fill("e2e");

  const initialize = page.getByRole("button", { name: "Initialize workspace" });
  const counter = page.getByText(/^Step \d+ of \d+$/);
  for (let step = 0; step < MAX_STEPS && !(await initialize.isVisible()); step += 1) {
    const before = await counter.textContent();
    await page.getByRole("button", { name: "Continue" }).click();
    await expect(counter).not.toHaveText(before ?? "");
  }
  await initialize.click();

  await expect(page).toHaveURL(/#\/$/);
  const status = await (await operatorApi(page.request)).setupStatus();
  expect(status.initialized).toBe(true);

  await page.context().storageState({ path: ADMIN_STATE });
});
