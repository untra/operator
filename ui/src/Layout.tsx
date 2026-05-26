import { NavLink, Outlet } from 'react-router-dom';
import styles from './Layout.module.css';

const NAV_ITEMS = [
  { to: '/', label: 'Dashboard' },
  { to: '/config', label: 'Configuration' },
  { to: '/issuetypes', label: 'Issue Types' },
  { to: '/queue', label: 'Queue' },
];

export function Layout() {
  return (
    <div className={styles.layout}>
      <nav className={styles.nav}>
        <div className={styles.brand}>Operator</div>
        <ul className={styles.navList}>
          {NAV_ITEMS.map((item) => (
            <li key={item.to}>
              <NavLink
                to={item.to}
                end={item.to === '/'}
                className={({ isActive }) =>
                  isActive ? `${styles.navLink} ${styles.active}` : styles.navLink
                }
              >
                {item.label}
              </NavLink>
            </li>
          ))}
        </ul>
      </nav>
      <main className={styles.main}>
        <Outlet />
      </main>
    </div>
  );
}
