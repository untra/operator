/**
 * The paywall renders a build-time URL as a link, so the one piece of logic
 * worth asserting is which values are allowed to become an href.
 */

import { describe, expect, test } from "bun:test";

import { safePurchaseDestination } from "./PremiumPaywall";

describe("safePurchaseDestination", () => {
  test("accepts https, whatever the case", () => {
    expect(safePurchaseDestination("https://operator.untra.io/premium")).toBe(
      "https://operator.untra.io/premium",
    );
    expect(safePurchaseDestination("HTTPS://operator.untra.io/premium")).toBe(
      "HTTPS://operator.untra.io/premium",
    );
  });

  test("refuses everything that is not https", () => {
    for (const hostile of [
      "http://operator.untra.io/premium",
      // oxlint-disable-next-line no-script-url
      "javascript:alert(1)",
      // oxlint-disable-next-line no-script-url
      "JavaScript:alert(1)",
      "data:text/html,<script>alert(1)</script>",
      "//operator.untra.io/premium",
      "operator.untra.io/premium",
      " https://operator.untra.io/premium",
      "",
    ]) {
      expect(safePurchaseDestination(hostile)).toBeNull();
    }
  });

  test("treats a missing destination as no link", () => {
    expect(safePurchaseDestination(null)).toBeNull();
    expect(safePurchaseDestination(undefined)).toBeNull();
  });
});
