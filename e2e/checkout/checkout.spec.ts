import { expect, test, type BrowserContext, type Page } from "@playwright/test";
import { BASE_URL, CHECKOUT_ENABLED, PLATFORM_APP_ORIGIN } from "../fixtures/env";
import { operatorApi } from "../fixtures/operator";
import { continueUntil, createAdmin } from "../fixtures/setup";
import { openCheckout, payAndAwaitReturn } from "../fixtures/stripe-checkout";

/**
 * Both ways a buyer reaches Premium, each a real sandbox purchase through the platform storefront:
 *  1. setup: a new workspace buys during onboarding; the key comes back and onboarding resumes.
 *  2. upgrade: an existing free workspace buys from the Premium page and lands back on it.
 *
 * Needs a fresh server, a build whose OPERATOR_PURCHASE_URL is the sandbox storefront, and Stripe
 * events reaching the platform webhook. See .forgejo/workflows/e2e-checkout.yaml.
 */
const REMOTE_TARGET = {
  name: "e2e-checkout-remote",
  kind: "ssh",
  ssh_alias: "e2e-checkout-remote",
  workdir: "/srv/e2e",
};
const PAYMENT_REQUIRED = 402;
const UI_TIMEOUT_MS = 20_000;
const DRAFT_PREFIX = "operator.onboarding-draft.";
const OPERATOR = new URL(BASE_URL);
const escape = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

test.skip(
  !CHECKOUT_ENABLED,
  "OPERATOR_E2E_CHECKOUT=1 is unset; the sandbox checkout suite is opt-in",
);
test.describe.configure({ mode: "serial", timeout: 240_000 });

let context: BrowserContext;
let page: Page;

test.beforeAll(async ({ browser }) => {
  context = await browser.newContext({ baseURL: BASE_URL, ignoreHTTPSErrors: true });
  page = await context.newPage();
});

test.afterAll(async () => {
  await context.close();
});

function expectReturnTo(href: string | null, flow: "setup" | "upgrade", profileId: string): void {
  const purchase = new URL(href ?? "");
  expect(purchase.origin).toBe(PLATFORM_APP_ORIGIN);
  expect(purchase.pathname).toBe("/checkout");
  expect(purchase.searchParams.get("profile_id")).toBe(profileId);
  expect(purchase.searchParams.get("return_to")).toBe(`${BASE_URL}/?checkout=${flow}`);
}

