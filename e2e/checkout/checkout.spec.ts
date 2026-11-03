import { expect, test } from "@playwright/test";
import { BASE_URL, CHECKOUT_ENABLED, PLATFORM_APP_ORIGIN } from "../fixtures/env";
import { operatorApi } from "../fixtures/operator";
import { continueUntil, createAdmin } from "../fixtures/setup";
import { payWithTestCard } from "../fixtures/stripe-checkout";

/**
 * First-time setup buys Premium through the real platform storefront and a Stripe sandbox
 * Checkout, then proves the platform hands the key back (`#license_key=…`), this server
 * installs it, and both tabs tell the user they are Premium without a copy and paste.
 *
 * Needs a fresh server, a build whose OPERATOR_PURCHASE_URL is the sandbox storefront, and Stripe
 * events reaching the platform webhook (CI runs `stripe listen`). See .forgejo/workflows/e2e-checkout.yaml.
 */
const REMOTE_TARGET = {
  name: "e2e-checkout-remote",
  kind: "ssh",
  ssh_alias: "e2e-checkout-remote",
  workdir: "/srv/e2e",
};
const PAYMENT_REQUIRED = 402;
/** Stripe → stripe listen → webhook inbox → license issuance → pickup poll (2 s ticks). */
const LICENSE_ISSUED_TIMEOUT_MS = 90_000;
const STRIPE_REDIRECT_TIMEOUT_MS = 60_000;
const UI_TIMEOUT_MS = 20_000;
const OPERATOR = new URL(BASE_URL);
const escape = (text: string) => text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

test.skip(
  !CHECKOUT_ENABLED,
  "OPERATOR_E2E_CHECKOUT=1 is unset; the sandbox checkout suite is opt-in",
);
test.setTimeout(240_000);

test("first-time setup buys Premium in the sandbox and the key comes back on its own", async ({
  page,
}) => {
  await createAdmin(page);

  const heading = page.getByRole("heading", { name: "Where will agents run?" });
  expect(await continueUntil(page, () => heading.isVisible())).toBe(true);

  const operator = await operatorApi(page.request);
  const before = await operator.license();
  expect(before).toMatchObject({ status: "missing", premium: false });
  expect((await operator.createTarget(REMOTE_TARGET)).status()).toBe(PAYMENT_REQUIRED);

  // Remote needs Premium: choosing it opens the paywall and the license panel.
  await page.getByText("Remote targets · Premium").click();
  const panel = page.getByRole("region", { name: "Operator license" });
  await expect(panel.locator("dt:text-is('Status') + dd")).toHaveText("Free");

  // The purchase link binds this configuration and says where to come back to.
  const buy = page.getByRole("link", { name: "Buy Premium", exact: true });
  const purchase = new URL((await buy.getAttribute("href")) ?? "");
  expect(purchase.origin).toBe(PLATFORM_APP_ORIGIN);
  expect(purchase.pathname).toBe("/checkout");
  expect(purchase.searchParams.get("profile_id")).toBe(before.profile_id);
  expect(purchase.searchParams.get("return_to")).toBe(`${BASE_URL}/`);

  const popupOpened = page.waitForEvent("popup");
  await buy.click();
  const shop = await popupOpened;

  // The tab that started checkout says what is happening while the buyer is away.
  await expect(panel.getByText(/finish checkout in the new tab/i)).toBeVisible();

  // Storefront → Stripe sandbox → back to the storefront's license pickup.
  await payWithTestCard(shop);
  await expect(shop).toHaveURL(
    new RegExp(`^${escape(PLATFORM_APP_ORIGIN)}/license\\?session_id=`),
    {
      timeout: STRIPE_REDIRECT_TIMEOUT_MS,
    },
  );
  await expect(
    shop.getByText(new RegExp(`returning you to operator at ${escape(OPERATOR.host)}`, "i")),
  ).toBeVisible({
    timeout: LICENSE_ISSUED_TIMEOUT_MS,
  });
  await expect(shop.getByRole("textbox", { name: "License key" })).not.toHaveValue("");

  // Back on this server: the key arrives in the fragment, is installed, then leaves the URL.
  await expect(shop).toHaveURL(new RegExp(`^${escape(OPERATOR.origin)}/#/license/return`), {
    timeout: UI_TIMEOUT_MS,
  });
  await expect(shop.getByText("Operator Premium is active.")).toBeVisible({
    timeout: UI_TIMEOUT_MS,
  });
  await expect(shop).not.toHaveURL(/license_key=/);

  // The onboarding tab polls and turns Premium without a reload; the paywall goes away.
  await expect(panel.getByText(/operator premium is active/i)).toBeVisible({
    timeout: UI_TIMEOUT_MS,
  });
  await expect(panel.locator("dt:text-is('Status') + dd")).toHaveText("Premium");
  await expect(buy).toBeHidden();

  const after = await operator.license();
  expect(after).toMatchObject({ status: "valid", premium: true, profile_id: before.profile_id });
  expect(after.terms?.profile_id).toBe(before.profile_id);
  expect((await operator.createTarget(REMOTE_TARGET)).ok()).toBe(true);
  expect((await operator.removeTarget(REMOTE_TARGET.name)).ok()).toBe(true);

  // The return tab hands back to onboarding, which is still in progress.
  await shop.getByRole("button", { name: "Continue setup" }).click();
  await expect(shop).toHaveURL(/#\/onboarding/);
});
