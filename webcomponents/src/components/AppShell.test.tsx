import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import { AccountFooter } from "./AppShell";

function handleSignOut() {}

describe("AccountFooter", () => {
  test("renders the account, configuration, and sign-out action", () => {
    const markup = renderToStaticMarkup(
      <AccountFooter username="admin" configurationName="Development" onSignOut={handleSignOut} />,
    );

    expect(markup).toContain("admin");
    expect(markup).toContain("Development");
    expect(markup).toContain(">Sign out</button>");
  });

  test("renders busy and failure states", () => {
    const markup = renderToStaticMarkup(
      <AccountFooter
        username="admin"
        configurationName="Development"
        busy
        failed
        onSignOut={handleSignOut}
      />,
    );

    expect(markup).toContain("Could not sign out.");
    expect(markup).toContain("disabled");
    expect(markup).toContain("Signing out…");
  });
});
