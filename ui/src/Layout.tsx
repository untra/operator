import { NavLink, Outlet, useLocation } from 'react-router-dom';
import styles from './Layout.module.css';
import { useTheme } from './theme';

interface NavItem {
  to: string;
  label: string;
}

// "Status" mirrors the canonical section order shared with the TUI and VS Code
// extension (the SectionId enum in src/ui/status_panel.rs). Configuration and
// Issue Types keep their dedicated editor pages; the rest deep-link into the
// unified Status page. The target section is passed as a `?s=` query param
// (not a URL fragment) because a secondary hash breaks HashRouter routing.
// "Pages" are web-only views with no section analog.
const STATUS_ITEMS: NavItem[] = [
  { to: '/config', label: 'Configuration' },
  { to: '/status?s=connections', label: 'Connections' },
  { to: '/status?s=kanban', label: 'Kanban' },
  { to: '/status?s=llm', label: 'LLM Tools' },
  { to: '/status?s=model-servers', label: 'Model Servers' },
  { to: '/status?s=git', label: 'Git' },
  { to: '/issuetypes', label: 'Issue Types' },
  { to: '/status?s=delegators', label: 'Delegators' },
  { to: '/status?s=projects', label: 'Managed Projects' },
];

const PAGE_ITEMS: NavItem[] = [
  { to: '/', label: 'Dashboard' },
  { to: '/queue', label: 'Queue' },
];

function NavGroup({ label, items }: { label: string; items: NavItem[] }) {
  const location = useLocation();

  const isActive = (to: string): boolean => {
    const [path, query] = to.split('?');
    if (location.pathname !== path) return false;
    if (query) {
      // e.g. "s=connections" must match the current ?s= param.
      return `?${query}` === location.search;
    }
    // Plain route: active only when no section query is present.
    return location.search === '';
  };

  return (
    <div className={styles.group}>
      <p className={styles.groupLabel}>{label}</p>
      <ul className={styles.navList}>
        {items.map((item) => (
          <li key={item.to}>
            <NavLink
              to={item.to}
              className={isActive(item.to) ? `${styles.navLink} ${styles.active}` : styles.navLink}
            >
              {item.label}
            </NavLink>
          </li>
        ))}
      </ul>
    </div>
  );
}

export function Layout() {
  const { theme, toggleTheme } = useTheme();

  return (
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
        <NavGroup label="Status" items={STATUS_ITEMS} />
        <NavGroup label="Pages" items={PAGE_ITEMS} />
      </nav>
      <main className={styles.main}>
        <Outlet />
      </main>
    </div>
  );
}
