/**
 * New Frontend System App
 *
 * This module provides a fully migrated Backstage app using the new
 * frontend system with extension-based architecture.
 *
 * To use this app, update index.tsx:
 *   import { createNewApp } from './AppNew';
 *   ReactDOM.createRoot(rootEl).render(createNewApp().createRoot());
 *
 * Or enable via environment variable:
 *   REACT_APP_USE_NEW_FRONTEND=true
 */

import React from 'react';
import { createApp } from '@backstage/frontend-defaults';
import {
  PageBlueprint,
  NavItemBlueprint,
  ApiBlueprint,
  createFrontendModule,
  createRouteRef,
} from '@backstage/frontend-plugin-api';
import {
  discoveryApiRef,
  fetchApiRef,
} from '@backstage/core-plugin-api';
import {
  catalogApiRef,
  starredEntitiesApiRef,
  MockStarredEntitiesApi,
} from '@backstage/plugin-catalog-react';
import { CatalogClient } from '@backstage/catalog-client';

// Import the new frontend system version of our plugin
import issueTypesPlugin from '@operator/plugin-issuetypes/alpha';

// Import homepage extensions
import {
  homeRouteRef,
  homepageExtensions,
} from './extensions';

// Route references for custom pages (homepage has its own)
const catalogRouteRef = createRouteRef();
const pluginsRouteRef = createRouteRef();
const boardRouteRef = createRouteRef();

const operatorCatalogPageExtension = PageBlueprint.make({
  name: 'operator-catalog',
  params: {
    path: '/catalog',
    routeRef: catalogRouteRef,
    loader: () =>
      import('./components/catalog/OperatorCatalogPage').then(m => (
        <m.OperatorCatalogPage />
      )),
  },
});

const pluginsPageExtension = PageBlueprint.make({
  name: 'plugins',
  params: {
    path: '/plugins',
    routeRef: pluginsRouteRef,
    loader: () =>
      import('./components/plugins').then(m => <m.PluginsPage />),
  },
});

const boardPageExtension = PageBlueprint.make({
  name: 'board',
  params: {
    path: '/board',
    routeRef: boardRouteRef,
    loader: () =>
      import('./components/kanban').then(m => <m.KanbanBoardPage />),
  },
});

// Navigation items
const HomeIcon = () => (
  <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
    <path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z" />
  </svg>
);

const CatalogIcon = () => (
  <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
    <path d="M4 8h4V4H4v4zm6 12h4v-4h-4v4zm-6 0h4v-4H4v4zm0-6h4v-4H4v4zm6 0h4v-4h-4v4zm6-10v4h4V4h-4zm-6 4h4V4h-4v4zm6 6h4v-4h-4v4zm0 6h4v-4h-4v4z" />
  </svg>
);

const PluginsIcon = () => (
  <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
    <path d="M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5C13 2.12 11.88 1 10.5 1S8 2.12 8 3.5V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.49 0 2.7 1.21 2.7 2.7s-1.21 2.7-2.7 2.7H2V20c0 1.1.9 2 2 2h3.8v-1.5c0-1.49 1.21-2.7 2.7-2.7 1.49 0 2.7 1.21 2.7 2.7V22H17c1.1 0 2-.9 2-2v-4h1.5c1.38 0 2.5-1.12 2.5-2.5S21.88 11 20.5 11z" />
  </svg>
);

const homeNavItem = NavItemBlueprint.make({
  name: 'home',
  params: {
    title: 'Home',
    routeRef: homeRouteRef,
    icon: HomeIcon,
  },
});

const catalogNavItem = NavItemBlueprint.make({
  name: 'catalog',
  params: {
    title: 'Catalog',
    routeRef: catalogRouteRef,
    icon: CatalogIcon,
  },
});

const pluginsNavItem = NavItemBlueprint.make({
  name: 'plugins',
  params: {
    title: 'Plugins',
    routeRef: pluginsRouteRef,
    icon: PluginsIcon,
  },
});

const BoardIcon = () => (
  <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
    <path d="M14 4h2v17h-2V4zM4 4h2v17H4V4zm14 0h2v17h-2V4z" />
  </svg>
);

const boardNavItem = NavItemBlueprint.make({
  name: 'board',
  params: {
    title: 'Board',
    routeRef: boardRouteRef,
    icon: BoardIcon,
  },
});

// Catalog API extension - provides catalog service for catalog components
const catalogApi = ApiBlueprint.make({
  name: 'catalog-api',
  params: defineParams =>
    defineParams({
      api: catalogApiRef,
      deps: { discoveryApi: discoveryApiRef, fetchApi: fetchApiRef },
      factory: ({ discoveryApi, fetchApi }) =>
        new CatalogClient({ discoveryApi, fetchApi }),
    }),
});

// Starred Entities API - provides in-memory starred entity storage
const starredEntitiesApi = ApiBlueprint.make({
  name: 'starred-entities-api',
  params: defineParams =>
    defineParams({
      api: starredEntitiesApiRef,
      deps: {},
      factory: () => new MockStarredEntitiesApi(),
    }),
});

// Custom module for Operator-specific extensions
const operatorAppModule = createFrontendModule({
  pluginId: 'app',
  extensions: [
    // APIs
    catalogApi,
    starredEntitiesApi,
    // Homepage (page + widgets)
    ...homepageExtensions,
    // Other pages
    operatorCatalogPageExtension,
    pluginsPageExtension,
    boardPageExtension,
    // Navigation
    homeNavItem,
    catalogNavItem,
    pluginsNavItem,
    boardNavItem,
  ],
});

/**
 * Create the new frontend system app
 */
export function createNewApp() {
  return createApp({
    features: [
      // Custom plugins
      issueTypesPlugin,
      // App module with custom pages and nav
      operatorAppModule,
    ],
  });
}

