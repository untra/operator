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

  // Checkout renders its whole form inside one Elements iframe.
  const form = page.frameLocator('iframe[title="Secure embedded form frame"]');

  const email = form.locator("#linkAuthentication-emailInput");
  await expect(email).toBeVisible({ timeout: PAGE_TIMEOUT_MS });
  await email.fill(`e2e+${randomUUID()}@example.com`);

  // With several payment methods enabled, card sits in a collapsed accordion.
  const cardNumber = form.locator("#payment-numberInput");
  if (!(await cardNumber.isVisible())) {
    await form.getByRole("button", { name: /^Card/ }).click();
  }

  await expect(cardNumber).toBeVisible({ timeout: PAGE_TIMEOUT_MS });
  await cardNumber.fill(TEST_CARD);
  await form.locator("#payment-expiryInput").fill(futureExpiry());
  await form.locator("#payment-cvcInput").fill(TEST_CVC);
  await fillIfVisible(form.locator("#payment-nameInput"), "Operator E2E");

  const country = form.locator("#payment-countryInput");
  if (await country.isVisible()) {
    await country.selectOption("US");
  }
  await fillIfVisible(form.locator("#payment-postalCodeInput"), TEST_POSTAL);

  // Never enroll the throwaway buyer in Link; it is opted in by default and demands a phone.
  const saveWithLink = form.locator("#payment-linkOptInInput");
  if ((await saveWithLink.isVisible()) && (await saveWithLink.isChecked())) {
    await saveWithLink.uncheck();
  }

  await form.getByTestId("payment-form-submit-button").click();
}
