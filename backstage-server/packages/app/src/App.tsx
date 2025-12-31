/**
 * Operator Backstage Frontend
 *
 * Full Backstage portal with Home, Catalog, Search, and Issue Types.
 * Uses custom theming based on Operator's branding configuration.
 *
 * This app uses a hybrid approach for incremental migration to the new
 * frontend system. Legacy plugins (catalog, search) work alongside the
 * new Blueprint-based plugin-issuetypes/alpha.
 *
 * Migration status:
 * - [x] plugin-issuetypes: Migrated to Blueprints (alpha.ts)
 * - [ ] catalog: Using legacy system
 * - [ ] search: Using legacy system
 * - [ ] home: Using legacy system
 */

import React from 'react';
import { Route } from 'react-router-dom';
import { createApp } from '@backstage/app-defaults';
import { FlatRoutes } from '@backstage/core-app-api';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
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
import { PluginsPage } from './components/plugins';
import { KanbanBoardPage } from './components/kanban';
import apis from './apis';
import { OperatorThemeProvider } from './theme';

const app = createApp({
  apis,
  plugins: [catalogPlugin, searchPlugin, issueTypesPlugin],
});

const AppProvider = app.getProvider();
const AppRouter = app.getRouter();

// Query client for server state management
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Retry once on failure
      retry: 1,
      // Keep data fresh for 30 seconds
      staleTime: 30000,
    },
  },
});

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

    {/* Plugins */}
    <Route path="/plugins" element={<PluginsPage />} />

    {/* Kanban Board */}
    <Route path="/board" element={<KanbanBoardPage />} />

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
    <QueryClientProvider client={queryClient}>
      <OperatorThemeProvider>
        <AppProvider>
          <AppRouter>
            <Root>
              {routes}
            </Root>
          </AppRouter>
        </AppProvider>
      </OperatorThemeProvider>
    </QueryClientProvider>
  );
}

/**
 * New Frontend System App (for future full migration)
 *
 * This export provides a path to fully migrate to the new frontend system.
 * When ready, replace the default export with createNewApp().createRoot().
 *
 * Example usage in index.tsx:
 *   import { createNewApp } from './App';
 *   ReactDOM.createRoot(rootEl).render(createNewApp().createRoot());
 */
export { createNewApp } from './AppNew';
