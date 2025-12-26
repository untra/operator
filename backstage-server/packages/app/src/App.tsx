/**
 * Operator Backstage Frontend
 *
 * Catalog browser with issue types management.
 */

import React from 'react';
import { Route } from 'react-router-dom';
import { createApp } from '@backstage/app-defaults';
import { FlatRoutes } from '@backstage/core-app-api';
import { catalogPlugin } from '@backstage/plugin-catalog';
import {
  issueTypesPlugin,
  IssueTypesPage,
  IssueTypeDetailPage,
  IssueTypeFormPage,
  CollectionsPage,
} from '@operator/plugin-issuetypes';

const app = createApp({
  plugins: [catalogPlugin, issueTypesPlugin],
});

const AppProvider = app.getProvider();
const AppRouter = app.getRouter();

const routes = (
  <FlatRoutes>
    <Route path="/issuetypes" element={<IssueTypesPage />}>
      <Route path="new" element={<IssueTypeFormPage />} />
      <Route path="collections" element={<CollectionsPage />} />
      <Route path=":key" element={<IssueTypeDetailPage />} />
      <Route path=":key/edit" element={<IssueTypeFormPage />} />
    </Route>
  </FlatRoutes>
);

export default function App() {
  return (
    <AppProvider>
      <AppRouter>
        {routes}
      </AppRouter>
    </AppProvider>
  );
}
