/**
 * Operator Backstage Frontend
 *
 * Full Backstage portal with Home, Catalog, Search, and Issue Types.
 * Uses custom theming based on Operator's branding configuration.
 */

import React from 'react';
import { Route } from 'react-router-dom';
import { createApp } from '@backstage/app-defaults';
import { FlatRoutes } from '@backstage/core-app-api';
import {
  CatalogEntityPage,
  catalogPlugin,
} from '@backstage/plugin-catalog';
import { searchPlugin, SearchPage } from '@backstage/plugin-search';
import {
  issueTypesPlugin,
  IssueTypesPage,
  IssueTypeDetailPage,
  IssueTypeFormPage,
  CollectionsPage,
} from '@operator/plugin-issuetypes';

import { Root } from './components/Root/Root';
import { HomePage } from './components/home/HomePage';
import { entityPage } from './components/catalog/EntityPage';
import { OperatorCatalogPage } from './components/catalog/OperatorCatalogPage';
import { apis } from './apis';
import { OperatorThemeProvider } from './theme';

const app = createApp({
  apis,
  plugins: [catalogPlugin, searchPlugin, issueTypesPlugin],
});

const AppProvider = app.getProvider();
const AppRouter = app.getRouter();

const routes = (
  <FlatRoutes>
    {/* Home */}
    <Route path="/" element={<HomePage />} />

    {/* Catalog - with Operator view toggle */}
    <Route path="/catalog" element={<OperatorCatalogPage />} />
    <Route path="/catalog/:namespace/:kind/:name" element={<CatalogEntityPage />}>
      {entityPage()}
    </Route>

    {/* Search */}
    <Route path="/search" element={<SearchPage />} />

    {/* Issue Types - flat routes (no nesting, each page is independent) */}
    <Route path="/issuetypes" element={<IssueTypesPage />} />
    <Route path="/issuetypes/new" element={<IssueTypeFormPage />} />
    <Route path="/issuetypes/collections" element={<CollectionsPage />} />
    <Route path="/issuetypes/:key" element={<IssueTypeDetailPage />} />
    <Route path="/issuetypes/:key/edit" element={<IssueTypeFormPage />} />
  </FlatRoutes>
);

export default function App() {
  return (
    <OperatorThemeProvider>
      <AppProvider>
        <AppRouter>
          <Root>
            {routes}
          </Root>
        </AppRouter>
      </AppProvider>
    </OperatorThemeProvider>
  );
}
