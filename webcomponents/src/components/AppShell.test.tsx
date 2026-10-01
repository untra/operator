import { describe, expect, test } from "bun:test";
import { renderToStaticMarkup } from "react-dom/server";

import { AccountFooter, NavRow } from "./AppShell";

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

describe("NavRow", () => {
  test("exposes a disabled reason to keyboard and screen-reader users", () => {
    const markup = renderToStaticMarkup(
      <NavRow label="Kanban" icon="project" disabledReason="Requires: Projects" />,
    );

    expect(markup).toContain('<button type="button"');
    expect(markup).toContain('aria-disabled="true"');
    expect(markup).toMatch(/aria-describedby="([^"]+)"[\s\S]*id="\1"/);
    expect(markup).not.toContain("title=");
  });
});
