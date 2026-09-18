import { useCallback, useState } from "react";
import { NavLink, Outlet, useNavigate } from "react-router-dom";
import {
  AppShell,
  BrandName,
  NavGroup,
  NavRow,
  RightPanel as RightPanelView,
  SignOutButton,
  ThemeToggle,
} from "@operator/webcomponents";
import { useTheme } from "./theme";
import type { Concept } from "./concepts";
import { CONCEPTS, STATUS_KEYS, PAGE_KEYS } from "./concepts";
import { SectionsProvider, useSections } from "./sections-context";
import { RightPanelProvider, useRightPanel } from "./right-panel";
import type { SectionDto } from "./api-client";
import { OperatorApi, setCsrfToken } from "./api-client";
import { useHost } from "./host";
import { ProfileSelector } from "./profiles-context";

// The "Status" group mirrors the canonical section order shared with the TUI and
// VS Code extension (the SectionId enum in src/ui/status_panel.rs) and reflects
// each section's live health from GET /api/v1/sections. A section whose
// prerequisites aren't met yet is shown disabled with a tooltip naming what it
// needs - the user sees it exists and why it isn't reachable. "Pages" are
// web-only views (Dashboard, Queue) with no section analog.

function ConceptNavRow({ concept, section }: { concept: Concept; section?: SectionDto }) {
  const renderLink = useCallback(
    (content: React.ReactNode, className: (state: { isActive: boolean }) => string) => (
      <NavLink to={concept.route} end={concept.route === "/"} className={className}>
        {content}
      </NavLink>
    ),
    [concept.route],
  );

  if (section && !section.met) {
    const needs = section.prerequisites.map((id) => CONCEPTS[id]?.label ?? id).join(", ");
    return (
      <NavRow
        label={concept.label}
        icon={concept.icon}
        health={section.health}
        disabledReason={needs ? `Requires: ${needs}` : "Not available yet"}
      />
    );
  }

  return (
    <NavRow
      label={concept.label}
      icon={concept.icon}
      health={section?.health}
      renderLink={renderLink}
    />
  );
}

function ConceptNavGroup({ label, keys }: { label: string; keys: readonly string[] }) {
  const { sections } = useSections();
  return (
    <NavGroup label={label}>
      {keys.map((key) => (
        <li key={key}>
          <ConceptNavRow concept={CONCEPTS[key]} section={sections?.find((s) => s.id === key)} />
        </li>
      ))}
    </NavGroup>
  );
}

function RightPanelController() {
  const { content, title, close } = useRightPanel();
  if (!content) {
    return null;
  }
  return (
    <RightPanelView title={title} onClose={close}>
      {content}
    </RightPanelView>
  );
}

export function Layout() {
  const { theme, toggleTheme } = useTheme();
  const host = useHost();
  const navigate = useNavigate();
  const [signingOut, setSigningOut] = useState(false);
  const [signOutError, setSignOutError] = useState(false);

  const signOut = useCallback(async () => {
    setSigningOut(true);
    setSignOutError(false);
    try {
      const api = new OperatorApi(host);
      await api.refreshCsrf();
      await api.logout();
      setCsrfToken(null);
      void navigate("/login", { replace: true });
    } catch {
      setSignOutError(true);
    } finally {
      setSigningOut(false);
    }
  }, [host, navigate]);

  return (
    <SectionsProvider>
      <RightPanelProvider>
        <AppShell
          brand={
            <>
              <BrandName />
              <ThemeToggle theme={theme} onToggle={toggleTheme} />
            </>
          }
          groups={
            <>
              <ProfileSelector />
              <ConceptNavGroup label="Status" keys={STATUS_KEYS} />
              <ConceptNavGroup label="Pages" keys={PAGE_KEYS} />
            </>
          }
          footer={<SignOutButton busy={signingOut} failed={signOutError} onClick={signOut} />}
          panel={<RightPanelController />}
        >
          <Outlet />
        </AppShell>
      </RightPanelProvider>
    </SectionsProvider>
  );
}
