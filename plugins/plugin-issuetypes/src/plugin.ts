/**
 * Plugin definition for the issuetypes plugin.
 */
import {
  createPlugin,
  createRoutableExtension,
  createApiFactory,
  discoveryApiRef,
  fetchApiRef,
} from '@backstage/core-plugin-api';

import {
  rootRouteRef,
  detailRouteRef,
  createRouteRef,
  editRouteRef,
  collectionsRouteRef,
} from './routes';
import { operatorApiRef, OperatorApiClient } from './api';

/** The issuetypes plugin instance */
export const issueTypesPlugin = createPlugin({
  id: 'issuetypes',
  apis: [
    createApiFactory({
      api: operatorApiRef,
      deps: {
        discoveryApi: discoveryApiRef,
        fetchApi: fetchApiRef,
      },
      factory: ({ discoveryApi, fetchApi }) =>
        new OperatorApiClient({ discoveryApi, fetchApi }),
    }),
  ],
  routes: {
    root: rootRouteRef,
    detail: detailRouteRef,
    create: createRouteRef,
    edit: editRouteRef,
    collections: collectionsRouteRef,
  },
});

/** Issue types list page extension */
export const IssueTypesPage = issueTypesPlugin.provide(
  createRoutableExtension({
    name: 'IssueTypesPage',
    component: () =>
      import('./components/IssueTypesPage').then(m => m.IssueTypesPage),
    mountPoint: rootRouteRef,
  }),
);

/** Issue type detail page extension */
export const IssueTypeDetailPage = issueTypesPlugin.provide(
  createRoutableExtension({
    name: 'IssueTypeDetailPage',
    component: () =>
      import('./components/IssueTypeDetailPage').then(
        m => m.IssueTypeDetailPage,
      ),
    mountPoint: detailRouteRef,
  }),
);

/** Issue type form page extension (create/edit) */
export const IssueTypeFormPage = issueTypesPlugin.provide(
  createRoutableExtension({
    name: 'IssueTypeFormPage',
    component: () =>
      import('./components/IssueTypeFormPage').then(m => m.IssueTypeFormPage),
    mountPoint: createRouteRef,
  }),
);

/** Collections management page extension */
export const CollectionsPage = issueTypesPlugin.provide(
  createRoutableExtension({
    name: 'CollectionsPage',
    component: () =>
      import('./components/CollectionsPage').then(m => m.CollectionsPage),
    mountPoint: collectionsRouteRef,
  }),
);
