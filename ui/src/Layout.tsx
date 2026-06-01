import { NavLink, Outlet } from 'react-router-dom';
import styles from './Layout.module.css';
import { useTheme } from './theme';
import type { Concept } from './concepts';
import { CONCEPTS, STATUS_KEYS, PAGE_KEYS } from './concepts';
import { ConceptIcon } from './components/ConceptIcon';
import { SectionsProvider, useSections } from './sections-context';
import { RightPanelProvider, useRightPanel } from './right-panel';
import type { SectionDto } from './api-client';

// The "Status" group mirrors the canonical section order shared with the TUI and
// VS Code extension (the SectionId enum in src/ui/status_panel.rs) and reflects
// each section's live health from GET /api/v1/sections. A section whose
// prerequisites aren't met yet is shown disabled with a tooltip naming what it
// needs — the user sees it exists and why it isn't reachable. "Pages" are
// web-only views (Dashboard, Queue) with no section analog.

function NavRow({ concept, section }: { concept: Concept; section?: SectionDto }) {
  const met = section ? section.met : true;

  const inner = (
    <>
      <ConceptIcon name={concept.icon} className={styles.navIcon} />
      <span className={styles.navLabel}>{concept.label}</span>
      {section && <span className={styles.navDot} data-health={section.health} />}
    </>
  );

  if (!met) {
    const needs = (section?.prerequisites ?? [])
      .map((id) => CONCEPTS[id]?.label ?? id)
      .join(', ');
    return (
      <span
        className={`${styles.navLink} ${styles.navDisabled}`}
        aria-disabled="true"
        title={needs ? `Requires: ${needs}` : 'Not available yet'}
      >
        {inner}
      </span>
    );
  }

  return (
    <NavLink
      to={concept.route}
      end={concept.route === '/'}
      className={({ isActive }) => (isActive ? `${styles.navLink} ${styles.active}` : styles.navLink)}
    >
      {inner}
    </NavLink>
  );
}

function NavGroup({ label, keys }: { label: string; keys: readonly string[] }) {
  const { sections } = useSections();
  return (
    <div className={styles.group}>
      <p className={styles.groupLabel}>{label}</p>
      <ul className={styles.navList}>
        {keys.map((key) => {
          const concept = CONCEPTS[key];
          const section = sections?.find((s) => s.id === key);
          return (
            <li key={key}>
              <NavRow concept={concept} section={section} />
            </li>
          );
        })}
      </ul>
    </div>
  );
}

// The detail sidepanel. Renders nothing until a view opens it via
// useRightPanel().open(...); when content is present it slides in on the right
// with a header (title + close) above the caller-supplied node.
function RightPanel() {
  const { content, title, close } = useRightPanel();
  if (!content) return null;
  return (
    <aside className={styles.rightPanel} aria-label={title ?? 'Detail panel'}>
      <div className={styles.rightPanelHeader}>
        <span className={styles.rightPanelTitle}>{title}</span>
        <button
          type="button"
          className={styles.rightPanelClose}
          onClick={close}
          aria-label="Close panel"
          title="Close panel"
        >
          ✕
        </button>
      </div>
      <div className={styles.rightPanelBody}>{content}</div>
    </aside>
  );
}

export function Layout() {
  const { theme, toggleTheme } = useTheme();

  return (
    <SectionsProvider>
      <RightPanelProvider>
        <div className={styles.layout}>
          <nav className={styles.nav}>
            <div className={styles.brandRow}>
              <span className={styles.brand}>Operator</span>
              <button
                type="button"
                className={styles.themeToggle}
                onClick={toggleTheme}
                aria-label={theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
                title={theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
              >
                {theme === 'dark' ? '☀' : '☾'}
              </button>
            </div>
            <NavGroup label="Status" keys={STATUS_KEYS} />
            <NavGroup label="Pages" keys={PAGE_KEYS} />
          </nav>
          <main className={styles.main}>
            <Outlet />
          </main>
          <RightPanel />
        </div>
      </RightPanelProvider>
    </SectionsProvider>
  );
}
