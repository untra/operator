/**
 * Route references for the issuetypes plugin.
 */
import { createRouteRef, createSubRouteRef } from '@backstage/core-plugin-api';

/** Root route for the issue types list page */
export const rootRouteRef = createRouteRef({
  id: 'issuetypes',
});

/** Route for viewing a single issue type */
export const detailRouteRef = createSubRouteRef({
  id: 'issuetypes-detail',
  parent: rootRouteRef,
  path: '/:key',
});

/** Route for creating a new issue type */
export const createRouteRef = createSubRouteRef({
  id: 'issuetypes-create',
  parent: rootRouteRef,
  path: '/new',
});

/** Route for editing an existing issue type */
export const editRouteRef = createSubRouteRef({
  id: 'issuetypes-edit',
  parent: rootRouteRef,
  path: '/:key/edit',
});

/** Route for managing collections */
export const collectionsRouteRef = createSubRouteRef({
  id: 'issuetypes-collections',
  parent: rootRouteRef,
  path: '/collections',
});
