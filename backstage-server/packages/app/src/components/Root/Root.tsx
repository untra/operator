/**
 * Root Layout Component
 *
 * Provides the sidebar navigation for the Backstage app.
 * Uses dynamic branding from Operator configuration.
 */

import { PropsWithChildren, useState, useEffect } from 'react';
import { makeStyles } from '@material-ui/core';
import HomeIcon from '@material-ui/icons/Home';
import CategoryIcon from '@material-ui/icons/Category';
import SearchIcon from '@material-ui/icons/Search';
import AssignmentIcon from '@material-ui/icons/Assignment';
// Tier icons
import LayersIcon from '@material-ui/icons/Layers';           // Foundation
import LibraryBooksIcon from '@material-ui/icons/LibraryBooks'; // Standards
import StorageIcon from '@material-ui/icons/Storage';         // Engines
import BuildIcon from '@material-ui/icons/Build';             // Ecosystem
import ArchiveIcon from '@material-ui/icons/Archive';         // Noncurrent
import {
  Sidebar,
  SidebarDivider,
  SidebarGroup,
  SidebarItem,
  SidebarPage,
  SidebarSpace,
  SidebarSubmenu,
  SidebarSubmenuItem,
  useSidebarOpenState,
} from '@backstage/core-components';
import { SidebarSearchModal } from '@backstage/plugin-search';
import { useOperatorTheme } from '../../theme';

const useSidebarLogoStyles = makeStyles((theme) => ({
  root: {
    width: '100%',
    height: 50,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '0 24px',
  },
  logo: {
    height: 36,
    maxWidth: '100%',
    objectFit: 'contain',
  },
  title: {
    fontSize: '1.2rem',
    fontWeight: 600,
    color: theme.palette.navigation?.color || theme.palette.common.white,
    textDecoration: 'none',
  },
  titleClosed: {
    fontSize: '1rem',
  },
}));

function SidebarLogo() {
  const classes = useSidebarLogoStyles();
  const { isOpen } = useSidebarOpenState();
  const theme = useOperatorTheme();
  const [logoError, setLogoError] = useState(false);

  // Reset logo error when theme changes
  useEffect(() => {
    setLogoError(false);
  }, [theme?.logoPath]);

  const appTitle = theme?.appTitle || 'Operator';
  const shortTitle = theme?.orgName?.substring(0, 2) || 'Op';
  const hasLogo = theme?.logoPath && !logoError;

  return (
    <div className={classes.root}>
      {isOpen && hasLogo ? (
        <img
          src="/branding/logo.svg"
          alt={appTitle}
          className={classes.logo}
          onError={() => setLogoError(true)}
        />
      ) : (
        <span className={`${classes.title} ${!isOpen ? classes.titleClosed : ''}`}>
          {isOpen ? appTitle : shortTitle}
        </span>
      )}
    </div>
  );
}

export function Root({ children }: PropsWithChildren<{}>) {
  return (
    <SidebarPage>
      <Sidebar>
        <SidebarLogo />
        <SidebarGroup label="Search" icon={<SearchIcon />} to="/search">
          <SidebarSearchModal />
        </SidebarGroup>
        <SidebarDivider />
        <SidebarGroup label="Menu" icon={<HomeIcon />}>
          <SidebarItem icon={HomeIcon} to="/" text="Home" />
          <SidebarItem icon={CategoryIcon} to="/catalog" text="Repositories">
            <SidebarSubmenu title="Repositories by Tier">
              <SidebarSubmenuItem
                title="Foundation"
                to="/catalog?filters[metadata.labels.operator-tier]=foundation"
                icon={LayersIcon}
              />
              <SidebarSubmenuItem
                title="Standards"
                to="/catalog?filters[metadata.labels.operator-tier]=standards"
                icon={LibraryBooksIcon}
              />
              <SidebarSubmenuItem
                title="Engines"
                to="/catalog?filters[metadata.labels.operator-tier]=engines"
                icon={StorageIcon}
              />
              <SidebarSubmenuItem
                title="Ecosystem"
                to="/catalog?filters[metadata.labels.operator-tier]=ecosystem"
                icon={BuildIcon}
              />
              <SidebarSubmenuItem
                title="Noncurrent"
                to="/catalog?filters[metadata.labels.operator-tier]=noncurrent"
                icon={ArchiveIcon}
              />
            </SidebarSubmenu>
          </SidebarItem>
          <SidebarItem icon={AssignmentIcon} to="/issuetypes" text="Issue Types">
            <SidebarSubmenu title="Issue Types">
              <SidebarSubmenuItem title="All Types" to="/issuetypes" />
              <SidebarSubmenuItem title="Collections" to="/issuetypes/collections" />
              <SidebarSubmenuItem title="New Type" to="/issuetypes/new" />
            </SidebarSubmenu>
          </SidebarItem>
        </SidebarGroup>
        <SidebarSpace />
        <SidebarDivider />
      </Sidebar>
      {children}
    </SidebarPage>
  );
}
