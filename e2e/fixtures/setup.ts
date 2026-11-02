import { randomUUID } from "node:crypto";
import { expect, type Page } from "@playwright/test";
import { BOOTSTRAP_PASSWORD } from "./env";

const MAX_STEPS = 25;

/** First visit to a fresh server: create the admin account, landing on onboarding. */
export async function createAdmin(page: Page): Promise<void> {
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
}

/** Click Continue through onboarding until `arrived` is visible; true when it was reached. */
export async function continueUntil(page: Page, arrived: () => Promise<boolean>): Promise<boolean> {
  const counter = page.getByText(/^Step \d+ of \d+$/);
  for (let step = 0; step < MAX_STEPS; step += 1) {
    if (await arrived()) {
      return true;
    }
    const before = await counter.textContent();
    await page.getByRole("button", { name: "Continue" }).click();
    await expect(counter).not.toHaveText(before ?? "");
  }
  return arrived();
}