test("setup: a new workspace buys Premium during onboarding and resumes where it left off", async () => {
  await createAdmin(page);

  const heading = page.getByRole("heading", { name: "Where will agents run?" });
  expect(await continueUntil(page, () => heading.isVisible())).toBe(true);

  const operator = await operatorApi(page.request);
  const before = await operator.license();
  expect(before).toMatchObject({ status: "missing", premium: false });
  expect((await operator.createTarget(REMOTE_TARGET)).status()).toBe(PAYMENT_REQUIRED);

  // Remote needs Premium: choosing it opens the paywall and the license panel.
  await page.getByText("Remote targets \u00B7 Premium").click();
  const panel = page.getByRole("region", { name: "Operator license" });
  await expect(panel.locator("dt:text-is('Status') + dd")).toHaveText("Free");

  const buy = page.getByRole("link", { name: "Buy Premium", exact: true });
  expectReturnTo(await buy.getAttribute("href"), "setup", before.profile_id);

  const shop = await openCheckout(page, buy);
  await expect(panel.getByText(/finish checkout in the new tab/i)).toBeVisible();
  await payAndAwaitReturn(shop, PLATFORM_APP_ORIGIN, OPERATOR);

  // The return tab installs the key, drops it from the URL and resumes onboarding at the license step.
  await expect(shop).toHaveURL(
    new RegExp(`^${escape(OPERATOR.origin)}/#/onboarding\\?step=license$`),
    { timeout: UI_TIMEOUT_MS },
  );
  const returned = shop.getByRole("region", { name: "Operator license" });
  await expect(returned.locator("dt:text-is('Status') + dd")).toHaveText("Premium", {
    timeout: UI_TIMEOUT_MS,
  });
  const drafts = await shop.evaluate(
    (prefix) =>
      Object.keys(localStorage)
        .filter((key) => key.startsWith(prefix))
        .map((key) => JSON.parse(localStorage.getItem(key) ?? "null")),
    DRAFT_PREFIX,
  );
  expect(drafts).toContainEqual(
    expect.objectContaining({ draft: expect.objectContaining({ configurationName: "e2e" }) }),
  );
  await shop.close();

  // The tab that started checkout polls and turns Premium without a reload.
  await expect(panel.getByText(/operator premium is active/i)).toBeVisible({
    timeout: UI_TIMEOUT_MS,
  });
  await expect(panel.locator("dt:text-is('Status') + dd")).toHaveText("Premium");
  await expect(buy).toBeHidden();

  const after = await operator.license();
  expect(after).toMatchObject({ status: "valid", premium: true, profile_id: before.profile_id });
  expect((await operator.createTarget(REMOTE_TARGET)).ok()).toBe(true);
  expect((await operator.removeTarget(REMOTE_TARGET.name)).ok()).toBe(true);

  const initialize = page.getByRole("button", { name: "Initialize workspace" });
  expect(await continueUntil(page, () => initialize.isVisible())).toBe(true);
  await initialize.click();
  await expect(page).toHaveURL(/#\/$/);
  expect((await operator.setupStatus()).initialized).toBe(true);
});

test("upgrade: an existing free workspace buys Premium from the Premium page", async () => {
  const operator = await operatorApi(page.request);
  expect((await operator.removeLicense()).ok()).toBe(true);
  const before = await operator.license();
  expect(before).toMatchObject({ status: "missing", premium: false });

  // In-app paywalls send the buyer to the Premium page, never straight to the storefront.
  // A hash-only goto keeps the document; reload so the UI reads the license fresh.
  await page.goto("/#/remote-targets");
  await page.reload();
  const seePremium = page.getByRole("link", { name: "See Premium" });
  await expect(seePremium).toHaveAttribute("href", "#/settings/premium");
  await seePremium.click();
  await expect(page).toHaveURL(/#\/settings\/premium$/);

  const upsell = page.getByRole("region", { name: "Upgrade" });
  await expect(page.getByRole("region", { name: "Operator online" })).toBeVisible();
  const buy = upsell.getByRole("link", { name: "Buy Operator Premium" });
  expectReturnTo(await buy.getAttribute("href"), "upgrade", before.profile_id);

  const shop = await openCheckout(page, buy);
  await expect(upsell.getByText(/finish checkout in the new tab/i)).toBeVisible();
  await payAndAwaitReturn(shop, PLATFORM_APP_ORIGIN, OPERATOR);

  // The return tab installs the key and lands on the Premium page's unlocked view.
  await expect(shop).toHaveURL(new RegExp(`^${escape(OPERATOR.origin)}/#/settings/premium$`), {
    timeout: UI_TIMEOUT_MS,
  });
  await expect(shop.getByRole("region", { name: "Premium active" })).toBeVisible({
    timeout: UI_TIMEOUT_MS,
  });
  await shop.close();

  // The tab that started checkout turns Premium on its own.
  await expect(page.getByRole("region", { name: "Premium active" })).toBeVisible({
    timeout: UI_TIMEOUT_MS,
  });

  // The License page reads the installed key and links out only to manage it on the platform.
  await page.goto("/#/settings/license");
  const panel = page.getByRole("region", { name: "Operator license" });
  await expect(panel.locator("dt:text-is('Status') + dd")).toHaveText("Premium");
  await expect(page.getByRole("link", { name: "Manage license" })).toHaveAttribute(
    "href",
    `${PLATFORM_APP_ORIGIN}/account`,
  );
  await expect(page.getByRole("link", { name: /buy/i })).toHaveCount(0);

  const after = await operator.license();
  expect(after).toMatchObject({ status: "valid", premium: true, profile_id: before.profile_id });
  expect((await operator.createTarget(REMOTE_TARGET)).ok()).toBe(true);
  expect((await operator.removeTarget(REMOTE_TARGET.name)).ok()).toBe(true);
});
