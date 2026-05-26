import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { HashRouter, Routes, Route } from 'react-router-dom';
import './index.css';
import { HostContext, createBrowserHost } from './host';
import { Layout } from './Layout';
import { DashboardPage } from './routes/DashboardPage';
import { ConfigPage } from './routes/ConfigPage';
import { IssueTypesPage } from './routes/IssueTypesPage';
import { QueuePage } from './routes/QueuePage';

const host = createBrowserHost();

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <HostContext.Provider value={host}>
      <HashRouter>
        <Routes>
          <Route element={<Layout />}>
            <Route index element={<DashboardPage />} />
            <Route path="config" element={<ConfigPage />} />
            <Route path="issuetypes" element={<IssueTypesPage />} />
            <Route path="queue" element={<QueuePage />} />
          </Route>
        </Routes>
      </HashRouter>
    </HostContext.Provider>
  </StrictMode>,
);
