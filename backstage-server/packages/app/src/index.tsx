/**
 * Operator Backstage Frontend Entry Point
 */

// Import BUI base styles (must be first)
import '@backstage/ui/css/styles.css';
// Import our custom theme overrides
import './theme/operator-theme.css';

import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';

const root = ReactDOM.createRoot(
  document.getElementById('root') as HTMLElement
);

root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
