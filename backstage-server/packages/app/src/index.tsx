/**
 * Operator Backstage Frontend Entry Point
 *
 * Uses the new Backstage frontend system with extension-based architecture.
 * Set localStorage key 'USE_LEGACY_APP' to 'true' to use legacy App.tsx.
 */

// Import BUI base styles (must be first)
import '@backstage/ui/css/styles.css';
// Import our custom theme overrides
import './theme/operator-theme.css';

import React from 'react';
import ReactDOM from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

// Query client for server state management (shared config with App.tsx)
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 30000,
    },
  },
});

// Feature flag for legacy app (check localStorage only - env handled by bundler)
const useLegacyApp = localStorage.getItem('USE_LEGACY_APP') === 'true';

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);

if (useLegacyApp) {
  // Legacy app with manual routing
  import('./App').then(({ default: App }) => {
    root.render(
      <React.StrictMode>
        <App />
      </React.StrictMode>
    );
  });
} else {
  // New frontend system with extensions
  import('./AppNew').then(({ createNewApp }) => {
    const app = createNewApp();
    root.render(
      <React.StrictMode>
        <QueryClientProvider client={queryClient}>
          {app.createRoot()}
        </QueryClientProvider>
      </React.StrictMode>
    );
  });
}
