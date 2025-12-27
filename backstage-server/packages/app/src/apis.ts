/**
 * API Factories
 *
 * Configures Backstage API clients to work with our Hono backend.
 */

import {
  AnyApiFactory,
  createApiFactory,
  discoveryApiRef,
  fetchApiRef,
} from '@backstage/core-plugin-api';
import { catalogApiRef } from '@backstage/plugin-catalog-react';
import { CatalogClient } from '@backstage/catalog-client';

const apis: AnyApiFactory[] = [
  // Catalog API - uses our Hono backend
  createApiFactory({
    api: catalogApiRef,
    deps: { discoveryApi: discoveryApiRef, fetchApi: fetchApiRef },
    factory: ({ discoveryApi, fetchApi }) =>
      new CatalogClient({ discoveryApi, fetchApi }),
  }),
];

export default apis;
