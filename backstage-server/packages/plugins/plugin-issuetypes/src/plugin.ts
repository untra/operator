/**
 * Operator Issue Types Plugin
 *
 * Backstage plugin for managing issue types and collections.
 */

import {
  createPlugin,
  createRoutableExtension,
  createRouteRef,
  createApiFactory,
  discoveryApiRef,
  fetchApiRef,
} from '@backstage/core-plugin-api';
import { operatorApiRef, OperatorApiClient } from './api';

// Route references
export const rootRouteRef = createRouteRef({
  id: 'issuetypes',
});

export const detailRouteRef = createRouteRef({
  id: 'issuetypes:detail',
  params: ['key'],
});

export const formRouteRef = createRouteRef({
  id: 'issuetypes:form',
  params: ['key'],
});

export const collectionsRouteRef = createRouteRef({
  id: 'issuetypes:collections',
});

// Plugin definition
export const issueTypesPlugin = createPlugin({
  id: 'issuetypes',
  routes: {
    root: rootRouteRef,
    detail: detailRouteRef,
    form: formRouteRef,
    collections: collectionsRouteRef,
  },
  apis: [
    createApiFactory({
      api: operatorApiRef,
      deps: { discoveryApi: discoveryApiRef, fetchApi: fetchApiRef },
      factory: ({ discoveryApi, fetchApi }) =>
        new OperatorApiClient({ discoveryApi, fetchApi }),
    }),
  ],
});

// Routable extensions
export const IssueTypesPage = issueTypesPlugin.provide(
  createRoutableExtension({
    name: 'IssueTypesPage',
    component: () =>
      import('./components/IssueTypesPage').then((m) => m.IssueTypesPage),
    mountPoint: rootRouteRef,
  }),
);

export const IssueTypeDetailPage = issueTypesPlugin.provide(
  createRoutableExtension({
    name: 'IssueTypeDetailPage',
    component: () =>
      import('./components/IssueTypeDetailPage').then(
        (m) => m.IssueTypeDetailPage,
      ),
    mountPoint: detailRouteRef,
  }),
);

export const IssueTypeFormPage = issueTypesPlugin.provide(
  createRoutableExtension({
    name: 'IssueTypeFormPage',
    component: () =>
      import('./components/IssueTypeFormPage').then((m) => m.IssueTypeFormPage),
    mountPoint: formRouteRef,
  }),
);

export const CollectionsPage = issueTypesPlugin.provide(
  createRoutableExtension({
    name: 'CollectionsPage',
    component: () =>
      import('./components/CollectionsPage').then((m) => m.CollectionsPage),
    mountPoint: collectionsRouteRef,
  }),
);
