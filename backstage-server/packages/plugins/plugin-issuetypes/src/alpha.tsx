/**
 * New Frontend System Plugin (Alpha)
 *
 * This module exports the plugin using Backstage's new frontend system
 * with PageBlueprint, NavItemBlueprint, and ApiBlueprint.
 *
 * Usage:
 *   import issueTypesPlugin from '@operator/plugin-issuetypes/alpha';
 *
 *   const app = createApp({
 *     features: [issueTypesPlugin],
 *   });
 */

import React from 'react';
import {
  createFrontendPlugin,
  PageBlueprint,
  NavItemBlueprint,
  ApiBlueprint,
  createRouteRef,
} from '@backstage/frontend-plugin-api';
import {
  discoveryApiRef,
  fetchApiRef,
} from '@backstage/core-plugin-api';
import { operatorApiRef, OperatorApiClient } from './api';

// Route References for the new frontend system
const rootRouteRef = createRouteRef();
const detailRouteRef = createRouteRef({ params: ['key'] });
const formRouteRef = createRouteRef();
const editRouteRef = createRouteRef({ params: ['key'] });
const collectionsRouteRef = createRouteRef();

// Icons for navigation
const IssueTypesIcon = () => (
  <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
    <path d="M4 6h16v2H4zm0 5h16v2H4zm0 5h16v2H4z" />
  </svg>
);

const CollectionsIcon = () => (
  <svg viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
    <path d="M3 3h8v8H3zm10 0h8v8h-8zM3 13h8v8H3zm10 0h8v8h-8z" />
  </svg>
);

// Page Extensions using PageBlueprint
const issueTypesPage = PageBlueprint.make({
  params: {
    path: '/issuetypes',
    routeRef: rootRouteRef,
    loader: () =>
      import('./components/IssueTypesPage').then(m => <m.IssueTypesPage />),
  },
});

const issueTypeDetailPage = PageBlueprint.make({
  name: 'detail',
  params: {
    path: '/issuetypes/:key',
    routeRef: detailRouteRef,
    loader: () =>
      import('./components/IssueTypeDetailPage').then(m => (
        <m.IssueTypeDetailPage />
      )),
  },
});

const issueTypeFormPage = PageBlueprint.make({
  name: 'form',
  params: {
    path: '/issuetypes/new',
    routeRef: formRouteRef,
    loader: () =>
      import('./components/IssueTypeFormPage').then(m => <m.IssueTypeFormPage />),
  },
});

const issueTypeEditPage = PageBlueprint.make({
  name: 'edit',
  params: {
    path: '/issuetypes/:key/edit',
    routeRef: editRouteRef,
    loader: () =>
      import('./components/IssueTypeFormPage').then(m => <m.IssueTypeFormPage />),
  },
});

const collectionsPage = PageBlueprint.make({
  name: 'collections',
  params: {
    path: '/issuetypes/collections',
    routeRef: collectionsRouteRef,
    loader: () =>
      import('./components/CollectionsPage').then(m => <m.CollectionsPage />),
  },
});

// Navigation Items using NavItemBlueprint
const issueTypesNavItem = NavItemBlueprint.make({
  params: {
    title: 'Issue Types',
    routeRef: rootRouteRef,
    icon: IssueTypesIcon,
  },
});

const collectionsNavItem = NavItemBlueprint.make({
  name: 'collections',
  params: {
    title: 'Collections',
    routeRef: collectionsRouteRef,
    icon: CollectionsIcon,
  },
});

// API Extension using ApiBlueprint with defineParams pattern
const operatorApi = ApiBlueprint.make({
  params: defineParams =>
    defineParams({
      api: operatorApiRef,
      deps: { discoveryApi: discoveryApiRef, fetchApi: fetchApiRef },
      factory: ({ discoveryApi, fetchApi }) =>
        new OperatorApiClient({ discoveryApi, fetchApi }),
    }),
});

// Plugin Definition
export default createFrontendPlugin({
  pluginId: 'issuetypes',
  routes: {
    root: rootRouteRef,
    detail: detailRouteRef,
    form: formRouteRef,
    edit: editRouteRef,
    collections: collectionsRouteRef,
  },
  extensions: [
    // Pages
    issueTypesPage,
    issueTypeDetailPage,
    issueTypeFormPage,
    issueTypeEditPage,
    collectionsPage,
    // Navigation
    issueTypesNavItem,
    collectionsNavItem,
    // API
    operatorApi,
  ],
});

// Re-export route refs for external use
export {
  rootRouteRef,
  detailRouteRef,
  formRouteRef,
  editRouteRef,
  collectionsRouteRef,
};

// Re-export API ref for convenience
export { operatorApiRef } from './api';
export type { OperatorApi } from './api';
