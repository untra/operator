import { randomUUID } from "node:crypto";
import { expect, type Locator, type Page } from "@playwright/test";

/**
 * Drives Stripe's hosted Checkout page in a sandbox with the always-succeeds test card.
 * Stripe owns this markup, so every selector lives here: when Stripe changes the page, fix it once.
 */
const TEST_CARD = "4242 4242 4242 4242";
const TEST_CVC = "424";
const TEST_POSTAL = "94103";
const PAGE_TIMEOUT_MS = 30_000;

/** MM / YY two years out, always in the future. */
function futureExpiry(): string {
  const now = new Date();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const year = String((now.getFullYear() + 2) % 100).padStart(2, "0");
  return `${month} / ${year}`;
}

async function fillIfVisible(field: Locator, value: string): Promise<void> {
  if (await field.isVisible()) {
    await field.fill(value);
  }
}

export async function payWithTestCard(page: Page): Promise<void> {
  await expect(page).toHaveURL(/^https:\/\/checkout\.stripe\.com\//, { timeout: PAGE_TIMEOUT_MS });

  const email = page.locator("#email");
  await expect(email).toBeVisible({ timeout: PAGE_TIMEOUT_MS });
  await email.fill(`e2e+${randomUUID()}@example.com`);

  // With several payment methods enabled, card sits in a collapsed accordion.
  const cardAccordion = page.getByTestId("card-accordion-item-button");
  if (await cardAccordion.isVisible()) {
    await cardAccordion.click();
  }

  await page.locator("#cardNumber").fill(TEST_CARD);
  await page.locator("#cardExpiry").fill(futureExpiry());
  await page.locator("#cardCvc").fill(TEST_CVC);
  await fillIfVisible(page.locator("#billingName"), "Operator E2E");

  const country = page.locator("#billingCountry");
  if (await country.isVisible()) {
    await country.selectOption("US");
  }
  await fillIfVisible(page.locator("#billingPostalCode"), TEST_POSTAL);

  // Never enroll the throwaway buyer in Link.
  const saveWithLink = page.locator("#enableStripePass");
  if ((await saveWithLink.isVisible()) && (await saveWithLink.isChecked())) {
    await saveWithLink.uncheck();
  }

  await page.getByTestId("hosted-payment-submit-button").click();
}
