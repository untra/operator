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

/** The storefront stays put with a role=alert when its api refuses checkout; surface that reason. */
async function reachStripe(page: Page): Promise<void> {
  const stripe = /^https:\/\/checkout\.stripe\.com\//;
  const refusal = page.getByRole("alert");
  const outcome = await Promise.race([
    page.waitForURL(stripe, { timeout: PAGE_TIMEOUT_MS }).then(() => "stripe" as const),
    refusal.waitFor({ timeout: PAGE_TIMEOUT_MS }).then(() => "refused" as const),
  ]).catch(() => "timeout" as const);
  if (outcome === "refused" && !stripe.test(page.url())) {
    throw new Error(`storefront refused checkout: ${(await refusal.innerText()).trim()}`);
  }
  await expect(page).toHaveURL(stripe, { timeout: PAGE_TIMEOUT_MS });
}

export async function payWithTestCard(page: Page): Promise<void> {
  await reachStripe(page);

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
