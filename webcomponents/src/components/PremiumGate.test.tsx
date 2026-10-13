import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import { PremiumGate } from "./PremiumGate";

function handleAddLicense() {}

const LICENSE = {
  status: "valid" as const,
  profile_id: "default",
  terms: null,
  purchase_url: "https://operator.untra.io/premium",
};

function render(license: Parameters<typeof PremiumGate>[0]["license"]) {
  return renderToStaticMarkup(
    <PremiumGate feature="Campaigns" license={license} onAddLicense={handleAddLicense}>
      <p>premium content</p>
    </PremiumGate>,
  );
}

describe("PremiumGate", () => {
  test("renders nothing while the license is loading", () => {
    expect(render(undefined)).toBe("");
    expect(render(null)).toBe("");
  });

  test("renders the paywall without a premium license", () => {
    const markup = render({ ...LICENSE, premium: false });
    expect(markup).toContain("Campaigns");
    expect(markup).toContain("Add license");
    expect(markup).not.toContain("premium content");
  });

  test("renders its children with a premium license", () => {
    const markup = render({ ...LICENSE, premium: true });
    expect(markup).toContain("premium content");
    expect(markup).not.toContain("Add license");
  });
});
