/**
 * The SPA's application chrome: nav rail, main column, and an optional detail panel.
 *
 * Presentation only. Routing, section health, theme persistence and sign-out all
 * stay in the host, which passes the rendered pieces in as slots.
 */

import type { ReactNode } from "react";
import { ConceptIcon } from "./ConceptIcon";
import styles from "./AppShell.module.css";

export interface AppShellProps {
  brand: ReactNode;
  /** Nav groups, in order. */
  groups: ReactNode;
  /** Rendered at the foot of the nav rail. */
  footer?: ReactNode;
  children: ReactNode;
  /** The right-hand detail panel, when one is open. */
  panel?: ReactNode;
}

export function AppShell({ brand, groups, footer, children, panel }: AppShellProps) {
  return (
    <div className={styles.layout}>
      <nav className={styles.nav}>
        <div className={styles.brandRow}>{brand}</div>
        {groups}
        {footer}
      </nav>
      <main className={styles.main}>{children}</main>
      {panel}
    </div>
  );
}

export interface NavGroupProps {
  label: string;
  children: ReactNode;
}

export function NavGroup({ label, children }: NavGroupProps) {
  return (
    <div className={styles.group}>
      <p className={styles.groupLabel}>{label}</p>
      <ul className={styles.navList}>{children}</ul>
    </div>
  );
}

export interface NavRowProps {
  label: string;
  icon: string;
  /** Section health dot. Omitted for rows with no section analog. */
  health?: string | null;
  /**
   * Set when the row's prerequisites are unmet. The row renders as a disabled
   * span naming what it needs, rather than a link.
   */
  disabledReason?: string;
  /**
   * Wraps the row content in the host's router link. The resolver is shaped for a
   * router link's own className callback, so the host adds no wrapper of its own.
   */
  renderLink?: (
    content: ReactNode,
    className: (state: { isActive: boolean }) => string,
  ) => ReactNode;
}

const linkClassName = ({ isActive }: { isActive: boolean }) =>
  isActive ? `${styles.navLink} ${styles.active}` : styles.navLink;

export function NavRow({ label, icon, health, disabledReason, renderLink }: NavRowProps) {
  const content = (
    <>
      <ConceptIcon name={icon} className={styles.navIcon} />
      <span className={styles.navLabel}>{label}</span>
      {health && <span className={styles.navDot} data-health={health} />}
    </>
  );

  if (disabledReason !== undefined) {
    return (
      <span
        className={`${styles.navLink} ${styles.navDisabled}`}
        aria-disabled="true"
        title={disabledReason}
      >
        {content}
      </span>
    );
  }

  return renderLink ? (
    renderLink(content, linkClassName)
  ) : (
    <span className={styles.navLink}>{content}</span>
  );
}

export interface ThemeToggleProps {
  theme: "light" | "dark";
  onToggle: () => void;
}

export function ThemeToggle({ theme, onToggle }: ThemeToggleProps) {
  const label = theme === "dark" ? "Switch to light theme" : "Switch to dark theme";
  return (
    <button
      type="button"
      className={styles.themeToggle}
      onClick={onToggle}
      aria-label={label}
      title={label}
    >
      {theme === "dark" ? "☀" : "☾"}
    </button>
  );
}

export interface SignOutButtonProps {
  busy?: boolean;
  failed?: boolean;
  onClick: () => void;
}

export function SignOutButton({ busy, failed, onClick }: SignOutButtonProps) {
  return (
    <>
      {failed && <p className={styles.signOutError}>Could not sign out.</p>}
      <button className={styles.signOut} type="button" onClick={onClick} disabled={busy}>
        {busy ? "Signing out…" : "Sign out"}
      </button>
    </>
  );
}

export function BrandName() {
  return <span className={styles.brand}>Operator</span>;
}
